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

//! The collected output of a program.
//!
//! Consists of the lines are read from either stdout or stderr.

use model::screen::{AddBytesResult, Cell, Event, Screen};

/// The full output of a program
#[derive(PartialEq)]
pub struct Response {
    /// Lines to be shown. Each line in a normal response is just a sequence of cells.
    pub lines: Vec<Vec<Cell>>,

    /// A temporary screen we add data to until they can be archived in *lines*.
    pub screen: Screen,
}

impl Response {
    /// Create an empty response with the given visibility.
    pub fn new() -> Response {
        Response {
            lines: vec![],
            screen: Screen::new(),
        }
    }

    /// Add a stream of bytes to the screen and possibly to the archive.
    ///
    /// Return an indication if TUI activity has been detected
    pub fn add_bytes<'a>(&mut self, bytes: &'a [u8]) -> AddBytesResult<'a> {
        for (i, b) in bytes.iter().enumerate() {
            match self.screen.add_byte(*b) {
                Event::NewLine => {
                    self.archive_screen();
                    return AddBytesResult::ShowStream(&bytes[(i + 1)..]);
                }
                Event::Cr => {
                    return AddBytesResult::ShowStream(&bytes[(i + 1)..]);
                }
                Event::StartTui => {
                    return AddBytesResult::StartTui(&bytes[(i + 1)..]);
                }
                _ => {}
            };
        }
        AddBytesResult::AllDone
    }

    /// Add all the lines on the screen to the archived lines
    pub fn archive_screen(&mut self) {
        for l in self.screen.line_iter() {
            self.lines.push(l.to_vec());
        }
        self.screen.reset();
    }
}
