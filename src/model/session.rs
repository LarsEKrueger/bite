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

use std::sync::mpsc::{Receiver, Sender};

use super::bash;
use super::conversation::*;
use super::interaction::*;
use super::iterators::*;
use super::types::*;
use super::error::*;
use super::execute;

/// A number of closed conversations and the current one
pub struct Session {
    /// Archived conversations
    pub archived: Vec<Conversation>,
    /// Current conversation
    pub current_conversation: Conversation,
    /// Current interaction
    current_interaction:
        Option<(Sender<String>, Receiver<execute::CommandOutput>, Interaction)>,

    /// Bash script interpreter.
    pub bash: bash::Bash,
}

impl Session {
    /// Create a new session with its own interpreter.
    pub fn new() -> Result<Self> {
        let bash = bash::Bash::new()?;

        Ok(Session {
            current_interaction: None,
            archived: vec![],
            current_conversation: Conversation::new(bash.expand_ps1()),
            bash,
        })
    }

    /// Open a new conversation and archive the current one.
    pub fn new_conversation(&mut self, prompt: String) {
        use std::mem;
        let cur = mem::replace(&mut self.current_conversation, Conversation::new(prompt));
        self.archived.push(cur);
    }

    /// Move the given interaction to the list of completed interactions.
    pub fn archive_interaction(&mut self, interaction: Interaction) {
        self.current_conversation.add_interaction(interaction);
    }

    /// Poll the channels to retrieve input from a running program.
    pub fn poll_interaction(&mut self) -> bool {
        let mut clear_spawned = false;
        let mut needs_marking = false;
        if let Some((_, ref cmd_output, ref mut inter)) = self.current_interaction {
            if let Ok(line) = cmd_output.try_recv() {
                needs_marking = true;
                match line {
                    execute::CommandOutput::FromOutput(line) => {
                        inter.add_output(line);
                    }
                    execute::CommandOutput::FromError(line) => {
                        inter.add_error(line);
                    }
                    execute::CommandOutput::Terminated(_exit_code) => {
                        // TODO: show the exit code if there is an error
                        clear_spawned = true;
                    }
                }
            }
        }
        if clear_spawned {
            if let Some((_, _, mut inter)) =
                ::std::mem::replace(&mut self.current_interaction, None)
            {
                inter.prepare_archiving();
                self.archive_interaction(inter);
            }
        }
        needs_marking
    }

    /// Process a line that the user has entered.
    ///
    /// If a program is running, send it to its stdin. If not, send it to the interpreter.
    pub fn add_line(&mut self, line: String) {
        let mut line_ret = line.clone();
        line_ret.push_str("\n");

        // If we have a current interaction, send it the line. Otherwise try to run the command.
        match self.current_interaction {
            Some((ref tx, _, ref mut inter)) => {
                inter.add_output(line.clone());
                // Send current_line to running program
                tx.send(line + "\n").unwrap();
            }
            None => {
                let cmd = self.bash.add_line(line_ret.as_str());
                match cmd {
                    Command::Incomplete => {}
                    Command::Error(err) => {
                        // Parser error. Create a fake interaction with the bad command line and
                        // the error message
                        let mut inter = Interaction::new(line);
                        for l in err.into_iter() {
                            inter.add_error(l);
                        }
                        inter.prepare_archiving();
                        self.archive_interaction(inter);
                    }
                    Command::SimpleCommand(v) => {
                        // Add to history
                        self.bash.history.add_command(line.clone());

                        // Run command or send to stdin
                        let cmd_res =  execute::spawn_command(&v,self.bash.variables.iter_exported()); 
                        match cmd_res {
                            Ok((tx, rx)) => {
                                ::std::mem::replace(
                                    &mut self.current_interaction,
                                    Some((tx, rx, Interaction::new(line.clone()))),
                                );
                            }
                            Err(msg) => {
                                // Something happened during program start
                                let mut inter = Interaction::new(line);
                                inter.add_error(format!("Error executing command: {}", msg));
                                inter.prepare_archiving();
                                self.archive_interaction(inter);
                            }
                        };
                    }
                }
            }
        };
    }

