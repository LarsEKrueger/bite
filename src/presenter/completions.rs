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
    Text(String),
}

pub struct Completions(CompleterMap);

impl Completions {
    pub fn new() -> Self {
        let mut map = CompleterMap::new();

        map.insert(script2::FOR, Completer::Text("for".to_string()));
        map.insert(script2::FUNCTION, Completer::Text("function".to_string()));

        Self(map)
    }

    pub fn lookup(&self, sym: SymbolId, start: &str) -> Vec<String> {
        let mut res = Vec::new();
        if let Some(c) = self.0.get(&sym) {
            match c {
                Completer::Text(s) => {
                    res.push(s.clone());
                }
            }
        }
        res
    }
}
