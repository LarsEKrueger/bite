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

//! The interface to the underlying bash interpreter.
//!
//! Be aware that bash is full of global variables and must not be started more than once.

use libc::{c_char, c_int};
use std::sync::{Arc, Mutex, Condvar, MutexGuard, PoisonError};

/// Line buffer to parse from
lazy_static!{
static ref bite_input_buffer: Mutex<String> = Mutex::new(String::new());
}

/// Condition variable to wait on if bite_input_buffer is empty
lazy_static!{
static ref bite_input_added: Condvar = Condvar::new();
}

pub fn bite_add_input(text: &str) {
    bite_input_buffer
        .lock()
        .map(|mut line| {
            line.push_str(text);
            Ok(())
        })
        .and_then(|_: Result<(), PoisonError<MutexGuard<String>>>| {
            bite_input_added.notify_all();
            Ok(())
        });
}

#[no_mangle]
pub extern "C" fn bite_getch() -> c_int {
    let mut line = bite_input_buffer.lock().unwrap();
    while line.len() == 0 {
        line = {
            bite_input_added.wait(line).unwrap()
        };
    }
    line.remove(0) as c_int
}

#[no_mangle]
pub extern "C" fn bite_ungetch(ch: c_int) -> c_int {
    bite_input_buffer.lock().map(|mut line| {
        line.insert(0, (ch & 255) as u8 as char)
    });
    ch
}

/// Start bash as a thread. Do not call more than once.
///
/// As bash is full of global variables and longjmps, we need to run its main function as a whole
/// in a thread.
pub fn start() {
    #[link(name = "Bash")]
    extern "C" {
        fn bash_main();
    }

    ::std::thread::spawn(move || unsafe { bash_main() });
}
