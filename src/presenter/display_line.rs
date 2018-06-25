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


//! One line to be displayed.
//!
//! Each line consists of segments that have the same color.

use super::*;
use std::borrow::Cow;

/// Item for the output iterator to be shown by the GUI.
///
/// Each line can have its own cursor, but the GUI might render them to blink synchronously.
pub struct DisplayLine<'a> {
    pub prefix: &'a [Cell],
    pub line: Cow<'a, [Cell]>,
    pub cursor_col: Option<usize>,
}

lazy_static!{
    static ref OUTPUT_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( b"  ");
    static ref PROMPT_PREFIX: Vec<Cell> = Vec::new();

    // TODO: Colors
    static ref NONE_OK_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "\x1b[42m » ".as_bytes());
    static ref OUTPUT_OK_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "\x1b[42mO» ".as_bytes());
    static ref ERROR_OK_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "\x1b[42mE» ".as_bytes());
    static ref NONE_FAIL_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "\x1b[41m » ".as_bytes());
    static ref OUTPUT_FAIL_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "\x1b[41mO» ".as_bytes());
    static ref ERROR_FAIL_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "\x1b[41mE» ".as_bytes());
    static ref NONE_RUNNING_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( " » ".as_bytes());
    static ref OUTPUT_RUNNING_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "O» ".as_bytes());
    static ref ERROR_RUNNING_PREFIX : Vec<Cell> = Screen::one_line_cell_vec( "E» ".as_bytes());

    static ref INPUT_PREFIX : Vec<Cell> = Vec::new();
    static ref MENU_DECO_PREFIX : Vec<Cell> = Vec::new();
    static ref MENU_SELECT_PREFIX : Vec<Cell> = Screen::one_line_cell_vec(b"==> ");
    static ref MENU_ITEM_PREFIX : Vec<Cell> = Screen::one_line_cell_vec(b"    ");
}

impl<'a> DisplayLine<'a> {
    /// Create an empty line.
    pub fn new(prefix: &'a [Cell], line: Cow<'a, [Cell]>, cursor_col: Option<usize>) -> Self {
        Self {
            prefix,
            line,
            cursor_col,
        }
    }

    /// Create a line to be displayed from an session item.
    ///
    /// Decorate the line according to its type and update the cursor position.
    pub fn from(line: LineItem) -> DisplayLine {
        // Depending on the type, choose the offset and draw the decoration
        let deco: &Vec<Cell> = match line.is_a {
            LineType::Output => &*OUTPUT_PREFIX,
            LineType::Prompt => &*PROMPT_PREFIX,
            LineType::Command(ref ov, _, es) => {
                match (ov, es.map(|es| es.success())) {
                    (OutputVisibility::None, None) => &*NONE_RUNNING_PREFIX,
                    (OutputVisibility::Output, None) => &*OUTPUT_RUNNING_PREFIX,
                    (OutputVisibility::Error, None) => &*ERROR_RUNNING_PREFIX,

                    (OutputVisibility::None, Some(true)) => &*NONE_OK_PREFIX,
                    (OutputVisibility::Output, Some(true)) => &*OUTPUT_OK_PREFIX,
                    (OutputVisibility::Error, Some(true)) => &*ERROR_OK_PREFIX,

                    (OutputVisibility::None, Some(false)) => &*NONE_FAIL_PREFIX,
                    (OutputVisibility::Output, Some(false)) => &*OUTPUT_FAIL_PREFIX,
                    (OutputVisibility::Error, Some(false)) => &*ERROR_FAIL_PREFIX,
                }
            }

            LineType::Input => &*INPUT_PREFIX,
            LineType::MenuDecoration => &*MENU_DECO_PREFIX,
            LineType::SelectedMenuItem(_) => &*MENU_SELECT_PREFIX,
            LineType::MenuItem(_) => &*MENU_ITEM_PREFIX,
        };
        // TODO: Fix cursor_col to account for prefix
        DisplayLine::new(deco, line.text, line.cursor_col)
    }
}
