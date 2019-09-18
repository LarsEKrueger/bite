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
use super::response::*;
use super::screen::{AddBytesResult, Cell, Matrix, Screen};
use model::screen;

/// Which output is visible.
///
/// The GUI concept dictates that at most one output (stdout or stderr) is visible. The internal
/// state allows both of them to be visible.
#[derive(Debug, PartialEq, Clone)]
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
/// way. It also does only hold completed commands.
#[derive(PartialEq)]
pub struct ArchivedInteraction {
    command: Matrix,
    pub output: Response,
    pub errors: Response,
    exit_status: Option<ExitStatus>,
}

/// The data of a currently running command.
pub struct CurrentInteraction {
    /// The process output so far
    archive: ArchivedInteraction,
    /// What is currently printed to the terminal on stdout
    output_screen: Screen,
    /// What is currently printed to the terminal on stderr
    error_screen: Screen,
}

impl ArchivedInteraction {
    /// Create a new command without any output yet.
    ///
    /// The command is a vector of cells as to support syntax coloring later.
    pub fn new(command: Matrix) -> Self {
        Self {
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

    /// Add a block as read from stdout.
    pub fn add_output(&mut self, data: Vec<Cell>) {
        self.output.add_data(data);
    }

    /// Add a block as if read from stderr.
    pub fn add_error(&mut self, data: Vec<Cell>) {
        self.errors.add_data(data);
    }

    /// Get the visible response, if any.
    fn visible_response(&self) -> Option<&Response> {
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
        let resp_lines = self
            .visible_response()
            .map(|r| r.line_iter(prompt_hash))
            .into_iter()
            .flat_map(|i| i);

        let ov = match (self.output.visible, self.errors.visible) {
            (true, _) => OutputVisibility::Output,
            (false, true) => OutputVisibility::Error,
            _ => OutputVisibility::None,
        };

        let lt = LineType::Command(ov, pos, self.exit_status);

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
        self.errors.visible = true;
        self.output.visible = false;
    }

    /// Make the output lines visible
    pub fn show_output(&mut self) {
        self.output.visible = true;
        self.errors.visible = false;
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

impl CurrentInteraction {
    pub fn new(command: Matrix) -> Self {
        Self {
            archive: ArchivedInteraction::new(command),
            output_screen: Screen::new(),
            error_screen: Screen::new(),
        }
    }

    /// Add a stream of bytes to the screen and possibly to the archive.
    ///
    /// Return true if there is progress bar activity going on.
    fn add_bytes_to_screen<'a>(
        screen: &mut Screen,
        response: &mut Response,
        bytes: &'a [u8],
    ) -> AddBytesResult<'a> {
        for (i, b) in bytes.iter().enumerate() {
            match screen.add_byte(*b) {
                // TODO: Handle TUI Switch
                screen::Event::NewLine => {
                    // Add all the lines on screen to the response
                    for l in screen.line_iter() {
                        response.add_data(l.to_vec());
                    }
                    screen.reset();
                }
                screen::Event::Cr => {
                    return AddBytesResult::ShowStream(&bytes[(i + 1)..]);
                }
                screen::Event::StartTui => {
                    return AddBytesResult::StartTui(&bytes[(i + 1)..]);
                }
                _ => {}
            };
        }
        AddBytesResult::AllDone
    }

    pub fn add_output<'a>(&mut self, bytes: &'a [u8]) -> AddBytesResult<'a> {
        Self::add_bytes_to_screen(&mut self.output_screen, &mut self.archive.output, bytes)
    }

