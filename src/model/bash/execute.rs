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
use std::sync::atomic::{Ordering, AtomicBool};
use std::time::Duration;
use std::process as spr;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::process::ExitStatusExt;
use std::error::Error;
use std::thread::JoinHandle;

use nix::unistd::{pipe, dup, read, write};
use nix::sys::select::{FdSet, select};
use nix::sys::time::{TimeVal, TimeValLike};

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
    Terminated(ExitStatus),
}

/// Read a line from a pipe and report if it worked.
fn read_line(fd: RawFd) -> Option<String> {
    // Convert complete line as lossy UTF-8
    let mut one = [b' '; 1];
    let mut line = vec![];
    while let Ok(1) = read(fd, &mut one) {
        if one[0] == b'\n' {
            return Some(String::from(String::from_utf8_lossy(&line[..])));
        }
        line.push(one[0]);
    }
    None
}

/// Things to wait for in run_pipeline
#[derive(Debug)]
enum WaitFor {
    /// A builtin function is executed in a thread
    ///
    /// (thread, running, is_last, reader)
    Builtin(JoinHandle<ExitStatus>, Arc<AtomicBool>, bool, Option<RawFd>),

    /// An external command
    ///
    /// (child, is_last, reader)
    Command(spr::Child, bool, Option<RawFd>),
}

#[derive(Debug)]
struct CloseMe(RawFd, bool);

impl Drop for CloseMe {
    fn drop(self: &mut CloseMe) {
        if self.1 {
            let _ = ::nix::unistd::close(self.0);
        }
    }
}

fn remove_handle(to_close: &mut Vec<CloseMe>, h: RawFd) {
    to_close.iter_mut().find(|hi| hi.0 == h).map(
        |cm| cm.1 = false,
    );
    to_close.retain(|hi| hi.0 != h);
}

impl Bash {
    /// Thread function to accept the output of the running program and provide it with input.
    pub fn run_command_sequence(
        bash: Arc<Mutex<Bash>>,
        mut output_tx: Sender<CommandOutput>,
        mut input_rx: Receiver<String>,
        sequence: Vec<CommandLogic>,
    ) {
        let mut ret_code = ExitStatus::from_raw(0);
        for expr in sequence {
            for pipe in expr.pipelines {
                match Bash::run_pipeline(bash.clone(), &mut output_tx, &mut input_rx, &pipe) {
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
        output_tx.send(CommandOutput::Terminated(ret_code)).unwrap();
    }

    /// Run the pipeline and wait for its completion.
    ///
    /// This method can block until the last command in the pipeline finished.
    fn run_pipeline(
        bash: Arc<Mutex<Bash>>,
        output_tx: &mut Sender<CommandOutput>,
        input_rx: &mut Receiver<String>,
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
        let mut to_close = Vec::new();
        to_close.push(CloseMe(pipe_stdin.0, true));
        to_close.push(CloseMe(pipe_stdin.1, true));

        // Build the pipe handles
        let handles = Bash::build_handles(&mut to_close, &pipeline.commands)?;
        debug_assert!(handles.len() == pipeline.commands.len());

        let (wait_for, pipe_stdout) = Bash::start_commands(
            bash,
            &mut to_close,
            &pipeline.commands,
            pipe_stdin.1,
            &handles,
        )?;

        // Read from all stderrs and the last stdout as well as write to the
        // first stdin.
        debug_assert!(handles.len() != 0);
        Ok(Bash::wait_for_commands(
            wait_for,
            output_tx,
            input_rx,
            pipe_stdin.0,
            pipe_stdout,
            handles,
        ))
    }

    /// Build the pipeline handles to connect the programs.
    ///
    /// Returns Array of tuple containing:
    /// ( stdin handle for command,
    ///   stdout handle for command,
    ///   stderr handle for command,
    ///   stderr handle for thread if exists)
    fn build_handles(
        to_close: &mut Vec<CloseMe>,
        commands: &Vec<self::Command>,
    ) -> Result<Vec<(RawFd, RawFd, RawFd, Option<RawFd>)>> {
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
                    (cmd_stdout.1, cmd_stdout.0, cmd_stderr.1, Some(cmd_stderr.0))
                }
                PipelineMode::StdOutStdErr => {
                    // Create a pipe and dup the input handle as stderr.
                    let cmd_stdout = pipe().map_err(|e| {
                        BashError::CouldNotCreatePipe(e.description().to_string())
                    })?;
                    let cmd_stderr = dup(cmd_stdout.1).map_err(|e| {
                        BashError::CouldNotCreatePipe(e.description().to_string())
                    })?;
                    (cmd_stdout.1, cmd_stdout.0, cmd_stderr, None)
                }
            };
            to_close.push(CloseMe(h.0, true));
            to_close.push(CloseMe(h.1, true));
            to_close.push(CloseMe(h.2, true));
            h.3.map(|fd| to_close.push(CloseMe(fd, true)));
            handles.push(h);
        }
        Ok(handles)
    }

