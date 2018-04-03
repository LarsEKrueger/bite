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
use std::time::Duration;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::os::unix::io::AsRawFd;
use std::os::unix::process::ExitStatusExt;
use std::io::{Read, Write};

use libc::{fd_set, select, timeval, FD_ISSET, FD_SET, FD_ZERO};

use tools::polling;
use super::*;

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

impl Bash {
    /// Thread function to accept the output of the running program and provide it with input.
    pub fn run_commands(
        mut self,
        mut output_tx: Sender<CommandOutput>,
        mut input_rx: Receiver<String>,
        terms: Vec<CommandTerm>,
    ) {
        let mut ret_code = ExitStatus::from_raw(0);
        for term in terms {
            for cmd in term.commands {
                match self.run_command(&mut output_tx, &mut input_rx, cmd.words) {
                    Ok(rc) => {
                        ret_code = rc;
                    }
                    Err(e) => {
                        e.send(&mut output_tx, "");
                    }
                }
                match cmd.reaction {
                    CommandReaction::Normal |
                    CommandReaction::Background => {
                        // TODO: Handle backgrounding
                    }
                    CommandReaction::And => {
                        if !ret_code.success() {
                            break;
                        }
                    }
                    CommandReaction::Or => {
                        if ret_code.success() {
                            break;
                        }
                    }
                }
            }
        }
        output_tx
            .send(CommandOutput::Terminated(ret_code, self))
            .unwrap();
    }

    fn run_command(
        &mut self,
        output_tx: &mut Sender<CommandOutput>,
        input_rx: &mut Receiver<String>,
        words: Vec<String>,
    ) -> Result<ExitStatus> {
        let words = self.expand_word_list(&words)?;
        let (assignments, words) = self.separate_out_assignments(words);

        // If there is no command, perform the assignments to global variables. If not,
        // perform them to temporary context, execute and drop the temporary context.
        if words.is_empty() {
            self.assign_to_global_context(assignments)?;
            Ok(ExitStatus::from_raw(0))
        } else {
            self.assign_to_temp_context(assignments)?;
            let res = self.run_simple_command(output_tx, input_rx, words);
            self.drop_temp_context();
            res
        }
    }

    /// Execute a builtin command or spawn a thread to run an external command
    fn run_simple_command(
        &mut self,
        output_tx: &mut Sender<CommandOutput>,
        input_rx: &mut Receiver<String>,
        words: Vec<String>,
    ) -> Result<ExitStatus> {
        debug_assert!(!words.is_empty());

        // Check if this a special builtin command
        if let Some(builtin) = SPECIAL_BUILTINS.get(words[0].as_str()) {
            return Ok(self.run_builtin(*builtin, output_tx, words));
        }

        // This is an external command.
        match Command::new(&words[0])
            .args(&words[1..])
            .env_clear()
            .envs(self.variables.iter_exported())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() {
            Ok(child) => Bash::wait_for_command(child, output_tx, input_rx),
            Err(e) => Err(Error::CouldNotStartProgram(format!("{}", e))),
        }
    }

    fn wait_for_command(
        mut child: Child,
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
                if let Some(line) = read_line::<ChildStdout>(child.stdout.as_mut().unwrap()) {
                    gate.mark();
                    output_tx
                        .send(CommandOutput::FromOutput(line))
                        .unwrap_or_else(|_| { let _ = child.kill(); });
                }
            }
            if chg_err {
                if let Some(line) = read_line::<ChildStderr>(child.stderr.as_mut().unwrap()) {
                    gate.mark();
                    output_tx
                        .send(CommandOutput::FromError(line))
                        .unwrap_or_else(|_| { let _ = child.kill(); });
                }
            }
        }
    }
}
