/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2020  Lars Krüger

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

//! Manage a number of jobs

use libc::c_uchar;
use nix::fcntl::{open, OFlag};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt};
use nix::sys::select::{select, FdSet};
use nix::sys::signal::{killpg, Signal};
use nix::sys::stat::Mode;
use nix::sys::time::{TimeVal, TimeValLike};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, read, write, Pid};
use std::ffi::OsStr;
use std::fs::File;
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::{spawn, JoinHandle};
use termios::os::target::*;
use termios::*;

use super::super::session::{InteractionHandle, OutputVisibility, SharedSession};
use super::builtins;
use super::builtins::BuiltinRunner;

/// Public info about the job.
///
/// Most internal info is stored in threads and can be kept private.
pub struct Job {
    /// Pseudo terminal handle for sending data to stdin of the whole pipeline.
    stdin_bite_side: RawFd,

    /// Other PTYs, mostly for changing the terminal size
    other_bite_side: Vec<RawFd>,

    /// Children for termination
    children: Vec<u32>,
}

/// Pair of PTS handles
#[derive(Debug)]
pub struct PtsPair {
    /// Stdin PTS master
    bite_side: RawFd,
    /// Stdin PTS slave
    command_side: RawFd,
}

/// All infos needed during starting a pipeline
#[derive(Debug)]
pub struct PipelineBuilder {
    /// Interaction to write the output to
    interaction_handle: InteractionHandle,

    /// Pseudo terminal handle for sending data to stdin of the whole pipeline. Will not be changed
    /// after creation
    stdin_bite_side: RawFd,

    /// Pseudo terminal handle for reading data from stdout of the whole pipeline. Created by last
    /// program in the pipeline.
    stdout_bite_side: Option<RawFd>,

    /// fd to use for stdin of the next program. Will be overwritten by every command in the
    /// pipeline.
    prev_stdout: RawFd,

    /// Stderr handles to read from
    stderr: Vec<RawFd>,

    /// Child processes to watch
    children: Vec<ChildOrThread>,

    /// Next program to start, might be a builtin
    next_program: ProgramOrBuiltin,
}

/// If next program is a builtin, store its function pointer instead of the name
enum ProgramOrBuiltin {
    /// Not set
    Nothing,

    /// Program, keep its name
    Program(String),

    /// Builtin, keep the function pointer
    Builtin(BuiltinRunner),
}

