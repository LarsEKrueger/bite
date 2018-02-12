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

use std::iter;

use super::iterators::*;
use super::interaction::*;

// A number of commands that are executed in the same folder.
pub struct Conversation {
    pub interactions: Vec<Interaction>,
    prompt: String,
}

impl Conversation {
    pub fn new(prompt: String) -> Conversation {
        Conversation {
            prompt,
            interactions: vec![],
        }
    }

    pub fn add_interaction(&mut self, interaction: Interaction) {
        self.interactions.push(interaction);
    }

    pub fn line_iter<'a>(&'a self, pos: CommandPosition) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.interactions
                .iter()
                .zip(pos.conv_iter())
                .flat_map(|(inter, index)| inter.line_iter(index))
                .chain(iter::once(
                    LineItem::new(&self.prompt, LineType::Prompt, None),
                )),
        )
    }

    #[allow(dead_code)]
    pub fn hide_output(&mut self) {
        for i in self.interactions.iter_mut() {
            i.hide_output();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_iter() {
        let mut conv = Conversation::new(String::from("prompt"));
        let mut inter_1_1 = Interaction::new(String::from("command 1.1"));
        inter_1_1.add_output(String::from("output 1.1.1"));
        inter_1_1.add_output(String::from("output 1.1.2"));
        conv.add_interaction(inter_1_1);
        let mut inter_1_2 = Interaction::new(String::from("command 1.2"));
        inter_1_2.add_error(String::from("error 1.2.1"));
        inter_1_2.add_error(String::from("error 1.2.2"));
        inter_1_2.output.visible = false;
        inter_1_2.errors.visible = true;
        conv.add_interaction(inter_1_2);

        let mut li = conv.line_iter(CommandPosition::Archived(0, 0));
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "command 1.1",
                is_a: LineType::Command(OutputVisibility::Output, CommandPosition::Archived(0, 0)),
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 1.1.1",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 1.1.2",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "command 1.2",
                is_a: LineType::Command(OutputVisibility::Error, CommandPosition::Archived(0, 1)),
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "error 1.2.1",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "error 1.2.2",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "prompt",
                is_a: LineType::Prompt,
                cursor_col: None,
            })
        );
        assert_eq!(li.next(), None);
    }

}
