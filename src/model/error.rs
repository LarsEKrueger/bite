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

//! Error codes for all bash operations

/// Errors codes
#[derive(Debug)]
pub enum Error {
    /// Tried to overwrite a read-only variable.
    VariableIsReadOnly(String),
    /// Unknown variable requests
    UnknownVariable(String),
    /// For some other reasons, we could not set the variable.
    CouldNotSetVariable(String),

    /// This is probably an implementation bug.
    InternalError(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn readable(self, suffix: &str) -> String {
        String::from(self.cause()) + " " + suffix
    }

    fn cause(self) -> String {
        match self {
            Error::VariableIsReadOnly(name) => {
                format!("tried to modify a read-only variable '{}'", name)
            }
            Error::UnknownVariable(name) => format!("tried to access unknown variable '{}'", name),
            Error::CouldNotSetVariable(name) => format!("failed to change variable '{}'", name),
            Error::InternalError(msg) => {
                format!("Internal error: {}\nReport to bugs@bugs.bugs", msg)
            }
        }
    }
}
