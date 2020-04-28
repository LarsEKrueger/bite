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

//! Helper functions to work with files that start with a format-describing header.

use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// Create a file and write the file format header
pub fn create<P: AsRef<Path>>(path: P, format_string: &str) -> Result<File> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let format_string_len = format_string.len();
    let file_format_len = file.write(format_string.as_bytes())?;

    if file_format_len != format_string_len {
        return Err(Error::new(ErrorKind::Other, "Could not write header"));
    }

    Ok(file)
}

/// Open a file and check if it begins with the desired header
pub fn open<P: AsRef<Path>>(path: P, format_string: &str) -> Result<File> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_EXCL)
        .open(path)?;

    let format_string_len = format_string.len();
    // Read the first format_string_len bytes and compare them to format_string
    let mut file_format: Vec<u8> = Vec::with_capacity(format_string_len);
    file_format.resize(format_string_len, 0);

    let file_format_len = file.read(file_format.as_mut_slice())?;
    if file_format_len != format_string_len {
        return Err(Error::new(ErrorKind::InvalidInput, "Could not read header"));
    }
    if file_format != format_string.as_bytes() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "File header does not match",
        ));
    }

    Ok(file)
}
