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

use std::iter;

use super::iterators::*;
use super::response::*;

// Which output is visible
#[derive(Debug, PartialEq)]
pub enum OutputVisibility {
    None,
    Output,
    Error,
}

// Where to find a command
#[derive(Debug, PartialEq, Clone)]
pub enum CommandPosition {
    Archived(usize, usize),
    CurrentConversation(usize),
    CurrentInteraction,
}

// A command and its output
#[derive(Debug, PartialEq)]
pub struct Interaction {
    command: String,
    pub output: Response,
    pub errors: Response,
}

impl Interaction {
    pub fn new(command: String) -> Interaction {
        Interaction {
            command,
            output: Response::new(true),
            errors: Response::new(false),
        }
    }

    pub fn add_output(&mut self, line: String) {
        self.output.add_line(line);
    }

    pub fn add_error(&mut self, line: String) {
        self.errors.add_line(line);
    }

    pub fn visible_response(&self) -> Option<&Response> {
        if self.output.visible {
            Some(&self.output)
        } else if self.errors.visible {
            Some(&self.errors)
        } else {
            None
        }
    }

    pub fn line_iter<'a>(&'a self, pos: CommandPosition) -> Box<Iterator<Item = LineItem> + 'a> {
        // In order to satisfy the type, we need to return a chain of iterators. Thus, if neither
        // response is visible, we take the output iterator and skip to the end.
        let resp_lines = match self.visible_response() {
            Some(ref r) => r.line_iter(),
            None => self.output.empty_line_iter(),
        };
        let ov = match (self.output.visible, self.errors.visible) {
            (true, _) => OutputVisibility::Output,
            (false, true) => OutputVisibility::Error,
            _ => OutputVisibility::None,
        };
        Box::new(
            iter::once(LineItem::new(&self.command, LineType::Command(ov, pos))).chain(resp_lines),
        )
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.lines.is_empty()
    }

    pub fn show_errors(&mut self) {
        self.errors.visible = true;
        self.output.visible = false;
    }

    pub fn hide_output(&mut self) {
        self.errors.visible = false;
        self.output.visible = false;
    }
}

impl CommandPosition {
    pub fn archive_iter() -> CpArchiveIter {
        CpArchiveIter { this: 0 }
    }
    pub fn conv_iter(&self) -> CpConvIter {
        CpConvIter { this: (*self).clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_iter() {
        let mut inter = Interaction::new(String::from("command"));
        inter.add_output(String::from("out 1"));
        inter.add_output(String::from("out 2"));
        inter.add_output(String::from("out 3"));
        inter.add_error(String::from("err 1"));
        inter.add_error(String::from("err 2"));

        // Test the iterator for visible output
        {
            let mut li = inter.line_iter(CommandPosition::CurrentConversation(0));
            assert_eq!(
                li.next(),
                Some(LineItem {
                    text: "command",
                    is_a: LineType::Command(
                        OutputVisibility::Output,
                        CommandPosition::CurrentConversation(0),
                    ),
                })
            );
            assert_eq!(
                li.next(),
                Some(LineItem {
                    text: "out 1",
                    is_a: LineType::Output,
                })
            );
            assert_eq!(li.count(), 2);
        }

        // Test for visible errors
        {
            inter.output.visible = false;
            inter.errors.visible = true;
            let mut li = inter.line_iter(CommandPosition::Archived(1, 0));
            assert_eq!(
                li.next(),
                Some(LineItem {
                    text: "command",
                    is_a: LineType::Command(
                        OutputVisibility::Error,
                        CommandPosition::Archived(1, 0),
                    ),
                })
            );
            assert_eq!(
                li.next(),
                Some(LineItem {
                    text: "err 1",
                    is_a: LineType::Output,
                })
            );
            assert_eq!(li.count(), 1);
        }

        // Test for nothing visible
        {
            inter.output.visible = false;
            inter.errors.visible = false;
            let mut li = inter.line_iter(CommandPosition::CurrentInteraction);
            assert_eq!(
                li.next(),
                Some(LineItem {
                    text: "command",
                    is_a: LineType::Command(
                        OutputVisibility::None,
                        CommandPosition::CurrentInteraction,
                    ),
                })
            );
            assert_eq!(li.next(), None);
        }
    }
}
