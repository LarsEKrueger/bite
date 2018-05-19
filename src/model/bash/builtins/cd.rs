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

use super::*;
use std::os::unix::io::{FromRawFd, RawFd};

/// Runner function for cd special builtin
///
/// cd [-L | [-P [-e]] -@ ] [dir]
pub fn cd_runner(
    mut bash: Arc<Mutex<Bash>>,
    stdin: RawFd,
    stdout: RawFd,
    stderr: RawFd,
    words: Vec<String>,
) -> ExitStatus {
    let mut force_link = true;
    let mut exit_nonzero = false;
    let mut dir = String::new();

    // Put this in a file to auto-close it on exit.
    let _stdin = unsafe { File::from_raw_fd(stdin) };
    let mut stdout = unsafe { File::from_raw_fd(stdout) };
    let mut stderr = unsafe { File::from_raw_fd(stderr) };

    let mut exit_status = ExitStatus::from_raw(0);

    if let Ok(()) = {
        let mut ap = ArgumentParser::new();
        ap.set_description("cd - builtin");
        // TODO: Better help text
        ap.refer(&mut force_link).add_option(
            &["-L"],
            StoreTrue,
            concat!(
                "force symbolic links to be followed: resolve symbolic\n",
                "links in DIR after processing instances of »..«"),
        )
        .add_option(
            &["-P"],
            StoreFalse,
            concat!(
                "use the physical directory structure without following\n",
                "symbolic links: resolve symbolic links in DIR before\n",
                "processing instances of »..«")
        );
        ap.refer(&mut exit_nonzero).add_option(
            &["-e"],
            StoreTrue,
            concat!(
                "if the -P option is supplied, and the current working\n",
                "directory cannot be determined successfully, exit with\n",
                "a non-zero status"),
        );
        ap.refer(&mut dir).add_argument(
            "dir",
            Store,
            concat!(
                "Change the current directory to DIR.\n",
                "The default DIR is the value of the HOME shell variable."),
        );

        ap.parse(words, &mut stdout, &mut stderr)
    } {
        if let Some(es) = do_with_lock_err(
            &mut bash,
            &mut stderr,
            "cd: ",
            |bash| {
                if dir.is_empty() {
                    dir=bash.get_current_user_home_dir().to_string();
                    Ok(())
                } else if dir == "-" {
                    dir = bash.variable.variable_as_str("OLDPWD");
                }
            }
            )
            {
                exit_status = es;
            }

        use std::io::Write;
        write!( stdout, "changing dir to »{}«\n", dir);

    exit_status
}
