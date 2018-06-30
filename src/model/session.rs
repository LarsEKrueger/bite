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

use super::conversation::*;
use super::interaction::*;
use super::iterators::*;
use super::screen::Matrix;

/// A number of closed conversations and the current one
pub struct Session {
    /// Archived conversations
    pub archived: Vec<Conversation>,
    /// Current conversation
    pub current_conversation: Conversation,
}

impl Session {
    /// Create a new session with its own interpreter.
    pub fn new(prompt: Matrix) -> Self {
        Session {
            archived: vec![],
            current_conversation: Conversation::new(prompt),
        }
    }

    /// Open a new conversation and archive the current one.
    pub fn new_conversation(&mut self, prompt: Matrix) {
        use std::mem;
        let cur = mem::replace(&mut self.current_conversation, Conversation::new(prompt));
        self.archived.push(cur);
    }

    /// Move the given interaction to the list of completed interactions.
    pub fn archive_interaction(&mut self, interaction: Interaction) {
        self.current_conversation.add_interaction(interaction);
    }

    /// Find the interaction that is referenced by the command position.
    ///
    /// # Errors
    ///
    /// Assumes that the CommandPosition was generated by line_iter.
    pub fn find_interaction_from_command<'a>(
        &'a mut self,
        pos: CommandPosition,
    ) -> Option<&'a mut Interaction> {
        match pos {
            CommandPosition::Archived(conv_index, inter_index) => {
                Some(&mut self.archived[conv_index].interactions[inter_index])
            }
            CommandPosition::CurrentConversation(inter_index) => {
                Some(&mut self.current_conversation.interactions[inter_index])
            }
            CommandPosition::CurrentInteraction => None,
        }
    }

    /// Return an iterator over the currently visible items.
    pub fn line_iter<'a>(&'a self) -> impl Iterator<Item = LineItem<'a>> {
        self.archived
            .iter()
            .zip(CommandPosition::archive_iter())
            .flat_map(|(c, pos)| c.line_iter(pos))
            .chain(self.current_conversation.line_iter(
                CommandPosition::CurrentConversation(0),
            ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::response::tests::check;
    use super::super::screen::Screen;

    fn new_test_session(prompt: &[u8]) -> Session {
        Session {
            archived: vec![],
            current_conversation: Conversation::new(Screen::one_line_matrix(prompt)),
        }
    }

    #[test]
    fn line_iter() {
        let mut session = new_test_session(b"prompt 1");

        let mut inter_1_1 = Interaction::new(Screen::one_line_cell_vec(b"command 1.1"));
        inter_1_1.add_output(b"output 1.1.1\r\n");
        inter_1_1.add_output(b"output 1.1.2\r\n");
        session.archive_interaction(inter_1_1);
        let mut inter_1_2 = Interaction::new(Screen::one_line_cell_vec(b"command 1.2"));
        inter_1_2.add_output(b"output 1.2.1\r\n");
        inter_1_2.add_output(b"output 1.2.2\r\n");
        session.archive_interaction(inter_1_2);

        session.new_conversation(Screen::one_line_matrix(b"prompt 2"));
        let mut inter_2_1 = Interaction::new(Screen::one_line_cell_vec(b"command 2.1"));
        inter_2_1.add_output(b"output 2.1.1\r\n");
        inter_2_1.add_output(b"output 2.1.2\r\n");
        session.archive_interaction(inter_2_1);
        let mut inter_2_2 = Interaction::new(Screen::one_line_cell_vec(b"command 2.2"));
        inter_2_2.add_output(b"output 2.2.1\r\n");
        inter_2_2.add_output(b"output 2.2.2\r\n");
        session.archive_interaction(inter_2_2);

        assert_eq!(session.archived.len(), 1);
        assert_eq!(session.archived[0].interactions.len(), 2);
        assert_eq!(session.current_conversation.interactions.len(), 2);

        let mut li = session.line_iter();
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                CommandPosition::Archived(0, 0),
                None,
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
                CommandPosition::Archived(0, 1),
                None,
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
                CommandPosition::CurrentConversation(0),
                None,
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
                CommandPosition::CurrentConversation(1),
                None,
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
