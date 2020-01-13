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
#![recursion_limit = "128"]

extern crate argparse;
extern crate boolinator;
extern crate libc;
extern crate time;
extern crate x11;

#[macro_use]
extern crate cstr;

#[macro_use]
extern crate lazy_static;

extern crate bincode;

extern crate glob;

extern crate nix;

extern crate nom;
extern crate nom_locate;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;
extern crate flexi_logger;

extern crate term;
extern crate termios;

use std::panic::PanicInfo;

pub mod model;
pub mod presenter;
pub mod tools;
pub mod view;

extern crate backtrace;

fn panic_hook(info: &PanicInfo) {
    let err_msg = match (info.payload().downcast_ref::<&str>(), info.location()) {
        (Some(msg), Some(loc)) => {
            error!(
                "Panic at {}:{}:{} with '{}'",
                loc.file(),
                loc.line(),
                loc.column(),
                msg
            );
            format!(
                "bite panicked at {}:{}:{} with '{}'\n",
                loc.file(),
                loc.line(),
                loc.column(),
                msg
            )
        }
        (None, Some(loc)) => {
            error!("Panic at {}:{}:{}", loc.file(), loc.line(), loc.column());
            format!(
                "bite panicked at {}:{}:{}\n",
                loc.file(),
                loc.line(),
                loc.column()
            )
        }
        _ => format!("bite panicked: {:?}\n", info),
    };
    bite_write_output(err_msg.as_str());

    let bt = backtrace::Backtrace::new();
    use std::fmt::Write;
    let mut msg = String::new();
    let _ = write!(msg, "{:?}", bt);
    bite_write_output(msg.as_str());
    error!("Error:{}\nStack Trace:\n{}", err_msg, msg);
}

/// Main function that starts the program.
pub fn main() {
    // Initialise env_logger first
    let _ = std::env::var("BITE_LOG").and_then(|bite_log| {
        let _ = flexi_logger::Logger::with_str(bite_log)
            .format(flexi_logger::with_thread)
            .log_to_file()
            .start();
        info!("Logging is ready");
        Ok(())
    });

    // Set up locale
    {
        let EMPTY = cstr!("");
        unsafe {
            ::libc::setlocale(::libc::LC_ALL, EMPTY.as_ptr());
        };
    }

    let params = ::tools::commandline::CommandLine::parse();

    #[cfg(debug_assertions)]
    info!("{:?}", params);

    // Create the session
    let session =
        model::session::SharedSession::new(model::screen::Screen::one_line_matrix(b"System"));

    // Start interpreter in a thread
    let interpreter = model::interpreter::Interpreter::new(session.clone());

    // Start the gui
    let mut gui = match ::view::Gui::new(session, interpreter) {
        Err(err) => {
            error!("Can't init GUI: {}", err);
            println!("Can't init GUI: {}", err);
            ::std::process::exit(1);
        }
        Ok(g) => g,
    };

    //   if params.single_program.len() != 0 {
    //       session.new_interaction(params.single_program[0].clone());
    //       spawned = Some(execute::spawn_command(&params.single_program));
    //   }

    // Write any panic messages to both log and the term bite was started from. Needs to be called
    // after bash::start.
    std::panic::set_hook(Box::new(&panic_hook));

    // Run the gui loop until the program is closed
    gui.main_loop();
    let interpreter = gui.finish();

    // Shutdown interpreter and wait for it to end
    interpreter.shutdown();

    let _ = std::panic::take_hook();
    info!("Exiting bite normally");
}
