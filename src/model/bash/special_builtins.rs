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

use std::sync::mpsc::Sender;
use std::process::ExitStatus;
use std::os::unix::process::ExitStatusExt;

use argparse::{ArgumentParser, StoreTrue, List};
use super::*;
use super::execute::*;
use super::script_parser;

/// Runner function for export special builtin
///
/// export [-fn] [-p] [name[=value]]
pub fn export_runner(
    bash: &mut Bash,
    output_tx: &mut Sender<CommandOutput>,
    words: Vec<String>,
) -> ExitStatus {
    let mut negat = false;
    let mut funct = false;
    let mut print = false;

    let mut assignments: Vec<String> = vec![];
    let mut output = vec![];
    let mut errors = vec![];

    let mut ret_code = ExitStatus::from_raw(0);
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

        ap.parse(words, &mut output, &mut errors)
    } {
        if !negat && !funct && assignments.is_empty() {
            print = true;
        }
        if print {
            for (name,variable) in bash.variables.iter()
                .filter(|&(_,ref v)| v.is_exported()) {
                    variable.print_for_builtins(name,&mut output)
                }
        }
        else {
            for assignment in assignments {
                if let IResult::Done(rest, (name , maybe_value)) =
                    script_parser::assignment_or_name(assignment.as_bytes()) {
                        let name = String::from_utf8_lossy(name);
                        if rest.is_empty() {
                            match bash.variables.find_variable_or_create_global( &name) {
                                Ok(variable) => {
                                    variable.set_exported(!negat);
                                    if let Some(value) = maybe_value {
                                        variable.set_value(&value);
                                    }
                                }
                                Err(e) => {
                                    use std::io::Write;
                                    write!(&mut output, "export: {}", e.readable("")).unwrap();
                                    ret_code=ExitStatus::from_raw(1);
                                }
                            }
                        }
                    }
            }
        }
    };

    for l in String::from_utf8_lossy(&output[..])
        .lines()
        .map(String::from)
        .map(CommandOutput::FromOutput)
    {
        output_tx.send(l).unwrap();
    }
    for l in String::from_utf8_lossy(&errors[..])
        .lines()
        .map(String::from)
        .map(CommandOutput::FromError)
    {
        output_tx.send(l).unwrap();
    }
    ret_code
}

/// Runner function for readonly special builtin
///
/// # Parameters
///
/// readonly [-aAf] [-p] [name[=value]]
pub fn readonly_runner(
    bash: &mut Bash,
    output_tx: &mut Sender<CommandOutput>,
    words: Vec<String>,
) -> ExitStatus {
    let mut array = false;
    let mut assoc = false;
    let mut funct = false;
    let mut print = false;
    let mut assignments: Vec<String> = vec![];
    let mut output = vec![];
    let mut errors = vec![];

    let mut ret_code = ExitStatus::from_raw(0);
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

        ap.parse(words, &mut output, &mut errors)
    } {
        if !array && !assoc && !funct && assignments.is_empty() {
            print = true;
        }
        if print {
            for (name,variable) in bash.variables.iter()
                .filter(|&(_,ref v)| v.is_readonly()) {
                    variable.print_for_builtins(name,&mut output)
                }
        }
        else {
            for assignment in assignments {
                if let IResult::Done(rest, (name , maybe_value)) =
                    script_parser::assignment_or_name(assignment.as_bytes()) {
                        let name = String::from_utf8_lossy(name);
                        if rest.is_empty() {
                            match bash.variables.find_variable_or_create_global( &name) {
                                Ok(variable) => {
                                    variable.set_readonly(true);
                                    if let Some(value) = maybe_value {
                                        variable.set_value(&value);
                                    }
                                }
                                Err(e) => {
                                    use std::io::Write;
                                    write!(&mut output, "readonly: {}", e.readable("")).unwrap();
ret_code = ExitStatus::from_raw(1);
                                }
                            }
                        }
                    }
            }
        }
    };
    for l in String::from_utf8_lossy(&output[..])
        .lines()
        .map(String::from)
        .map(CommandOutput::FromOutput)
    {
        output_tx.send(l).unwrap();
    }
    for l in String::from_utf8_lossy(&errors[..])
        .lines()
        .map(String::from)
        .map(CommandOutput::FromError)
    {
        output_tx.send(l).unwrap();
    }
    ret_code
}
