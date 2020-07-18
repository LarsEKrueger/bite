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
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use super::screen::Screen;
use tools::logging::unwrap_log;

mod builtins;
mod byte_code;
mod data_stack;
pub mod grammar;
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

    /// Channel to send instructions to be run in a given interaction
    ///
    /// If the data holds None, the thread is to be stopped.
    sender: Sender<Option<(byte_code::Instructions, InteractionHandle)>>,

    /// Atomic to stop the interpreter
    is_running: Arc<AtomicBool>,

    /// In which interaction is the interpreter running code
    ///
    /// None: Interpreter is waiting for commands.
    is_busy: Arc<Mutex<Option<InteractionHandle>>>,
}

/// Processing function that gets input from the mutex
fn interpreter_loop(
    mut runner: byte_code::Runner,
    is_running: Arc<AtomicBool>,
    is_busy: Arc<Mutex<Option<InteractionHandle>>>,
    input: Receiver<Option<(byte_code::Instructions, InteractionHandle)>>,
) {
    while is_running.load(Ordering::Acquire) {
        trace!("Waiting for new command");
        // Wait for new input
        if let Ok(Some((instructions, interaction_handle))) = input.recv() {
            trace!("Got instructions: »{:?}«", instructions);
            *is_busy.lock().unwrap() = Some(interaction_handle);
            runner.run(Arc::new(instructions), interaction_handle);
            *is_busy.lock().unwrap() = None;
        } else {
            return;
        }
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
        let _ = shell_stack.import_from_environment();
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
        let is_busy = Arc::new(Mutex::new(None));
        let (sender, receiver) = channel();
        let thread = {
            let is_running = is_running.clone();
            let is_busy = is_busy.clone();
            let runner = self.runner;
            std::thread::Builder::new()
                .name("interpreter".to_string())
                .spawn(move || interpreter_loop(runner, is_running, is_busy, receiver))
                .unwrap()
        };

        InteractiveInterpreter {
            session: self.session,
            thread,
            sender,
            is_running,
            is_busy,
        }
    }
}

impl InteractiveInterpreter {
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
        trace!("Send input");
        let _ = self.sender.send(Some((instructions, interaction)));
        trace!("Input sent");
        interaction
    }

    /// Shut down the interpreter.
    ///
    /// This function will block until the interpreter has completed the last command.
    pub fn shutdown(self) {
        self.is_running.store(false, Ordering::Release);
        let _ = self.sender.send(None);
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

    /// Check if the interpreter is busy
    pub fn is_busy(&self) -> Option<InteractionHandle> {
        let handle = *self.is_busy.lock().unwrap();
        handle
    }
}
