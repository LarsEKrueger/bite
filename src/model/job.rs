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

//! Launch programs and store their output in a Session

use libc::c_uchar;
use nix::fcntl::{open, OFlag};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt};
use nix::sys::select::{select, FdSet};
use nix::sys::stat::Mode;
use nix::sys::time::{TimeVal, TimeValLike};
use nix::unistd::{close, read, write};
use std::ffi::OsStr;
use std::mem;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::thread::spawn;
use std::time::Duration;
use termios::os::target::*;
use termios::*;

use super::session::{InteractionHandle, OutputVisibility, RunningStatus, SharedSession};

/// std::process::Child that can be shared between threads
type SharedChild = Arc<Mutex<Child>>;

/// Public info about the job.
///
/// Most internal info is kept in threads and can be kept private.
pub struct Job {
    /// Interaction Handle for this job
    interactionHandle: InteractionHandle,

    /// The child process that has been created and its stdin PTS handle.
    ///
    /// Might be None if the start failed
    child: Option<(SharedChild, RawFd)>,
}

/// Handles for the Pseudo Terminals
struct PtsHandles {
    /// Stdin PTS master (bite side)
    stdin_m: RawFd,
    /// Stdin PTS slave (command side)
    stdin_s: RawFd,
    /// Stdout PTS master (bite side)
    stdout_m: RawFd,
    /// Stdout PTS slave (command side)
    stdout_s: RawFd,
    /// Stderr PTS master (bite side)
    stderr_m: RawFd,
    /// Stderr PTS slave (command side)
    stderr_s: RawFd,
}

/// Compute the matching control character of a letter
const fn control(x: char) -> c_uchar {
    ((x as u32) & 0x1f) as c_uchar
}

/// Fix the control characters
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

/// Convert an error to a string.
fn as_description<T>(err: T) -> String
where
    T: ::std::error::Error,
{
    err.description().to_string()
}

/// Define a fallback termios value
fn fallback_termios() -> Termios {
    let mut termios: Termios = unsafe { mem::zeroed() };
    termios.c_iflag = ICRNL | IXON;
    termios.c_oflag = TAB3 | ONLCR | OPOST;
    fix_termios_cc(&mut termios);
    termios
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

/// Create handles for stdin, stdout, stderr as pseudo terminals.
///
/// Might fail with an error message.
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

    Ok(PtsHandles {
        stdin_m,
        stdin_s,
        stdout_m,
        stdout_s,
        stderr_m,
        stderr_s,
    })
}

/// Read from a RawFd until fails and send to session
fn read_data(
    fd: RawFd,
    mut session: SharedSession,
    interactionHandle: InteractionHandle,
    stream: OutputVisibility,
) {
    // The loop will exit on error
    loop {
        // If there is input, read it.
        let mut rdfs = FdSet::new();
        rdfs.insert(fd);
        let mut timeout = TimeVal::milliseconds(20);
        let data_available = match select(None, Some(&mut rdfs), None, None, Some(&mut timeout)) {
            Ok(0) | Err(_) => false,
            Ok(_) => true,
        };

        if data_available {
            let mut buffer = [0; 4096];
            if let Ok(len) = read(fd, &mut buffer) {
                // TODO: Handle switching to TUI mode
                session.add_bytes_raw(stream, interactionHandle, &buffer[0..len]);
            } else {
                // There was some serious error reading from command, so drop everything and leave.
                break;
            }
        }
    }
}

/// Wait for a child to finish.
///
/// This needs to be implemented by polling as not to drop stdin.
fn wait_for_child(
    child: SharedChild,
    mut session: SharedSession,
    interactionHandle: InteractionHandle,
) {
    let time_between_polls = Duration::from_millis(10);
    let mut error_reported = false;
    loop {
        if let Ok(mut child) = child.lock() {
            match child.try_wait().map_err(as_description) {
                Ok(Some(es)) => {
                    session.set_running_status(interactionHandle, RunningStatus::Exited(es));
                    break;
                }
                Err(msg) => {
                    if !error_reported {
                        session.add_error_raw(
                            interactionHandle,
                            b"BiTE: Failed to poll for exit code. Cause: ",
                        );
                        session.add_error_raw(interactionHandle, msg.as_bytes());
                        session.add_error_raw(interactionHandle, b"\n");
                        error_reported = true;
                    }
                    session.set_running_status(interactionHandle, RunningStatus::Unknown);
                }
                _ => {}
            }
        }
        sleep(time_between_polls);
    }
}

/// Start a command and set up the reader threads
///
/// This is the core of Job::new.
fn start_command<I, S>(
    session: SharedSession,
    interactionHandle: InteractionHandle,
    program: S,
    args: I,
) -> Result<(SharedChild, RawFd), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let handles = create_terminals()?;
    let child = Command::new(program)
        .args(args)
        .stdin(unsafe { Stdio::from_raw_fd(handles.stdin_s) })
        .stderr(unsafe { Stdio::from_raw_fd(handles.stderr_s) })
        .stdout(unsafe { Stdio::from_raw_fd(handles.stdout_s) })
        .spawn()
        .map_err(as_description)?;

    {
        let stdout_m = handles.stdout_m;
        let stdout_session = session.clone();
        spawn(move || {
            read_data(
                stdout_m,
                stdout_session,
                interactionHandle,
                OutputVisibility::Output,
            )
        });
    }

    {
        let stderr_m = handles.stderr_m;
        let stderr_session = session.clone();
        spawn(move || {
            read_data(
                stderr_m,
                stderr_session,
                interactionHandle,
                OutputVisibility::Error,
            )
        });
    }

    let child = Arc::new(Mutex::new(child));
    {
        let wait_child = child.clone();
        spawn(move || wait_for_child(wait_child, session, interactionHandle));
    }
    Ok((child, handles.stdin_m))
}

impl Job {
    /// Start a command and the threads that read from stdin/stdout
    ///
    /// TODO: Decouple from bite's CWD and env: Use interpreter-internal values
    pub fn new<I, S>(
        mut session: SharedSession,
        interactionHandle: InteractionHandle,
        program: S,
        args: I,
    ) -> Job
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let child = match start_command(session.clone(), interactionHandle, program, args) {
            Err(msg) => {
                session.add_error_raw(interactionHandle, b"BiTE: Failed to launch job. Cause: ");
                session.add_error_raw(interactionHandle, msg.as_bytes());
                session.add_error_raw(interactionHandle, b"\n");
                None
            }
            Ok(child) => Some(child),
        };

        session.set_running_status(interactionHandle, RunningStatus::Running);

        Job {
            interactionHandle,
            child,
        }
    }

    /// Write data to stdin of job.
    ///
    /// This might block if the job doesn't accept data and the pipe is full.
    ///
    /// Does not echo data to session.
    pub fn write_stdin(&mut self, bytes: &[u8]) {
        if let Some((_, stdin)) = self.child {
            // TODO: Check result
            let _ = write(stdin, bytes);
        }
    }
}
