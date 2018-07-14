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
#[derive(PartialEq)]
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

    /// Add a line.
    pub fn add_data(&mut self, data: Vec<Cell>) {
        self.lines.push(data);
    }

    /// Iterate over the lines
    pub fn line_iter<'a>(&'a self, prompt_hash: u64) -> impl Iterator<Item = LineItem<'a>> {
        self.lines.iter().map(move |l| {
            LineItem::new(&l[..], LineType::Output, None, prompt_hash)
        })
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
    fn line_iter() {
        let mut resp = Response::new(true);

        let mut s = Screen::new();
        s.place_str("line 1");
        s.new_line();
        s.place_str("line 2");
        s.new_line();
        s.new_line();
        s.place_str("line 4");

        let m = s.freeze();
        for l in m.line_iter() {
            resp.add_data(l.to_vec());
        }

        let mut li = resp.line_iter(0);

        check(li.next(), LineType::Output, None, "line 1");
        check(li.next(), LineType::Output, None, "line 2");
        check(li.next(), LineType::Output, None, "");
        check(li.next(), LineType::Output, None, "line 4");
        assert_eq!(li.next(), None);
    }

    #[test]
    fn empty_line_iter() {
        let mut resp = Response::new(true);
        let mut s = Screen::new();
        s.place_str("line 1");
        s.new_line();
        s.place_str("line 2");
        s.new_line();
        s.new_line();
        s.place_str("line 4");

        let m = s.freeze();
        for l in m.line_iter() {
            resp.add_data(l.to_vec());
        }

        let mut li = resp.empty_line_iter(0);
        assert_eq!(li.next(), None);
    }
}
