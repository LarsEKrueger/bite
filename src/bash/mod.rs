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

mod types;
mod parser;
mod prompt_parser;

use nom::IResult;
use std::env;
use std::mem;
use std::ptr;
use std::ffi::CStr;
use libc::{c_char, gethostname, gid_t, uid_t};

pub use self::types::*;
pub use self::prompt_parser::*;

// All relevant info about the current user.
struct UserInfo {
    uid: uid_t,
    _gid: gid_t,
    name: String,
    _home_dir: String,
    _shell: String,
}

pub struct Bash {
    current_host_name: String,
    current_user: UserInfo,
    line: String,
}

const VERSION: &'static str = "0.0";
const PATCHLEVEL: &'static str = "0";

const FALLBACK_HOSTNAME: &'static str = "??host??";

fn format_error_message(error: ::nom::Err<&[u8]>, line: &String) -> Vec<String> {
    let mut msg = line.clone();

    match error {
        ::nom::Err::NodePosition(_, p, _) |
        ::nom::Err::Position(_, p) => {
            let l_ptr = line.as_ptr();
            let p_ptr = p.as_ptr();
            let o = p_ptr as usize - l_ptr as usize;
            for _ in 0..o {
                msg.push_str(" ");
            }
            msg.push_str("^---- Syntax error");
        }
        _ => msg.push_str("Syntax error"),
    };

    msg.lines().map(String::from).collect()
}

impl Bash {
    pub fn new() -> Self {
        // It's highly unlikely that this will change.
        let current_host_name = {
            // Host names are at most 255 bytes long, plus the zero.
            let len: usize = 256;
            let mut buf: Vec<u8> = vec![0; len];

            let err = unsafe { gethostname(buf.as_mut_ptr() as *mut c_char, len) };
            if err == 0 {
                // find the first 0 byte (i.e. just after the data that gethostname wrote)
                let actual_len = buf.iter().position(|byte| *byte == 0).unwrap_or(len);

                String::from(::std::str::from_utf8(&buf[..actual_len]).unwrap_or(
                    FALLBACK_HOSTNAME,
                ))
            } else {
                String::from(FALLBACK_HOSTNAME)
            }
        };

        // We get the user right here, because we are almost always an interactive shell.
        let current_user = unsafe {
            let amt = match ::libc::sysconf(::libc::_SC_GETPW_R_SIZE_MAX) {
                n if n < 0 => 512 as usize,
                n => n as usize,
            };
            let mut buf = Vec::with_capacity(amt);
            let mut passwd: ::libc::passwd = mem::zeroed();
            let mut result = ptr::null_mut();
            match ::libc::getpwuid_r(
                ::libc::getuid(),
                &mut passwd,
                buf.as_mut_ptr(),
                buf.capacity(),
                &mut result,
            ) {
                0 if !result.is_null() => UserInfo {
                    uid: passwd.pw_uid,
                    _gid: passwd.pw_gid,
                    name: CStr::from_ptr(passwd.pw_name as *const c_char)
                        .to_string_lossy()
                        .into_owned(),
                    _home_dir: CStr::from_ptr(passwd.pw_dir as *const c_char)
                        .to_string_lossy()
                        .into_owned(),
                    _shell: CStr::from_ptr(passwd.pw_shell as *const c_char)
                        .to_string_lossy()
                        .into_owned(),
                },
                _ => UserInfo {
                    uid: ::std::u32::MAX,
                    _gid: ::std::u32::MAX,
                    name: String::from("I have no name"),
                    _home_dir: String::from("I have no home"),
                    _shell: String::from("I have no shell"),
                },
            }
        };

        Self {
            line: String::new(),
            current_host_name,
            current_user,
        }
    }

    pub fn version() -> String {
        String::from(VERSION)
    }

    pub fn version_and_patchlevel() -> String {
        let mut s = String::from(VERSION);
        s.push_str(".");
        s.push_str(&PATCHLEVEL);
        s
    }

    pub fn add_line(&mut self, l: &str) -> types::Command {
        // Append the line to the last one and try to (re-)parse.
        self.line.push_str(l);
        use std::mem;
        let line = mem::replace(&mut self.line, String::new());
        let bytes = line.as_bytes();
        match parser::parse_bash(bytes) {
            IResult::Incomplete(_) => types::Command::Incomplete,
            IResult::Error(e) => types::Command::Error(format_error_message(e, &line)),
            IResult::Done(r, o) => {
                // Delete the input from line and return the command.
                let rest = self.line.split_off(r.len());
                self.line = rest;
                o
            }
        }
    }

    pub fn expand_ps1(&self) -> String {
        // Get the string from the environment.
        // TODO: Get it from own variables.

        let ps1_string = env::var("PS1").unwrap_or(String::from("\\s-\\v\\$ "));

        self.decode_prompt_string(ps1_string.as_str())
    }

    fn decode_prompt_string(&self, input: &str) -> String {
        // For the moment use nom to parse the string until proven too slow.
        match parse_prompt(input.as_bytes(), self) {
            IResult::Done(_, s) => s,
            _ => String::from("$ "),
        }
    }

    pub fn get_current_host_name<'a>(&'a self) -> &'a str {
        &self.current_host_name
    }

    pub fn get_current_user_name<'a>(&'a self) -> &'a str {
        &self.current_user.name
    }

    pub fn current_user_is_root(&self) -> bool {
        self.current_user.uid == 0
    }
}
