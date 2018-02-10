
use lmdb::*;
use std::path::Path;

use std::io::{BufReader, BufRead};
use std::fs::File;

type HistoryItem = u64;
const HistoryItemSize: usize = 8;

// The history is stored in an lmdb database. The keys are the commands as strings, the values
// count how many times this command has been executed.
pub struct History {
    env: Environment,
    db: Database,
}

impl History {
    pub fn new(home_dir: &str) -> ::lmdb::Result<Self> {

        let bite_history_path = ::std::path::Path::new(home_dir).join(".bite_history");
        let bash_history_path = Path::new(home_dir).join(".bash_history");

        let env = Environment::new()
            .set_flags(NO_SUB_DIR)
            .open_with_permissions(&bite_history_path, 0o600)?;

        let db = env.create_db(None, DatabaseFlags::empty())?;

        let mut hist = Self { env, db };

        // Import the history from bash if we can
        if let Ok(file) = File::open(bash_history_path) {
            for line in BufReader::new(file).lines() {
                if let Ok(line) = line {
                    hist.import_command(line);
                }
            }
        };

        Ok(hist)
    }

    // If line does not exist, create an entry
    pub fn import_command(&mut self, line: String) {

        // Write some data in a transaction
        if let Ok(mut txn) = self.env.begin_rw_txn() {
            match txn.get(self.db, &line) {
                Err(Error::NotFound) => {
                    let one: HistoryItem = 1;
                    let _ =
                        txn.put(
                            self.db,
                            &line,
                            unsafe { ::std::mem::transmute::<&u64, &[u8; HistoryItemSize]>(&one) },
                            WriteFlags::empty(),
                        );
                }
                _ => {}
            }
            // Commit the changes so they are visible to later transactions
            let _ = txn.commit();
        }
        let _ = self.env.sync(true);
    }
}
