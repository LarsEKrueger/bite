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

//! Organizes the past and current programs and their outputs.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::process::ExitStatus;

use super::iterators::*;
use super::response::*;
use super::screen::{AddBytesResult, Matrix};

use std::sync::{Arc, Mutex};

/// Which output is visible.
///
/// The GUI concept dictates that at most one output (stdout or stderr) is visible.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OutputVisibility {
    None,
    Output,
    Error,
}

/// Running status of an interaction
#[derive(PartialEq, Debug, Clone)]
pub enum RunningStatus {
    Running,
    Unknown,
    Exited(ExitStatus),
}

/// A command and its output.
///
/// This is just a visual representation of a command and not connected to a running process in any
/// way.
#[derive(PartialEq)]
struct Interaction {
    /// Visual representation of the command that was called to create these responses
    command: Matrix,
    /// Collected stdout lines
    output: Response,
    /// Collected stderr lines
    errors: Response,
    /// Which response to show
    visible: OutputVisibility,
    /// status of the command
    running_status: RunningStatus,
}

/// A number of commands that are executed with the same prompt string.
struct Conversation {
    /// List of programs and their outputs for this prompt.
    interactions: Vec<InteractionHandle>,
    /// The prompt for this conversation.
    prompt: Matrix,
    /// Hash value of the prompt for displaying a color
    prompt_hash: u64,
}

/// An ordered list of conversations.
///
/// Conversations and Interactions are only supposed to be accessed through a session as their
/// indices need to be consistent.
pub struct Session {
    /// History of conversations, oldest first.
    conversations: Vec<Conversation>,

    /// History of interactions, oldest first.
    interactions: Vec<Interaction>,
}

/// Session that can be shared between threads
#[derive(Clone)]
pub struct SharedSession(Arc<Mutex<Session>>);

/// Index of an interaction in a session.
///
/// While there will be usually less than 2^64 interactions in a session, this is a usize to avoid
/// error handling now. Opening too many interactions will eat up all the memory before the program
/// runs out of indices.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct InteractionHandle(usize);

impl Interaction {
    /// Create a new command without any output yet.
    ///
    /// The command is a vector of cells as to support syntax coloring later.
    pub fn new(command: Matrix) -> Self {
        Self {
            command,
            output: Response::new(),
            errors: Response::new(),
            visible: OutputVisibility::Output,
            running_status: RunningStatus::Unknown,
        }
    }

    /// Get the visible response, if any.
    fn visible_response(&self) -> Option<&Response> {
        match self.visible {
            OutputVisibility::None => None,
            OutputVisibility::Output => Some(&self.output),
            OutputVisibility::Error => Some(&self.errors),
        }
    }

    /// Get the iterator over the items in this interaction.
    fn line_iter<'a>(
        &'a self,
        handle: InteractionHandle,
        prompt_hash: u64,
    ) -> impl Iterator<Item = LineItem<'a>> {
        // We always have the command, regardless if there is any output to show.
        let resp_lines = self
            .visible_response()
            .map(|r| r.line_iter(prompt_hash))
            .into_iter()
            .flat_map(|i| i);

        let visible = self.visible;
        let lt = LineType::Command(visible, handle, self.running_status.clone());

        self.command
            .line_iter()
            .map(move |r| LineItem::new(r, lt.clone(), None, prompt_hash))
            .chain(resp_lines)
    }

    /// Check if there are any error lines.
    fn has_errors(&self) -> bool {
        !self.errors.lines.is_empty()
    }

    /// Make the error lines visible
    pub fn show_errors(&mut self) {
        self.visible = OutputVisibility::Error;
    }

    /// If there are errors, show them.
    pub fn show_potential_errors(&mut self) {
        let failure = match self.running_status {
            RunningStatus::Exited(es) => !es.success(),
            _ => false,
        };
        if self.has_errors() || failure {
            self.show_errors();
        }
    }
}

