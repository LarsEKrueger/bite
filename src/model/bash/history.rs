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

/// List of all commands so far
pub struct History {
    /// Path to store the history when this struct is dropped.
    store_in: PathBuf,

    /// List of commands.
    pub items: Vec<String>,
}

/// What to search for in the history.
pub enum HistorySearchMode {
    /// Everything.
    Browse,
    /// Lines that start with the given string.
    Prefix(String),
    /// Lines that contain the given string.
    Contained(String),
}

/// Search result
pub struct HistorySearchCursor {
    /// Index into History::items that matched the search request.
    pub matching_items: Vec<usize>,

    /// Currently selected item.
    pub item_ind: usize,
}

const HISTORY_FILE_FORMAT: &str = "BITE history V0.1";

const BITE_HISTORY_NAME: &str = ".bite_history";
const BASH_HISTORY_NAME: &str = ".bash_history";

impl Drop for History {
    /// Save history to file when the object is deallocated.
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
    /// Load the history from the database or the bash history.
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

    /// Loads the items from a file.
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

    /// Saves the items to a file.
    ///
    /// TODO: merge with already exisiting file
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

    /// Adds a command string to the history.
    ///
    /// Ensures that the command is unique as not to pollute the history like bash does.
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

    /// Searches the history for matching lines and returns a list of indices into History::items.
    ///
    /// # Safety
    ///
    /// The user must ensure that the search is dropped when the history changes.
    pub fn search(&self, mode: HistorySearchMode, reverse: bool) -> HistorySearchCursor {
        let matching_items: Vec<usize> = self.items
            .iter()
            .zip(0..)
            .filter(|&(it, _)| mode.matches(it))
            .map(|x| x.1)
            .collect();

        let item_ind = if reverse {
            let l = matching_items.len();
            if l == 0 { 0 } else { l - 1 }
        } else {
            0
        };
        HistorySearchCursor {
            matching_items,
            item_ind,
        }
    }
}


impl HistorySearchMode {
    /// Check if the given string matches this search mode
    pub fn matches(&self, other: &str) -> bool {
        match self {
            &HistorySearchMode::Browse => true,
            &HistorySearchMode::Prefix(ref pref) => other.starts_with(pref.as_str()),
            &HistorySearchMode::Contained(ref cont) => other.contains(cont.as_str()),
        }
    }
}

impl HistorySearchCursor {
    /// Go to the previous item and wrap around if necessary.
    pub fn prev1(&mut self) {
        if self.item_ind < self.matching_items.len() {
            if self.item_ind > 0 {
                self.item_ind -= 1;
            } else {
                self.item_ind = self.matching_items.len() - 1;
            }
        }
    }

    /// Go to the next item and wrap around if necessary.
    pub fn next1(&mut self) {
        if self.item_ind + 1 < self.matching_items.len() {
            self.item_ind += 1;
        } else {
            self.item_ind = 0;
        }
    }

    /// Go *n* items back in the search and stop at the beginning.
    pub fn prev(&mut self, n: usize) {
        if self.item_ind < self.matching_items.len() {
            if self.item_ind > n {
                self.item_ind -= n;
            } else {
                self.item_ind = 0;
            }
        }
    }

    /// Go *n* items forward in the search and stop at the end.
    pub fn next(&mut self, n: usize) {
        if self.item_ind + n < self.matching_items.len() {
            self.item_ind += n;
        } else {
            self.item_ind = if self.matching_items.len() > 0 {
                self.matching_items.len() - 1
            } else {
                0
            };
        }
    }
}