/// Different things to wait on for completion
#[derive(Debug)]
enum ChildOrThread {
    Child(Child),
    Thread(JoinHandle<ExitStatus>),
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
    err.to_string()
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
        info!("Opened /dev/tty for termios");
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

/// Prepare termios struct for use with pts
fn fixup_termios(termios: &mut Termios) {
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
    fix_termios_cc(termios);
    cfmakeraw(termios);
}

/// Create a single pts master/slave pair.
///
/// Returns: (master, slave)
fn create_terminal(termios: Termios) -> Result<(RawFd, RawFd), String> {
    let ptsm =
        posix_openpt(OFlag::O_RDWR | OFlag::O_EXCL | OFlag::O_NONBLOCK).map_err(as_description)?;
    trace!("PTY: bite side fd {:?}", ptsm);
    grantpt(&ptsm).map_err(as_description)?;
    unlockpt(&ptsm).map_err(as_description)?;
    let sname = unsafe { ptsname(&ptsm).map_err(as_description) }?;
    trace!("PTY: command side name {}", sname);
    let sfd = open(Path::new(&sname), OFlag::O_RDWR, Mode::empty()).map_err(as_description)?;
    trace!("PTY: command side fd {}", sfd);

    let ptsm = ptsm.into_raw_fd();
    tcsetattr(sfd, TCSANOW, &termios).map_err(as_description)?;
    tcflush(sfd, TCOFLUSH).map_err(as_description)?;

    Ok((ptsm, sfd))
}

/// Create a pair of PTS handles
fn create_handle_pair() -> Result<PtsPair, String> {
    let mut termios = default_termios();
    fixup_termios(&mut termios);
    let (bite_side, command_side) = create_terminal(termios)?;
    Ok(PtsPair {
        bite_side,
        command_side,
    })
}

use nix::libc::{c_ushort, winsize, TIOCSWINSZ};
ioctl_write_ptr_bad!(ioctl_set_winsize, TIOCSWINSZ, winsize);

fn set_winsize(fd: RawFd, width: usize, height: usize) {
    let ws = winsize {
        ws_row: height as c_ushort,
        ws_col: width as c_ushort,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let res = unsafe { ioctl_set_winsize(fd, &ws) };
    trace!("set_winsize({}x{}): {:?}", width, height, res);
}

/// Read from a RawFd until fails and send to session
fn read_data(
    fd: RawFd,
    mut session: SharedSession,
    interactionHandle: InteractionHandle,
    stream: OutputVisibility,
) {
    trace!("Reading data from file descriptor {}", fd);
    session.register_thread(interactionHandle);
    // The loop will exit on error
    loop {
        // If there is input, read it.
        let mut rdfs = FdSet::new();
        rdfs.insert(fd);
        let mut exfs = FdSet::new();
        exfs.insert(fd);
        let mut timeout = TimeVal::milliseconds(20);
        let data_available = match select(
            None,
            Some(&mut rdfs),
            None,
            Some(&mut exfs),
            Some(&mut timeout),
        ) {
            Ok(0) | Err(_) => false,
            Ok(_) => true,
        };
        if data_available {
            trace!("read_data maybe got data: {:?}, {:?}", rdfs, exfs);
            let mut buffer = [0; 4096];
            match read(fd, &mut buffer) {
                Ok(len) => {
                    session.add_bytes(stream, interactionHandle, &buffer[0..len]);
                }
                Err(err) => {
                    // There was some serious error reading from command, so drop everything and
                    // leave. This is a first detector if a program exited.
                    debug!(
                        "Stopped reading from file descriptor {} due to {:?}",
                        fd, err
                    );
                    break;
                }
            }
        }
    }
    trace!("Done reading data from file descriptor {}", fd);
    session.unregister_thread(interactionHandle);
    let _ = close(fd);
}

impl std::fmt::Debug for ProgramOrBuiltin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProgramOrBuiltin::Nothing => write!(f, "ProgramOrBuiltin::Nothing"),
            ProgramOrBuiltin::Program(s) => write!(f, "ProgramOrBuiltin::Program {{ {:?} }}", s),
            ProgramOrBuiltin::Builtin(b) => {
                write!(f, "ProgramOrBuiltin::Builtin {{ {:?} }}", *b as *const ())
            }
        }
    }
}

impl PipelineBuilder {
    /// Prepare a new pipeline
    pub fn new(
        interaction_handle: InteractionHandle,
        window_width: usize,
        window_height: usize,
    ) -> Result<Self, String> {
        let stdin_pair = create_handle_pair()?;

        // Set window size on bite side of PTY
        set_winsize(stdin_pair.bite_side, window_width, window_height);

        Ok(Self {
            interaction_handle,
            stdin_bite_side: stdin_pair.bite_side,
            stdout_bite_side: None,
            prev_stdout: stdin_pair.command_side,
            stderr: Vec::new(),
            children: Vec::new(),
            next_program: ProgramOrBuiltin::Nothing,
        })
    }

    /// Set the name of the next program to launch
    pub fn set_program(&mut self, name: String) {
        trace!(
            "Overwriting program »{:?}« with »{}«",
            self.next_program,
            name
        );
        self.next_program = if let Some(b) = builtins::runner(&name) {
            ProgramOrBuiltin::Builtin(b)
        } else {
            ProgramOrBuiltin::Program(name)
        };
    }

