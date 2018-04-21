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

//! Special builtin runnemutrs

use std::sync::{Arc, Mutex};
use std::os::unix::io::{FromRawFd, RawFd};
use std::fs::File;

use argparse::{ArgumentParser, StoreTrue, List};
use super::*;
use super::script_parser;


/// Lock the mutex and run a function with error handling
fn do_with_lock<T, F>(thing: &mut Arc<Mutex<T>>, stderr: &mut File, fun: F)
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
        }
        Ok(ref mut inner) => fun(inner),
    }
}

/// Lock the mutex and run a function with double error handling
fn do_with_lock_err<T, F>(thing: &mut Arc<Mutex<T>>, stderr: &mut File, prefix: &str, fun: F)
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
        }
        Ok(ref mut inner) => {
            fun(inner).map_err(|e| {
                use std::io::Write;
                write!(stderr, "{}", e.cause(prefix, "")).unwrap();
            });
        }
    }
}


/// Runner function for export special builtin
///
/// export [-fn] [-p] [name[=value]]
pub fn export_runner(
    mut bash: Arc<Mutex<Bash>>,
    stdin: RawFd,
    stdout: RawFd,
    stderr: RawFd,
    words: Vec<String>,
) {
    let mut negat = false;
    let mut funct = false;
    let mut print = false;

    let _stdin = unsafe { File::from_raw_fd(stdin) };
    let mut stdout = unsafe { File::from_raw_fd(stdout) };
    let mut stderr = unsafe { File::from_raw_fd(stderr) };

    let mut assignments: Vec<String> = vec![];

    if let Ok(()) = {
        let mut ap = ArgumentParser::new();
        ap.set_description("export - builtin");
        // TODO: Better help text
        ap.refer(&mut negat).add_option(
            &["-n"],
            StoreTrue,
            "negate",
        );
        ap.refer(&mut funct).add_option(
            &["-f"],
            StoreTrue,
            "Function",
        );
        ap.refer(&mut print).add_option(&["-p"], StoreTrue, "Print");
        ap.refer(&mut assignments).add_argument(
            "name",
            List,
            "Names",
        );

        ap.parse(words, &mut stdout, &mut stderr)
    } {
        if !negat && !funct && assignments.is_empty() {
            print = true;
        }
        if print {
            do_with_lock(
                &mut bash,
                &mut stderr,
                |bash|
                {
                    for (name,variable) in bash.variables.iter()
                        .filter(|&(_,ref v)| v.is_exported()) {
                            variable.print_for_builtins(name,&mut stdout)
                        }
                });
        }
        else {
            for assignment in assignments {
                if let IResult::Done(rest, (name , maybe_value)) =
                    script_parser::assignment_or_name(assignment.as_bytes()) {
                        let name = String::from_utf8_lossy(name);
                        if rest.is_empty() {
                            do_with_lock_err(
                                &mut bash,
                                &mut stderr,
                                "export: ",
                                |bash| {
                                    bash.variables.find_variable_or_create_global( &name).map(
                                        |variable| {
                                            variable.set_exported(!negat);
                                            if let Some(value) = maybe_value {
                                                variable.set_value(&value);
                                            }
                                        })
                                }
                                );
                        }
                    }
            }
        }
    };
}

/// Runner function for readonly special builtin
///
/// # Parameters
///
/// readonly [-aAf] [-p] [name[=value]]
pub fn readonly_runner(
    mut bash: Arc<Mutex<Bash>>,
    stdin: RawFd,
    stdout: RawFd,
    stderr: RawFd,
    words: Vec<String>,
) {
    let mut array = false;
    let mut assoc = false;
    let mut funct = false;
    let mut print = false;
    let mut assignments: Vec<String> = vec![];

    let _stdin = unsafe { File::from_raw_fd(stdin) };
    let mut stdout = unsafe { File::from_raw_fd(stdout) };
    let mut stderr = unsafe { File::from_raw_fd(stderr) };

    if let Ok(()) = {
        let mut ap = ArgumentParser::new();
        ap.set_description("readonly - builtin");
        // TODO: Better help text
        ap.refer(&mut array).add_option(
            &["-a"],
            StoreTrue,
            "Indexed array",
        );
        ap.refer(&mut assoc).add_option(
            &["-A"],
            StoreTrue,
            "Associative array",
        );
        ap.refer(&mut funct).add_option(
            &["-f"],
            StoreTrue,
            "Function",
        );
        ap.refer(&mut print).add_option(&["-p"], StoreTrue, "Print");
        ap.refer(&mut assignments).add_argument(
            "name",
            List,
            "Names",
        );

        ap.parse(words, &mut stdout, &mut stderr)
    } {
        if !array && !assoc && !funct && assignments.is_empty() {
            print = true;
        }
        if print {
            do_with_lock(
                &mut bash,
                &mut stderr,
                |bash|
                for (name,variable) in bash.variables.iter()
                .filter(|&(_,ref v)| v.is_readonly()) {
                    variable.print_for_builtins(name,&mut stdout)
                }
                );
        }
        else {
            for assignment in assignments {
                if let IResult::Done(rest, (name , maybe_value)) =
                    script_parser::assignment_or_name(assignment.as_bytes()) {
                        let name = String::from_utf8_lossy(name);
                        if rest.is_empty() {
                            do_with_lock(
                                &mut bash,
                                &mut stderr,
                                |bash|
                                match bash.variables.find_variable_or_create_global( &name) {
                                    Ok(variable) => {
                                        variable.set_readonly(true);
                                        if let Some(value) = maybe_value {
                                            variable.set_value(&value);
                                        }
                                    }
                                    Err(e) => {
                                        use std::io::Write;
                                        write!(&mut stdout,
                                               "readonly: {}", e.readable("")).unwrap();
                                    }
                                }
                                );
                        }
                    }
            }
        }
    };
}
