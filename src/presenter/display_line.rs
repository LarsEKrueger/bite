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

/// Item for the output iterator to be shown by the GUI.
///
/// Each line can have its own cursor, but the GUI might render them to blink synchronously.
pub struct DisplayLine {
    pub strips: Vec<ColorString>,
    pub cursor_col: Option<usize>,
}

/// Symbolic colors
#[derive(Clone)]
pub enum Color {
    Background = 0,
    Normal,
    StatusError,
    StatusOk,

    MaxColor,
}

/// A string and its colors.
pub struct ColorString {
    pub text: String,
    pub color: Color,
}

impl DisplayLine {
    /// Create an empty line.
    pub fn new(cursor_col: Option<usize>) -> Self {
        Self {
            strips: Vec::new(),
            cursor_col,
        }
    }

    /// Add a strip.
    pub fn push_strip(&mut self, text: String, color: Color) {
        self.strips.push(ColorString { text, color });
    }

    /// Create a line to be displayed from an session item.
    ///
    /// Decorate the line according to its type and update the cursor position.
    pub fn from(line: LineItem) -> DisplayLine {
        // Depending on the type, choose the offset and draw the decoration
        let mut dl = DisplayLine::new(line.cursor_col);
        let (deco_col, deco_text) = match line.is_a {
            LineType::Output => (Color::Background, "  "),
            LineType::Prompt => (Color::Background, ""),
            LineType::Command(ref ov, _, es) => {
                (
                    match es {
                        None => Color::Normal,
                        Some(es) => {
                            if es.success() {
                                Color::StatusOk
                            } else {
                                Color::StatusError
                            }
                        }
                    },
                    match ov {
                        &OutputVisibility::None => " » ",
                        &OutputVisibility::Output => "O» ",
                        &OutputVisibility::Error => "E» ",
                    },
                )
            }
            LineType::Input => (Color::Background, ""),
            LineType::MenuDecoration => (Color::Background, ""),
            LineType::SelectedMenuItem(_) => (Color::Normal, "==> "),
            LineType::MenuItem(_) => (Color::Normal, "    "),
        };
        if !deco_text.is_empty() {
            dl.push_strip(String::from(deco_text), deco_col);
        }
        dl.push_strip(line.text.to_string(), Color::Normal);
        dl
    }
}
