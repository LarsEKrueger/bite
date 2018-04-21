/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2018  Lars Krüger

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

//! Handle command execution and reading output and errors from the command.
//!
//! Spawns the threads that control the executed command and provides the channels to communicate
//! with them.
//!
//! The channels must be polled from the outside.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::process as spr;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::process::ExitStatusExt;
use std::io::{Read, Write};
use std::error::Error;
use std::thread::JoinHandle;
use std::collections::HashSet;

use libc::{fd_set, select, timeval, FD_ISSET, FD_SET, FD_ZERO};
use nix::unistd::{pipe, dup};

use tools::polling;
use super::*;
use super::Error as BashError;

/// Data to be sent to the receiver of the program's output.
pub enum CommandOutput {
    /// A line was read from stdout.
    FromOutput(String),

    /// A line was read from stderr.
    FromError(String),

    /// The program terminated.
    Terminated(ExitStatus, Bash),
}


/// Read a line from a pipe and report if it worked.
fn read_line<T>(pipe: &mut T) -> Option<String>
where
    T: Read,
{
    // Convert complete line as lossy UTF-8
    let mut one = [b' '; 1];
    let mut line = vec![];
    while let Ok(1) = pipe.read(&mut one) {
        if one[0] == b'\n' {
            return Some(String::from(String::from_utf8_lossy(&line[..])));
        }
        line.push(one[0]);
    }
    None
}

/// Things to wait for in run_pipeline
enum WaitFor {
    Builtin(JoinHandle<()>, Option<RawFd>),
    Command(spr::Child, bool, Option<RawFd>),
}


#[derive(PartialEq, Eq, Hash)]
struct CloseMe(RawFd);

impl Drop for CloseMe {
    fn drop(self: &mut CloseMe) {
        let _ = ::nix::unistd::close(self.0);
    }
}

impl Bash {
    /// Thread function to accept the output of the running program and provide it with input.
    pub fn run_command_sequence(
        self,
        mut output_tx: Sender<CommandOutput>,
        mut input_rx: Receiver<String>,
        sequence: Vec<CommandLogic>,
    ) {
        let mut ret_code = ExitStatus::from_raw(0);
        let bash_mutex = Arc::new(Mutex::new(self));
        for expr in sequence {
            for pipe in expr.pipelines {
                match Bash::run_pipeline(bash_mutex.clone(), &mut output_tx, &mut input_rx, &pipe) {
                    Ok(rc) => {
                        ret_code = rc;
                    }
                    Err(e) => {
                        e.send(&mut output_tx, "");
                    }
                }
                match pipe.reaction {
                    CommandReaction::Normal |
                    CommandReaction::Background => {
                        // TODO: Handle backgrounding
                    }
                    CommandReaction::And => {
                        if !(pipe.invert ^ ret_code.success()) {
                            break;
                        }
                    }
                    CommandReaction::Or => {
                        if pipe.invert ^ ret_code.success() {
                            break;
                        }
                    }
                }
            }
        }
        //output_tx
        //    .send(CommandOutput::Terminated(ret_code, self))
        //    .unwrap();
    }