    /// Start the commands of the pipeline.
    ///
    /// This function will link the pipes together.
    ///
    /// * `last_stdout` - RawFd of the stdout of the last command.
    fn start_commands(
        bash: Arc<Mutex<Bash>>,
        to_close: &mut Vec<CloseMe>,
        commands: &Vec<self::Command>,
        mut last_stdout: RawFd,
        handles: &Vec<(RawFd, RawFd, RawFd, Option<RawFd>)>,
    ) -> Result<(Vec<WaitFor>, RawFd)> {
        // Array of things to wait for (threads and commands).
        let mut wait_for = Vec::new();

        // Start a thread for builtins or a command if not.
        for cmd_ind in 0..handles.len() {
            let is_last = cmd_ind + 1 == handles.len();
            let words = &commands[cmd_ind].words;

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
                let _ = bash.lock()
                    .map_err(|e| BashError::CouldNotLock(e.description().to_string()))
                    .map(|mut bash| bash.assign_to_global_context(assignments))?;
            } else {
                {
                    // TODO: Do this in the thread for builtins?
                    let mut bash = bash.lock().map_err(|e| {
                        BashError::CouldNotLock(e.description().to_string())
                    })?;
                    bash.assign_to_temp_context(assignments)?;
                }

                let stdout = handles[cmd_ind].0;
                let next_stdin = handles[cmd_ind].1;
                let stderr = handles[cmd_ind].2;
                let reader = handles[cmd_ind].3;

                // Check if this a special builtin command
                if let Some(builtin) = SPECIAL_BUILTINS.get(words[0].as_str()) {
                    // Case 2: Buildin. Run in thread and let it close the handles.
                    let bmc = bash.clone();

                    // Remove the handles before spawning. What could go wrong?
                    remove_handle(to_close, last_stdout);
                    remove_handle(to_close, stdout);
                    remove_handle(to_close, stderr);

                    let running = Arc::new(AtomicBool::new(true));
                    let rc = running.clone();

                    wait_for.push(WaitFor::Builtin(
                        ::std::thread::spawn(move || {
                            let es = builtin(bmc, last_stdout, stdout, stderr, words);
                            rc.store(false, Ordering::SeqCst);
                            es
                        }),
                        running,
                        is_last,
                        reader,
                    ));
                } else {
                    // Case 3: Command.
                    let bash = bash.lock().map_err(|e| {
                        BashError::CouldNotLock(e.description().to_string())
                    })?;

                    spr::Command::new(&words[0])
                        .args(&words[1..])
                        .env_clear()
                        .envs(bash.variables.iter_exported())
                        .stdin(unsafe { spr::Stdio::from_raw_fd(last_stdout) })
                        .stdout(unsafe { spr::Stdio::from_raw_fd(stdout) })
                        .stderr(unsafe { spr::Stdio::from_raw_fd(stderr) })
                        .spawn()
                        .map_err(|e| {
                            BashError::CouldNotStartProgram(e.description().to_string())
                        })
                        .map(|child| {
                            let res = wait_for.push(WaitFor::Command(child, is_last, reader));
                            // Remove the handles after successful spawning.
                            remove_handle(to_close, last_stdout);
                            remove_handle(to_close, stdout);
                            remove_handle(to_close, stderr);
                            res
                        })?;
                }
                last_stdout = next_stdin;
            }
            {
                let mut bash = bash.lock().map_err(|e| {
                    BashError::CouldNotLock(e.description().to_string())
                })?;
                bash.drop_temp_context();
            }
        }
        Ok((wait_for, last_stdout))
    }

    fn wait_for_commands(
        mut wait_for: Vec<WaitFor>,
        output_tx: &mut Sender<CommandOutput>,
        input_rx: &mut Receiver<String>,
        pipe_stdin: RawFd,
        pipe_stdout: RawFd,
        handles: Vec<(RawFd, RawFd, RawFd, Option<RawFd>)>,
    ) -> ExitStatus {
        let mut gate = polling::Gate::new(Duration::from_millis(1));
        let mut exit_status = ExitStatus::from_raw(0);

        'reader: loop {
            if gate.can_exit() {
                // Go over the things we wait for and check if they exited
                let mut i = 0;
                while i < wait_for.len() {
                    let mut remove_me = false;

                    // Flag if we need to join the thread later
                    let mut join_thread = false;
                    match wait_for[i] {
                        WaitFor::Builtin(_, ref running, _, _) => {
                            if !running.load(Ordering::SeqCst) {
                                // Can't join here as we need to consume the WaitFor.
                                join_thread = true;
                                remove_me = true;
                            }
                        }
                        WaitFor::Command(ref mut child, ref is_last, _) => {
                            match child.try_wait() {
                                Err(_) => {
                                    // TODO: Report this error.
                                }
                                Ok(Some(status)) => {
                                    if *is_last {
                                        exit_status = status;
                                    }
                                    remove_me = true;
                                }
                                Ok(None) => {}
                            }
                        }
                    }

                    if remove_me {
                        let wf = wait_for.remove(i);
                        if join_thread {
                            if let WaitFor::Builtin(thread, _, is_last, _) = wf {
                                match thread.join() {
                                    Err(_) => {
                                        // TODO: Report this error.
                                    }
                                    Ok(es) => {
                                        if is_last {
                                            exit_status = es;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        i += 1;
                    }
                }

                // Exit if there is nothing to wait for
                if wait_for.is_empty() {
                    break;
                }
            }

            // TODO: handle ErrorKind::Interrupted
            if let Ok(line) = input_rx.try_recv() {
                println!("sending '{}' to child", line);
                // For now, kill the child and let the loop exit
                match write(pipe_stdin, line.as_bytes()) {
                    Ok(_n) => {
                        // TODO handle n != line.len()
                    }
                    Err(_) => {
                        // TODO handle error here
                    }
                }
            }
            gate.wait();

            {
                let mut rdfs = FdSet::new();
                rdfs.insert(pipe_stdout);
                for h in handles.iter() {
                    if let Some(fd) = h.3 {
                        rdfs.insert(fd);
                    }
                }
                let mut timeout = TimeVal::milliseconds(10);
                let (chg_out, first_err) =
                    match select(None, Some(&mut rdfs), None, None, Some(&mut timeout)) {
                        Ok(0) | Err(_) => {
                            // Error or timeout
                            (false, None)
                        }
                        Ok(_) => {
                            // Who was ready?
                            let first_err = handles.iter().find(|h| match h.3 {
                                Some(fd) => rdfs.contains(fd),
                                None => false,
                            });
                            (rdfs.contains(pipe_stdout), first_err)
                        }
                    };
                if chg_out {
                    if let Some(line) = read_line(pipe_stdout) {
                        gate.mark();
                        output_tx.send(CommandOutput::FromOutput(line)).unwrap();
                    }
                }
                if let Some(h) = first_err {
                    if let Some(fd) = h.3 {
                        if let Some(line) = read_line(fd) {
                            gate.mark();
                            output_tx.send(CommandOutput::FromError(line)).unwrap();
                        }
                    }
                }
            }
        }
        exit_status
    }
}
