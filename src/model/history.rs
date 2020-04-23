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
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;

/// Map a command to the number of times it was entered
type CommandCountMap = qptrie::Trie<String, u32>;

/// Zero-cost abstraction around the trie to add some operations
#[derive(Debug)]
struct EnteredCommands(CommandCountMap);

/// History of all entered commands, sorted by folder.
///
/// The history is in charge of making predictions of the next command given the start of the
/// current one. As the prediction will be read on every render, it is cached.
#[derive(Debug)]
pub struct History {
    /// Count frequency of commands, ordered by directory
    commands: HashMap<String, EnteredCommands>,

    /// Last prediction, most frequent first
    pub prediction: Vec<String>,
}

impl History {
    /// Create empty history
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            prediction: Vec::new(),
        }
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
            .create(true)
            .truncate(true)
            .open(file_name)
            .map_err(|e| e.description().to_string())?;
        self.serialize_into(file_handle);
        Ok(())
    }

    /// Enter a command in the history
    pub fn enter(&mut self, dir: &str, cmd: &String) {
        if !self.commands.contains_key(dir) {
            self.commands
                .insert(dir.to_string(), EnteredCommands(CommandCountMap::new()));
        }
        if let Some(dir_cmds) = self.commands.get_mut(dir) {
            dir_cmds.enter(cmd);
        }
    }

    /// Compute a new prediction
    pub fn predict(&mut self, dir: &str, start: &String) {
        self.prediction.clear();
        if let Some(ec) = self.commands.get(dir) {
            for p in ec.predict(start) {
                self.prediction.push(p);
            }
        }
    }

    /// Get the latest prediction
    pub fn prediction<'a>(&'a self) -> &'a Vec<String> {
        &self.prediction
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
        Ok(History {
            commands: hm,
            prediction: Vec::new(),
        })
    }

    /// As radix_trie does not support serde, serialize a HashMap of HashMaps.
    fn serialize_into<W>(&self, writer: W)
    where
        W: Write,
    {
        let mut hm = HashMap::new();

        for (dir, cmds) in self.commands.iter() {
            let mut ccm = HashMap::new();
            for (cmd, cnt) in cmds.0.prefix_iter(&String::new()) {
                let _ = ccm.insert(cmd.to_string(), *cnt);
            }
            let _ = hm.insert(dir.to_string(), ccm);
        }
        let _ = bincode::serialize_into(writer, &hm);
    }
}

impl EnteredCommands {
    /// Put the string in the map, increment the count if it was already there.
    fn enter(&mut self, command: &String) {
        if let Some(counter) = self.0.get_mut(command) {
            *counter += 1;
        } else {
            self.0.insert(command.to_string(), 1);
        }
    }

    fn predict<'a>(&'a self, start: &'a String) -> impl Iterator<Item = String> + 'a {
        let start_len = start.len();
        self.0
            .prefix_iter(start)
            .sorted_by(|a, b| Ord::cmp(b.1, a.1))
            .map(move |(s, _)| s[start_len..].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter() {
        let mut trie = EnteredCommands(CommandCountMap::new());

        trie.enter(&"ab cd ef".to_string());
        trie.enter(&"ab cd ef".to_string());
        trie.enter(&"ab cd ef".to_string());
        trie.enter(&"ab ef cd".to_string());
        trie.enter(&"ab ef cd".to_string());
        trie.enter(&"ef cd ab".to_string());

        assert_eq!(trie.0.get(&"ab cd ef".to_string()), Some(&3));
        assert_eq!(trie.0.get(&"ab ef cd".to_string()), Some(&2));
        assert_eq!(trie.0.get(&"ef cd ab".to_string()), Some(&1));
    }

    #[test]
    fn predict() {
        let trie = {
            let mut trie = EnteredCommands(CommandCountMap::new());

            trie.enter(&"abzzz ef".to_string());
            trie.enter(&"abzzz ef".to_string());
            trie.enter(&"abzzz ef".to_string());
            trie.enter(&"ab ef cd".to_string());
            trie.enter(&"ab ef cd".to_string());
            trie.enter(&"abxxx".to_string());
            trie.enter(&"yyyyy".to_string());
            trie
        };

        let options: Vec<String> = trie.predict(&"ab".to_string()).collect();
        assert_eq!(options.len(), 3);
        assert_eq!(options[0], "zzz ef");
        assert_eq!(options[1], " ef cd");
        assert_eq!(options[2], "xxx");
    }

    #[test]
    fn serde() {
        // Build a history
        let mut history = History {
            commands: HashMap::new(),
            prediction: Vec::new(),
        };

        history.enter("/home/user", &"ab cd ef".to_string());
        history.enter("/home/user", &"ab cd ef".to_string());
        history.enter("/home/user", &"ab cd ef".to_string());
        history.enter("/home/user/stuff", &"ab cd ef".to_string());
        history.enter("/home/user/stuff", &"ab cd ef".to_string());
        history.enter("/home/user", &"cd ef".to_string());

        let mut buffer = Vec::new();
        history.serialize_into(&mut buffer);

        let maybe_readback = History::deserialize_from(&buffer[..]);
        assert_eq!(maybe_readback.is_ok(), true);
        if let Ok(mut readback) = maybe_readback {
            readback.predict("/home/user", &String::new());
            assert_eq!(*readback.prediction(), ["ab cd ef", "cd ef"]);
            readback.predict("/home/user/stuff", &String::new());
            assert_eq!(*readback.prediction(), vec!["ab cd ef"]);
        }
    }
}
