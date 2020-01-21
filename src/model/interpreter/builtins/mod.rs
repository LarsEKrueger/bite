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

//! Builtin commands

pub mod change_dir;

use std::io::Write;
use std::os::unix::process::ExitStatusExt;

use super::super::session::{InteractionHandle, OutputVisibility, RunningStatus, SharedSession};

pub trait SetReturnCode {
    fn set_return_code(&mut self, i32);
}

#[derive(Clone)]
pub struct SessionOutput {
    pub session: SharedSession,
    pub handle: InteractionHandle,
}

pub struct SessionStdout(pub SessionOutput);
pub struct SessionStderr(pub SessionOutput);

impl SetReturnCode for SessionOutput {
    fn set_return_code(&mut self, return_code: i32) {
        self.session.set_running_status(
            self.handle,
            RunningStatus::Exited(ExitStatusExt::from_raw(return_code)),
        );
    }
}

impl Write for SessionStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .session
            .add_bytes(OutputVisibility::Output, self.0.handle, buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Write for SessionStderr {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .session
            .add_bytes(OutputVisibility::Error, self.0.handle, buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub type BuiltinRunner =
fn (
    words: Vec<String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    set_return_code: &mut dyn SetReturnCode,
);

pub fn runner( cmd:&str) -> Option< BuiltinRunner> {
  if cmd == "cd" {
      Some(change_dir::run)
  } else { None }
}
