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
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use super::screen::Screen;
use tools::logging::unwrap_log;

mod builtins;
mod byte_code;
mod data_stack;
pub mod jobs;
mod parser;
mod variables;

use self::variables::ContextStack;

pub struct StartupInterpreter {
    /// Session to print output to.
    session: SharedSession,

    /// Interpreter State to be initialized by the init scripts.
    pub runner: byte_code::Runner,
}

pub struct InteractiveInterpreter {
    /// Session to print output to.
    session: SharedSession,

    /// Interpreter thread
    thread: JoinHandle<()>,

    /// Mutex and condition around the input string and the interaction handle.
    ///
    /// If the mutex holds None, there is no new input
    ///
    /// TODO: Use a channel
    input: Arc<(
        Condvar,
        Mutex<Option<(byte_code::Instructions, InteractionHandle)>>,
    )>,

    /// Atomic to stop the interpreter
    is_running: Arc<AtomicBool>,
}

/// Processing function that gets input from the mutex
fn interpreter_loop(
    mut runner: byte_code::Runner,
    is_running: Arc<AtomicBool>,
    input: Arc<(
        Condvar,
        Mutex<Option<(byte_code::Instructions, InteractionHandle)>>,
    )>,
) {
    while is_running.load(Ordering::Acquire) {
        trace!("Waiting for new command");
        // Wait for condition variable and extract the string and the interaction handle.
        let (instructions, interaction_handle) = {
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

        trace!("Got instructions: »{:?}«", instructions);

        runner.run(Arc::new(instructions), interaction_handle);
    }
}

/// Parse a (partial) script and either return the byte code array or an error message
pub fn parse_script(script: &String) -> Result<byte_code::Instructions, String> {
    let mut instructions: byte_code::Instructions = Vec::new();

    let mut input = parser::Span::new(script);
    while !input.fragment.is_empty() {
        match parser::script(input) {
            Ok((rest, ast)) => {
                // Compile AST into instructions
                byte_code::compile(&mut instructions, ast)?;
                input = rest;
            }
            Err(nom::Err::Incomplete(needed)) => {
                // TODO: Create error message
                let mut msg = String::new();
                msg.push_str(&format!("Incomplete: {:?}", needed));
                return Err(msg);
            }
            Err(nom::Err::Error(code)) => {
                // TODO: Create error message
                let mut msg = String::new();
                msg.push_str(&format!("Error: {:?}", code));
                return Err(msg);
            }
            Err(nom::Err::Failure(code)) => {
                // TODO: Create error message
                let mut msg = String::new();
                msg.push_str(&format!("Failure: {:?}", code));
                return Err(msg);
            }
        }
    }

    Ok(instructions)
}

impl StartupInterpreter {
    /// Create a new interpreter.
    pub fn new(session: SharedSession) -> Self {
        let mut shell_stack = ContextStack::new();
        shell_stack.import_from_environment();
        let runner = byte_code::Runner::new(session.clone(), shell_stack);
        Self { session, runner }
    }

    /// Run a script in a given interaction
    fn run_script(&mut self, script_name: &PathBuf, interaction_handle: InteractionHandle) {
        match std::fs::File::open(script_name) {
            Ok(mut file) => {
                let mut content = String::new();
                use std::io::Read;
                match file.read_to_string(&mut content) {
                    Ok(_) => {
                        // Parse the content of the file
                        match parse_script(&content) {
                            Ok(instructions) => {
                                // Send the instructions to the runner
                                self.runner.run(Arc::new(instructions), interaction_handle);
                            }
                            Err(msg) => {
                                self.session.add_bytes(
                                    OutputVisibility::Error,
                                    interaction_handle,
                                    format!(
                                        "BiTE: Error parsing script »{}«: {}",
                                        script_name.to_string_lossy(),
                                        msg
                                    )
                                    .as_bytes(),
                                );
                                self.session.set_running_status(
                                    interaction_handle,
                                    RunningStatus::Exited(1),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        self.session.add_bytes(
                            OutputVisibility::Error,
                            interaction_handle,
                            format!(
                                "BiTE: Error reading script »{}«: {}",
                                script_name.to_string_lossy(),
                                e
                            )
                            .as_bytes(),
                        );
                        self.session
                            .set_running_status(interaction_handle, RunningStatus::Exited(1));
                    }
                }
            }
            Err(e) => {
                self.session.add_bytes(
                    OutputVisibility::Error,
                    interaction_handle,
                    format!(
                        "BiTE: Error opening script »{}«: {}",
                        script_name.to_string_lossy(),
                        e
                    )
                    .as_bytes(),
                );
                self.session
                    .set_running_status(interaction_handle, RunningStatus::Exited(1));
            }
        }
    }

    /// Run an init script in a new interaction
    pub fn run_init_script(&mut self, script_name: &PathBuf) -> InteractionHandle {
        trace!("Want to run init script »{:?}«", script_name);
        let interaction = self
            .session
            .add_interaction(Screen::one_line_matrix(b"Startup"));
        self.run_script(script_name, interaction);
        interaction
    }

    /// Start the interpreter threat and transfer the interpreter state to it.
    ///
    /// Return the interface to this thread
    pub fn complete_startup(self) -> InteractiveInterpreter {
        let is_running = Arc::new(AtomicBool::new(true));
        let input = Arc::new((Condvar::new(), Mutex::new(None)));
        let thread = {
            let session = self.session.clone();
            let is_running = is_running.clone();
            let input = input.clone();
            let runner = self.runner;
            std::thread::Builder::new()
                .name("interpreter".to_string())
                .spawn(move || interpreter_loop(runner, is_running, input))
                .unwrap()
        };

        InteractiveInterpreter {
            session: self.session,
            thread,
            input,
            is_running,
        }
    }
}

impl InteractiveInterpreter {
    /// Check if the interpreter is ready for a new command.
    pub fn is_ready(&self) -> bool {
        self.input.1.lock().unwrap().is_none()
    }

    /// Execute a set of instructions.
    ///
    /// If the interpreter is still busy with another one, this call will block. The output of any
    /// command started from this call will be added to the given interaction.
    pub fn run(
        &mut self,
        command: String,
        instructions: byte_code::Instructions,
    ) -> InteractionHandle {
        trace!("Want to run »{}«", command);
        let interaction = self
            .session
            .add_interaction(Screen::one_line_matrix(command.as_bytes()));
        let mut input = self.input.1.lock().unwrap();
        *input = Some((instructions, interaction));
        trace!("Set input");
        self.input.0.notify_one();
        trace!("Sent notification");
        interaction
    }

    /// Shut down the interpreter.
    ///
    /// This function will block until the interpreter has completed the last command.
    pub fn shutdown(self) {
        self.is_running.store(false, Ordering::Release);
        {
            let mut input = self.input.1.lock().unwrap();
            *input = Some((Vec::new(), InteractionHandle::INVALID));
        }
        self.input.0.notify_one();
        let _ = self.thread.join();
    }

    /// Get the current working directory
    pub fn get_cwd(&self) -> PathBuf {
        unwrap_log(
            nix::unistd::getcwd(),
            "getting current work directory",
            PathBuf::from("."),
        )
    }
}
