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

type CompleterAlgo = fn(&mut LookupState, usize, &str) -> Vec<(String, String)>;

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
fn file_completion(start: usize, word: &str) -> Vec<(usize, String, String)> {
    let mut completion = Vec::new();
    // Remember if the word started with ./ because glob removes that.
    let dot_slash = if word.starts_with("./") { "./" } else { "" };

    // Find all files and folders that match '<word>*'
    if let Ok(g) = glob::glob(&(word.to_string() + "*")) {
        // Get the matches after word
        for path in g.filter_map(std::result::Result::ok) {
            let mut p = dot_slash.to_string();
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
            completion.push((start, p, help));
        }
    }
    completion
}

/// Keep track of the starting positions of elements to complete.
///
/// This struct is used to compensate the fact that the CST is processed bottom-up. Thus, the
/// SIMPLE_COMMAND_ELEMENT to complete is seen before the COMMAND.
///
/// In order to distinguish whether the command is to be complete or an argument, the starting
/// position of the last SIMPLE_COMMAND_ELEMENT as to be tracked as well as the starting position
/// of the last COMMAND. If both starting positions are identical, the command name is being
/// completed. If not, a parameter is being completed.
///
/// In addition, it needs to be tracked if the SIMPLE_COMMAND_ELEMENT/COMMAND pair has been seen at
/// all.
pub struct LookupState {
    /// Starting position of the last SIMPLE_COMMAND_ELEMENT.
    simple_command_element_pos: Option<usize>,
    /// Starting position of the last COMMAND.
    command_pos: Option<usize>,
}

/// If a SIMPLE_COMMAND is to be completed, check if there is a space/tab/newline in the input. If
/// not, find all executables that begin with the given string.
///
/// Later, this will run a completion script in the interpreter.
fn simple_command_completion(start: usize, word: &str) -> Vec<(usize, String, String)> {
    trace!("simple_command_completion");
    let mut res = Vec::new();
    // Go through the PATH variable
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            if path.is_dir() {
                debug!("simple_commands in dir {:?}:", path);
                // Got through the files in that path
                if let Ok(entries) = std::fs::read_dir(&path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            // if the file is executable
                            if is_executable(&path) {
                                // If the file name starts with start, add it to the
                                // completions
                                let file_name = entry.file_name();
                                if file_name.as_os_str().to_string_lossy().starts_with(word) {
                                    res.push((
                                        start,
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
    res
}

fn simple_command_element_update(
    state: &mut LookupState,
    pos: usize,
    _start: &str,
) -> Vec<(String, String)> {
    state.simple_command_element_pos = state
        .simple_command_element_pos
        .map_or(Some(pos), |x| Some(std::cmp::max(x, pos)));

    Vec::new()
}

fn command_update(state: &mut LookupState, pos: usize, _start: &str) -> Vec<(String, String)> {
    state.command_pos = state
        .command_pos
        .map_or(Some(pos), |x| Some(std::cmp::max(x, pos)));

    Vec::new()
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
            script2::SIMPLE_COMMAND_ELEMENT,
            Completer::Code(simple_command_element_update),
        );

        map.insert(script2::COMMAND, Completer::Code(command_update));

        Self(map)
    }

    /// Begin tracking a new CST traversal
    pub fn begin(&self) -> LookupState {
        LookupState {
            simple_command_element_pos: None,
            command_pos: None,
        }
    }

    /// Given a symbol and a starting string, compute all possible completions.
    ///
    /// Return (complete, help)
    pub fn lookup(
        &self,
        state: &mut LookupState,
        sym: SymbolId,
        pos: usize,
        start: &str,
    ) -> Vec<(String, String)> {
        let mut res = Vec::new();
        if let Some(c) = self.0.get(&sym) {
            match c {
                Completer::Text(s, h) => {
                    res.push((s.to_string(), h.to_string()));
                }
                Completer::Code(fun) => {
                    return fun(state, pos, start);
                }
            }
        }
        res
    }

    /// Complete the CST traversal. Emit all completions.
    pub fn end(
        &self,
        state: &LookupState,
        editor: &crate::presenter::Editor,
        cursor_position: usize,
    ) -> Vec<(usize, String, String)> {
        if let Some(simple_command_element_pos) = state.simple_command_element_pos {
            if let Some(command_pos) = state.command_pos {
                let word = editor.span_string(simple_command_element_pos, cursor_position);
                if simple_command_element_pos == command_pos {
                    // complete command
                    return simple_command_completion(simple_command_element_pos, &word);
                } else {
                    // Complete file
                    return file_completion(simple_command_element_pos, &word);
                }
            }
        }

        Vec::new()
    }
}
