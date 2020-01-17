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

//! Bash Script Interpreter
//!
//! Processes the source, starts jobs etc.

use super::session::{InteractionHandle, OutputVisibility, RunningStatus, SharedSession};
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use super::job::Job;
use super::screen::Screen;

mod jobs;
mod parser;

pub struct Interpreter {
    /// Session to print output to.
    session: SharedSession,

    /// Interpreter thread
    thread: JoinHandle<()>,

    /// Mutex and condition around the input string and the interaction handle.
    ///
    /// If the mutex holds None, there is no new input
    input: Arc<(Condvar, Mutex<Option<(String, InteractionHandle)>>)>,

    /// Atomic to stop the interpreter
    is_running: Arc<AtomicBool>,

    /// List of active jobs
    jobs: jobs::SharedJobs,
}

/// Processing function that gets input from the mutex
fn interpreter_loop(
    mut session: SharedSession,
    is_running: Arc<AtomicBool>,
    input: Arc<(Condvar, Mutex<Option<(String, InteractionHandle)>>)>,
    mut jobs: jobs::SharedJobs,
) {
    while is_running.load(Ordering::Acquire) {
        trace!( "Waiting for new command");
        assert!(!jobs.has_foreground());
        // Wait for condition variable and extract the string and the interaction handle.
        let (input_string, interaction_handle) = {
            // The lock must not be held for too long to allow other threads to check the readiness.
            let mut input_data = input.1.lock().unwrap();
            while input_data.is_none() {
                input_data = input.0.wait(input_data).unwrap();
                if !is_running.load(Ordering::Acquire) {
                    return;
                }
            }

            // Extract the data
            std::mem::replace(&mut *input_data, None).unwrap()
        };

        trace!("Got new input »{}«", input_string);

        // Process string
        let mut input = parser::Span::new(&input_string);
        while !input.fragment.is_empty() {
            session.add_bytes(
                OutputVisibility::Output,
                interaction_handle,
                format!("Parse: »{}«\n", input.fragment).as_bytes(),
            );
            match parser::script(input) {
                Ok((rest, cmd)) => {
                    // TODO: Run command
                    session.add_bytes(
                        OutputVisibility::Output,
                        interaction_handle,
                        format!("OK: Would run »{:?}«\n", cmd).as_bytes(),
                    );

                   let job = {
                       let session = session.clone();
                       Job::new(
                           session,
                           interaction_handle,
                           cmd.words[0].fragment,
                           cmd.words[1..].iter().map(|s| s.fragment),
                       )
                   };

                   jobs.set_foreground(job);

                   // TODO: Wait for foreground job to be finished

                    // Process the rest of the input
                    input = rest;
                }
                Err(nom::Err::Incomplete(n)) => {
                    // TODO: Complain about incomplete parse
                    session.add_bytes(
                        OutputVisibility::Error,
                        interaction_handle,
                        format!("Error: Incomplete »{:?}«\n", n).as_bytes(),
                    );
                    session.set_running_status(
                        interaction_handle,
                        RunningStatus::Exited(ExitStatus::from_raw(1)),
                    );
                    session.set_visibility(interaction_handle, OutputVisibility::Error);
                }
                Err(nom::Err::Error(sp)) => {
                    // TODO: Complain about error
                    session.add_bytes(
                        OutputVisibility::Error,
                        interaction_handle,
                        format!("Error: Error »{:?}«\n", sp).as_bytes(),
                    );
                    session.set_running_status(
                        interaction_handle,
                        RunningStatus::Exited(ExitStatus::from_raw(1)),
                    );
                    session.set_visibility(interaction_handle, OutputVisibility::Error);
                }
                Err(nom::Err::Failure(sp)) => {
                    // TODO: Complain about failure
                    session.add_bytes(
                        OutputVisibility::Error,
                        interaction_handle,
                        format!("Error: Failure »{:?}«\n", sp).as_bytes(),
                    );
                    session.set_running_status(
                        interaction_handle,
                        RunningStatus::Exited(ExitStatus::from_raw(1)),
                    );
                    session.set_visibility(interaction_handle, OutputVisibility::Error);
                }
            }
        }
    }
}

impl Interpreter {
    /// Create a new interpreter. This will spawn a thread.
    pub fn new(session: SharedSession) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let input = Arc::new((Condvar::new(), Mutex::new(None)));
        let jobs = jobs::SharedJobs::new();
        let thread = {
            let session = session.clone();
            let is_running = is_running.clone();
            let input = input.clone();
            let jobs = jobs.clone();
            std::thread::spawn(move || interpreter_loop(session, is_running, input, jobs))
        };

        Self {
            session,
            thread,
            input,
            is_running,
            jobs,
        }
    }

    /// Check if the interpreter is ready for a new command.
    pub fn is_ready(&self) -> bool {
        self.input.1.lock().unwrap().is_none()
    }

    /// Execute a (partial) script.
    ///
    /// If the interpreter is still busy with another one, this call will block. The output of any
    /// command started from this call will be added to the given interaction.
    pub fn run_command(&self, command: String, interaction: InteractionHandle) {
        trace!("Want to run »{}«", command);
        let mut input = self.input.1.lock().unwrap();
        *input = Some((command, interaction));
        trace!("Set input");
        self.input.0.notify_one();
        trace!("Sent notification");
    }

    /// Shut down the interpreter.
    ///
    /// This function will block until the interpreter has completed the last command.
    pub fn shutdown(self) {
        self.is_running.store(false, Ordering::Release);
        self.run_command(String::new(), InteractionHandle::INVALID);
        let _ = self.thread.join();
    }
}
