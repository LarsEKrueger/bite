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

//! Completions based on the grammar

use sesd::SymbolId;
use std::collections::HashMap;

use std::os::linux::fs::MetadataExt;

use crate::model::interpreter::grammar::script2;

/// Map a grammar symbol to the competler algo
type CompleterMap = HashMap<SymbolId, Completer>;

type CompleterAlgo = fn(&str) -> Vec<(String, String)>;

/// Possible completer algos
///
/// * TODO: glob + filter
/// * TODO  variables
enum Completer {
    /// A text template
    /// (completion,help)
    Text(&'static str, &'static str),

    /// Find the completions by running a function
    Code(CompleterAlgo),
}

pub struct Completions(CompleterMap);

/// Check if path is executable
///
/// This checks if the executable bit for *other* is set.
///
/// TODO: Do what -x does.
fn is_executable(path: &std::path::Path) -> bool {
    if let Ok(metadata) = path.metadata() {
        if (metadata.st_mode() & 0o001) != 0 {
            return true;
        }
    }
    false
}

/// Return a permission character if the mode has the permission
fn perm(c: char, mode: u32, permission: u32) -> char {
    if (mode & permission) != 0 {
        c
    } else {
        '-'
    }
}

/// Find files that begin with `word`.
///
/// TODO: Filter files with known ignorable extensions
fn file_completion(completion: &mut Vec<(String, String)>, prefix: &str, word: &str) {
    // Remember if the word started with ./ because glob removes that.
    let dot_slash = if word.starts_with("./") { "./" } else { "" };

    // Find all files and folders that match '<word>*'
    if let Ok(g) = glob::glob(&(word.to_string() + "*")) {
        // Get the matches after word
        for path in g.filter_map(std::result::Result::ok) {
            let mut p = prefix.to_string();
            p.push_str(dot_slash);
            p.push_str(&path.display().to_string());
            // If the path is a directory, add a slash.
            if path.is_dir() {
                p.push_str("/");
            }
            let help = if let Ok(metadata) = path.metadata() {
                let mode = metadata.st_mode();
                format!(
                    "{}{}{} {}{}{} {}{}{}",
                    perm('r', mode, 0o400),
                    perm('w', mode, 0o200),
                    perm('x', mode, 0o100),
                    perm('r', mode, 0o040),
                    perm('w', mode, 0o020),
                    perm('x', mode, 0o010),
                    perm('r', mode, 0o004),
                    perm('w', mode, 0o002),
                    perm('x', mode, 0o001)
                )
            } else {
                "\x1b[31m?????????\x1b[39m".to_string()
            };
            completion.push((p, help));
        }
    }
}

/// If a SIMPLE_COMMAND is to be completed, check if there is a space/tab/newline in the input. If
/// not, find all executables that begin with the given string.
///
/// Later, this will run a completion script in the interpreter.
fn simple_command_completion(start: &str) -> Vec<(String, String)> {
    trace!("simple_command_completion");
    let mut res = Vec::new();
    if !start.is_empty() {
        if let Some(last_space_pos) = start.rfind(&[' ', '\t', '\n'][..]) {
            trace!(
                "simple_command_completion: File completion after {:?}",
                &start[last_space_pos..]
            );
            if last_space_pos + 1 < start.len() {
                let prefix = &start[0..(last_space_pos + 1)];
                let param_start = &start[(last_space_pos + 1)..];
                file_completion(&mut res, prefix, param_start);
            }
        } else {
            // No space found in a non-empty start -> search for programs in path.
            // TODO: Handle names with . or / in them differently

            // Go through the PATH variable
            if let Some(paths) = std::env::var_os("PATH") {
                for path in std::env::split_paths(&paths) {
                    trace!("simple_commands check {:?}:", path);
                    if path.is_dir() {
                        trace!("simple_commands in {:?}:", path);
                        // Got through the files in that path
                        if let Ok(entries) = std::fs::read_dir(&path) {
                            for entry in entries {
                                if let Ok(entry) = entry {
                                    let path = entry.path();
                                    trace!("simple_commands found {:?}", path);
                                    // if the file is executable
                                    if is_executable(&path) {
                                        // If the file name starts with start, add it to the
                                        // completions
                                        let file_name = entry.file_name();
                                        if file_name
                                            .as_os_str()
                                            .to_string_lossy()
                                            .starts_with(start)
                                        {
                                            res.push((
                                                file_name.to_string_lossy().into_owned(),
                                                path.to_string_lossy().into_owned(),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    res
}

impl Completions {
    pub fn new() -> Self {
        let mut map = CompleterMap::new();

        map.insert(script2::FOR, Completer::Text("for", "begin a for loop"));
        map.insert(
            script2::FUNCTION,
            Completer::Text("function", "declare a function"),
        );

        // map.insert(script2::SIMPLE_COMMAND , 0, "ls "

        map.insert(
            script2::AND_GREATER_GREATER,
            Completer::Text("&>>", "append output and error to file"),
        );
        map.insert(
            script2::AND_GREATER,
            Completer::Text("&>", "redirect output and error to file, preferred form"),
        );
        map.insert(
            script2::GREATER_AND,
            Completer::Text(">&", "redirect output and error to file, obsolete form"),
        );

        map.insert(
            script2::LESS_AND,
            Completer::Text("<&", "duplicate file descriptor"),
        );
        map.insert(
            script2::LESS_LESS_LESS,
            Completer::Text("<<<", "send a \x1b[3mhere\x1b[23m string to input"),
        );
        map.insert(
            script2::LESS_LESS_MINUS,
            Completer::Text("<<-", "begin a \x1b[3mhere\x1b[23m document, remove tabs"),
        );
        map.insert(
            script2::LESS_LESS,
            Completer::Text("<<", "begin a \x1b[3mhere\x1b[23m document"),
        );
        map.insert(
            script2::LESS_GREATER,
            Completer::Text("<>", "open file for reading and writing"),
        );
        map.insert(
            script2::GREATER_BAR,
            Completer::Text(">|", "redirect output, overwrite file"),
        );
        map.insert(
            script2::GREATER_GREATER,
            Completer::Text(">>", "append output to file"),
        );
        map.insert(
            script2::LOGICAL_SEP_BG,
            Completer::Text("&", "start program on the left in background"),
        );
        map.insert(
            script2::LOGICAL_SEP_FG,
            Completer::Text(";", "start program on the left in foreground"),
        );
        map.insert(
            script2::OR_OR,
            Completer::Text(
                "||",
                "if program on the left fails, run program on the right",
            ),
        );
        map.insert(
            script2::AND_AND,
            Completer::Text(
                "&&",
                "if program on the left succeeds, run program on the right",
            ),
        );
        map.insert(
            script2::BAR_AND,
            Completer::Text(
                "|&",
                "pipe output and error of program on the left to the right",
            ),
        );
        map.insert(script2::WS, Completer::Text(" ", "separate the items"));

        map.insert(
            script2::SIMPLE_COMMAND,
            Completer::Code(simple_command_completion),
        );

        Self(map)
    }

    /// Given a symbol and a starting string, compute all possible completions.
    ///
    /// Return (complete, help)
    pub fn lookup(&self, sym: SymbolId, start: &str) -> Vec<(String, String)> {
        let mut res = Vec::new();
        if let Some(c) = self.0.get(&sym) {
            match c {
                Completer::Text(s, h) => {
                    res.push((s.to_string(), h.to_string()));
                }
                Completer::Code(fun) => {
                    return fun(start);
                }
            }
        }
        res
    }
}
