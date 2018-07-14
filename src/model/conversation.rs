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

//! A conversation is a number of commands run with the same prompt string.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::iterators::*;
use super::interaction::{ArchivedInteraction, CommandPosition};
use super::screen::Matrix;

/// A number of commands that are executed with the same prompt string.
pub struct Conversation {
    /// List of programs and their outputs for this prompt.
    pub interactions: Vec<ArchivedInteraction>,
    /// The prompt for this conversation.
    pub prompt: Matrix,

    /// Hash value of the prompt for displaying a color
    prompt_hash: u64,
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

    /// Add an interaction to the conversation.
    pub fn add_interaction(&mut self, interaction: ArchivedInteraction) {
        self.interactions.push(interaction);
    }

    /// Return an iterator for this conversation.
    ///
    /// The provided CommandPosition is that of the conversation.
    pub fn line_iter<'a>(&'a self, pos: CommandPosition) -> impl Iterator<Item = LineItem<'a>> {
        let prompt_hash = self.prompt_hash;
        self.interactions
            .iter()
            .zip(pos.conv_iter())
            .flat_map(move |(inter, index)| inter.line_iter(index, prompt_hash))
            .chain(self.prompt.line_iter().map(move |r| {
                LineItem::new(r, LineType::Prompt, None, prompt_hash)
            }))
    }

    /// Return the hash value of the prompt
    pub fn prompt_hash(&self) -> u64 {
        self.prompt_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::response::tests::check;
    use model::interaction::OutputVisibility;
    use model::interaction::tests::{test_add_output, test_add_error};
    use super::super::screen::Screen;

    #[test]
    fn line_iter() {
        let mut conv = Conversation::new(Screen::one_line_matrix(b"prompt"));
        let mut inter_1_1 = ArchivedInteraction::new(Screen::one_line_matrix(b"command 1.1"));
        test_add_output(&mut inter_1_1, b"output 1.1.1\noutput 1.1.2\n");
        conv.add_interaction(inter_1_1);
        let mut inter_1_2 = ArchivedInteraction::new(Screen::one_line_matrix(b"command 1.2"));
        test_add_error(&mut inter_1_2, b"error 1.2.1\nerror 1.2.2\n");
        inter_1_2.output.visible = false;
        inter_1_2.errors.visible = true;
        conv.add_interaction(inter_1_2);

        let mut li = conv.line_iter(CommandPosition::Archived(0, 0));
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
                OutputVisibility::Error,
                CommandPosition::Archived(0, 1),
                None,
            ),
            None,
            "command 1.2",
        );
        check(li.next(), LineType::Output, None, "error 1.2.1");
        check(li.next(), LineType::Output, None, "error 1.2.2");
        check(li.next(), LineType::Prompt, None, "prompt");
        assert_eq!(li.next(), None);
    }

}
