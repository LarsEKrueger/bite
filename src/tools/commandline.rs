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

//! Handles all sources of configuration.
//!
//! Currently only reads the command line arguments.

use argparse::{ArgumentParser, List};

/// All parameters passed to the executable on the command line.
#[derive(Debug)]
pub struct CommandLine {
    pub single_program: Vec<String>,
}

impl CommandLine {
    /// Parse the command line arguments and fill a CommandLine struct.
    pub fn parse() -> CommandLine {
        let mut result = CommandLine { single_program: vec![] };
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("BiTE - Bash-Integrated Terminal Emulator");
            ap.refer(&mut result.single_program).add_option(
                &["-e"],
                List,
                "Single program to be run. bite will exit after it is completed.",
            );
            ap.parse_args_or_exit();
        }

        result
    }
}
