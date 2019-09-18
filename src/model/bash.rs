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

use std::error::Error;
use std::ffi::CStr;
use std::fs::File;
use std::io::Write;
use std::mem;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
use std::path::Path;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Barrier;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::spawn;
use termios::os::target::*;
use termios::*;

use nix::fcntl::{open, OFlag};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt, SessionId};
use nix::sys::select::{select, FdSet};
use nix::sys::signal::{kill, Signal};
use nix::sys::stat::Mode;
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::{close, dup, dup2, read, write, Pid};

use libc::{c_char, c_int, c_uchar, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};

/// Line buffer to parse from
lazy_static! {
    static ref bite_input_buffer: Mutex<String> = Mutex::new(String::new());
}

/// Condition variable to wait on if bite_input_buffer is empty
lazy_static! {
    static ref bite_input_added: Condvar = Condvar::new();
}

/// Marker that bash is waiting for input
static bash_is_waiting: AtomicBool = AtomicBool::new(false);

/// Check if bash is waiting for input
pub fn is_bash_waiting() -> bool {
    unsafe {
        bash_is_waiting.load(Ordering::SeqCst)
            && bash_out_blocked.load(Ordering::SeqCst)
            && bash_err_blocked.load(Ordering::SeqCst)
    }
}

/// Bite side interface to send text to bash.
pub fn bash_add_input(text: &str) {
    let _ = bite_input_buffer
        .lock()
        .map(|mut line| {
            line.push_str(text);
            info!("bash_add_input: {} remaining", line.len());
            Ok(())
        })
        .and_then(|_: Result<(), PoisonError<MutexGuard<String>>>| {
            bite_input_added.notify_all();
            Ok(())
        });
}

static mut bash_sender: Option<Mutex<Sender<BashOutput>>> = None;

#[no_mangle]
pub extern "C" fn bite_print_prompt() {
    #[link(name = "Bash")]
    extern "C" {
        static current_decoded_prompt: *const c_char;
    }
    unsafe {
        // Send via channel
        if let Some(ref mut sender) = bash_sender {
            let prompt = CStr::from_ptr(current_decoded_prompt);

            // Remove "\[" and "\]"
            let prompt = prompt.to_string_lossy().to_owned();
            let prompt = prompt.replace("\\[", "").replace("\\]", "");

            let prompt = Vec::from(prompt.as_bytes());
            let _ = sender.lock().unwrap().send(BashOutput::Prompt(prompt));
        }
    };
}

#[no_mangle]
pub extern "C" fn bite_set_exit_status(exit_status: c_int) {
    unsafe {
        if let Some(ref mut sender) = bash_sender {
            use std::os::unix::process::ExitStatusExt;
            let _ = sender
                .lock()
                .unwrap()
                .send(BashOutput::Terminated(ExitStatusExt::from_raw(exit_status)));
        }
    }
}

#[no_mangle]
pub extern "C" fn bite_getch() -> c_int {
    let mut line = bite_input_buffer.lock().unwrap();
    bash_is_waiting.store(true, Ordering::SeqCst);
    // Handle spurious wakeups
    while line.len() == 0 {
        line = bite_input_added.wait(line).unwrap();
    }
    bash_is_waiting.store(false, Ordering::SeqCst);
    info!("bite_getch: {} remaining", line.len());
    line.remove(0) as c_int
}

#[no_mangle]
pub extern "C" fn bite_ungetch(ch: c_int) -> c_int {
    let _ = bite_input_buffer
        .lock()
        .map(|mut line| line.insert(0, (ch & 255) as u8 as char));
    ch
}

/// Convert an error to a string.
fn as_description<T>(err: T) -> String
where
    T: ::std::error::Error,
{
    err.description().to_string()
}

struct PtsHandles {
    /// Stdin PTS master (bite side)
    prg_stdin: RawFd,
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
    bite_stdout: Arc<Mutex<File>>,
    /// Stdout PTS master (bite side)
    stderr_m: RawFd,
    /// Stdout PTS slave (bash side)
    stderr_s: RawFd,
    /// Stderr backup. This will print to the terminal that started us.
    _bite_stderr: Arc<Mutex<File>>,
}

/// Create a single pts master/slave pair.
///
/// Returns: (master, slave)
fn create_terminal(termios: Termios) -> Result<(RawFd, RawFd), String> {
    let ptsm =
        posix_openpt(OFlag::O_RDWR | OFlag::O_EXCL | OFlag::O_NONBLOCK).map_err(as_description)?;
    grantpt(&ptsm).map_err(as_description)?;
    unlockpt(&ptsm).map_err(as_description)?;
    let sname = unsafe { ptsname(&ptsm).map_err(as_description) }?;
    let sfd = open(Path::new(&sname), OFlag::O_RDWR, Mode::empty()).map_err(as_description)?;

    let ptsm = ptsm.into_raw_fd();
    tcsetattr(sfd, TCSANOW, &termios).map_err(as_description)?;
    tcflush(sfd, TCOFLUSH).map_err(as_description)?;

    Ok((ptsm, sfd))
}

