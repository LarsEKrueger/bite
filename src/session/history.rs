
use lmdb::*;
use std::path::{Path, PathBuf};

use std::io::{BufReader, BufRead};
use std::fs::File;

// The history is stored in a BTreeSet for deduplication.
pub struct History {
    store_in: PathBuf,
    items: Vec<String>,
}

pub struct HistorySeqIter {
    ind: usize,
}

pub struct HistoryPrefIter {
    prefix: String,
    ind: usize,
}

type HistoryDbKey = u64;
const HistoryDbKeySize: usize = 8;

const DB_COUNTER_NAME: &str = "counter";
const DB_HISTORY_NAME: &str = "history";

const DB_COUNTER_KEY: &[u8; 7] = b"counter";

/* Database design:
 * - two databases
 *   - counter: holds the number of items in "history" as u64
 *   - history: holds the items, sorted by u64 key
 * - load: Load all values from "history" in order
 * - save
 *   - For each line in memory, check if it exists.
 *   - If it doesn't, add it.
 *   - If it does, delete the entry and add a new one at the end.
 *   - At the end, recreate the indices by a linear scan.
 */

fn read_counter<'txn,T> (db_count:Database,txn:&'txn mut T) -> Result<HistoryDbKey>
where T:Transaction 
{
    let cnt_res = txn.get(db_count, DB_COUNTER_KEY);
    match cnt_res {
        Err(Error::NotFound) => Ok(0),
        Ok(cnt_bytes) => 
            if cnt_bytes.len() == HistoryDbKeySize {
                Ok(unsafe { *(&cnt_bytes[0] as *const u8 as *const HistoryDbKey) })
            } else {
                Ok(0)
            }
        Err(e) => Err(e),
    }
}

// If line does not exist, create an entry
fn load_from_database(path: &Path) -> Result<Vec<String>> {
    let env = Environment::new()
        .set_flags(NO_SUB_DIR)
        .set_max_dbs(2)
        .open_with_permissions(&path, 0o600)?;
    let db_hist = env.create_db(Some(DB_HISTORY_NAME), DatabaseFlags::empty())?;
    let db_count = env.create_db(Some(DB_COUNTER_NAME), DatabaseFlags::empty())?;
    let mut txn = env.begin_ro_txn()?;

    let counter = read_counter(db_count,&mut txn)?;

    let mut items = Vec::new();

    for k in 0..counter {
        if let Ok(v) = txn.get(
            db_hist,
            unsafe {
                ::std::mem::transmute::<&u64, &[u8; HistoryDbKeySize]>(&k)
            }) {
            let line = String::from_utf8_lossy(v);
            items.push(String::from(line));
        }
    }

    if items.len() == 0 { Err(Error::NotFound) } else { Ok(items) }
}

fn save_to_database(path: &Path, items: &Vec<String>) -> ::lmdb::Result<()> {
    let env = Environment::new()
        .set_flags(NO_SUB_DIR)
        .set_max_dbs(2)
        .open_with_permissions(path, 0o600)?;
    let db_hist = env.create_db(Some(DB_HISTORY_NAME), DatabaseFlags::empty())?;
    let db_count = env.create_db(Some(DB_COUNTER_NAME), DatabaseFlags::empty())?;
    let mut txn = env.begin_rw_txn()?;

    // Get the counter
    let mut counter: HistoryDbKey = read_counter(db_count,&mut txn)?;

    // Iterate over the items to bubble the known ones to the end and to add the unknown ones.
    for line in items.iter() {
        // Delete all items that have a value of line
        {
            let mut first = None;
            {
                let mut ro_cursor = txn.open_ro_cursor(db_hist)?;
                // Check if there are no items
                if let Ok(_) = ro_cursor.get(None,None, 0 /*MDB_FIRST*/) {
                    for (k, v) in ro_cursor.iter_start() {
                        let db_line = String::from_utf8_lossy(v);
                        if db_line == line.as_str() {
                            first = Some(k.clone());
                            break;
                        }
                    }
                }
            }
            if let Some(k) = first {
                txn.del(db_hist, &k, None)?;
            }
        }
        // Add line with key=counter
        txn.put(
            db_hist,
            unsafe {
                ::std::mem::transmute::<&u64, &[u8; HistoryDbKeySize]>(&counter)
            },
            &line,
            WriteFlags::empty(),
        )?;
        counter += 1;
    }

    // TODO: Now everything is in correct order, re-write them beginning at 0.

    // Write back the counter
    txn.put(
        db_count,
        DB_COUNTER_KEY,
        unsafe {
            ::std::mem::transmute::<&u64, &[u8; HistoryDbKeySize]>(&counter)
        },
        WriteFlags::empty(),
    )?;

    txn.commit()?;

    Ok(())
}

impl Drop for History {
    // Save to database in correct order
    fn drop(&mut self) {
        let _e = save_to_database(&self.store_in, &self.items);
    }
}


impl History {
    // Load the history from the database or the bash history.
    pub fn new(home_dir: &str) -> Self {

        let bite_history_path = ::std::path::Path::new(home_dir).join(".bite_history");
        let bash_history_path = Path::new(home_dir).join(".bash_history");

        // Try to load from database
        if let Ok(items) = load_from_database(&bite_history_path) {
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

    pub fn prefix_iter(&self, prefix:&str,reverse:bool) -> HistoryPrefIter {
        let ind = if reverse {
            let l = self.items.len();
            if l == 0 { 0 } else { l - 1 }
        } else {
            0
        };
        HistoryPrefIter { prefix:String::from(prefix), ind }
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
                if history.items[ind].starts_with( self.prefix.as_str()) {
                return Some(history.items[ind].clone())
                }
            } else {
                self.ind = history.items.len();
                if history.items[0].starts_with( self.prefix.as_str()) {
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
                if history.items[ind].starts_with( self.prefix.as_str()) {
            return Some(history.items[ind].clone());
                }
        }
        None
    }
}
