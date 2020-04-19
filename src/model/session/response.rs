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

use super::{LineItem, LineType};
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

    /// Add a stream of bytes to the screen and possibly to the archive.
    ///
    /// Ignore all TUI activity
    pub fn add_bytes_raw(&mut self, bytes: &[u8]) {
        for b in bytes.iter() {
            match self.screen.add_byte(*b) {
                Event::NewLine => {
                    self.archive_screen();
                    // Keep going
                }
                _ => {}
            };
        }
    }

    /// Add all the lines on the screen to the archived lines
    pub fn archive_screen(&mut self) {
        for l in self.screen.line_iter() {
            self.lines.push(l.to_vec());
        }
        self.screen.reset();
    }

    /// Iterate over the lines
    pub fn line_iter<'a>(&'a self, prompt_hash: u64) -> impl Iterator<Item = LineItem<'a>> {
        let screen_lines = self
            .screen
            .line_iter()
            .map(move |line| LineItem::new(&line[..], LineType::Output, None, prompt_hash));

        self.lines
            .iter()
            .map(move |l| LineItem::new(&l[..], LineType::Output, None, prompt_hash))
            .chain(screen_lines)
    }

    /// Number of lines line_iter would return
    pub fn count_lines(&self) -> usize {
        (self.screen.height() as usize) + self.lines.len()
    }

    /// Return a correctly typed iterator without any data in it.
    pub fn empty_line_iter<'a>(&'a self, prompt_hash: u64) -> impl Iterator<Item = LineItem<'a>> {
        let mut iter = self.line_iter(prompt_hash);
        iter.nth(self.lines.len());
        iter
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn l2s(item: &LineItem) -> String {
        item.text.iter().map(|c| c.code_point()).collect()
    }

    /// Export this so other module can check their iterators
    pub fn check(item: Option<LineItem>, gt_is_a: LineType, gt_col: Option<usize>, gt_txt: &str) {
        assert!(item.is_some());
        if let Some(item) = item {
            assert_eq!(item.is_a, gt_is_a);
            assert_eq!(item.cursor_col, gt_col);
            assert_eq!(l2s(&item).as_str(), gt_txt);
        }
    }

    #[test]
    fn line_iter_non_archived() {
        let mut resp = Response::new();

        let _ = resp.add_bytes(b"line 1\n");
        let _ = resp.add_bytes(b"line 2\n");
        let _ = resp.add_bytes(b"\n");
        let _ = resp.add_bytes(b"line 4");

        assert_eq!(resp.line_iter(0).count(), 4);

        let mut li = resp.line_iter(0);
        check(li.next(), LineType::Output, None, "line 1");
        check(li.next(), LineType::Output, None, "line 2");
        check(li.next(), LineType::Output, None, "");
        check(li.next(), LineType::Output, None, "line 4");
        assert_eq!(li.next(), None);
    }

    #[test]
    fn line_iter_archived() {
        let mut resp = Response::new();

        let _ = resp.add_bytes(b"line 1\n");
        let _ = resp.add_bytes(b"line 2\n");
        let _ = resp.add_bytes(b"\n");
        let _ = resp.add_bytes(b"line 4");

        assert_eq!(resp.line_iter(0).count(), 4);

        let mut li = resp.line_iter(0);
        check(li.next(), LineType::Output, None, "line 1");
        check(li.next(), LineType::Output, None, "line 2");
        check(li.next(), LineType::Output, None, "");
        check(li.next(), LineType::Output, None, "line 4");
        assert_eq!(li.next(), None);
    }

    #[test]
    fn empty_line_iter() {
        let mut resp = Response::new();
        resp.add_bytes(b"line 1\nline 2\n\nline 4\n");
        let mut li = resp.empty_line_iter(0);
        assert_eq!(li.next(), None);
    }
}
