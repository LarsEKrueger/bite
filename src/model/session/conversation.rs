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

//! Organizes the output of programs with the same prompt

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::InteractionHandle;
use model::screen::Matrix;

/// A number of commands that are executed with the same prompt string.
pub struct Conversation {
    /// List of programs and their outputs for this prompt.
    pub interactions: Vec<InteractionHandle>,
    /// The prompt for this conversation.
    pub prompt: Matrix,
    /// Hash value of the prompt for displaying a color
    pub prompt_hash: u64,
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