/// Compute the matching control character of a letter
const fn control(x: char) -> c_uchar {
    ((x as u32) & 0x1f) as c_uchar
}

/// Define a fallback termios value
fn fallback_termios() -> Termios {
    let mut termios: Termios = unsafe { mem::zeroed() };
    termios.c_iflag = ICRNL | IXON;
    termios.c_oflag = TAB3 | ONLCR | OPOST;
    fix_termios_cc(&mut termios);
    termios
}

fn fix_termios_cc(termios: &mut Termios) {
    termios.c_cc[VINTR] = control('C');
    termios.c_cc[VQUIT] = control('\\');
    termios.c_cc[VERASE] = 0o177;
    termios.c_cc[VKILL] = control('U');
    termios.c_cc[VEOF] = control('D');
    termios.c_cc[VEOL] = 0xff;
    termios.c_cc[VSTART] = control('Q');
    termios.c_cc[VSTOP] = control('S');
    termios.c_cc[VSUSP] = control('Z');
    termios.c_cc[VREPRINT] = control('R');
    termios.c_cc[VDISCARD] = control('O');
    termios.c_cc[VWERASE] = control('W');
    termios.c_cc[VLNEXT] = control('V');
    termios.c_cc[VEOL2] = 0;
}

/// Create a default termios struct either from /dev/tty or from built-in values.
fn default_termios() -> Termios {
    if let Ok(ttyfd) = open(Path::new("/dev/tty"), OFlag::O_RDWR, Mode::empty()) {
        info!("Could open /dev/tty for termios");
        let termios = Termios::from_fd(ttyfd);
        let _ = close(ttyfd);
        if let Ok(termios) = termios {
            info!("Got termios from /dev/tty: {:?}", termios);
            termios
        } else {
            warn!("Could not get termios from /dev/tty");
            fallback_termios()
        }
    } else {
        warn!("Could not open /dev/tty for termios");
        fallback_termios()
    }
}

/// Reassign stdin, stdout, stderr to pseudo terminals.
///
/// As bash uses the variables stdin, stderr, stdout as well as the raw handle numbers 0, 1, 2 as
/// integer literal, we have to overwrite both. We also have to reattach rust's stdin, stdout,
/// stderr to the new PTSM handles.
///
/// If this fails, we fail with an error message.
fn create_terminals() -> Result<PtsHandles, String> {
    // Create an initial termios struct
    let mut termios = default_termios();

    // Fix termios struct
    termios.c_iflag &= !(INLCR | IGNCR);
    termios.c_iflag |= ICRNL;
    termios.c_iflag |= IUTF8;
    termios.c_oflag |= OPOST | ONLCR;
    termios.c_cflag &= !CBAUD;
    termios.c_lflag |= ISIG | ICANON | ECHO | ECHOE | ECHOK;
    termios.c_lflag |= ECHOKE | IEXTEN;
    termios.c_lflag |= ECHOCTL | IEXTEN;
    termios.c_cflag |= CS8;
    termios.c_cflag |= B38400;
    trace!(
        "ttySetAttr: c_iflag={:x}, c_oflag={:x}, c_cflag={:x}, c_lflag={:x}\n",
        termios.c_iflag,
        termios.c_oflag,
        termios.c_cflag,
        termios.c_lflag
    );
    // c_iflag=4500, c_oflag=5, c_cflag=bf, c_lflag=8a3b
    fix_termios_cc(&mut termios);
    cfmakeraw(&mut termios);

    // Create the pts pairs
    let (stdin_m, stdin_s) = create_terminal(termios)?;
    let (stdout_m, stdout_s) = create_terminal(termios)?;
    let (stderr_m, stderr_s) = create_terminal(termios)?;

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
    unsafe {
        Ok(PtsHandles {
            prg_stdin: stdin_m,
            stdin_s,
            stdin_b: save_stdin,
            stdout_m,
            stdout_s,
            bite_stdout: Arc::new(Mutex::new(::std::fs::File::from_raw_fd(save_stdout))),
            stderr_m,
            stderr_s,
            _bite_stderr: Arc::new(Mutex::new(::std::fs::File::from_raw_fd(save_stderr))),
        })
    }
}

static mut pts_handles: Option<PtsHandles> = None;

pub fn bite_write_output(line: &str) {
    unsafe {
        pts_handles
            .as_ref()
            .map(|h| h.bite_stdout.lock().map(|mut f| f.write(line.as_bytes())))
    };
}

/// Send input to the foreground program running in bash.
pub fn program_add_input(line: &str) {
    unsafe {
        pts_handles.as_ref().map(|h| {
            write(h.prg_stdin, line.as_bytes()).map_err(|e| {
                h.bite_stdout
                    .lock()
                    .map(|mut f| write!(f, "write to {}: {}\n", h.prg_stdin, e.description()))
            })
        })
    };
}

