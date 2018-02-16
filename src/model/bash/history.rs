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

use std::path::{Path, PathBuf};

use std::error::Error;
use std::io::{BufReader, BufRead, Result, Error as IoError, ErrorKind};
use std::fs::File;

use tools::versioned_file;

// The history is stored in a plain vector
pub struct History {
    store_in: PathBuf,
    pub items: Vec<String>,
}

pub struct HistorySeqIter {
    ind: usize,
}

pub struct HistoryPrefIter {
    prefix: String,
    ind: usize,
}

pub struct HistoryInteractiveSearch {
    pub matching_items: Vec<usize>,
    pub ind_item: usize,
}

const HISTORY_FILE_FORMAT: &str = "BITE history V0.1";

const BITE_HISTORY_NAME: &str = ".bite_history";
const BASH_HISTORY_NAME: &str = ".bash_history";

// History file format
// <header>
// <items> -- Serde / bincode format

impl Drop for History {
    // Save to database in correct order
    fn drop(&mut self) {
        if let Err(e) = self.save_to_file() {
            println!(
                "Warning: Could not write to {:?}: {}",
                self.store_in,
                e.description()
            );
        }
    }
}

impl History {
    // Load the history from the database or the bash history.
    pub fn new(home_dir: &str) -> Self {

        let bite_history_path = ::std::path::Path::new(home_dir).join(BITE_HISTORY_NAME);
        let bash_history_path = Path::new(home_dir).join(BASH_HISTORY_NAME);

        // Try to load from database
        match Self::load_from_file(&bite_history_path) {
            Ok(items) => {
                return Self {
                    store_in: bite_history_path,
                    items,
                }
            }
            Err(e) => {
                println!(
                    "Warning: Could not read {:?}: {}",
                    bite_history_path,
                    e.description()
                );
            }
        }

        let mut hist = Self {
            store_in: bite_history_path,
            items: Vec::new(),
        };

        // Import the history from bash if we can
        if let Ok(file) = File::open(bash_history_path) {
            for line in BufReader::new(file).lines() {
                if let Ok(line) = line {
                    hist.add_command(line);
                }
            }
        }
        hist
    }

    // Load the items from a file
    fn load_from_file(path: &Path) -> Result<Vec<String>> {
        let mut file = versioned_file::open(path, HISTORY_FILE_FORMAT)?;
        match ::bincode::deserialize_from(&mut file, ::bincode::Infinite) {
            Ok(items) => Ok(items),
            Err(_) => Err(IoError::new(
                ErrorKind::InvalidData,
                "Can't read history file",
            )),
        }
    }

    fn save_to_file(&self) -> Result<()> {
        let mut file = versioned_file::create(&self.store_in, HISTORY_FILE_FORMAT)?;
        match ::bincode::serialize_into(&mut file, &self.items, ::bincode::Infinite) {
            Ok(_) => Ok(()),
            Err(_) => Err(IoError::new(
                ErrorKind::WriteZero,
                "Can't write history file",
            )),
        }
    }

    pub fn add_command(&mut self, line: String) {
        let mut first = self.items.len();
        for i in 0..self.items.len() {
            if self.items[i] == line {
                first = i;
                break;
            }
        }

        if first != self.items.len() {
            self.items.remove(first);
        }
        self.items.push(line);
    }

    pub fn seq_iter(&self, reverse: bool) -> HistorySeqIter {
        let ind = if reverse {
            let l = self.items.len();
            if l == 0 { 0 } else { l - 1 }
        } else {
            0
        };
        HistorySeqIter { ind }
    }

    pub fn prefix_iter(&self, prefix: &str, reverse: bool) -> HistoryPrefIter {
        let ind = if reverse {
            let l = self.items.len();
            if l == 0 { 0 } else { l - 1 }
        } else {
            0
        };
        HistoryPrefIter {
            prefix: String::from(prefix),
            ind,
        }
    }

    pub fn begin_interactive_search(&self) -> HistoryInteractiveSearch {
        let l = self.items.len();
        HistoryInteractiveSearch {
            matching_items: (0..self.items.len()).collect(),
            ind_item: if l == 0 { 0 } else { l - 1 },
        }
    }
}

impl HistorySeqIter {
    pub fn prev(&mut self, history: &History) -> Option<String> {
        if self.ind < history.items.len() {
            if self.ind > 0 {
                let ind = self.ind;
                self.ind -= 1;
                Some(history.items[ind].clone())
            } else {
                self.ind = history.items.len();
                Some(history.items[0].clone())
            }
        } else {
            None
        }
    }

    pub fn next(&mut self, history: &History) -> Option<String> {
        if self.ind < history.items.len() {
            let ind = self.ind;
            self.ind += 1;
            Some(history.items[ind].clone())
        } else {
            None
        }
    }
}

impl HistoryPrefIter {
    pub fn prev(&mut self, history: &History) -> Option<String> {
        // Loop backwards until we find an item that starts with self.prefix
        while self.ind < history.items.len() {
            if self.ind > 0 {
                let ind = self.ind;
                self.ind -= 1;
                if history.items[ind].starts_with(self.prefix.as_str()) {
                    return Some(history.items[ind].clone());
                }
            } else {
                self.ind = history.items.len();
                if history.items[0].starts_with(self.prefix.as_str()) {
                    return Some(history.items[0].clone());
                }
            }
        }
        None
    }

    pub fn next(&mut self, history: &History) -> Option<String> {
        while self.ind < history.items.len() {
            let ind = self.ind;
            self.ind += 1;
            if history.items[ind].starts_with(self.prefix.as_str()) {
                return Some(history.items[ind].clone());
            }
        }
        None
    }
}

fn abs_diff(a: usize, b: usize) -> usize {
    if a < b { b - a } else { a - b }
}

impl HistoryInteractiveSearch {
    pub fn set_prefix(&mut self, history: &History, pref: &str) {
        // Get index of history item that is selected.
        let current_history_ind = if self.ind_item < self.matching_items.len() {
            self.matching_items[self.ind_item]
        } else {
            0
        };
        // Find the indices of all history items that contain this search string
        self.matching_items = (0..history.items.len())
            .filter(|i| history.items[*i].contains(pref))
            .collect();

        // Find the index into matching_items that is closest to current_history_ind to move the
        // highlight only a litte.
        let mut ind_item = 0;
        let mut dist = history.items.len();
        for i in 0..self.matching_items.len() {
            let history_ind = self.matching_items[i];
            let d = abs_diff(current_history_ind, history_ind);
            if d < dist {
                dist = d;
                ind_item = i;
            }
        }
        self.ind_item = ind_item;
    }

    pub fn prev(&mut self) {
        if self.ind_item > 0 {
            self.ind_item -= 1;
        } else {
            let l = self.matching_items.len();
            self.ind_item = if l == 0 { 0 } else { l - 1 };
        }
    }

    pub fn next(&mut self) {
        if self.ind_item < self.matching_items.len() {
            self.ind_item += 1;
        } else {
            self.ind_item = 0;
        }
    }
}
