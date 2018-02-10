
use lmdb::*;
use std::path::{Path, PathBuf};

use std::io::{BufReader, BufRead};
use std::fs::File;

// The history is stored in a BTreeSet for deduplication.
pub struct History {
    store_in: PathBuf,
    items: Vec<String>,
}

pub struct HistoryIter {
    ind: usize,
}

// If line does not exist, create an entry
fn load_from_database(path: &Path) -> Option<Vec<String>> {
    if let Ok(env) = Environment::new()
        .set_flags(NO_SUB_DIR)
        .open_with_permissions(&path, 0o600)
    {
        if let Ok(db) = env.create_db(None, DatabaseFlags::empty()) {
            // Read the whole DB into the set
            if let Ok(mut txn) = env.begin_ro_txn() {
                if let Ok(mut cursor) = txn.open_ro_cursor(db) {
                    let mut items = Vec::new();
                    for (k, _) in cursor.iter() {
                        let line = String::from_utf8_lossy(k);
                        items.push(String::from(line));
                    }
                    return Some(items);
                }
            }
        }
    }
    None
}

impl Drop for History {
    fn drop(&mut self) {
        if let Ok(env) = Environment::new()
            .set_flags(NO_SUB_DIR)
            .open_with_permissions(&self.store_in, 0o600)
        {
            // Dummy value to put into lmdb file
            let value: u32 = 1;

            if let Ok(db) = env.create_db(None, DatabaseFlags::empty()) {
                if let Ok(mut txn) = env.begin_rw_txn() {
                    for line in self.items.iter() {
                        let _ = txn.put(
                            db,
                            &line,
                            unsafe { ::std::mem::transmute::<&u32, &[u8; 4]>(&value) },
                            WriteFlags::empty(),
                        );

                    }
                    let _ = txn.commit();
                }
            }
        }
    }
}


impl History {
    // Load the history from the database or the bash history.
    pub fn new(home_dir: &str) -> Self {

        let bite_history_path = ::std::path::Path::new(home_dir).join(".bite_history");
        let bash_history_path = Path::new(home_dir).join(".bash_history");

        // Try to load from database
        if let Some(items) = load_from_database(&bite_history_path) {
            return Self {
                store_in: bite_history_path,
                items,
            };
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

    pub fn add_command(&mut self, line: String) {
        if !self.items.contains(&line) {
            self.items.push(line);
        }
    }

    pub fn iter(&self, reverse: bool) -> HistoryIter {
        let ind = if reverse {
            let l = self.items.len();
            if l == 0 { 0 } else { l - 1 }
        } else {
            0
        };
        HistoryIter { ind }
    }
}

impl HistoryIter {
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
