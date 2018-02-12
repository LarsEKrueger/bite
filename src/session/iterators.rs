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

use super::interaction::*;

// Type of line
#[derive(Debug, PartialEq)]
pub enum LineType {
    Prompt,
    Command(OutputVisibility, CommandPosition),
    Output,
    Input,
    MenuItem(usize),
    MenuDecoration,
}

// A line to be displayed.
#[derive(Debug, PartialEq)]
pub struct LineItem<'a> {
    // TODO Color
    // Type/Prefix
    pub is_a: LineType,

    // Text to be displayed
    pub text: &'a str,

    // Cursor position, if any
    pub cursor_col: Option<usize>,
}

pub struct CpArchiveIter {
    pub this: usize,
}

pub struct CpConvIter {
    pub this: CommandPosition,
}


impl<'a> LineItem<'a> {
    pub fn new(l: &'a str, is_a: LineType, cursor_col: Option<usize>) -> Self {
        Self {
            text: l,
            is_a,
            cursor_col,
        }
    }
}

impl Iterator for CpArchiveIter {
    type Item = CommandPosition;

    fn next(&mut self) -> Option<Self::Item> {
        let this = self.this;
        self.this += 1;
        Some(CommandPosition::Archived(this, 0))
    }
}

impl Iterator for CpConvIter {
    type Item = CommandPosition;

    fn next(&mut self) -> Option<Self::Item> {
        match self.this {
            CommandPosition::CurrentInteraction => None,
            CommandPosition::Archived(conv_ind, ref mut this_inter) => {
                let next = Some(CommandPosition::Archived(conv_ind, *this_inter));
                *this_inter += 1;
                next
            }
            CommandPosition::CurrentConversation(ref mut this_inter) => {
                let next = Some(CommandPosition::CurrentConversation(*this_inter));
                *this_inter += 1;
                next
            }
        }
    }
}