    /// Start a program and hook it into the pipeline
    ///
    /// If it's the first program in the pipeline, connect the stdin to the command_side of the
    /// stdin pts, otherwise connect it to the stdout of the previous program.
    ///
    /// If it's the last program in the pipeline, connect to the command_side of the stdout/stderr
    /// pts, otherwise create them as pipes.
    pub fn start<I, S>(
        &mut self,
        is_last: bool,
        window_width: usize,
        window_height: usize,
        args: I,
    ) -> Result<(), String>
    where
        I: IntoIterator<Item = S> + std::fmt::Debug,
        S: AsRef<OsStr>,
    {
        match &self.next_program {
            ProgramOrBuiltin::Nothing => {
                error!("No program set for argument »{:?}«", args);
                Err("Internal error".to_string())
            }
            ProgramOrBuiltin::Program(s) => {
                trace!(
                    "start program »{:}«, window: {}x{}",
                    s,
                    window_width,
                    window_height
                );
                let mut cmd = Command::new(s);

                cmd.args(args)
                    .env("TERM", "xterm")
                    .stdin(unsafe { Stdio::from_raw_fd(self.prev_stdout) });

                // If stderr isn't redirected, it will go out to the pts directly.
                {
                    let stderr_pair = create_handle_pair()?;
                    set_winsize(stderr_pair.bite_side, window_width, window_height);
                    self.stderr.push(stderr_pair.bite_side);
                    cmd.stderr(unsafe { Stdio::from_raw_fd(stderr_pair.command_side) });
                }

                // The last stdout goes to the pts, all others are piped.
                if is_last {
                    // If this is the last command of the pipeline, create pts for the outputs.
                    let stdout_pair = create_handle_pair()?;
                    set_winsize(stdout_pair.bite_side, window_width, window_height);
                    self.stdout_bite_side = Some(stdout_pair.bite_side);

                    cmd.stdout(unsafe { Stdio::from_raw_fd(stdout_pair.command_side) });
                } else {
                    cmd.stdout(Stdio::piped());
                }

                // Set the process group of this process. This makes the process its own program
                // group. If the command is a shell script, all programs run from that script can
                // be killed or their screen size changed.
                unsafe {
                    cmd.pre_exec(|| {
                        if -1 == libc::setpgid(0, 0) {
                            Err(std::io::Error::last_os_error())
                        } else {
                            Ok(())
                        }
                    })
                };

                trace!("about to spawn »{:}«", s);
                let child = cmd.spawn().map_err(as_description)?;

                // Get the last stdout, then keep the child for waiting, then check if the pipe
                // creation worked.
                let prev_stdout = child
                    .stdout
                    .as_ref()
                    .map(|o| o.as_raw_fd())
                    .ok_or_else(|| format!("Could not create output pipeline for »{}«", s));

                self.children.push(ChildOrThread::Child(child));
                // If this isn't the last command in a pipe, keep the stdout around for the next
                // command as stdin.
                if !is_last {
                    trace!("prev_stdout: {:?}", prev_stdout);
                    self.prev_stdout = prev_stdout?;
                }

                Ok(())
            }
            ProgramOrBuiltin::Builtin(b) => {
                trace!("start builtin");
                // stdin isn't used and can be closed.
                let _ = nix::unistd::close(self.prev_stdout);

                // If stderr isn't redirected, it will go out to the pts directly.
                let stderr_pair = create_handle_pair()?;
                self.stderr.push(stderr_pair.bite_side);

                // The last stdout goes to the pts, all others are piped.
                let stdout_command_side = if is_last {
                    // If this is the last command of the pipeline, create pts for the outputs.
                    let stdout_pair = create_handle_pair()?;
                    self.stdout_bite_side = Some(stdout_pair.bite_side);
                    stdout_pair.command_side
                } else {
                    // Create a normal pipe
                    let (read_end, write_end) = nix::unistd::pipe().map_err(as_description)?;
                    self.prev_stdout = read_end;
                    write_end
                };

                let mut args: Vec<String> = args
                    .into_iter()
                    .map(|s| s.as_ref().to_string_lossy().into_owned())
                    .collect();
                // Insert a fake argv[0]. Replace with name of builtin if that is important.
                args.insert(0, "builtin".to_string());
                let b = b.clone();
                trace!("about to spawn thread");
                let t = spawn(move || {
                    b(
                        args,
                        &mut unsafe { File::from_raw_fd(stdout_command_side) },
                        &mut unsafe { File::from_raw_fd(stderr_pair.command_side) },
                    )
                });
                trace!("thread spawned");

                self.children.push(ChildOrThread::Thread(t));

                Ok(())
            }
        }
    }

