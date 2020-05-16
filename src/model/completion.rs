/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2020  Lars Krüger

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

//! Complete the current command line

/// Find files that begin with `word`.
///
/// TODO: Filter files with known ignorable extensions
pub fn file_completion(word: &str) -> Vec<String> {
    // Remember if the word started with ./ because glob removes that.
    let dot_slash = if word.starts_with("./") { "./" } else { "" };

    // Find all files and folders that match '<word>*'
    if let Ok(g) = glob::glob(&(word.to_string() + "*")) {
        // Get the matches after word
        g.filter_map(std::result::Result::ok)
            .map(|path| {
                let mut p = dot_slash.to_string();
                p.push_str(&path.display().to_string());
                // If the path is a directory, add a slash.
                if path.is_dir() {
                    p.push_str("/");
                }
                p
            })
            .collect()
    } else {
        Vec::new()
    }
}
