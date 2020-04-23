/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2020  Lars Krüger

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

//! Organizes the output of a sequence of programs

use super::response::Response;
use model::interpreter::jobs::Job;
use model::screen::{Matrix, Screen};

/// Which output is visible.
///
/// The GUI concept dictates that at most one output (stdout or stderr) is visible.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OutputVisibility {
    None,
    Output,
    Error,
}

/// Running status of an interaction
#[derive(PartialEq, Debug, Clone)]
pub enum RunningStatus {
    Running,
    Unknown,
    Exited(i32),
}

/// A command and its output.
///
/// This is just a visual representation of a command and not connected to a running process in any
/// way.
pub struct Interaction {
    /// Visual representation of the command that was called to create these responses
    pub command: Matrix,
    /// Collected stdout lines
    pub output: Response,
    /// Collected stderr lines
    pub errors: Response,
    /// Which response to show
    pub visible: OutputVisibility,
    /// status of the command
    pub running_status: RunningStatus,
    /// True if TUI is running
    pub tui_mode: bool,
    /// Screen used for TUI mode
    pub tui_screen: Screen,
    /// Number of threads that feed data into the interaction.
    pub threads: usize,
    /// Job currently writing output to this interaction
    pub job: Option<Job>,
}

impl RunningStatus {
    pub fn is_running(&self) -> bool {
        match self {
            Self::Exited(_) => false,
            _ => true,
        }
    }
}

impl Interaction {
    /// Create a new command without any output yet.
    ///
    /// The command is a vector of cells as to support syntax coloring later.
    pub fn new(command: Matrix) -> Self {
        let mut tui_screen = Screen::new();
        tui_screen.make_room_for(79, 24);
        tui_screen.fixed_size();
        Self {
            command,
            output: Response::new(),
            errors: Response::new(),
            visible: OutputVisibility::Output,
            running_status: RunningStatus::Unknown,
            tui_mode: false,
            tui_screen,
            threads: 0,
            job: None,
        }
    }

    /// Get the visible response, if any.
    pub fn visible_response(&self) -> Option<&Response> {
        match self.visible {
            OutputVisibility::None => None,
            OutputVisibility::Output => Some(&self.output),
            OutputVisibility::Error => Some(&self.errors),
        }
    }

    /// Check if there are any error lines.
    fn has_errors(&self) -> bool {
        !self.errors.lines.is_empty()
    }

    /// Make the error lines visible
    pub fn show_errors(&mut self) {
        self.visible = OutputVisibility::Error;
    }

    /// If there are errors, show them.
    pub fn show_potential_errors(&mut self) {
        let failure = match self.running_status {
            RunningStatus::Exited(es) => es != 0,
            _ => false,
        };
        if !failure {
            self.visible = OutputVisibility::Output;
        } else if self.has_errors() {
            self.show_errors();
        }
    }

    /// If there is data in the TUI screen, add it to the end of output
    pub fn exit_cleanup(&mut self) {
        trace!("exit cleanup on interaction");
        if self.tui_mode {
            for l in self.tui_screen.line_iter_full() {
                self.output.lines.push(l.to_vec());
            }
            self.tui_mode = false;
            self.tui_screen.reset();
        }
    }
}
