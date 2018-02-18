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

pub enum HistorySearchMode {
    Browse,
    Prefix(String),
    Contained(String),
}

pub struct HistorySearchCursor {
    pub matching_items: Vec<usize>,
    pub item_ind: usize,
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
    fn matches(&self, other: &str) -> bool {
        match self {
            &HistorySearchMode::Browse => true,
            &HistorySearchMode::Prefix(ref pref) => other.starts_with(pref.as_str()),
            &HistorySearchMode::Contained(ref cont) => other.contains(cont.as_str()),
        }
    }
}

impl HistorySearchCursor {
    pub fn prev1(&mut self) {
        if self.item_ind < self.matching_items.len() {
            if self.item_ind > 0 {
                self.item_ind -= 1;
            } else {
                self.item_ind = self.matching_items.len() - 1;
            }
        }
    }

    pub fn next1(&mut self) {
        if self.item_ind + 1 < self.matching_items.len() {
            self.item_ind += 1;
        } else {
            self.item_ind = 0;
        }
    }

    pub fn prev(&mut self, n: usize) {
        if self.item_ind < self.matching_items.len() {
            if self.item_ind > n {
                self.item_ind -= n;
            } else {
                self.item_ind = 0;
            }
        }
    }

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
