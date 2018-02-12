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

pub mod line;
pub mod response;
pub mod interaction;
pub mod conversation;
pub mod iterators;
pub mod history;

use std::iter::*;
use std::sync::mpsc::{Receiver, Sender};

use self::interaction::*;
use self::conversation::*;
use self::iterators::*;

use super::bash;
use super::execute;
use super::runeline;

enum HistorySearchMode {
    None,
    Sequential(history::HistorySeqIter),
    Prefix(history::HistoryPrefIter),
    Interactive(history::HistoryInteractiveSearch),
}

// A number of closed conversations and the current one
pub struct Session {
    pub archived: Vec<Conversation>,
    pub current_conversation: Conversation,
    current_interaction: Option<(Sender<String>, Receiver<execute::CommandOutput>, Interaction)>,
    current_line: runeline::Runeline,
    last_line_shown: usize,

    bash: bash::Bash,
    history: history::History,

    history_search: HistorySearchMode,
}

impl Session {
    pub fn new() -> Result<Self, String> {
        let bash = bash::Bash::new();

        let history = {
            let home_dir = bash.get_current_user_home_dir();
            history::History::new(home_dir)
        };

        // Load the history from ~/.bite_history or import from ~/.bash_history
        let mut session = Session {
            current_interaction: None,
            current_line: runeline::Runeline::new(),
            last_line_shown: 0,
            archived: vec![],
            current_conversation: Conversation::new(bash.expand_ps1()),
            bash,
            history,
            history_search: HistorySearchMode::None,
        };
        let last_line_shown = session.line_iter().count() - 1;
        session.last_line_shown = last_line_shown;
        Ok(session)
    }

    #[allow(dead_code)]
    pub fn new_conversation(&mut self, prompt: String) {
        use std::mem;
        let cur = mem::replace(&mut self.current_conversation, Conversation::new(prompt));
        self.archived.push(cur);
    }

    pub fn archive_interaction(&mut self, interaction: Interaction) {
        self.current_conversation.add_interaction(interaction);
    }

