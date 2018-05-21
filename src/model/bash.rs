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

//! The interface to the underlying bash interpreter.
//!
//! Be aware that bash is full of global variables and must not be started more than once.

use libc::{c_char, c_int};

/// Start bash as a thread. Do not call more than once.
///
/// As bash is full of global variables and longjmps, we need to run its main function as a whole
/// in a thread.
pub fn start() {
    #[link(name = "Bash")]
    extern "C" {
        fn bash_main();
    }

    ::std::thread::spawn(|| unsafe { bash_main() });
}
