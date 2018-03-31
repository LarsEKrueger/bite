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

//! Bash script interpreter.
//!
//! This is the basic component for a non-interactive bash interpreter. The interactive part in
//! handled in [`Session`].
//!
//! [`Session`]: ../session/struct.Session.html

use nom::IResult;
use std::env;
use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::collections::HashMap;
use libc::{c_char, gethostname, gid_t, uid_t};
use std::sync::mpsc::{Receiver, Sender};

pub mod script_parser;
pub mod expansion_parser;
pub mod prompt_parser;
pub mod history;
pub mod execute;
pub mod special_builtins;

use super::types::*;
use super::error::*;
use super::variables;
use self::special_builtins::*;

/// All relevant info about the current user.
///
/// This is the same structure as in the C version.
struct UserInfo {
    /// User ID
    uid: uid_t,

    /// Group ID
    _gid: gid_t,

    /// Account Name
    name: String,

    /// equivalent to $HOME
    home_dir: String,

    /// Path of the defaul shell to execute
    _shell: String,
}

/// Output and error lines of a builtin command
pub struct BuiltinOutput {
    pub output: Vec<String>,
    pub errors: Vec<String>,
}

/// Result of spawning a new command.
pub enum ExecutionResult {
    Ignore,
    Spawned((Sender<String>, Receiver<execute::CommandOutput>)),
    Builtin(BuiltinOutput),
    Internal,
    Err(String),
}

/// Function for builtin commands
type BuiltinRunner = fn(&mut Bash, Vec<String>) -> Result<BuiltinOutput>;

/// Complete interpreter state.
pub struct Bash {
    /// Name of the computer this program is running.
    current_host_name: String,

    /// Info about the user that runs this program.
    current_user: UserInfo,

    /// Accumulate source in this string until it parses completely.
    ///
    /// This is required as NOM cannot continue a partial parse.
    line: String,

    /// List of all lines we have successfully parsed.
    pub history: history::History,

    /// Stack of variables
    pub variables: variables::Stack,
}

const VERSION: &'static str = "0.0";
const PATCHLEVEL: &'static str = "0";

const FALLBACK_HOSTNAME: &'static str = "??host??";

/// Convert a parsing error message into a readable format, similar to Rust's messages.
fn format_error_message(error: ::nom::Err<&[u8]>, line: &String) -> Vec<String> {
    let mut msg = line.clone();

    match error {
        ::nom::Err::NodePosition(_, p, _) |
        ::nom::Err::Position(_, p) => {
            let l_ptr = line.as_ptr();
            let p_ptr = p.as_ptr();
            let o = p_ptr as usize - l_ptr as usize;
            for _ in 0..o {
                msg.push_str(" ");
            }
            msg.push_str("^---- Syntax error");
        }
        _ => msg.push_str("Syntax error"),
    };

    msg.lines().map(String::from).collect()
}


/// Table of special builtin commands and their runners
lazy_static!{
    static ref SPECIAL_BUILTINS: HashMap<&'static str, BuiltinRunner> = {
        let mut map: HashMap<&'static str, BuiltinRunner> = HashMap::new();
        map.insert("export", export_runner);
        map.insert("readonly", readonly_runner);
        map
    };
}

