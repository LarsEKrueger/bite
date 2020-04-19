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

//! Locate a specific line in a session
//!
//! This module does not prescribe an order in which the lines are to be rendered.

/// Describe where in which part of a response the line is.
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseLocator {
    Lines(usize),
    Screen(usize),
}

/// Describe where in which part of an interaction the line is.
#[derive(Clone, Debug, PartialEq)]
pub enum InteractionLocator {
    Command(usize),
    Tui(usize),
    /// Index of the line in the response. Which response (output, error, visible) is used, is up
    /// to to caller.
    Response(ResponseLocator),
}

/// Describe where in which part of a converation the line is.
#[derive(Clone, Debug, PartialEq)]
pub enum ConversationLocator {
    /// Index of interaction and position in it.
    Interaction(usize, InteractionLocator),
    Prompt(usize),
}

/// Describe the location of a line in a session.
///
/// This assumes that responses are only extended at the end to keep referring to the same line
/// when the locator doesn't change its value.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionLocator {
    /// Index of conversation
    pub conversation: usize,

    /// Where in the conversation is the line
    pub in_conversation: ConversationLocator,
}

/// Common return type
pub type MaybeSessionLocator = Option<SessionLocator>;

impl SessionLocator {
    /// Decrement the line parameter in each element by at most `lines`.
    ///
    /// `lines` is updated by the respective amount.
    pub fn dec_line(&mut self, lines: &mut usize) {
        match self.in_conversation {
            ConversationLocator::Prompt(ref mut line)
            | ConversationLocator::Interaction(_, InteractionLocator::Command(ref mut line))
            | ConversationLocator::Interaction(_, InteractionLocator::Tui(ref mut line))
            | ConversationLocator::Interaction(
                _,
                InteractionLocator::Response(ResponseLocator::Lines(ref mut line)),
            )
            | ConversationLocator::Interaction(
                _,
                InteractionLocator::Response(ResponseLocator::Screen(ref mut line)),
            ) => {
                if lines <= line {
                    *line -= *lines;
                    *lines = 0;
                } else {
                    *lines -= *line;
                    *line = 0;
                }
            }
        }
    }

    /// Check if the `line` parameter is zero.
    ///
    /// Return true if line is zero.
    pub fn is_start_line(&self) -> bool {
        match self.in_conversation {
            ConversationLocator::Prompt(line)
            | ConversationLocator::Interaction(_, InteractionLocator::Command(line))
            | ConversationLocator::Interaction(_, InteractionLocator::Tui(line))
            | ConversationLocator::Interaction(
                _,
                InteractionLocator::Response(ResponseLocator::Lines(line)),
            )
            | ConversationLocator::Interaction(
                _,
                InteractionLocator::Response(ResponseLocator::Screen(line)),
            ) => line == 0,
        }
    }
}