    /// Run the pipeline and wait for its completion.
    ///
    /// This method can block until the last command in the pipeline finished.
    fn run_pipeline(
        bash: Arc<Mutex<Bash>>,
        _output_tx: &mut Sender<CommandOutput>,
        _input_rx: &mut Receiver<String>,
        pipeline: &Pipeline,
    ) -> Result<ExitStatus> {
        // Algo
        // * map the commands to the respective file handles.
        // * start the commands given these file handles
        // * Read from input_rx and write to output_tx
        // * wait until all commands stopped

        // Pipe for talking to the first program
        let pipe_stdin = pipe().map_err(|e| {
            BashError::CouldNotCreatePipe(e.description().to_string())
        })?;

        // pipes we need to close on error.
        let mut to_close = HashSet::new();
        to_close.insert(CloseMe(pipe_stdin.0));
        to_close.insert(CloseMe(pipe_stdin.1));

        // Build the pipe handles
        let handles = {
            let bash = bash.lock().map_err(|e| {
                BashError::CouldNotLock(e.description().to_string())
            })?;
            bash.build_handles(&mut to_close, &pipeline.commands)?
        };

        debug_assert!(handles.len() == pipeline.commands.len());

        // Array of things to wait for (threads and commands).
        let mut wait_for = Vec::new();

        let exit_status = Ok(ExitStatus::from_raw(0));

        // Start a thread for builtins or a command if not.
        for cmd_ind in 0..handles.len() {
            let is_last = cmd_ind + 1 == handles.len();
            let words = &pipeline.commands[cmd_ind].words;

            let (assignments, words) = {
                let bash = bash.lock().map_err(|e| {
                    BashError::CouldNotLock(e.description().to_string())
                })?;
                let words = bash.expand_word_list(&words)?;
                bash.separate_out_assignments(words)
            };

            // There are three cases:
            // 1. Assignments only. Do the assignments and close the handles. This is possible as
            //    the assignment operation will never print anything. The bash instance must be
            //    protected with a mutex as other commands in the pipeline could change variables
            //    as well.
            // 2. Builtin. Do the assignment, then run the builtin in a separate thread. The
            //    builtin will close its handles at the end, just as command would do.
            // 3. Command. Start the command and give it its handles. If that worked, add it to the
            //    list of things to wait for.

            // If there is no command, perform the assignments to global variables. If not,
            // perform them to temporary context, execute and drop the temporary context.
            if words.is_empty() {
                // Case 1: Simple assignment.
                // If we can't get the mutex, tell the user why and skip assignment.
                bash.lock()
                    .map_err(|e| BashError::CouldNotLock(e.description().to_string()))
                    .map(|mut bash| bash.assign_to_global_context(assignments))?;
            } else {
                {
                    let mut bash = bash.lock().map_err(|e| {
                        BashError::CouldNotLock(e.description().to_string())
                    })?;
                    bash.assign_to_temp_context(assignments)?;
                }

                // Check if this a special builtin command
                if let Some(builtin) = SPECIAL_BUILTINS.get(words[0].as_str()) {
                    // Case 2: Buildin. Run in thread and let it close the handles.
                    let bmc = bash.clone();
                    let stdin = handles[cmd_ind].0;
                    let stdout = handles[cmd_ind].1;
                    let stderr = handles[cmd_ind].2;
                    let reader = handles[cmd_ind].3;
                    wait_for.push(WaitFor::Builtin(
                        ::std::thread::spawn(
                            move || builtin(bmc, stdin, stdout, stderr, words),
                        ),
                        reader,
                    ));
                    to_close.remove(&CloseMe(stdin));
                    to_close.remove(&CloseMe(stdout));
                    to_close.remove(&CloseMe(stderr));

                    reader.map(|fd| to_close.remove(&CloseMe(fd)));
                } else {
                    // Case 3: Command.
                    {
                        let bash = bash.lock().map_err(|e| {
                            BashError::CouldNotLock(e.description().to_string())
                        })?;

                        // TODO: Handles
                        spr::Command::new(&words[0])
                            .args(&words[1..])
                            .env_clear()
                            .envs(bash.variables.iter_exported())
                            .stdin(spr::Stdio::piped())
                            .stdout(spr::Stdio::piped())
                            .stderr(spr::Stdio::piped())
                            .spawn()
                            .map_err(|e| {
                                BashError::CouldNotStartProgram(e.description().to_string())
                            })
                            .map(|child| {
                                wait_for.push(WaitFor::Command(child, is_last, handles[cmd_ind].3))
                            })?;
                    }
                }

            }
            {
                let mut bash = bash.lock().map_err(|e| {
                    BashError::CouldNotLock(e.description().to_string())
                })?;
                bash.drop_temp_context();
            }
        }

        // Read from all stderrs and the last stdout as well as write to the
        // first stdin.

        // TODO

        exit_status
    }