impl Bash {
    /// Create a new bash script interpreter.
    pub fn new() -> Result<Self> {
        // It's highly unlikely that this will change.
        let current_host_name = {
            // Host names are at most 255 bytes long, plus the zero.
            let len: usize = 256;
            let mut buf: Vec<u8> = vec![0; len];

            let err = unsafe { gethostname(buf.as_mut_ptr() as *mut c_char, len) };
            if err == 0 {
                // find the first 0 byte (i.e. just after the data that gethostname wrote)
                let actual_len = buf.iter().position(|byte| *byte == 0).unwrap_or(len);

                String::from(::std::str::from_utf8(&buf[..actual_len]).unwrap_or(
                    FALLBACK_HOSTNAME,
                ))
            } else {
                String::from(FALLBACK_HOSTNAME)
            }
        };

        // We get the user right here, because we are almost always an interactive shell.
        let current_user = unsafe {
            let amt = match ::libc::sysconf(::libc::_SC_GETPW_R_SIZE_MAX) {
                n if n < 0 => 512 as usize,
                n => n as usize,
            };
            let mut buf = Vec::with_capacity(amt);
            let mut passwd: ::libc::passwd = mem::zeroed();
            let mut result = ptr::null_mut();
            match ::libc::getpwuid_r(
                ::libc::getuid(),
                &mut passwd,
                buf.as_mut_ptr(),
                buf.capacity(),
                &mut result,
            ) {
                0 if !result.is_null() => UserInfo {
                    uid: passwd.pw_uid,
                    _gid: passwd.pw_gid,
                    name: CStr::from_ptr(passwd.pw_name as *const c_char)
                        .to_string_lossy()
                        .into_owned(),
                    home_dir: CStr::from_ptr(passwd.pw_dir as *const c_char)
                        .to_string_lossy()
                        .into_owned(),
                    _shell: CStr::from_ptr(passwd.pw_shell as *const c_char)
                        .to_string_lossy()
                        .into_owned(),
                },
                // TODO: Return this as an error
                _ => UserInfo {
                    uid: ::std::u32::MAX,
                    _gid: ::std::u32::MAX,
                    name: String::from("I have no name"),
                    home_dir: String::from("I have no home"),
                    _shell: String::from("I have no shell"),
                },
            }
        };

        let history = history::History::new(&current_user.home_dir);

        let mut variables = variables::Stack::new();
        variables.import_from_environment()?;

        Ok(Self {
            line: String::new(),
            current_host_name,
            current_user,
            history,
            variables,
        })
    }

    /// Returns the version string for display.
    pub fn version() -> String {
        String::from(VERSION)
    }

    /// Returns the detailed version string for display.
    pub fn version_and_patchlevel() -> String {
        let mut s = String::from(VERSION);
        s.push_str(".");
        s.push_str(&PATCHLEVEL);
        s
    }

    /// Accepts another line and try to parse it.
    ///
    /// If the parse was successful and complete, return a command to be executed.
    ///
    /// If the parse failed, indicate so.
    pub fn add_line(&mut self, l: &str) -> Command {
        // Append the line to the last one and try to (re-)parse.
        self.line.push_str(l);

        // Keep the line in a local variable for a moment
        let line = ::std::mem::replace(&mut self.line, String::new());
        let command = {
            let bytes = line.as_bytes();
            match script_parser::parse_script(bytes) {
                IResult::Incomplete(_) => Command::Incomplete,
                IResult::Error(e) => Command::Error(format_error_message(e, &line)),
                IResult::Done(_, o) => o,
            }
        };
        match command {
            Command::Incomplete => {
                // Put line back
                self.line = line;
            }
            Command::Error(_) => {}
            _ => {}
        }
        command
    }

    /// Parses $PS1 and returns the generated string.
    pub fn expand_ps1(&self) -> String {
        // Get the string from the environment.
        // TODO: Get it from own variables.

        let ps1_string = env::var("PS1").unwrap_or(String::from("\\s-\\v\\$ "));

        self.decode_prompt_string(ps1_string.as_str())
    }

    /// Parses $PS1 and returns the generated string.
    fn decode_prompt_string(&self, input: &str) -> String {
        // For the moment use nom to parse the string until proven too slow.
        match prompt_parser::parse_prompt(input.as_bytes(), self) {
            IResult::Done(_, s) => s,
            _ => String::from("$ "),
        }
    }

