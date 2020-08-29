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

//! Completions based on the grammar

use sesd::SymbolId;
use std::collections::HashMap;

use crate::model::interpreter::grammar::script2;

/// Map a grammar symbol to the competler algo
type CompleterMap = HashMap<SymbolId, Completer>;

/// Possible completer algos
///
/// * TODO: glob + filter
/// * TODO  variables
enum Completer {
    /// A text template
    /// (completion,help)
    Text(&'static str, &'static str),
}

pub struct Completions(CompleterMap);

impl Completions {
    pub fn new() -> Self {
        let mut map = CompleterMap::new();

        map.insert(script2::FOR, Completer::Text("for", "begin a for loop"));
        map.insert(
            script2::FUNCTION,
            Completer::Text("function", "declare a function"),
        );

        // map.insert(script2::SIMPLE_COMMAND , 0, "ls "

        map.insert(
            script2::AND_GREATER_GREATER,
            Completer::Text("&>>", "append output and error to file"),
        );
        map.insert(
            script2::AND_GREATER,
            Completer::Text("&>", "redirect output and error to file, preferred form"),
        );
        map.insert(
            script2::GREATER_AND,
            Completer::Text(">&", "redirect output and error to file, obsolete form"),
        );

        map.insert(
            script2::LESS_AND,
            Completer::Text("<&", "duplicate file descriptor"),
        );
        map.insert(
            script2::LESS_LESS_LESS,
            Completer::Text("<<<", "send a \x1b[3mhere\x1b[23m string to input"),
        );
        map.insert(
            script2::LESS_LESS_MINUS,
            Completer::Text("<<-", "begin a \x1b[3mhere\x1b[23m document, remove tabs"),
        );
        map.insert(
            script2::LESS_LESS,
            Completer::Text("<<", "begin a \x1b[3mhere\x1b[23m document"),
        );
        map.insert(
            script2::LESS_GREATER,
            Completer::Text("<>", "open file for reading and writing"),
        );
        map.insert(
            script2::GREATER_BAR,
            Completer::Text(">|", "redirect output, overwrite file"),
        );
        map.insert(
            script2::GREATER_GREATER,
            Completer::Text(">>", "append output to file"),
        );
        map.insert(
            script2::LOGICAL_SEP_BG,
            Completer::Text("&", "start program on the left in background"),
        );
        map.insert(
            script2::LOGICAL_SEP_FG,
            Completer::Text(";", "start program on the left in foreground"),
        );
        map.insert(
            script2::OR_OR,
            Completer::Text(
                "||",
                "if program on the left fails, run program on the right",
            ),
        );
        map.insert(
            script2::AND_AND,
            Completer::Text(
                "&&",
                "if program on the left succeeds, run program on the right",
            ),
        );
        map.insert(
            script2::BAR_AND,
            Completer::Text(
                "|&",
                "pipe output and error of program on the left to the right",
            ),
        );
        map.insert(script2::WS, Completer::Text(" ", "separate the items"));

        Self(map)
    }

    /// Given a symbol and a starting string, compute all possible completions.
    ///
    /// Return (complete, help)
    pub fn lookup(&self, sym: SymbolId, start: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        if let Some(c) = self.0.get(&sym) {
            match c {
                Completer::Text(s, h) => {
                    res.push((s.to_string(), h.to_string()));
                }
            }
        }
        res
    }
}
