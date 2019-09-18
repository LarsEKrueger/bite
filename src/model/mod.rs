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

//! The model component of the model-view-presenter pattern.
//!
//! All modules deal with the bash script interpreter, either interactively or non-interactively.

pub mod bash;
pub mod control_sequence;
pub mod conversation;
pub mod error;
pub mod history;
pub mod interaction;
pub mod iterators;
pub mod response;
pub mod screen;
pub mod session;
pub mod types;
