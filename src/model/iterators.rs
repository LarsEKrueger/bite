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

use super::screen::*;
use super::session::{InteractionHandle, OutputVisibility, RunningStatus};

use std::borrow::Cow;

/// Type of a line for internal processing.
#[derive(Debug, PartialEq, Clone)]
pub enum LineType {
    /// A command prompt.
    Prompt,
    /// A command with its visibility and position for changing that.
    Command(OutputVisibility, InteractionHandle, RunningStatus),
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
    /// Line of a TUI
    Tui,
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
