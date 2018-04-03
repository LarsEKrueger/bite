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

//! Generic types to be used in the model.

/// Command as returned by the bash parser.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Parser demands more input
    Incomplete,
    /// Parsing error
    Error(Vec<String>),
    /// No command, e.g. due to empty input
    None,
    /// Command expression.
    ///
    /// This is always a tree of maximal depth 2. Level 0 corresponds to commands separated by &
    /// and ;. Level 1 corresponds to commands separated by && and ||. Commands at a level are
    /// evaluated left to right.
    Expression(Vec<CommandTerm>),
}

/// A list of commands and the reaction to them.
#[derive(Debug, PartialEq)]
pub struct CommandTerm {
    pub commands: Vec<CommandInfo>,
}

/// Command and flags.
#[derive(Debug, PartialEq)]
pub struct CommandInfo {
    pub words: Vec<String>,
    pub reaction: CommandReaction,
    pub invert: bool,
}

/// How to react on the failure of a command
#[derive(Debug, PartialEq)]
pub enum CommandReaction {
    /// Execute the next command
    Normal,
    /// Send to background
    Background,
    /// Short-cut AND
    And,
    /// Short-cut OR
    Or,
}

/// Assignment part of a command
#[derive(Debug, PartialEq)]
pub struct Assignment {
    /// name of the variable to assign
    pub name: String,
    /// Value to be assigned
    pub value: String,

    // TODO: Assignment operation (assign or add)
}

/// The structure that comes from parsing an expansion
pub type Expansion = Vec<ExpSpan>;

/// A segment that can be expanded.
#[derive(Debug, PartialEq)]
pub enum ExpSpan {
    /// Copy this string
    Verbatim(String),

    /// Add the content of this variable.
    ///
    /// TODO: Add operator
    Variable(String),

    /// Add $HOME
    Tilde,

    /// Data for bracket expansion
    Bracket(Vec<String>),

    /// Add file names
    Glob(String),
}

impl Command {
    pub fn new_expression(
        mut terms: Vec<CommandTerm>,
        last_reaction: Option<CommandReaction>,
    ) -> Self {
        if let Some(last_reaction) = last_reaction {
            terms.last_mut().map(|ct| {
                ct.commands.last_mut().map(
                    |ci| ci.set_reaction(last_reaction),
                )
            });
        };
        Command::Expression(terms)
    }
}

impl Assignment {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

impl CommandTerm {
    pub fn new(commands: Vec<CommandInfo>) -> Self {
        Self { commands }
    }

    //./ Set the reaction of the last CommandInfo.
    pub fn set_reaction(&mut self, reaction: CommandReaction) {
        self.commands.last_mut().map(|ci| ci.set_reaction(reaction));
    }
}

impl CommandInfo {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            words,
            reaction: CommandReaction::Normal,
            invert: false,
        }
    }

    pub fn set_reaction(&mut self, reaction: CommandReaction) {
        self.reaction = reaction;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_new_expression() {
        assert_eq!(
            Command::new_expression(
                vec![CommandTerm::new(vec![
                                      CommandInfo::new(vec![String::from("ab")]),
                                      CommandInfo::new(vec![String::from("bc")])
                ])],
                Some(CommandReaction::Background)
                ),
            Command::Expression(
                vec![CommandTerm {
                    commands:vec![CommandInfo {
                        words: vec![String::from("ab")],
                        reaction: CommandReaction::Normal,
                        invert : false
                    }, CommandInfo {
                        words: vec![String::from("bc")],
                        reaction: CommandReaction::Background,
                        invert : false
                    }
                    ],
                }]),
        );
    }

}