impl Conversation {
    /// Creates a new conversation without any interactions.
    pub fn new(prompt: Matrix) -> Conversation {
        let mut h = DefaultHasher::new();
        prompt.hash(&mut h);
        let prompt_hash = h.finish();

        Conversation {
            prompt,
            interactions: vec![],
            prompt_hash,
        }
    }
}

impl Session {
    /// Create a new session.
    pub fn new(prompt: Matrix) -> Self {
        Session {
            conversations: vec![Conversation::new(prompt)],
            interactions: vec![],
        }
    }

    /// Open a new conversation if the prompts are different
    pub fn new_conversation(&mut self, prompt: Matrix) {
        if let Some(current) = self.conversations.last_mut() {
            if current.prompt == prompt {
                return;
            }
        }
        self.conversations.push(Conversation::new(prompt));
    }

    /// Return an iterator over the currently visible items.
    pub fn line_iter<'a>(&'a self, show_last_prompt: bool) -> impl Iterator<Item = LineItem<'a>> {
        let num_conversations = self.conversations.len();
        self.conversations
            .iter()
            .enumerate()
            .flat_map(move |(conversation_index, conversation)| {
                let prompt_hash = conversation.prompt_hash;
                let is_last_conv = (conversation_index + 1) >= num_conversations;
                let show_this_prompt = !is_last_conv || show_last_prompt;

                conversation
                    .interactions
                    .iter()
                    .flat_map(move |interHandle| {
                        self.interactions[interHandle.0].line_iter(*interHandle, prompt_hash)
                    })
                    .chain(
                        conversation
                            .prompt
                            .line_iter()
                            .map(move |r| LineItem::new(r, LineType::Prompt, None, prompt_hash))
                            .take_while(move |_| show_this_prompt),
                    )
            })
    }

    /// Add a new interaction to the latest conversation.
    pub fn add_interaction(&mut self, command: Matrix) -> InteractionHandle {
        let handle = InteractionHandle(self.interactions.len());
        self.interactions.push(Interaction::new(command));
        if let Some(current) = self.conversations.last_mut() {
            current.interactions.push(handle);
        }
        handle
    }

    /// Quick access to an interaction by handle.
    ///
    /// Returns the default for illegal handles.
    fn interaction_mut<F, R>(&mut self, handle: InteractionHandle, default: R, f: F) -> R
    where
        F: FnOnce(&mut Interaction) -> R,
    {
        if handle.0 < self.interactions.len() {
            f(&mut self.interactions[handle.0])
        } else {
            default
        }
    }

    /// Show the output of a given interaction
    pub fn show_output(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| i.visible = OutputVisibility::Output)
    }

    /// Show the errors of a given interaction
    pub fn show_errors(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), Interaction::show_errors)
    }

    /// Add bytes to the output of the given interaction
    pub fn add_output<'a>(
        &mut self,
        handle: InteractionHandle,
        bytes: &'a [u8],
    ) -> AddBytesResult<'a> {
        self.interaction_mut(handle, AddBytesResult::AllDone, |i| {
            i.output.add_bytes(bytes)
        })
    }

    /// Add bytes to the errors of the given interaction
    pub fn add_error<'a>(
        &mut self,
        handle: InteractionHandle,
        bytes: &'a [u8],
    ) -> AddBytesResult<'a> {
        self.interaction_mut(handle, AddBytesResult::AllDone, |i| {
            i.errors.add_bytes(bytes)
        })
    }

    /// Archive the given interaction
    pub fn archive_interaction(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            i.output.archive_screen();
            i.errors.archive_screen();
        })
    }

    /// Cycle the visibility of an interaction
    pub fn cycle_visibility(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            let v = match i.visible {
                OutputVisibility::Output => OutputVisibility::Error,
                OutputVisibility::Error => OutputVisibility::None,
                OutputVisibility::None => OutputVisibility::Output,
            };
            i.visible = v;
        })
    }

    /// Set the exit status of an interaction
    ///
    /// This is an obsolete method
    pub fn set_exit_status(&mut self, handle: InteractionHandle, status: ExitStatus) {
        self.interaction_mut(handle, (), |i| {
            i.running_status = RunningStatus::Exited(status);
            i.show_potential_errors();
        });
    }
}

