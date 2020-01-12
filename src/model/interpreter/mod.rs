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

use super::session::{InteractionHandle, SharedSession};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

mod parser;

struct Interpreter {
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
}

/// Processing function that gets input from the mutex
fn interpreter_loop(
    session: SharedSession,
    is_running: Arc<AtomicBool>,
    input: Arc<(Condvar, Mutex<Option<(String, InteractionHandle)>>)>,
) {
    while is_running.load(Ordering::Acquire) {
        // Wait for condition variable and extract the string and the interaction handle.
        let (input_string, interaction_handle) = {
            // The lock must not be held for too long to allow other threads to check the readiness.
            let mut input_data = input.1.lock().unwrap();
            while input_data.is_none() {
                input_data = input.0.wait(input_data).unwrap();
            }

            // Extract the data
            std::mem::replace(&mut *input_data, None).unwrap()
        };

        // Process string
    }
}

impl Interpreter {
    /// Create a new interpreter. This will spawn a thread.
    pub fn new(session: SharedSession) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let input = Arc::new((Condvar::new(), Mutex::new(None)));
        let thread = {
            let session = session.clone();
            let is_running = is_running.clone();
            let input = input.clone();
            std::thread::spawn(move || interpreter_loop(session, is_running, input))
        };

        Self {
            session,
            thread,
            input,
            is_running,
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
        let mut input = self.input.1.lock().unwrap();
        *input = Some((command, interaction));
        self.input.0.notify_one();
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
