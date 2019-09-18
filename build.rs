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
//!
//! This is based on https://github.com/gpg-rs/libgcrypt

extern crate gcc;
extern crate rustc_version;

use rustc_version::{version, Version};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, Stdio};
use std::str;

fn main() {
    // Check the rust version first
    if let Ok(ver) = version() {
        if ver < Version::new(1, 26, 0) {
            let _ = writeln!(
                &mut io::stderr(),
                "bite requires rustc >= 1.26.0 to compile."
            );
            process::exit(1);
        }
    } else {
        let _ = writeln!(&mut io::stderr(), "Can't get rustc verion.");
        process::exit(1);
    }

    // Internal C code
    gcc::Build::new()
        .file("c_src/myCreateIC.c")
        .compile("mystuff");

    // Bash-as-a-library

    // Download bash source if it's not there
    if !Path::new("c_src/bash/configure").exists() {
        run(Command::new("git").args(&["submodule", "update", "--init"]));

        // Patch source
        run(Command::new("git").args(&["apply", "c_src/bash.patch", "--directory=c_src/bash"]));
    }

    // Build bash
    if let Some(_build) = try_build() {
        return;
    }

    process::exit(1);
}

fn spawn(cmd: &mut Command) -> Option<Child> {
    println!("running: {:?}", cmd);
    match cmd.stdin(Stdio::null()).spawn() {
        Ok(child) => Some(child),
        Err(e) => {
            println!("failed to execute command: {:?}\nerror: {}", cmd, e);
            None
        }
    }
}

fn run(cmd: &mut Command) -> bool {
    if let Some(mut child) = spawn(cmd) {
        match child.wait() {
            Ok(status) => {
                if !status.success() {
                    println!(
                        "command did not execute successfully: {:?}\n\
                         expected success, got: {}",
                        cmd, status
                    );
                } else {
                    return true;
                }
            }
            Err(e) => {
                println!("failed to execute command: {:?}\nerror: {}", cmd, e);
            }
        }
    }
    false
}

fn try_build() -> Option<PathBuf> {
    let src = PathBuf::from(env::current_dir().unwrap())
        .join("c_src")
        .join("bash");
    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let build = dst.clone().join("build");
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    let compiler = gcc::Build::new().get_compiler();
    let mut cflags = compiler.args().iter().fold(OsString::new(), |mut c, a| {
        c.push(a);
        c.push(" ");
        c
    });

    cflags.push("-fPIC");

    let _ = fs::create_dir_all(&build);

    // Run configure if required
    if !build.clone().join("Makefile").exists() {
        if !run(Command::new("sh")
            .current_dir(&build)
            .env("CC", compiler.path())
            .env("CFLAGS", &cflags)
            .arg(src.join("configure"))
            .args(&[
                "--build",
                &gnu_target(&host),
                "--host",
                &gnu_target(&target),
                &format!("--prefix={}", msys_compatible(&dst)),
                "--without-bash-malloc",
                "--disable-readline",
            ]))
        {
            return None;
        }
    }

    if !run(Command::new("make")
        .current_dir(&build)
        .arg("-j")
        .arg(env::var("NUM_JOBS").unwrap())
        .arg("libBash.a"))
    {
        return None;
    }

    println!("cargo:rustc-link-search=native={}", build.display());
    println!("cargo:rustc-link-lib=curses");
    println!("cargo:rustc-link-lib=dl");
    Some(build)
}

fn msys_compatible<P: AsRef<Path>>(path: P) -> String {
    let mut path = path.as_ref().to_string_lossy().into_owned();
    if !cfg!(windows) || Path::new(&path).is_relative() {
        return path;
    }

    let mut is_letter = false;
    if let Some(c) = path.as_bytes().first() {
        match c {
            b'a'...b'z' | b'A'...b'Z' => {
                is_letter = true;
            }
            _ => {}
        }
    }
    if is_letter {
        if path.split_at(1).1.starts_with(":\\") {
            (&mut path[..1]).make_ascii_lowercase();
            path.remove(1);
            path.insert(0, '/');
        }
    }
    path.replace("\\", "/")
}

fn gnu_target(target: &str) -> &str {
    match target {
        "i686-pc-windows-gnu" => "i686-w64-mingw32",
        "x86_64-pc-windows-gnu" => "x86_64-w64-mingw32",
        s => s,
    }
}