    /// Build the pipeline handles to connect the programs.
    ///
    /// Returns Array of tuple containing:
    /// ( stdin handle for command,
    ///   stdout handle for command,
    ///   stderr handle for command,
    ///   stderr handle for thread if exists)
    fn build_handles(
        &self,
        to_close: &mut HashSet<CloseMe>,
        commands: &Vec<self::Command>,
    ) -> Result<(Vec<(RawFd, RawFd, RawFd, Option<RawFd>)>)> {
        let mut handles = Vec::new();
        for cmd in commands {
            let h = match cmd.mode {
                PipelineMode::Nothing | PipelineMode::StdOut => {
                    // Create a pipe for either file and connect.
                    let cmd_stdout = pipe().map_err(|e| {
                        BashError::CouldNotCreatePipe(e.description().to_string())
                    })?;
                    let cmd_stderr = pipe().map_err(|e| {
                        BashError::CouldNotCreatePipe(e.description().to_string())
                    })?;
                    (cmd_stdout.0, cmd_stdout.1, cmd_stderr.0, Some(cmd_stderr.1))
                }
                PipelineMode::StdOutStdErr => {
                    // Create a pipe and dup the input handle as stderr.
                    let cmd_stdout = pipe().map_err(|e| {
                        BashError::CouldNotCreatePipe(e.description().to_string())
                    })?;
                    let cmd_stderr = dup(cmd_stdout.0).map_err(|e| {
                        BashError::CouldNotCreatePipe(e.description().to_string())
                    })?;
                    (cmd_stdout.0, cmd_stdout.1, cmd_stderr, None)
                }
            };
            to_close.insert(CloseMe(h.0));
            to_close.insert(CloseMe(h.1));
            to_close.insert(CloseMe(h.2));
            h.3.map(|fd| to_close.insert(CloseMe(fd)));
            handles.push(h);
        }
        Ok(handles)
    }

    fn wait_for_command(
        mut child: spr::Child,
        output_tx: &mut Sender<CommandOutput>,
        input_rx: &mut Receiver<String>,
    ) -> Result<ExitStatus> {
        let fd_out = child.stdout.as_ref().map(|c| c.as_raw_fd());
        let fd_err = child.stderr.as_ref().map(|c| c.as_raw_fd());

        let mut gate = polling::Gate::new(Duration::from_millis(1));
        'reader: loop {
            if gate.can_exit() {
                match child.try_wait() {
                    Err(_) => {}
                    Ok(Some(status)) => {
                        return Ok(status);
                    }
                    Ok(None) => {}
                }
            }

            let kill_child = if let Ok(line) = input_rx.try_recv() {
                if let Some(ref mut stdin) = child.stdin.as_mut() {
                    println!("sending '{}' to child", line);
                    // TODO: handle ErrorKind::Interrupted
                    // For now, kill the child and let the loop exit
                    match stdin.write(line.as_bytes()) {
                        Ok(_n) => {
                            // TODO handle n != line.len()
                            false
                        }
                        Err(_) => true,
                    }
                } else {
                    false
                }
            } else {
                false
            };
            if kill_child {
                let _ = child.kill();
            }
            gate.wait();

            let (chg_out, chg_err) = unsafe {
                let mut rfds: fd_set = ::std::mem::uninitialized();
                let mut tv = timeval {
                    tv_sec: 0,
                    tv_usec: 10000,
                };
                FD_ZERO(&mut rfds);
                let mut fd_max = 0;
                if let Some(fd_out) = fd_out {
                    FD_SET(fd_out, &mut rfds);
                    fd_max = ::std::cmp::max(fd_max, fd_out + 1);
                }
                if let Some(fd_err) = fd_err {
                    FD_SET(fd_err, &mut rfds);
                    fd_max = ::std::cmp::max(fd_max, fd_err + 1);
                }
                let retval = select(
                    fd_max,
                    &mut rfds,
                    ::std::ptr::null_mut(),
                    ::std::ptr::null_mut(),
                    &mut tv,
                );
                // Error or timeout
                if retval <= 0 {
                    (false, false)
                } else {
                    (
                        fd_out.map_or(false, |f| FD_ISSET(f, &mut rfds)),
                        fd_err.map_or(false, |f| FD_ISSET(f, &mut rfds)),
                    )
                }
            };

            if chg_out {
                if let Some(line) = read_line::<spr::ChildStdout>(child.stdout.as_mut().unwrap()) {
                    gate.mark();
                    output_tx
                        .send(CommandOutput::FromOutput(line))
                        .unwrap_or_else(|_| { let _ = child.kill(); });
                }
            }
            if chg_err {
                if let Some(line) = read_line::<spr::ChildStderr>(child.stderr.as_mut().unwrap()) {
                    gate.mark();
                    output_tx
                        .send(CommandOutput::FromError(line))
                        .unwrap_or_else(|_| { let _ = child.kill(); });
                }
            }
        }
    }
}
