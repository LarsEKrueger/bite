/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2020  Lars Krüger

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

//! Rendering style for the syntax highlighting of the command input

use sesd;
use sesd::{char::CharMatcher, CompiledGrammar, ERROR_ID};

/// Styling instructions for rendering the command input
///
/// TODO: Indentation
#[derive(Debug)]
pub struct Style {
    /// Escape codes to be sent before the text.
    ///
    /// Can contain line breaks and decoration
    pub pre: String,
    /// Escape codes to be sent after the text
    ///
    /// Can contain line breaks and decoration
    pub post: String,
}

pub type StyleSheet = sesd::style_sheet::StyleSheet<Style>;
pub type StyleMatcher = sesd::style_sheet::StyleMatcher<Style>;
pub type LookedUp<'a> = sesd::style_sheet::LookedUp<'a, Style>;

lazy_static! {
    /// Default: Normal color
    pub static ref DEFAULT: Style = Style {
        pre: String::new(),
        post: String::new()
    };

    /// Unparsed input: Yellow on red
    pub static ref UNPARSED: Style = Style {
        pre: "\x1b[33;41m".to_string(),
        post: "\x1b[0m".to_string(),
    };
}

fn s(pre: &str, post: &str) -> Style {
    Style {
        pre: pre.to_string(),
        post: post.to_string(),
    }
}

/// Create the stye sheet for the script input
pub fn script(grammar: &CompiledGrammar<char, CharMatcher>) -> StyleSheet {
    let mut sheet = StyleSheet::new();

    // Simple command: green on black
    sheet.add(StyleMatcher::new(s("\x1b[32m", "\x1b[0m")).skip_to(grammar.nt_id("simple_command")));

    // Logical operators: cyan on black
    sheet.add(StyleMatcher::new(s("\x1b[36m", "\x1b[0m")).skip_to(grammar.nt_id("AND_AND")));
    sheet.add(StyleMatcher::new(s("\x1b[36m", "\x1b[0m")).skip_to(grammar.nt_id("OR_OR")));

    // Comment: Yellow on black
    sheet.add(StyleMatcher::new(s("\x1b[33m", "\x1b[0m")).skip_to(grammar.nt_id("comment")));

    // Error: White on red
    sheet.add(StyleMatcher::new(s("\x1b[37;41m", "\x1b[0m")).skip_to(ERROR_ID));

    sheet
}