    /// Construct an iterator to yield the pids of the child programs
    fn child_pids<'a>(&'a self) -> impl Iterator<Item = u32> + 'a {
        self.children.iter().filter_map(|cot| match cot {
            ChildOrThread::Child(c) => Some(c.id()),
            _ => None,
        })
    }

    /// Construct a Job to be stored in the session
    pub fn create_job(&self) -> Job {
        let mut other_bite_side = Vec::new();
        if let Some(stdout) = self.stdout_bite_side {
            other_bite_side.push(stdout);
        }
        for e in self.stderr.iter() {
            other_bite_side.push(e.clone());
        }
        Job {
            children: self.child_pids().collect(),
            stdin_bite_side: self.stdin_bite_side,
            other_bite_side,
        }
    }

    // Run the programs in the pipeline to completion, return the exit status of the last program
    pub fn wait(self, session: SharedSession, interaction_handle: InteractionHandle) -> i32 {
        let mut reader_threads = Vec::new();

        // Start a reader thread for each stderr
        for e in self.stderr.iter() {
            let session = session.clone();
            let e = *e;
            let jh =
                spawn(move || read_data(e, session, interaction_handle, OutputVisibility::Error));
            reader_threads.push(jh);
        }

        // Start a reader thread for the last stdout
        if let Some(stdout) = self.stdout_bite_side {
            let session = session.clone();
            let jh = spawn(move || {
                read_data(
                    stdout,
                    session,
                    interaction_handle,
                    OutputVisibility::Output,
                )
            });
            reader_threads.push(jh);
        }

        let mut exit_status = 0;
        // Waiting for the reader threads to complete  doesn't require locking the mutex around
        // the children, but it also does not catch all cases (e.g. gvim going into background)
        // because the file handle might be passed down to the forked process and still be open.
        //
        // In case of gvim, waiting for the reader threads is actually bad as it would
        // (incorrectly) block until gvim exits.
        //
        // Instead, get the process ids and call waitpid.
        for pid in self.child_pids() {
            // Wait for each child, report on the exit code of each failing program. Keep the exit code
            // of the last failing program.
            trace!("Waiting for pid {:?}", pid);
            match waitpid(Some(Pid::from_raw(pid as i32)), None) {
                Err(e) => {
                    debug!("Error waiting for pid: »{:?}«", e);
                }
                Ok(WaitStatus::Exited(_, es)) => {
                    exit_status = es;
                }
                ret => {
                    debug!("waitpid returned with unexpected reason: {:?}", ret);
                }
            }
        }
        // Close bite's side of stdin
        let _ = close(self.stdin_bite_side);

        exit_status
    }
}

impl Job {
    /// Terminate all the programs
    pub fn terminate(&self) {
        for pid in self.children.iter() {
            let res = killpg(Pid::from_raw(*pid as i32), Some(Signal::SIGTERM));
            trace!("terminate: killpg({}) = {:?}", pid, res);
        }
    }

    /// Send some bytes to the stdin of this job
    pub fn write_stdin(&mut self, bytes: &[u8]) {
        let _ = write(self.stdin_bite_side, bytes);
    }

    pub fn set_tui_size(&mut self, w: usize, h: usize) {
        set_winsize(self.stdin_bite_side, w, h);
        for fd in self.other_bite_side.iter() {
            set_winsize(fd.clone(), w, h);
        }
        for pid in self.children.iter() {
            trace!("sending SIGWINCH to {}", pid);
            let res = killpg(Pid::from_raw(*pid as i32), Some(Signal::SIGWINCH));
            trace!("set_tui_size: killpg({}) = {:?}", pid, res);
        }
    }
}
