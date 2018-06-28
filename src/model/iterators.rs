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

//! Various iterators and their items as used in the model.

use super::interaction::*;
use super::screen::*;

use std::process::ExitStatus;
use std::borrow::Cow;

/// Type of a line for internal processing.
#[derive(Debug, PartialEq)]
pub enum LineType {
    /// A command prompt.
    Prompt,
    /// A command with its visibility and position for changing that.
    Command(OutputVisibility, CommandPosition, Option<ExitStatus>),
    /// Output from a program (error or normal).
    Output,
    /// The input line.
    Input,

    /// A menu item that has been selected and its position in the menu.
    SelectedMenuItem(usize),
    /// A non-selected menu item and its position in the menu.
    MenuItem(usize),
    /// Non-interactive lines in a menu
    MenuDecoration,
}

/// A line to be displayed.
#[derive(Debug, PartialEq)]
pub struct LineItem<'a> {
    /// Type/Prefix
    pub is_a: LineType,

    /// Text to be displayed
    pub text: Cow<'a, [Cell]>,

    /// Cursor position, if any
    pub cursor_col: Option<usize>,

    /// Hash value of the prompt for coloring
    pub prompt_hash: u64,
}

/// Iterator to generate CommandPosition elements over the archived elements of a session.
pub struct CpArchiveIter {
    /// The index of the current archived conversation.
    pub this: usize,
}

/// Iterator to generate CommandPosition elements over the elements of a conversation.
pub struct CpConvIter {
    /// The index of the current conversation.
    pub this: CommandPosition,
}


impl<'a> LineItem<'a> {
    /// Create a new line item.
    pub fn new(l: &'a [Cell], is_a: LineType, cursor_col: Option<usize>, prompt_hash: u64) -> Self {
        Self {
            text: Cow::Borrowed(l),
            is_a,
            cursor_col,
            prompt_hash,
        }
    }

    /// Create a new line item that owns the text
    pub fn new_owned(
        l: Vec<Cell>,
        is_a: LineType,
        cursor_col: Option<usize>,
        prompt_hash: u64,
    ) -> Self {
        Self {
            text: Cow::Owned(l),
            is_a,
            cursor_col,
            prompt_hash,
        }
    }
}

impl Iterator for CpArchiveIter {
    type Item = CommandPosition;

    /// Return the next CommandPosition for an archived conversation.
    fn next(&mut self) -> Option<Self::Item> {
        let this = self.this;
        self.this += 1;
        Some(CommandPosition::Archived(this, 0))
    }
}

impl Iterator for CpConvIter {
    type Item = CommandPosition;

    /// Return the next CommandPosition for an interaction inside a conversation.
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
