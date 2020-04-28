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
use std::io::{Read, Write};

use tools::versioned_file;

/// Map a String key to the number of times it was entered
type KeyCountMap = qptrie::Trie<String, u32>;

/// Zero-cost abstraction around the trie to add some operations
#[derive(Debug)]
struct Predictor(KeyCountMap);

/// History of all entered commands, sorted by folder and previous command.
///
/// In order to allow commands to be found regardless where and after which command they were
/// entered, the following qptries are kept:
///
/// * directory, previous command, command
/// * directory, command
/// * command
///
/// In order to distinguish them, the individual parts of the keys are separated by \u{0}.
///
/// The history is in charge of making predictions of the next command given the start of the
/// current one. As the prediction will be read on every render, it is cached.
#[derive(Debug)]
pub struct History {
    /// Count frequency of commands, ordered by directory, then last command
    dir_prev_cmd: Predictor,

    /// Count frequency of commands, ordered by directory
    dir_cmd: Predictor,

    /// Count frequency of commands
    cmd: Predictor,

    /// Last dir for which a command was entered
    last_dir: String,

    /// Last command entered
    last_cmd: String,

    /// Last prediction, most frequent first
    pub prediction: Vec<String>,
}

const HISTORY_FORMAT_100: &str = "BITE HISTORY 1.0.0";

impl History {
    /// Create empty history
    pub fn new() -> Self {
        Self {
            dir_prev_cmd: Predictor::new(),
            dir_cmd: Predictor::new(),
            cmd: Predictor::new(),
            last_dir: String::new(),
            last_cmd: String::new(),
            prediction: Vec::new(),
        }
    }

    /// Load the history from the given file.
    pub fn load(file_name: &str) -> Result<History, String> {
        let file_handle = versioned_file::open(file_name, HISTORY_FORMAT_100)
            .map_err(|e| e.description().to_string())?;
        History::deserialize_from(file_handle)
    }

    /// Save the history
    pub fn save(&self, file_name: &str) -> Result<(), String> {
        let file_handle = versioned_file::create(file_name, HISTORY_FORMAT_100)
            .map_err(|e| e.description().to_string())?;
        self.serialize_into(file_handle);
        Ok(())
    }

    /// Enter a command in the history
    pub fn enter(&mut self, dir: &str, cmd: &String) {
        // Prepare the last command of a new directory
        if self.last_dir != dir {
            self.last_dir.clear();
            self.last_dir.push_str(dir);
            self.last_cmd.clear();
        }

        // Update dir_prev_cmd
        let mut key = self.last_dir.clone();
        key.push_str("\0");
        key.push_str(&self.last_cmd);
        key.push_str("\0");
        key.push_str(&cmd);
        self.dir_prev_cmd.enter(&key);

        // Update dir_cmd, reuse key to save allocations
        key.clear();
        key.push_str(&self.last_dir);
        key.push_str("\0");
        key.push_str(&cmd);
        self.dir_cmd.enter(&key);

        // Update cmd
        self.cmd.enter(cmd);

        // Remember the last command
        self.last_cmd.clear();
        self.last_cmd.push_str(cmd);
    }

