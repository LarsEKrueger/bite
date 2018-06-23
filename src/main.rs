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
//! The software is designed as [`model`]-[`view`]-[`presenter`]. You can find the respective
//! components in their own modules.
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

extern crate bincode;

extern crate glob;

extern crate nix;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate bitflags;

use std::panic::PanicInfo;

pub mod tools;
pub mod model;
pub mod presenter;
pub mod view;

use model::bash::{bash_add_input, bite_write_output};

extern crate backtrace;

fn panic_hook(info: &PanicInfo) {
    let msg = match (info.payload().downcast_ref::<&str>(), info.location()) {
        (Some(msg), Some(loc)) => {
            format!(
                "bite panicked at {}:{}:{} with '{}'\n",
                loc.file(),
                loc.line(),
                loc.column(),
                msg
            )
        }
        (None, Some(loc)) => {
            format!(
                "bite panicked at {}:{}:{}\n",
                loc.file(),
                loc.line(),
                loc.column()
            )
        }
        _ => format!("bite panicked: {:?}\n", info),
    };
    bite_write_output(msg.as_str());

    let bt = backtrace::Backtrace::new();
    use std::fmt::Write;
    let mut msg = String::new();
    let _ = write!(msg, "{:?}", bt);
    bite_write_output(msg.as_str());
}

/// Main function that starts the program.
pub fn main() {
    let EMPTY = cstr!("");
    unsafe {
        ::libc::setlocale(::libc::LC_ALL, EMPTY.as_ptr());
    };

    let params = ::tools::commandline::CommandLine::parse();

    #[cfg(debug_assertions)]
    println!("Command Line\n{:?}", params);

    let (receiver, reader_barrier, bash_barrier) = match model::bash::start() {
        Err(err) => {
            println!("Can't start integrated bash: {}", err);
            ::std::process::exit(1);
        }
        Ok(r) => r,
    };

    let mut gui = match ::view::Gui::new(receiver) {
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

    std::panic::set_hook(Box::new(&panic_hook));

    reader_barrier.wait();
    bash_barrier.wait();

    gui.main_loop();
    gui.finish();

    // Make bash shutdown cleanly by killing a potentially running program and then telling bash to
    // exit, which will make it terminate its thread.
    // bash_kill_last();
    bash_add_input("exit 0");
    model::bash::stop();

    let _ = std::panic::take_hook();
}
