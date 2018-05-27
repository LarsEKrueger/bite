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

use std::sync::{Mutex, Condvar, MutexGuard, PoisonError};
use std::os::unix::io::{RawFd, IntoRawFd};
use std::path::Path;
use std::error::Error;

use nix::pty::*;
use nix::unistd::{dup, dup2};
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;

use libc::{c_int, STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO};

/// Line buffer to parse from
lazy_static!{
static ref bite_input_buffer: Mutex<String> = Mutex::new(String::new());
}

/// Condition variable to wait on if bite_input_buffer is empty
lazy_static!{
static ref bite_input_added: Condvar = Condvar::new();
}

/// Bite side interface to send text to bash.
pub fn bite_add_input(text: &str) {
    let _ = bite_input_buffer
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
    // Print prompt once
    if line.len() == 0 {
        #[link(name = "Bash")]
        extern "C" {
            fn print_prompt();
            fn prompt_again();
        }
        unsafe {
            prompt_again();
            // TODO: Send via channel
            print_prompt()
        };
    }
    // Handle spurious wakeups
    while line.len() == 0 {
        line = bite_input_added.wait(line).unwrap();
    }
    line.remove(0) as c_int
}

#[no_mangle]
pub extern "C" fn bite_ungetch(ch: c_int) -> c_int {
    let _ = bite_input_buffer.lock().map(|mut line| {
        line.insert(0, (ch & 255) as u8 as char)
    });
    ch
}

/// Convert an error to a string.
fn as_description<T>(err: T) -> String
where
    T: ::std::error::Error,
{
    err.description().to_string()
}

#[allow(dead_code)]
struct PtsHandles {
    /// Stdin PTS master (bite side)
    stdin_m: RawFd,
    /// Stdin PTS slave (bash side)
    stdin_s: RawFd,
    /// Stdin backup.
    ///
    /// Do we ever need that?
    stdin_b: RawFd,
    /// Stdout PTS master (bite side)
    stdout_m: RawFd,
    /// Stdout PTS slave (bash side)
    stdout_s: RawFd,
    /// Stdout backup. This will print to the terminal that started us.
    stdout_b: RawFd,
    /// Stdout PTS master (bite side)
    stderr_m: RawFd,
    /// Stdout PTS slave (bash side)
    stderr_s: RawFd,
    /// Stderr backup. This will print to the terminal that started us.
    stderr_b: RawFd,
}

/// Create a single pts master/slave pair.
///
/// Returns: (master, slave)
fn create_terminal() -> Result<(RawFd, RawFd), String> {
    let ptsm = posix_openpt(OFlag::O_EXCL).map_err(as_description)?;
    grantpt(&ptsm).map_err(as_description)?;
    unlockpt(&ptsm).map_err(as_description)?;
    let sname = unsafe { ptsname(&ptsm).map_err(as_description) }?;
    let sfd = open(Path::new(&sname), OFlag::O_RDWR, Mode::empty())
        .map_err(as_description)?;

    Ok((ptsm.into_raw_fd(), sfd))
}

/// Reassign stdin, stdout, stderr to pseudo terminals.
///
/// As bash uses the variables stdin, stderr, stdout as well as the raw handle numbers 0, 1, 2 as
/// integer literal, we have to overwrite both. We also have to reattach rust's stdin, stdout,
/// stderr to the new PTSM handles.
///
/// If this fails, we fail with an error message.
fn create_terminals() -> Result<PtsHandles, String> {
    // Create the pts pairs
    let (stdin_m, stdin_s) = create_terminal()?;
    let (stdout_m, stdout_s) = create_terminal()?;
    let (stderr_m, stderr_s) = create_terminal()?;

    // Backup handles 0,1,2
    let save_stdin = dup(STDIN_FILENO).map_err(as_description)?;
    let save_stdout = dup(STDOUT_FILENO).map_err(as_description)?;
    let save_stderr = dup(STDERR_FILENO).map_err(as_description)?;

    // Rust uses the handles 0, 1, 2. See rust.git/src/libstd/sys/unix/stdio.rs. Thus reassigning
    // the handles to the slaves redirects all errors to us. If we really want to print something,
    // we need to use the backup handles.

    // We close 0, 1, 2 and dup the slaves onto them. If something goes wrong, we restore all of
    // them and report the error.
    let stdin_ok = dup2(stdin_s, STDIN_FILENO);
    let stdout_ok = dup2(stdout_s, STDOUT_FILENO);
    let stderr_ok = dup2(stderr_s, STDERR_FILENO);

    if stdin_ok.is_err() || stdout_ok.is_err() || stderr_ok.is_err() {
        // If something went wrong here, we'll never know.
        let _ = dup2(save_stdin, STDIN_FILENO);
        let _ = dup2(save_stdout, STDOUT_FILENO);
        let _ = dup2(save_stderr, STDERR_FILENO);

        // Build a somewhat useful error message.
        let mut error = String::new();
        if let Err(e) = stdin_ok {
            error.push_str(e.description());
        }
        if let Err(e) = stdout_ok {
            if !error.is_empty() {
                error.push_str("\n");
            }
            error.push_str(e.description());
        }
        if let Err(e) = stderr_ok {
            if !error.is_empty() {
                error.push_str("\n");
            }
            error.push_str(e.description());
        }
        return Err(error);
    }

    // Make C use the pts for their stdin, stdout, sterr FILE structs.
    {
        #[link(name = "Bash")]
        extern "C" {
            fn bash_use_pts(stdin: c_int, out: c_int, err: c_int);
        }
        unsafe {
            bash_use_pts(stdin_s, stdout_s, stderr_s);
        }
    }

    // We should be good to go. Keep the handles somewhere safe
    Ok(PtsHandles {
        stdin_m,
        stdin_s,
        stdin_b: save_stdin,
        stdout_m,
        stdout_s,
        stdout_b: save_stdout,
        stderr_m,
        stderr_s,
        stderr_b: save_stderr,
    })
}

static mut pts_handles: Option<PtsHandles> = None;

/// Start bash as a thread. Do not call more than once.
///
/// As bash is full of global variables and longjmps, we need to run its main function as a whole
/// in a thread.
pub fn start() -> Result<(), String> {
    #[link(name = "Bash")]
    extern "C" {
        fn bash_main();
    }

    unsafe {
        pts_handles = Some(create_terminals()?);

        // If we got here, we can print stuff through the backup handles.
        use std::os::unix::io::FromRawFd;
        use std::io::Write;
        pts_handles.as_ref().map(|h| {
            ::std::fs::File::from_raw_fd(h.stdout_b).write(
                b"bite: Pseudo terminals correctly set up.\n",
            )
        });
    }

    ::std::thread::spawn(move || unsafe { bash_main() });

    Ok(())
}
