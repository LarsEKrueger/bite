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

use std::iter;
use std::process::ExitStatus;

use super::iterators::*;
use super::response::*;
use super::screen::Cell;

/// Which output is visible.
///
/// The GUI concept dictates that at most one output (stdout or stderr) is visible. The internal
/// state allows both of them to be visible.
#[derive(Debug, PartialEq)]
pub enum OutputVisibility {
    None,
    Output,
    Error,
}

/// Where to find a command
#[derive(Debug, PartialEq, Clone)]
pub enum CommandPosition {
    Archived(usize, usize),
    CurrentConversation(usize),
    CurrentInteraction,
}

/// A command and its output.
///
/// This is just a visual representation of a command and not connected to a running process in any
/// way.
#[derive(PartialEq)]
pub struct Interaction {
    command: Vec<Cell>,
    pub output: Response,
    pub errors: Response,
    exit_status: Option<ExitStatus>,
}

impl Interaction {
    /// Create a new command without any output yet.
    ///
    /// The command is a vector of cells as to support syntax coloring later.
    pub fn new(command: Vec<Cell>) -> Interaction {
        Interaction {
            command,
            exit_status: None,
            output: Response::new(true),
            errors: Response::new(false),
        }
    }

    /// Set the exit status of the interaction.
    pub fn set_exit_status(&mut self, exit_status: ExitStatus) {
        self.exit_status = Some(exit_status);
    }

    // /// Add a block as read from stdout.
    pub fn add_output(&mut self, data: &[u8]) {
        self.output.add_data(data);
    }

    /// Add a block as if read from stderr.
    pub fn add_error(&mut self, data: &[u8]) {
        self.errors.add_data(data);
    }

    /// Get the visible response, if any.
    pub fn visible_response(&self) -> Option<&Response> {
        if self.output.visible {
            Some(&self.output)
        } else if self.errors.visible {
            Some(&self.errors)
        } else {
            None
        }
    }

    /// Get the iterator over the items in this interaction.
    pub fn line_iter<'a>(
        &'a self,
        pos: CommandPosition,
        prompt_hash: u64,
    ) -> impl Iterator<Item = LineItem<'a>> {
        // We always have the command, regardless if there is any output to show.
        let resp_lines = self.visible_response()
            .map(|r| r.line_iter(prompt_hash))
            .into_iter()
            .flat_map(|i| i);

        let ov = match (self.output.visible, self.errors.visible) {
            (true, _) => OutputVisibility::Output,
            (false, true) => OutputVisibility::Error,
            _ => OutputVisibility::None,
        };

        iter::once(LineItem::new(
            &self.command,
            LineType::Command(ov, pos, self.exit_status),
            None,
            prompt_hash,
        )).chain(resp_lines)

    }

    /// Check if there are any errror lines.
    pub fn has_errors(&self) -> bool {
        !self.errors.lines.is_empty()
    }

    /// Make the error lines visible
    pub fn show_errors(&mut self) {
        self.errors.visible = true;
        self.output.visible = false;
    }

    /// Hide all output.
    pub fn hide_output(&mut self) {
        self.errors.visible = false;
        self.output.visible = false;
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
        let (ov, ev) = match (self.output.visible, self.errors.visible) {
            (true, false) => (false, true),
            (false, true) => (false, false),
            _ => (true, false),
        };
        self.output.visible = ov;
        self.errors.visible = ev;
    }
}

impl CommandPosition {
    /// Iterator to create CommandPosition elements over the whole vector of archived
    /// conversations.
    pub fn archive_iter() -> CpArchiveIter {
        CpArchiveIter { this: 0 }
    }

    /// Iterator to create CommandPostion elements starting at a given command position.
    pub fn conv_iter(&self) -> CpConvIter {
        CpConvIter { this: (*self).clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::response::tests::check;
    use super::super::screen::Screen;

    #[test]
    fn basic_iter() {
        let mut inter = Interaction::new(Screen::one_line_cell_vec(b"command"));
        inter.add_output(b"out 1\r\nout 2\r\nout3\r\n");
        inter.add_error(b"err 1\r\nerr 2\r\n");

        // Test the iterator for visible output
        {
            let mut li = inter.line_iter(CommandPosition::CurrentConversation(0), 0);
            check(
                li.next(),
                LineType::Command(
                    OutputVisibility::Output,
                    CommandPosition::CurrentConversation(0),
                    None,
                ),
                None,
                "command",
            );
            check(li.next(), LineType::Output, None, "out 1");
            assert_eq!(li.count(), 2);
        }

        // Test for visible errors
        {
            inter.output.visible = false;
            inter.errors.visible = true;
            let mut li = inter.line_iter(CommandPosition::Archived(1, 0), 0);
            check(
                li.next(),
                LineType::Command(
                    OutputVisibility::Error,
                    CommandPosition::Archived(1, 0),
                    None,
                ),
                None,
                "command",
            );
            check(li.next(), LineType::Output, None, "err 1");
            assert_eq!(li.count(), 1);
        }

        // Test for nothing visible
        {
            inter.output.visible = false;
            inter.errors.visible = false;
            let mut li = inter.line_iter(CommandPosition::CurrentInteraction, 0);
            check(
                li.next(),
                LineType::Command(
                    OutputVisibility::None,
                    CommandPosition::CurrentInteraction,
                    None,
                ),
                None,
                "command",
            );
            assert_eq!(li.next(), None);
        }
    }
}
