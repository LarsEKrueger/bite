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

//! Keep track of previously entered commands
//!
//! TODO: Merge histories on save

use itertools::Itertools;
use radix_trie::TrieCommon;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;

/// Map a command to the number of times it was entered
type CommandCountMap = radix_trie::Trie<String, u32>;

/// Zero-cost abstraction around the trie to add some operations
#[derive(PartialEq, Debug)]
struct EnteredCommands(CommandCountMap);

/// The history is a sorted by folders
#[derive(PartialEq, Debug)]
pub struct History(HashMap<String, EnteredCommands>);

impl History {
    /// Create empty history
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Load the history from the given file.
    pub fn load(file_name: &str) -> Result<History, String> {
        let file_handle = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_EXCL)
            .open(file_name)
            .map_err(|e| e.description().to_string())?;

        History::deserialize_from(file_handle)
    }

    /// Save the history
    pub fn save(&self, file_name: &str) -> Result<(), String> {
        let file_handle = OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_EXCL)
            .open(file_name)
            .map_err(|e| e.description().to_string())?;
        self.serialize_into(file_handle);
        Ok(())
    }

    /// Enter a command in the history
    pub fn enter(&mut self, dir: &str, cmd: &str) {
        if !self.0.contains_key(dir) {
            self.0
                .insert(dir.to_string(), EnteredCommands(CommandCountMap::new()));
        }
        if let Some(dir_cmds) = self.0.get_mut(dir) {
            dir_cmds.enter(cmd);
        }
    }

    /// As radix_trie does not support serde, obtain a HashMap of HashMaps.
    fn deserialize_from<R>(reader: R) -> Result<History, String>
    where
        R: Read,
    {
        let input: HashMap<String, HashMap<String, u32>> =
            bincode::deserialize_from(reader).map_err(|e| e.description().to_string())?;
        let mut hm = HashMap::new();

        for (dir, cmds) in input.iter() {
            let mut ccm = CommandCountMap::new();
            for (cmd, cnt) in cmds.iter() {
                let _ = ccm.insert(cmd.to_string(), *cnt);
            }
            let _ = hm.insert(dir.to_string(), EnteredCommands(ccm));
        }
        Ok(History(hm))
    }

    /// As radix_trie does not support serde, serialize a HashMap of HashMaps.
    fn serialize_into<W>(&self, writer: W)
    where
        W: Write,
    {
        let mut hm = HashMap::new();

        for (dir, cmds) in self.0.iter() {
            let mut ccm = HashMap::new();
            for (cmd, cnt) in cmds.0.iter() {
                let _ = ccm.insert(cmd.to_string(), *cnt);
            }
            let _ = hm.insert(dir.to_string(), ccm);
        }
        let _ = bincode::serialize_into(writer, &hm);
    }
}

impl EnteredCommands {
    /// Put the string in the map, increment the count if it was already there.
    fn enter(&mut self, command: &str) {
        if let Some(counter) = self.0.get_mut(command) {
            *counter += 1;
        } else {
            self.0.insert(command.to_string(), 1);
        }
    }

    fn predict(&self, start: &str) -> impl Iterator<Item = &str> {
        let start_len = start.len();
        self.0
            .subtrie(start)
            .iter()
            .flat_map(|subtrie| subtrie.iter())
            .sorted_by(|a, b| Ord::cmp(b.1, a.1))
            .map(move |(s, _)| &s[start_len..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter() {
        let mut trie = EnteredCommands(CommandCountMap::new());

        trie.enter("ab cd ef");
        trie.enter("ab cd ef");
        trie.enter("ab cd ef");
        trie.enter("ab ef cd");
        trie.enter("ab ef cd");
        trie.enter("ef cd ab");

        assert_eq!(trie.0.len(), 3);
        assert_eq!(trie.0.get("ab cd ef"), Some(&3));
        assert_eq!(trie.0.get("ab ef cd"), Some(&2));
        assert_eq!(trie.0.get("ef cd ab"), Some(&1));
    }

    #[test]
    fn predict() {
        let trie = {
            let mut trie = EnteredCommands(CommandCountMap::new());

            trie.enter("abzzz ef");
            trie.enter("abzzz ef");
            trie.enter("abzzz ef");
            trie.enter("ab ef cd");
            trie.enter("ab ef cd");
            trie.enter("abxxx");
            trie.enter("yyyyy");
            trie
        };

        let mut options = trie.predict("ab");
        assert_eq!(options.next(), Some("zzz ef"));
        assert_eq!(options.next(), Some(" ef cd"));
        assert_eq!(options.next(), Some("xxx"));
        assert_eq!(options.next(), None);
    }

    #[test]
    fn serde() {
        // Build a history
        let mut history = History(HashMap::new());

        history.enter("/home/user", "ab cd ef");
        history.enter("/home/user", "ab cd ef");
        history.enter("/home/user", "ab cd ef");
        history.enter("/home/user/stuff", "ab cd ef");
        history.enter("/home/user/stuff", "ab cd ef");
        history.enter("/home/user", "cd ef");

        let mut buffer = Vec::new();
        history.serialize_into(&mut buffer);

        let maybe_readback = History::deserialize_from(&buffer[..]);
        assert_eq!(maybe_readback.is_ok(), true);
        if let Ok(readback) = maybe_readback {
            assert_eq!(history, readback);
        }
    }
}
