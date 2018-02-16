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

use super::iterators::*;
use super::line::*;

// The full output of a program
#[derive(Debug, PartialEq)]
pub struct Response {
    pub visible: bool,
    pub lines: Vec<Line>,
}

impl Response {
    pub fn new(visible: bool) -> Response {
        Response {
            visible,
            lines: vec![],
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.lines.push(Line::new(line));
    }

    pub fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(self.lines.iter().map(|l| {
            LineItem::new(&l.text, LineType::Output, None)
        }))
    }

    pub fn empty_line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        let mut iter = self.lines.iter().map(|l| {
            LineItem::new(&l.text, LineType::Output, None)
        });
        iter.nth(self.lines.len());
        Box::new(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_iter() {
        let mut resp = Response::new(true);
        resp.add_line(String::from("line 1"));
        resp.add_line(String::from("line 2"));
        resp.add_line(String::from(""));
        resp.add_line(String::from("line 4"));

        let mut li = resp.line_iter();
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "line 1",
                is_a: LineType::Output,
                cursor_col: None,
            })
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
