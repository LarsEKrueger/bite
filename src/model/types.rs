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
pub enum ParsedCommand {
    /// Parser demands more input
    Incomplete,
    /// Parsing error
    Error(Vec<String>),
    /// No command, e.g. due to empty input
    None,
    /// Command sequence.
    ///
    /// This is always a tree of maximal depth 3. Level 0 corresponds to the commands in a
    /// pipeline. Level 1 corresponds to commands separated by & and ;. Level 2 corresponds to
    /// commands separated by && and ||. Commands at a level are evaluated left to right.
    CommandSequence(Vec<CommandLogic>),
}

/// A list of commands as
#[derive(Debug, PartialEq)]
pub struct CommandLogic {
    pub pipelines: Vec<Pipeline>,
}

/// A pipeline
#[derive(Debug, PartialEq)]
pub struct Pipeline {
    pub commands: Vec<Command>,
    pub reaction: CommandReaction,
    pub invert: bool,
}

/// A command and its parameters
#[derive(Debug, PartialEq)]
pub struct Command {
    pub words: Vec<String>,
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

impl ParsedCommand {
    pub fn new_sequence(
        mut logic: Vec<CommandLogic>,
        last_reaction: Option<CommandReaction>,
    ) -> Self {
        if let Some(last_reaction) = last_reaction {
            logic.last_mut().map(|exp| {
                exp.pipelines.last_mut().map(
                    |pi| pi.set_reaction(last_reaction),
                )
            });
        };
        ParsedCommand::CommandSequence(logic)
    }
}

impl Assignment {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

impl CommandLogic {
    pub fn new(pipelines: Vec<Pipeline>) -> Self {
        Self { pipelines }
    }

    //./ Set the reaction of the last CommandInfo.
    pub fn set_reaction(&mut self, reaction: CommandReaction) {
        self.pipelines.last_mut().map(
            |pi| pi.set_reaction(reaction),
        );
    }
}

impl Pipeline {
    pub fn new(command: Command) -> Self {
        Self {
            commands: vec![command],
            reaction: CommandReaction::Normal,
            invert: false,
        }
    }

    pub fn set_reaction(&mut self, reaction: CommandReaction) {
        self.reaction = reaction;
    }
}

impl Command {
    pub fn new(words: Vec<String>) -> Self {
        Self { words }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence() {
        assert_eq!(
            ParsedCommand::new_sequence(
                vec![CommandLogic::new(vec![
                                      Pipeline::new(Command::new(vec![String::from("ab")])),
                                      Pipeline::new(Command::new( vec![String::from("bc")]))
                ])],
                Some(CommandReaction::Background)
                ),
            ParsedCommand::CommandSequence(
                vec![CommandLogic {
                    pipelines:vec![Pipeline {
                        commands : vec![
                        Command { words: vec![String::from("ab")] },
                        ],
                        reaction: CommandReaction::Normal,
                        invert : false
                    }, Pipeline {
                        commands : vec![
                        Command { words: vec![String::from("bc")] },
                        ],
                        reaction: CommandReaction::Background,
                        invert : false
                    }
                    ],
                }]),
        );
    }
}