impl SharedSession {
    /// Quick access to an interaction by handle.
    ///
    /// Returns the default if something goes wrong.
    fn interaction_mut<F, R>(&mut self, handle: InteractionHandle, default: R, f: F) -> R
    where
        F: FnOnce(&mut Interaction) -> R,
    {
        if let Ok(mut s) = self.0.lock() {
            s.interaction_mut(handle, default, f)
        } else {
            default
        }
    }

    /// Add bytes to error stream of interaction.
    ///
    /// Ignore the AddBytesResult and keep going.
    pub fn add_error_raw(&mut self, handle: InteractionHandle, bytes: &[u8]) {
        self.interaction_mut(handle, (), |i| i.errors.add_bytes_raw(bytes));
    }

    /// Add bytes to selected stream of interaction
    ///
    /// Ignore the AddBytesResult and keep going.
    pub fn add_bytes_raw(
        &mut self,
        stream: OutputVisibility,
        handle: InteractionHandle,
        bytes: &[u8],
    ) {
        self.interaction_mut(handle, (), |i| match stream {
            OutputVisibility::None => {}
            OutputVisibility::Output => i.output.add_bytes_raw(bytes),
            OutputVisibility::Error => i.errors.add_bytes_raw(bytes),
        });
    }

    /// Set the running status of an interaction
    pub fn set_running_status(&mut self, handle: InteractionHandle, status: RunningStatus) {
        self.interaction_mut(handle, (), |i| {
            i.running_status = status;
            i.show_potential_errors();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::response::tests::check;
    use super::super::screen::Screen;
    use super::*;

    fn new_test_session(prompt: &[u8]) -> Session {
        Session::new(Screen::one_line_matrix(prompt))
    }

    #[test]
    fn line_iter() {
        let mut session = new_test_session(b"prompt 1");

        let inter_1_1 = session.add_interaction(Screen::one_line_matrix(b"command 1.1"));
        session.add_output(inter_1_1, b"output 1.1.1\noutput 1.1.2\n");
        let inter_1_2 = session.add_interaction(Screen::one_line_matrix(b"command 1.2"));
        session.add_output(inter_1_2, b"output 1.2.1\noutput 1.2.2\n");

        session.new_conversation(Screen::one_line_matrix(b"prompt 2"));
        let inter_2_1 = session.add_interaction(Screen::one_line_matrix(b"command 2.1"));
        session.add_output(inter_2_1, b"output 2.1.1\noutput 2.1.2\n");
        let inter_2_2 = session.add_interaction(Screen::one_line_matrix(b"command 2.2"));
        session.add_output(inter_2_2, b"output 2.2.1\noutput 2.2.2\n");

        assert_eq!(session.conversations.len(), 2);
        assert_eq!(session.conversations[0].interactions.len(), 2);
        assert_eq!(session.conversations[1].interactions.len(), 2);

        let mut li = session.line_iter(true);
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 0 },
                RunningStatus::Unknown,
            ),
            None,
            "command 1.1",
        );
        check(li.next(), LineType::Output, None, "output 1.1.1");
        check(li.next(), LineType::Output, None, "output 1.1.2");
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 1 },
                RunningStatus::Unknown,
            ),
            None,
            "command 1.2",
        );
        check(li.next(), LineType::Output, None, "output 1.2.1");
        check(li.next(), LineType::Output, None, "output 1.2.2");
        check(li.next(), LineType::Prompt, None, "prompt 1");

        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 2 },
                RunningStatus::Unknown,
            ),
            None,
            "command 2.1",
        );
        check(li.next(), LineType::Output, None, "output 2.1.1");
        check(li.next(), LineType::Output, None, "output 2.1.2");
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 3 },
                RunningStatus::Unknown,
            ),
            None,
            "command 2.2",
        );
        check(li.next(), LineType::Output, None, "output 2.2.1");
        check(li.next(), LineType::Output, None, "output 2.2.2");
        check(li.next(), LineType::Prompt, None, "prompt 2");
        assert_eq!(li.next(), None);
    }

}
