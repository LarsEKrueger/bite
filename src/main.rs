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

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate argparse;
extern crate libc;
extern crate time;
extern crate x11;

#[macro_use]
extern crate cstr;

#[macro_use]
extern crate nom;

mod session;
mod commandline;
mod polling;
mod execute;
mod runeline;
mod bash;
mod gui;

fn main() {
    let EMPTY = cstr!("");
    unsafe {
        libc::setlocale(libc::LC_ALL, EMPTY.as_ptr());
    };

    let params = commandline::CommandLine::parse();

    #[cfg(debug_assertions)]
    println!("Command Line\n{:?}", params);

    let mut gui = match gui::Gui::new() {
        Err(err) => {
            println!("Can't init GUI: {}", err);
            std::process::exit(1);
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
