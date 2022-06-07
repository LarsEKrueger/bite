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

//! Build script for own code and bash wrapper.

extern crate cc;
extern crate rustc_version;

use rustc_version::{version, Version};
use std::io;
use std::io::Write;
use std::process;

fn main() {
    // Check the rust version first
    if let Ok(ver) = version() {
        if ver < Version::new(1, 37, 0) {
            let _ = writeln!(
                &mut io::stderr(),
                "bite requires rustc >= 1.37.0 to compile."
            );
            process::exit(1);
        }
    } else {
        let _ = writeln!(&mut io::stderr(), "Can't get rustc verion.");
        process::exit(1);
    }

    // Internal C code
    cc::Build::new()
        .file("c_src/myCreateIC.c")
        .compile("mystuff");
}
