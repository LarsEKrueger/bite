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

//! Builtin runners

use std::sync::{Arc, Mutex};
use std::fs::File;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

use argparse::{ArgumentParser, StoreTrue, Store, StoreFalse, List};
use super::script_parser;

use super::*;

/// Lock the mutex and run a function with error handling
fn do_with_lock<T, F>(thing: &mut Arc<Mutex<T>>, stderr: &mut File, fun: F) -> Option<ExitStatus>
where
    F: FnOnce(&mut T),
{
    match thing.lock() {
        Err(e) => {
            use std::io::Write;
            use std::error::Error;
            write!(
                stderr,
                "{}\n",
                self::Error::InternalError(file!(), line!(), e.description().to_string())
                    .cause("export: ", "")
            ).unwrap();
            Some(ExitStatus::from_raw(1))
        }
        Ok(ref mut inner) => {
            fun(inner);
            None
        }
    }
}

/// Lock the mutex and run a function with double error handling
fn do_with_lock_err<T, F>(
    thing: &mut Arc<Mutex<T>>,
    stderr: &mut File,
    prefix: &str,
    fun: F,
) -> Option<ExitStatus>
where
    F: FnOnce(&mut T) -> Result<()>,
{
    match thing.lock() {
        Err(e) => {
            use std::io::Write;
            use std::error::Error;
            write!(
                stderr,
                "{}\n",
                self::Error::InternalError(file!(), line!(), e.description().to_string())
                    .cause("export: ", "")
            ).unwrap();
            Some(ExitStatus::from_raw(1))
        }
        Ok(ref mut inner) => {
            match fun(inner) {
                Err(e) => {
                    use std::io::Write;
                    write!(stderr, "{}", e.cause(prefix, "")).unwrap();
                    Some(ExitStatus::from_raw(1))
                }
                Ok(_) => None,
            }
        }
    }
}

pub mod export;
pub mod readonly;
pub mod cd;