    /// Reads the host name from the interpreter for display purposes.
    pub fn get_current_host_name<'a>(&'a self) -> &'a str {
        &self.current_host_name
    }

    /// Reads the account name of the user from the interpreter for display purposes.
    pub fn get_current_user_name<'a>(&'a self) -> &'a str {
        &self.current_user.name
    }

    /// Reads the path of the home directory from the interpreter for display purposes.
    #[allow(dead_code)]
    pub fn get_current_user_home_dir<'a>(&'a self) -> &'a str {
        &self.current_user.home_dir
    }

    /// Checks if the current user is root.
    pub fn current_user_is_root(&self) -> bool {
        self.current_user.uid == 0
    }

    /// Execute a command.
    ///
    /// Ignore any error cases.
    pub fn execute(&mut self, cmd: Command) -> ExecutionResult {
        match cmd {
            Command::Incomplete |
            Command::Error(_) => ExecutionResult::Ignore,
            Command::None => ExecutionResult::Ignore,
            Command::SimpleCommand(ci) => {
                let words = match self.expand_word_list(ci.words) {
                    Ok(w) => w,
                    Err(e) => {
                        return ExecutionResult::Err(e.readable("during expansion"));
                    }
                };
                let (assignments, words) = self.separate_out_assignments(words);

                // If there is no command, perform the assignments to global variables. If not,
                // perform them to temporary context, execute and drop the temporary context.
                if words.is_empty() {
                    match self.assign_to_global_context(assignments) {
                        Ok(_) => ExecutionResult::Internal,
                        Err(e) => ExecutionResult::Err(e.readable("while setting variables")),
                    }
                } else {
                    match self.assign_to_temp_context(assignments) {
                        Ok(_) => {
                            let res = self.execute_simple_command(words);
                            self.drop_temp_context();
                            res
                        }
                        Err(e) => ExecutionResult::Err(
                            e.readable("while setting temporary variables"),
                        ),

                    }
                }
            }
            Command::Expression(_exp) => {
                // TODO: Run the expression.
                ExecutionResult::Err(
                    Error::InternalError(file!(), line!(), String::from("not implemented"))
                        .readable(""),
                )
            }
        }
    }

    /// Execute a builtin command or spawn a thread to run an external command
    fn execute_simple_command(&mut self, words: Vec<String>) -> ExecutionResult {
        if words.is_empty() {
            ExecutionResult::Internal
        } else {
            // Check if this a special builtin command
            if let Some(builtin) = SPECIAL_BUILTINS.get(words[0].as_str()) {
                match self.run_builtin(*builtin, words) {
                    Ok(r) => ExecutionResult::Builtin(r),
                    Err(e) => ExecutionResult::Err(e.readable("")),
                }
            } else {
                // Spawn the command if there is one
                match execute::spawn_command(&words[..], self.variables.iter_exported()) {
                    Ok(r) => ExecutionResult::Spawned(r),
                    Err(e) => ExecutionResult::Err(e),
                }
            }
        }
    }

    /// Run a builtin command and retrieve its output and error lines
    fn run_builtin(&mut self, runner: BuiltinRunner, ps: Vec<String>) -> Result<BuiltinOutput> {
        runner(self, ps)
    }

    /// Assign variables in global context
    fn assign_to_global_context(&mut self, assignments: Vec<Assignment>) -> Result<()> {
        let global = self.variables.get_global_context()?;
        for assignment in assignments {
            global.bind_variable(&assignment.name, &assignment.value)?;
        }
        Ok(())
    }

    /// Assign variables in temporary context
    fn assign_to_temp_context(&mut self, assignments: Vec<Assignment>) -> Result<()> {
        let context = self.variables.create_temp_context();
        for assignment in assignments {
            context
                .bind_variable(&assignment.name, &assignment.value)?
                .set_exported(true);
        }
        Ok(())
    }

    /// Drop a temporary context.
    fn drop_temp_context(&mut self) {
        self.variables.drop_temp_context();
    }

    /// Expand all words in the list.
    fn expand_word_list(&self, words: Vec<String>) -> Result<Vec<String>> {
        // TODO: use map
        let mut out_words = vec![];
        for w in words {
            self.expand_word(&mut out_words, w)?;
        }
        Ok(out_words)
    }

    /// Expand a single word
    fn expand_word(&self, out_words: &mut Vec<String>, word: String) -> Result<()> {
        match expansion_parser::expansion(word.as_bytes()) {
            IResult::Done(_, exp) => self.rebuild_expansion(out_words, exp)?,
            IResult::Error(_) |
            IResult::Incomplete(_) => out_words.push(word),
        };
        Ok(())
    }

    /// Add all expanded combinations to out_words.
    ///
    /// Only bracket expansion and globbing can produce multiple outputs. We perform globbing last.
    /// Bracket expansion is done using the classic outer-product indexing.
    /// As bracket expansion is quite rare, we perform a test to simplify the indexing.
    fn rebuild_expansion(&self, out_words: &mut Vec<String>, exp: Expansion) -> Result<()> {
        // bracket_idx [(alternatives, current)]
        let mut bracket_idx: Vec<(usize, usize)> = exp.iter()
            .filter_map(|s| if let ExpSpan::Bracket(ref v) = *s {
                Some((v.len(), 0 as usize))
            } else {
                None
            })
            .collect();

        // Outer-product indexing,
        loop {
            // Concat the items and then glob.
            let (pat, has_glob) = self.expand(&exp, &bracket_idx)?;
            if has_glob {
                Bash::glob2words(out_words, pat)?;
            } else {
                out_words.push(pat);
            }

            // Now increment
            let mut i_bracket_idx = 0;
            while i_bracket_idx < bracket_idx.len() {
                let ref mut bii = bracket_idx[i_bracket_idx];
                if bii.1 + 1 < bii.0 {
                    bii.1 += 1;
                    break;
                }
                bii.1 = 0;
                i_bracket_idx += 1;
            }
            if i_bracket_idx == bracket_idx.len() {
                break;
            }
        }
        Ok(())
    }

    fn expand(&self, exp: &Expansion, bracket_idx: &Vec<(usize, usize)>) -> Result<(String, bool)> {
        let mut result = String::new();
        let mut has_glob = false;

        // Index into bracket_idx
        let mut i_bracket_idx = 0;
        for i in 0..exp.len() {
            match exp[i] {
                ExpSpan::Verbatim(ref s) => result.push_str(s),
                ExpSpan::Variable(ref n) => {
                    let v = self.variables.variable_as_str(n.as_str())?;
                    result.push_str(v);
                }
                ExpSpan::Tilde => result.push_str(self.current_user.home_dir.as_str()),
                ExpSpan::Bracket(ref v) => {
                    if i_bracket_idx >= bracket_idx.len() {
                        return Err(Error::InternalError(
                            file!(),
                            line!(),
                            format!(
                                "could not index into bracket index (exp=»{:?}«<<,bracket_idx=»{:?}«,i={},i_bracket_idx={})",
                                exp,
                                bracket_idx,
                                i,
                                i_bracket_idx
                            ),
                        ));
                    }

                    let idx = bracket_idx[i_bracket_idx].1;
                    if idx >= v.len() {
                        return Err(Error::InternalError(
                            file!(),
                            line!(),
                            format!(
                                "could not index into bracket vector (exp=»{:?}«<<,bracket_idx=»{:?}«,i={},i_bracket_idx={})",
                                exp,
                                bracket_idx,
                                i,
                                i_bracket_idx
                            ),
                        ));
                    }
                    result.push_str(v[idx].as_str());
                    i_bracket_idx += 1;
                }
                ExpSpan::Glob(ref g) => {
                    has_glob = true;
                    result.push_str(g)
                }
            }
        }
        return Ok((result, has_glob));
    }

    fn glob2words(out_words: &mut Vec<String>, pat: String) -> Result<()> {
        use glob::glob;

        for entry in glob(pat.as_str()).map_err(|pe| {
            Error::IllegalGlob(String::from(pe.msg))
        })?
        {
            match entry {
                Ok(path) => out_words.push(path.to_string_lossy().into_owned()),
                _ => {}
            }
        }
        Ok(())
    }

    /// Split the word list into assignments and regular words.
    ///
    /// This is an instance method as we need to access the shell flags later.
    fn separate_out_assignments(&self, words: Vec<String>) -> (Vec<Assignment>, Vec<String>) {
        let mut out_words = vec![];
        let mut out_assignments = vec![];

        // Iterate over the words and make assignments. Break the loop at the first non-assignment.
        let mut i = words.into_iter();
        loop {
            match i.next() {
                None => break,
                Some(w) => {
                    // Parse the word as an assignment.
                    match script_parser::assignment(w.as_bytes()) {
                        IResult::Done(_, a) => out_assignments.push(a),
                        IResult::Error(_) |
                        IResult::Incomplete(_) => {
                            out_words.push(w);
                            break;
                        }
                    }

                }
            }
        }

        // Move the remaining words.
        loop {
            match i.next() {
                None => break,
                Some(w) => out_words.push(w),
            }
        }
        (out_assignments, out_words)
    }
}
