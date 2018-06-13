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

use super::iterators::*;
use super::screen::*;

/// The full output of a program
#[derive(Debug, PartialEq)]
pub struct Response {
    /// Is it to be shown in the GUI?
    pub visible: bool,
    /// Lines to be shown. Each line in a normal response is just a sequence of cells.
    pub lines: Vec<Vec<Cell>>,
}

impl Response {
    /// Create an empty response with the given visibility.
    pub fn new(visible: bool) -> Response {
        Response {
            visible,
            lines: vec![],
        }
    }

    /// Add a line to the response.
    pub fn add_matrix(&mut self, matrix: Matrix) {
        for i in 0..matrix.rows() {
            self.lines.push(matrix.compacted_row(i));
        }
    }

    /// Iterate over the lines
    pub fn line_iter<'a>(&'a self) -> impl Iterator<Item = LineItem<'a>> {
        self.lines.iter().map(|l| {
            LineItem::new(&l[..], LineType::Output, None)
        })
    }

    /// Return a correctly typed iterator without any data in it.
    pub fn empty_line_iter<'a>(&'a self) -> impl Iterator<Item = LineItem<'a>> {
        let mut iter = self.lines.iter().map(|l| {
            LineItem::new(&l[..], LineType::Output, None)
        });
        iter.nth(self.lines.len());
        iter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_iter() {
        let mut resp = Response::new(true);

        let mut s = Screen::new();
        s.place_str("line 1");
        s.new_line();
        s.place_str("line 2");
        s.new_line();
        s.new_line();
        s.place_str("line 4");

        resp.add_matrix(s.freeze());

        let mut li = resp.line_iter();

        let l0 = li.next();
        assert!(l0.is_some());
        let Some(l0) = l0;
        assert_eq!(l0.is_a, LineType::Output);
        assert_eq!(l0.cursor_col, None);
        assert_eq!(
            String::from(l0.text.iter().map(|c| c.code_point)).as_str(),
            "line 1"
        );

        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "line 2",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "line 4",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(li.next(), None);
    }

    #[test]
    fn empty_line_iter() {
        let mut resp = Response::new(true);
        resp.add_line(String::from("line 1"));
        resp.add_line(String::from("line 2"));
        resp.add_line(String::from(""));
        resp.add_line(String::from("line 4"));

        let mut li = resp.empty_line_iter();
        assert_eq!(li.next(), None);
    }
}