    /// Add a stream of bytes to the screen and possibly to the archive.
    ///
    /// Return true if there is progress bar activity going on.
    pub fn add_error<'a>(&mut self, bytes: &'a [u8]) -> AddBytesResult<'a> {
        Self::add_bytes_to_screen(&mut self.error_screen, &mut self.archive.errors, bytes)
    }

    /// Get the iterator over the items in this interaction.
    pub fn line_iter<'a>(
        &'a self,
        pos: CommandPosition,
        prompt_hash: u64,
    ) -> impl Iterator<Item = LineItem<'a>> {
        let resp = match (self.archive.output.visible, self.archive.errors.visible) {
            (true, _) => Some((&self.archive.output, &self.output_screen)),
            (false, true) => Some((&self.archive.errors, &self.error_screen)),
            _ => None,
        };

        let resp_lines = resp
            .map(|(r, _)| r.line_iter(prompt_hash))
            .into_iter()
            .flat_map(|i| i);

        let screen_lines = resp
            .map(|(_, s)| {
                s.line_iter().zip(0..).map(move |(line, nr)| {
                    let cursor_x = if s.cursor_y() == nr {
                        Some(s.cursor_x() as usize)
                    } else {
                        None
                    };
                    LineItem::new(&line[..], LineType::Output, cursor_x, prompt_hash)
                })
            })
            .into_iter()
            .flat_map(|i| i);

        let ov = match (self.archive.output.visible, self.archive.errors.visible) {
            (true, _) => OutputVisibility::Output,
            (false, true) => OutputVisibility::Error,
            _ => OutputVisibility::None,
        };

        let lt = LineType::Command(ov, pos, self.archive.exit_status);

        self.archive
            .command
            .line_iter()
            .map(move |r| LineItem::new(r, lt.clone(), None, prompt_hash))
            .chain(resp_lines)
            .chain(screen_lines)
    }

    /// Set the exit status of the interaction.
    pub fn set_exit_status(&mut self, exit_status: ExitStatus) {
        self.archive.set_exit_status(exit_status);
    }

    /// Update the archive with the last state of the screen and return it.
    pub fn prepare_archiving(mut self) -> ArchivedInteraction {
        for sr in [
            (&mut self.output_screen, &mut self.archive.output),
            (&mut self.error_screen, &mut self.archive.errors),
        ]
        .iter_mut()
        {
            let (ref mut screen, ref mut response) = sr;
            for l in screen.line_iter() {
                response.add_data(l.to_vec());
            }
            screen.reset();
        }
        self.archive
    }

    pub fn get_archive(&mut self) -> &mut ArchivedInteraction {
        &mut self.archive
    }

    /// Check if there are any error lines.
    pub fn has_errors(&self) -> bool {
        self.archive.has_errors()
    }

    /// Make the error lines visible
    pub fn show_errors(&mut self) {
        self.archive.show_errors();
    }

    /// Make the output lines visible
    pub fn show_output(&mut self) {
        self.archive.show_output();
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
        CpConvIter {
            this: (*self).clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::response::tests::check;
    use super::super::screen::Screen;
    use super::*;

    pub fn test_add_output(inter: &mut ArchivedInteraction, bytes: &[u8]) {
        let m = Screen::one_line_matrix(bytes);
        for l in m.line_iter() {
            inter.add_output(l.to_vec());
        }
    }

    pub fn test_add_error(inter: &mut ArchivedInteraction, bytes: &[u8]) {
        let m = Screen::one_line_matrix(bytes);
        for l in m.line_iter() {
            inter.add_error(l.to_vec());
        }
    }

    #[test]
    fn archived_line_iter() {
        let mut inter = ArchivedInteraction::new(Screen::one_line_matrix(b"command"));

        test_add_output(&mut inter, b"out 1\nout 2\nout3\n");
        test_add_error(&mut inter, b"err 1\nerr 2\n");

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

    #[test]
    fn current_line_iter() {
        let mut inter = CurrentInteraction::new(Screen::one_line_matrix(b"command"));

        assert_eq!(
            inter
                .line_iter(CommandPosition::CurrentInteraction, 0)
                .count(),
            1
        );
        inter.add_output(b"out 1\n");

        assert_eq!(
            inter
                .line_iter(CommandPosition::CurrentInteraction, 0)
                .count(),
            2
        );

        inter.add_output(b"out 2");

        assert_eq!(
            inter
                .line_iter(CommandPosition::CurrentInteraction, 0)
                .count(),
            3
        );
    }
}
