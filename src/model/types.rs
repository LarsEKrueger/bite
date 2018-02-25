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

//! Generic types to be used in the model.

/// Command as returned by the bash parser.
#[derive(Debug, PartialEq)]
pub enum Command {
    Incomplete,
    Error(Vec<String>),
    SimpleCommand(SimpleCommand),
}

/// Simple command can have assignements for temporary variables and the words that make up the
/// actual command.
///
/// Either of the two can be empty.
#[derive(Debug, PartialEq)]
pub struct SimpleCommand {
    pub assignments: Vec<Assignment>,
    pub words: Vec<String>,
}

/// Assignment part of a command
#[derive(Debug, PartialEq)]
pub struct Assignment {
    /// name of the variable to assign
    pub name: String,
    /// Value to be assigned
    pub value: String,

    // TODO: Assignment operation (assign or add)
}


impl Command {
    pub fn new_simple_command(assignments: Vec<Assignment>, words: Vec<String>) -> Self {
        Command::SimpleCommand(SimpleCommand { assignments, words })

    }
}


impl Assignment {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