    /// Compute a new prediction
    pub fn predict(&mut self, dir: &str, start: &String) {
        self.prediction.clear();
        // most specific search first
        if dir == self.last_dir {
            let mut key = String::new();
            key.push_str(dir);
            key.push_str("\0");
            key.push_str(&self.last_cmd);
            key.push_str("\0");
            key.push_str(start);
            for p in self.dir_prev_cmd.predict(&key) {
                self.prediction.push(p);
            }
            if !self.prediction.is_empty() {
                return;
            }
            // Search without previous command, reuse key to reduce allocations
            key.clear();
            key.push_str(dir);
            key.push_str("\0");
            key.push_str(start);
            for p in self.dir_cmd.predict(&key) {
                self.prediction.push(p);
            }
            if !self.prediction.is_empty() {
                return;
            }
        }
        // Search global list
        for p in self.cmd.predict(start) {
            self.prediction.push(p);
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
        let hm: HashMap<String, u32> =
            bincode::deserialize_from(reader).map_err(|e| e.description().to_string())?;

        let mut dir_prev_cmd = Predictor::new();
        let mut dir_cmd = Predictor::new();
        let mut cmd = Predictor::new();
        for (c, n) in hm.iter() {
            let (pred, key) = if c.starts_with("\0\0") {
                (&mut cmd, &c[2..])
            } else if c.starts_with("\0") {
                (&mut dir_cmd, &c[1..])
            } else {
                (&mut dir_prev_cmd, &c[..])
            };
            let _ = pred.0.insert(key.to_string(), *n);
        }
        Ok(History {
            dir_prev_cmd,
            dir_cmd,
            cmd,
            last_dir: String::new(),
            last_cmd: String::new(),
            prediction: Vec::new(),
        })
    }

    /// As radix_trie does not support serde, serialize a HashMap. Use \u{0} prefixes to
    /// distignuish the entries
    fn serialize_into<W>(&self, writer: W)
    where
        W: Write,
    {
        let mut hm: HashMap<String, u32> = HashMap::new();

        for (c, n) in self.dir_prev_cmd.0.prefix_iter(&String::new()) {
            let _ = hm.insert(c.to_string(), *n);
        }
        for (c, n) in self.dir_cmd.0.prefix_iter(&String::new()) {
            let mut key = String::new();
            key.push_str("\0");
            key.push_str(c);
            let _ = hm.insert(key, *n);
        }
        for (c, n) in self.cmd.0.prefix_iter(&String::new()) {
            let mut key = String::new();
            key.push_str("\0\0");
            key.push_str(c);
            let _ = hm.insert(key, *n);
        }
        let _ = bincode::serialize_into(writer, &hm);
    }
}

impl Predictor {
    fn new() -> Self {
        Self(KeyCountMap::new())
    }

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
        let mut trie = Predictor::new();

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
            let mut trie = Predictor::new();

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
        let mut history = History::new();
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
        if let Ok(readback) = maybe_readback {
            for ((k_gt, v_gt), (k, v)) in history
                .dir_prev_cmd
                .0
                .prefix_iter(&String::new())
                .zip(readback.dir_prev_cmd.0.prefix_iter(&String::new()))
            {
                assert_eq!(k_gt, k);
                assert_eq!(v_gt, v);
            }
            for ((k_gt, v_gt), (k, v)) in history
                .dir_cmd
                .0
                .prefix_iter(&String::new())
                .zip(readback.dir_cmd.0.prefix_iter(&String::new()))
            {
                assert_eq!(k_gt, k);
                assert_eq!(v_gt, v);
            }
            for ((k_gt, v_gt), (k, v)) in history
                .cmd
                .0
                .prefix_iter(&String::new())
                .zip(readback.cmd.0.prefix_iter(&String::new()))
            {
                assert_eq!(k_gt, k);
                assert_eq!(v_gt, v);
            }
        }
    }

    #[test]
    fn zero_sep() {
        let mut ccm = KeyCountMap::new();
        assert_eq!(ccm.insert("abc\0def\0xyz".to_string(), 1), true);
        assert_eq!(ccm.insert("abc\0def\0001".to_string(), 2), true);
        assert_eq!(ccm.insert("abc\0def\0002".to_string(), 3), true);
        assert_eq!(ccm.insert("def\0abc\0xyz".to_string(), 3), true);

        let start = String::from("ab");
        let mut pref_abc = ccm.prefix_iter(&start).sorted_by(|a, b| Ord::cmp(a.0, b.0));
        assert_eq!(
            pref_abc.next(),
            Some((&("abc\0def\0001".to_string()), &2u32))
        );
        assert_eq!(
            pref_abc.next(),
            Some((&("abc\0def\0002".to_string()), &3u32))
        );
        assert_eq!(
            pref_abc.next(),
            Some((&("abc\0def\0xyz".to_string()), &1u32))
        );
        assert_eq!(pref_abc.next(), None);
    }
}
