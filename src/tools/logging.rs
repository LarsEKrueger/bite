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

//! Tools that help with logging

/// Unwrap a Result, print debug message and return default value in case of error.
pub fn unwrap_log<T, E>(result: Result<T, E>, desc: &str, default: T) -> T
where
    E: core::fmt::Debug,
{
    match result {
        Err(e) => {
            debug!("BiTE: {}, due to »{:?}«", desc, e);
            default
        }
        Ok(x) => x,
    }
}
