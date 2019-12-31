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

use super::interaction::*;
use super::iterators::*;
use super::screen::{AddBytesResult, Matrix};

/// A number of commands that are executed with the same prompt string.
pub struct Conversation {
    /// List of programs and their outputs for this prompt.
    pub interactions: Vec<InteractionHandle>,
    /// The prompt for this conversation.
    pub prompt: Matrix,
    /// Hash value of the prompt for displaying a color
    prompt_hash: u64,
}

/// An ordered list of conversations.
///
/// Conversations and Interactions are only supposed to be accessed through a session as their
/// indices need to be consistent.
pub struct Session {
    /// History of conversations, oldest first.
    pub conversations: Vec<Conversation>,

    /// History of interactions, oldest first.
    pub interactions: Vec<Interaction>,
}

/// Index of an interaction in a session.
///
/// While there will be usually less than 2^64 interactions in a session, this is a usize to avoid
/// error handling now. Opening too many interactions will eat up all the memory before the program
/// runs out of indices.
#[derive(Clone,Copy)]
pub struct InteractionHandle(usize);

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

    /// Add an interaction to the conversation.
    pub fn add_interaction(&mut self, interaction: InteractionHandle) {
        self.interactions.push(interaction);
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
    pub fn line_iter<'a>(&'a self) -> impl Iterator<Item = LineItem<'a>> {
        self.conversations.iter().flat_map(move |c| {
            let prompt_hash = c.prompt_hash;
            c.interactions
                .iter()
                .flat_map(move |interHandle| {
                    let inter : &'a Interaction  = &(self.interactions[interHandle.0]);
                    inter.line_iter(prompt_hash)
                })
                .chain(
                    c.prompt
                        .line_iter()
                        .map(move |r| LineItem::new(r, LineType::Prompt, None, prompt_hash)),
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
    /// Does nothing for illegal handles.
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

    /// Set the exit status of an interaction
    pub fn set_exit_status(&mut self, handle: InteractionHandle, exit_status: ExitStatus) {
        self.interaction_mut(handle, (), |i| i.set_exit_status(exit_status))
    }

    /// Show the output of a given interaction
    pub fn show_output(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), Interaction::show_output)
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
        self.interaction_mut(handle, AddBytesResult::AllDone, |i| i.add_output(bytes))
    }

    /// Add bytes to the errors of the given interaction
    pub fn add_error<'a>(
        &mut self,
        handle: InteractionHandle,
        bytes: &'a [u8],
    ) -> AddBytesResult<'a> {
        self.interaction_mut(handle, AddBytesResult::AllDone, |i| i.add_error(bytes))
    }

    /// Archive the given interaction
    pub fn archive_interaction( &mut self, handle: InteractionHandle) {
        self.interaction_mut( handle, (), Interaction::archive)
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
        session.add_output( inter_2_1, b"output 2.1.1\noutput 2.1.2\n");
        let inter_2_2 = session.add_interaction(Screen::one_line_matrix(b"command 2.2"));
        session.add_output(inter_2_2, b"output 2.2.1\noutput 2.2.2\n");

        assert_eq!(session.conversations.len(), 2);
        assert_eq!(session.conversations[0].interactions.len(), 2);
        assert_eq!(session.conversations[1].interactions.len(), 2);

        let mut li = session.line_iter();
        check(
            li.next(),
            LineType::Command(OutputVisibility::Output, None),
            None,
            "command 1.1",
        );
        check(li.next(), LineType::Output, None, "output 1.1.1");
        check(li.next(), LineType::Output, None, "output 1.1.2");
        check(
            li.next(),
            LineType::Command(OutputVisibility::Output, None),
            None,
            "command 1.2",
        );
        check(li.next(), LineType::Output, None, "output 1.2.1");
        check(li.next(), LineType::Output, None, "output 1.2.2");
        check(li.next(), LineType::Prompt, None, "prompt 1");

        check(
            li.next(),
            LineType::Command(OutputVisibility::Output, None),
            None,
            "command 2.1",
        );
        check(li.next(), LineType::Output, None, "output 2.1.1");
        check(li.next(), LineType::Output, None, "output 2.1.2");
        check(
            li.next(),
            LineType::Command(OutputVisibility::Output, None),
            None,
            "command 2.2",
        );
        check(li.next(), LineType::Output, None, "output 2.2.1");
        check(li.next(), LineType::Output, None, "output 2.2.2");
        check(li.next(), LineType::Prompt, None, "prompt 2");
        assert_eq!(li.next(), None);
    }

}
