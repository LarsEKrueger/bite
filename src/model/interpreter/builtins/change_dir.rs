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

//! Change directory builtin

use std::env::VarError;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use nix::unistd::chdir;

use argparse::{ArgumentParser, Store};

fn change_dir(mut dir: String, _stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    // Fix dir
    if dir.is_empty() {
        dir = match std::env::var("HOME") {
            Ok(d) => d,
            Err(var_err) => {
                return match var_err {
                    VarError::NotPresent => {
                        let _ = write!(stderr, "BiTE: cd can't value of $HOME\n");
                        3
                    }
                    VarError::NotUnicode(_) => {
                        let _ = write!(stderr, "BiTE: cd: Vlue of $HOME isn't unicode\n");
                        4
                    }
                };
            }
        };
    }
    // Change directory
    match chdir(dir.as_str()) {
        Ok(()) => 0,
        Err(e) => {
            let _ = write!(stderr, "BiTE: cd can't change to »{}«: {}\n", dir, e);
            5
        }
    }
}

/// Run function for the *change directory* builtin.
///
/// cd [dir]
pub fn run(words: Vec<String>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> ExitStatus {
    let mut dir = String::new();

    let parse_res = {
        let mut ap = ArgumentParser::new();
        ap.set_description("Change directory");
        ap.refer(&mut dir)
            .add_argument("dir", Store, "Directory to change into");

        ap.parse(words, stdout, stderr)
    };
    let ret_code = match parse_res {
        Ok(()) => change_dir(dir, stdout, stderr),
        Err(ret_code) => ret_code,
    };

    ExitStatusExt::from_raw(ret_code)
}