/// Kill the last program we created.
///
/// This should always be the foreground program. Also, race conditions do not really matter here.
pub fn bash_kill_last() {
    #[link(name = "Bash")]
    extern "C" {
        static last_made_pid: SessionId;
    }
    unsafe {
        if last_made_pid != -1 {
            let _ = kill(Pid::from_raw(last_made_pid), Signal::SIGQUIT);
        }
    }
}

/// Data to be sent to the receiver of the program's output.
pub enum BashOutput {
    /// A line was read from stdout.
    FromOutput(Vec<u8>),

    /// A line was read from stderr.
    FromError(Vec<u8>),

    /// The program terminated.
    Terminated(ExitStatus),

    /// Bash wanted to issue a prompt.
    Prompt(Vec<u8>),
}

static mut read_lines_quit: AtomicBool = AtomicBool::new(false);

static mut bash_out_blocked: AtomicBool = AtomicBool::new(false);
static mut bash_err_blocked: AtomicBool = AtomicBool::new(false);

pub fn read_lines_running() -> bool {
    !unsafe { read_lines_quit.load(Ordering::Relaxed) }
}

/// Read from a RawFd until fails and send to the channel with the constructor
fn read_data(
    fd: RawFd,
    sender: Sender<BashOutput>,
    fd_blocked: &mut AtomicBool,
    construct: &Fn(Vec<u8>) -> BashOutput,
    _error: Arc<Mutex<File>>,
) {
    // Is it time to quit the threat?
    while read_lines_running() {
        // If there is input, read it.
        let mut rdfs = FdSet::new();
        rdfs.insert(fd);
        let mut timeout = TimeVal::milliseconds(20);
        let data_available = match select(None, Some(&mut rdfs), None, None, Some(&mut timeout)) {
            Ok(0) | Err(_) => false,
            Ok(_) => true,
        };

        fd_blocked.store(!data_available, Ordering::SeqCst);
        if data_available {
            let mut buffer = [0; 4096];
            if let Ok(len) = read(fd, &mut buffer) {
                let v = Vec::from(&buffer[0..len]);
                let _ = sender.send(construct(v));
            } else {
                use std::fmt::Write;
                let mut s = String::new();
                let _ = write!(
                    s,
                    "Read failed from handle {} (data_available={:?})\nExiting thread.\n",
                    fd, data_available
                );
                bite_write_output(s.as_str());
                // There was some serious error reading from bash, so drop everything and leave.
                unsafe {
                    read_lines_quit.store(true, Ordering::Relaxed);
                }
            }
        }
    }
}

/// Start bash as a thread. Do not call more than once.
///
/// As bash is full of global variables and longjmps, we need to run its main function as a whole
/// in a thread.
pub fn start() -> Result<(Receiver<BashOutput>, Arc<Barrier>, Arc<Barrier>), String> {
    #[link(name = "Bash")]
    extern "C" {
        fn bash_main();
    }

    let handles = create_terminals()?;

    let (sender, receiver) = channel();

    let reader_barrier = Arc::new(Barrier::new(3));
    let bash_barrier = Arc::new(Barrier::new(2));

    let stdout_sender = sender.clone();
    let (stdout_m, stderr_m) = (handles.stdout_m, handles.stderr_m);
    let stdout_bo = handles.bite_stdout.clone();
    let out_reader_barrier = reader_barrier.clone();
    spawn(move || {
        out_reader_barrier.wait();
        unsafe {
            read_data(
                stdout_m,
                stdout_sender,
                &mut bash_out_blocked,
                &BashOutput::FromOutput,
                stdout_bo,
            )
        }
    });

    let stdout_be = handles.bite_stdout.clone();
    let stderr_sender = sender.clone();
    let err_reader_barrier = reader_barrier.clone();
    spawn(move || {
        err_reader_barrier.wait();
        unsafe {
            read_data(
                stderr_m,
                stderr_sender,
                &mut bash_err_blocked,
                &BashOutput::FromError,
                stdout_be,
            )
        }
    });

    unsafe { bash_sender = Some(Mutex::new(sender)) };

    let bash_main_barrier = bash_barrier.clone();
    spawn(move || {
        bash_main_barrier.wait();
        unsafe { bash_main() }
    });

    unsafe { pts_handles = Some(handles) };

    // If we got here, we can print stuff through the backup handles.
    info!("Pseudo terminals correctly set up.");

    Ok((receiver, reader_barrier, bash_barrier))
}

/// Try to stop the machinery.
pub fn stop() {
    // Make bash shutdown cleanly by killing a potentially running program and then telling bash to
    // exit, which will make it terminate its thread.
    bash_kill_last();
    bash_add_input("exit 0\n");

    // Then close all handles
    unsafe {
        read_lines_quit.store(true, Ordering::Relaxed);
        pts_handles.as_mut().map(|h| {
            let _ = close(h.prg_stdin);
            let _ = close(h.stdin_s);
            let _ = close(h.stdin_b);
            let _ = close(h.stdout_m);
            let _ = close(h.stdout_s);
            let _ = close(h.stderr_m);
            let _ = close(h.stderr_s);
        });
        pts_handles = None;
    }
}
