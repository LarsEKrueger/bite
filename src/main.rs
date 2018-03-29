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

//! BiTE - Bash-integrated Terminal Emulator
//!
//! BiTE combines bash and xterm into one program.
//!
//! The software is designed as [`model`]-[`view`]-[`presenter`]. You can find the respective components in
//! their own modules.
//!
//! [`model`]: ../model/index.html
//! [`view`]: ../view/index.html
//! [`presenter`]: ../presenter/index.html

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate argparse;
extern crate libc;
extern crate time;
extern crate x11;
extern crate boolinator;

#[macro_use]
extern crate cstr;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate nom;

extern crate bincode;

extern crate glob;

pub mod tools;
pub mod model;
pub mod presenter;
pub mod view;

/// Main function that starts the program.
pub fn main() {
    let EMPTY = cstr!("");
    unsafe {
        ::libc::setlocale(::libc::LC_ALL, EMPTY.as_ptr());
    };

    let params = ::tools::commandline::CommandLine::parse();

    #[cfg(debug_assertions)]
    println!("Command Line\n{:?}", params);

    let mut gui = match ::view::Gui::new() {
        Err(err) => {
            println!("Can't init GUI: {}", err);
            ::std::process::exit(1);
        }
        Ok(g) => g,
    };

    //   if params.single_program.len() != 0 {
    //       session.new_interaction(params.single_program[0].clone());
    //       spawned = Some(execute::spawn_command(&params.single_program));
    //   }

    gui.main_loop();
    gui.finish();
}
