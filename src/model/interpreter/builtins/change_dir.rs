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

use std::io::Write;

use argparse::{ArgumentParser, Store};

use super::SetReturnCode;

/// Run function for the *change directory* builtin.
///
/// cd [dir]
pub fn run(
    words: Vec<String>,
    stdout: &mut Write,
    stderr: &mut Write,
    set_return_code: &mut SetReturnCode,
) {
    let mut dir = String::new();

    let mut ap = ArgumentParser::new();
    ap.set_description("Change directory");
    ap.refer(&mut dir)
        .add_argument("dir", Store, "Directory to change into");
    match ap.parse(words, stdout, stderr) {
        Ok(()) => {
            // TODO: Change directory

        }
        Err(ret_code) => {

        set_return_code.set_return_code( ret_code); 
        }
    }
}