    /// Find the interaction that is referenced by the command position.
    ///
    /// # Errors
    ///
    /// Assumes that the CommandPosition was generated by line_iter.
    pub fn find_interaction_from_command<'a>(
        &'a mut self,
        pos: CommandPosition,
    ) -> &'a mut Interaction {
        match pos {
            CommandPosition::Archived(conv_index, inter_index) => {
                &mut self.archived[conv_index].interactions[inter_index]
            }
            CommandPosition::CurrentConversation(inter_index) => {
                &mut self.current_conversation.interactions[inter_index]
            }
            CommandPosition::CurrentInteraction => {
                if let Some((_, _, ref mut inter)) = self.current_interaction {
                    inter
                } else {
                    panic!("find_interaction_from_command: Expected current interaction")
                }
            }
        }
    }

    /// Return an iterator over the currently visible items.
    pub fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        let archived_iter = self.archived
            .iter()
            .zip(CommandPosition::archive_iter())
            .flat_map(|(c, pos)| c.line_iter(pos))
            .chain(self.current_conversation.line_iter(
                CommandPosition::CurrentConversation(0),
            ));

        // If we have a current interaction, we display it. We don't need to draw the line editor
        // as it is special and will be drawn accordingly.
        match self.current_interaction {
            None => Box::new(archived_iter),
            Some((_, _, ref inter)) => {
                Box::new(archived_iter.chain(inter.line_iter(
                    CommandPosition::CurrentInteraction,
                )))
            }
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_session(prompt: String) -> Session {
        let bash = bash::Bash::new().expect("Can't make test bash instance");
        Session {
            current_interaction: None,
            archived: vec![],
            current_conversation: Conversation::new(prompt),
            bash,
        }
    }

    #[test]
    fn line_iter() {
        let mut session = new_test_session(String::from("prompt 1"));

        let mut inter_1_1 = Interaction::new(String::from("command 1.1"));
        inter_1_1.add_output(String::from("output 1.1.1"));
        inter_1_1.add_output(String::from("output 1.1.2"));
        session.archive_interaction(inter_1_1);
        let mut inter_1_2 = Interaction::new(String::from("command 1.2"));
        inter_1_2.add_output(String::from("output 1.2.1"));
        inter_1_2.add_output(String::from("output 1.2.2"));
        session.archive_interaction(inter_1_2);

        session.new_conversation(String::from("prompt 2"));

        let mut inter_2_1 = Interaction::new(String::from("command 2.1"));
        inter_2_1.add_output("output 2.1.1".to_string());
        inter_2_1.add_output("output 2.1.2".to_string());
        session.archive_interaction(inter_2_1);
        let mut inter_2_2 = Interaction::new(String::from("command 2.2"));
        inter_2_2.add_output("output 2.2.1".to_string());
        inter_2_2.add_output("output 2.2.2".to_string());
        session.archive_interaction(inter_2_2);

        assert_eq!(session.archived.len(), 1);
        assert_eq!(session.archived[0].interactions.len(), 2);
        assert_eq!(session.current_conversation.interactions.len(), 2);

        let mut li = session.line_iter();
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
                is_a: LineType::Command(OutputVisibility::Output, CommandPosition::Archived(0, 1)),
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 1.2.1",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 1.2.2",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "prompt 1",
                is_a: LineType::Prompt,
                cursor_col: None,
            })
        );

        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "command 2.1",
                is_a: LineType::Command(
                    OutputVisibility::Output,
                    CommandPosition::CurrentConversation(0),
                ),
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 2.1.1",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 2.1.2",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "command 2.2",
                is_a: LineType::Command(
                    OutputVisibility::Output,
                    CommandPosition::CurrentConversation(1),
                ),
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 2.2.1",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "output 2.2.2",
                is_a: LineType::Output,
                cursor_col: None,
            })
        );
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "prompt 2",
                is_a: LineType::Prompt,
                cursor_col: None,
            })
        );
        assert_eq!(li.next(), None);
    }

}