    pub fn line_iter_history_search<'a>(
        &'a self,
        hsi: &'a history::HistoryInteractiveSearch,
    ) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            hsi.matching_items
                .iter()
                .zip(0..)
                .map(move |(hist_ind, match_ind)| {
                    LineItem::new(
                        self.history.items[*hist_ind].as_str(),
                        if match_ind == hsi.ind_item {
                            LineType::SelectedMenuItem(*hist_ind)
                        } else {
                            LineType::MenuItem(*hist_ind)
                        },
                        None,
                    )
                })
                .chain(::std::iter::once(LineItem::new(
                    self.current_line.text(),
                    LineType::Input,
                    Some(self.current_line_pos()),
                ))),
        )
    }

    pub fn line_iter_normal<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        let archived_iter = self.archived
            .iter()
            .zip(CommandPosition::archive_iter())
            .flat_map(|(c, pos)| c.line_iter(pos))
            .chain(self.current_conversation.line_iter(
                CommandPosition::CurrentConversation(0),
            ));

        // If we have a current interaction, we display it. We don't need to draw the line editor
        // as it is special and will be drawn accordingly.
        let iter: Box<Iterator<Item = LineItem> + 'a> = match self.current_interaction {
            None => Box::new(archived_iter),
            Some((_, _, ref inter)) => {
                Box::new(archived_iter.chain(inter.line_iter(
                    CommandPosition::CurrentInteraction,
                )))
            }
        };

        Box::new(iter.chain(::std::iter::once(LineItem::new(
            self.current_line.text(),
            LineType::Input,
            Some(self.current_line_pos()),
        ))))
    }

    pub fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        if let HistorySearchMode::Interactive(ref hsi) = self.history_search {
            self.line_iter_history_search(hsi)
        } else {
            self.line_iter_normal()
        }
    }

    pub fn start_line(&self, lines_per_window: usize) -> usize {
        if self.last_line_shown > lines_per_window {
            self.last_line_shown + 1 - lines_per_window
        } else {
            0
        }
    }

    pub fn current_line_pos(&self) -> usize {
        self.current_line.char_index()
    }

    pub fn last_line_visible(&self) -> bool {
        self.line_iter().count() == (self.last_line_shown + 1)
    }

    pub fn to_last_line(&mut self) {
        let last_line_shown = self.line_iter().count();
        self.last_line_shown = last_line_shown - 1;
    }

    pub fn poll_interaction(&mut self) -> bool {
        let last_line_visible_pre = self.last_line_visible();
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
        if last_line_visible_pre {
            self.to_last_line();
        }
        needs_marking
    }

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

    pub fn move_left(&mut self) {
        self.clear_history_mode();
        self.current_line.move_left();
    }

    pub fn move_right(&mut self) {
        self.clear_history_mode();
        self.current_line.move_right();
    }

    pub fn delete_left(&mut self) {
        self.clear_history_mode();
        self.current_line.delete_left();
    }

    pub fn delete_right(&mut self) {
        self.clear_history_mode();
        self.current_line.delete_right();
    }

    pub fn end_line(&mut self) {
        if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
            if hsi.ind_item < hsi.matching_items.len() {
                self.current_line.replace(
                    self.history.items[hsi.matching_items[hsi.ind_item]].clone(),
                    false,
                );
            }
        };
        self.clear_history_mode();
        let line = self.current_line.clear();
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
                    bash::Command::Incomplete => {}
                    bash::Command::Error(err) => {
                        // Parser error. Create a fake interaction with the bad command line and
                        // the error message
                        let mut inter = Interaction::new(line);
                        for l in err.into_iter() {
                            inter.add_error(l);
                        }
                        inter.prepare_archiving();
                        self.archive_interaction(inter);
                    }
                    bash::Command::SimpleCommand(v) => {
                        // Add to history
                        self.history.add_command(line.clone());

                        // Run command or send to stdin
                        match execute::spawn_command(&v) {
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
                        self.to_last_line();
                    }
                }
            }
        };
    }

    pub fn insert_str(&mut self, s: &str) {
        if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
            self.current_line.insert_str(s);
            hsi.set_prefix(&self.history, self.current_line.text());
        } else {
            self.clear_history_mode();
            self.current_line.insert_str(s);
        }
        self.to_last_line();
    }

    pub fn scroll_down(&mut self) -> bool {
        // Scroll down -> increment last_line_shown
        if self.last_line_shown + 1 < self.line_iter().count() {
            self.last_line_shown += 1;
            true
        } else {
            false
        }
    }

    pub fn scroll_up(&mut self, lines_per_window: usize) -> bool {
        // Scroll up -> decrement last_line_shown
        if self.last_line_shown > lines_per_window {
            self.last_line_shown -= 1;
            true
        } else {
            false
        }
    }

    fn clear_history_mode(&mut self) {
        self.history_search = HistorySearchMode::None;
    }

    fn history_search_seq(&mut self, reverse: bool) {
        match self.history_search {
            HistorySearchMode::Sequential(_) => {}
            _ => {
                self.history_search = HistorySearchMode::Sequential(self.history.seq_iter(reverse));
            }
        }
    }

    pub fn previous_history(&mut self) {
        if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
            hsi.prev();
            return;
        };
        self.history_search_seq(true);
        let line = match self.history_search {
            HistorySearchMode::Sequential(ref mut iter) => iter.prev(&self.history),
            _ => None,
        };
        match line {
            Some(s) => {
                self.current_line.replace(s, true);
                self.to_last_line();
                // TODO: Go to end of line
            }
            None => self.clear_history_mode(),
        }
    }

    pub fn next_history(&mut self) {
        if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
            hsi.next();
            return;
        };
        self.history_search_seq(false);
        let line = match self.history_search {
            HistorySearchMode::Sequential(ref mut iter) => iter.next(&self.history),
            _ => None,
        };
        match line {
            Some(s) => {
                self.current_line.replace(s, true);
                self.to_last_line();
                // TODO: Go to end of line
            }
            None => self.clear_history_mode(),
        }
    }

    fn history_search_pref(&mut self, reverse: bool) {
        match self.history_search {
            HistorySearchMode::Prefix(_) => {}
            _ => {
                let iter = self.history.prefix_iter(
                    self.current_line.text_before_cursor(),
                    reverse,
                );
                self.history_search = HistorySearchMode::Prefix(iter);
            }
        }
    }

    pub fn history_search_forward(&mut self) {
        self.history_search_pref(false);
        let line = match self.history_search {
            HistorySearchMode::Prefix(ref mut iter) => iter.next(&self.history),
            _ => None,
        };
        match line {
            Some(s) => {
                self.current_line.replace(s, true);
                self.to_last_line();
            }
            None => self.clear_history_mode(),
        }
    }

    pub fn history_search_backward(&mut self) {
        self.history_search_pref(true);

        let line = match self.history_search {
            HistorySearchMode::Prefix(ref mut iter) => iter.prev(&self.history),
            _ => None,
        };
        match line {
            Some(s) => {
                self.current_line.replace(s, true);
                self.to_last_line();
            }
            None => self.clear_history_mode(),
        }
    }

    pub fn history_search_interactive(&mut self) {
        println!("history_search_interactive");
        match self.history_search {
            HistorySearchMode::Interactive(_) => {}
            _ => {
                self.current_line.clear();
                self.history_search =
                    HistorySearchMode::Interactive(self.history.begin_interactive_search());
                self.to_last_line();
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_session(prompt: String) -> Session {
        let bash = bash::Bash::new();
        let history = {
            let home_dir = bash.get_current_user_home_dir();
            history::History::new(home_dir)
        };
        let mut session = Session {
            current_interaction: None,
            current_line: runeline::Runeline::new(),
            last_line_shown: 0,
            archived: vec![],
            current_conversation: Conversation::new(prompt),
            bash,
            history,
            history_search: HistorySearchMode::None,
        };
        let last_line_shown = session.line_iter().count() - 1;
        session.last_line_shown = last_line_shown;
        session
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
        assert_eq!(
            li.next(),
            Some(LineItem {
                text: "",
                is_a: LineType::Input,
                cursor_col: Some(0),
            })
        );
        assert_eq!(li.next(), None);
    }

}
