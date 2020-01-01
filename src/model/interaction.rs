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

//! A command and its output.
//!
//! This command might be still running.

use std::process::ExitStatus;

use super::iterators::*;
use super::session::InteractionHandle;
use super::response::*;
use super::screen::{AddBytesResult, Matrix};

/// Which output is visible.
///
/// The GUI concept dictates that at most one output (stdout or stderr) is visible.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OutputVisibility {
    None,
    Output,
    Error,
}

/// A command and its output.
///
/// This is just a visual representation of a command and not connected to a running process in any
/// way.
#[derive(PartialEq)]
pub struct Interaction {
    /// Visual representation of the command that was called to create these responses
    command: Matrix,
    /// Collected stdout lines
    pub output: Response,
    /// Collected stderr lines
    pub errors: Response,
    /// Which response to show
    visible: OutputVisibility,
    /// exit status of the command, None if command is still runinng
    exit_status: Option<ExitStatus>,
}

impl Interaction {
    /// Create a new command without any output yet.
    ///
    /// The command is a vector of cells as to support syntax coloring later.
    pub fn new(command: Matrix) -> Self {
        Self {
            command,
            output: Response::new(),
            errors: Response::new(),
            visible: OutputVisibility::Output,
            exit_status: None,
        }
    }

    /// Set the exit status of the interaction.
    pub fn set_exit_status(&mut self, exit_status: ExitStatus) {
        self.exit_status = Some(exit_status);
    }

    /// Get the visible response, if any.
    fn visible_response(&self) -> Option<&Response> {
        match self.visible {
            OutputVisibility::None => None,
            OutputVisibility::Output => Some(&self.output),
            OutputVisibility::Error => Some(&self.errors),
        }
    }

    /// Get the iterator over the items in this interaction.
    pub fn line_iter<'a>(&'a self, handle : InteractionHandle, prompt_hash: u64) -> impl Iterator<Item = LineItem<'a>> {
        // We always have the command, regardless if there is any output to show.
        let resp_lines = self
            .visible_response()
            .map(|r| r.line_iter(prompt_hash))
            .into_iter()
            .flat_map(|i| i);

        let visible = self.visible;
        let lt = LineType::Command(visible, handle, self.exit_status);

        self.command
            .line_iter()
            .map(move |r| LineItem::new(r, lt.clone(), None, prompt_hash))
            .chain(resp_lines)
    }

    /// Check if there are any error lines.
    pub fn has_errors(&self) -> bool {
        !self.errors.lines.is_empty()
    }

    /// Make the error lines visible
    pub fn show_errors(&mut self) {
        self.visible = OutputVisibility::Error;
    }

    /// Make the output lines visible
    pub fn show_output(&mut self) {
        self.visible = OutputVisibility::Output;
    }

    /// Add a stream of bytes to the output screen and possibly to the archive.
    pub fn add_output<'a>(&mut self, bytes: &'a [u8]) -> AddBytesResult<'a> {
        self.output.add_bytes(bytes)
    }

    /// Add a stream of bytes to the error screen and possibly to the archive.
    pub fn add_error<'a>(&mut self, bytes: &'a [u8]) -> AddBytesResult<'a> {
        self.errors.add_bytes(bytes)
    }

    /// Archive both responses
    pub fn archive(&mut self) {
        self.output.archive_screen();
        self.errors.archive_screen();
    }

    /// If there are errors, show them.
    ///
    /// This is to be called before archiving the interaction, i.e. after a program has finished
    /// running.
    pub fn prepare_archiving(&mut self) {
        if self.has_errors() {
            self.show_errors();
        }
    }

    /// Cycle through the visibility flags
    pub fn cycle_visibility(&mut self) {
        let v = match self.visible {
            OutputVisibility::Output => OutputVisibility::Error,
            OutputVisibility::Error => OutputVisibility::None,
            OutputVisibility::None => OutputVisibility::Output,
        };
        self.visible = v;
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::response::tests::check;
    use super::super::screen::Screen;
    use super::*;

    #[test]
    fn archived_line_iter() {
        let mut inter = Interaction::new(Screen::one_line_matrix(b"command"));

        inter.output.add_bytes(b"out 1\nout 2\nout3\n");
        inter.errors.add_bytes(b"err 1\nerr 2\n");

        // Test the iterator for visible output
        {
            let mut li = inter.line_iter(InteractionHandle{0:0}, 0);
            check(
                li.next(),
                LineType::Command(OutputVisibility::Output, InteractionHandle{0:0}, None),
                None,
                "command",
            );
            check(li.next(), LineType::Output, None, "out 1");
            assert_eq!(li.count(), 2);
        }

        // Test for visible errors
        {
            inter.visible = OutputVisibility::Error;
            let mut li = inter.line_iter(InteractionHandle{0:0}, 0);
            check(
                li.next(),
                LineType::Command(OutputVisibility::Error, InteractionHandle{0:0}, None),
                None,
                "command",
            );
            check(li.next(), LineType::Output, None, "err 1");
            assert_eq!(li.count(), 1);
        }

        // Test for nothing visible
        {
            inter.visible = OutputVisibility::None;
            let mut li = inter.line_iter(InteractionHandle{0:0}, 0);
            check(
                li.next(),
                LineType::Command(OutputVisibility::None, InteractionHandle{0:0}, None),
                None,
                "command",
            );
            assert_eq!(li.next(), None);
        }
    }
}
