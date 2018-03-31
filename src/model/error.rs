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
    /// Illegal pattern in globbing
    IllegalGlob(String),

    /// This is probably an implementation bug.
    InternalError(&'static str, u32, String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn readable(self, suffix: &str) -> String {
        String::from(self.cause(suffix))
    }

    fn cause(self, suffix: &str) -> String {
        match self {
            Error::VariableIsReadOnly(name) => {
                format!("tried to modify a read-only variable '{}' {}", name, suffix)
            }
            Error::UnknownVariable(name) => {
                format!("tried to access unknown variable '{}' {}", name, suffix)
            }
            Error::CouldNotSetVariable(name) => {
                format!("failed to change variable '{}' {}", name, suffix)
            }
            Error::IllegalGlob(msg) => format!("illegal pattern '{}' {}", msg, suffix),
            Error::InternalError(file, line, msg) => {
                format!(
                    concat!(
                        "Internal error '{}' in {}:{}\n",
                        "Report at https://github.com/LarsEKrueger/bite/issues"
                    ),
                    msg,
                    file,
                    line
                )
            }
        }
    }
}
