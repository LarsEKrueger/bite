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

//! Sub presenter for executing programs.

use super::*;
use std::str::from_utf8_unchecked;
use model::bash::{bash_kill_last, is_bash_waiting};

/// Presenter to run commands and send input to their stdin.
#[allow(dead_code)]
pub struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: Interaction,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<Vec<Cell>>,
}

#[allow(dead_code)]
impl ExecuteCommandPresenter {
    pub fn new(commons: Box<PresenterCommons>, prompt: Vec<Cell>) -> Box<Self> {
        let mut presenter = ExecuteCommandPresenter {
            commons,
            current_interaction: Interaction::new(prompt),
            next_prompt: None,
        };
        presenter.to_last_line();
        Box::new(presenter)
    }

    /// Ensure that the last line is visible, even if the number of lines was changed.
    fn to_last_line(&mut self) {
        let len = self.line_iter().count();
        self.commons.last_line_shown = len;
    }
}

impl SubPresenter for ExecuteCommandPresenter {
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn poll_interaction(mut self: Box<Self>) -> (Box<SubPresenter>, bool) {
        let mut needs_marking = false;
        if let Ok(line) = self.commons_mut().receiver.try_recv() {
            needs_marking = true;
            match line {
                BashOutput::FromOutput(line) => {
                    self.current_interaction.add_output(&line);
                }
                BashOutput::FromError(line) => {
                    self.current_interaction.add_error(&line);
                }
                BashOutput::Terminated(exit_code) => {
                    self.current_interaction.set_exit_status(exit_code);
                }
                BashOutput::Prompt(prompt) => {
                    self.next_prompt = Some(Screen::one_line_cell_vec(&prompt));
                }
            }
        }

        if !needs_marking && is_bash_waiting() {
            let next_prompt = ::std::mem::replace(&mut self.next_prompt, None);
            if let Some(prompt) = next_prompt {
                self.current_interaction.prepare_archiving();
                let ci = ::std::mem::replace(
                    &mut self.current_interaction,
                    Interaction::new(Vec::new()),
                );
                self.commons.session.archive_interaction(ci);

                if prompt != self.commons.session.current_conversation.prompt {
                    self.commons.session.new_conversation(prompt);
                }
                return (ComposeCommandPresenter::new(self.commons), needs_marking);
            }
        }
        (self, needs_marking)
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.commons
                .session
                .line_iter()
                .chain(self.current_interaction.line_iter(
                    CommandPosition::CurrentInteraction,
                    0,
                ))
                .chain(::std::iter::once(LineItem::new_owned(
                    Screen::one_line_cell_vec(
                        self.commons.current_line.text().as_bytes(),
                    ),
                    LineType::Input,
                    Some(self.commons.current_line_pos()),
                    0,
                ))),
        )

    }

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Enter) => {
                let line = self.commons.current_line.clear();
                // TODO: disable write-back in bash and mark this line as input
                // self.current_interaction.add_output(line.clone());
                ::model::bash::programm_add_input(line.as_str());
                ::model::bash::programm_add_input("\n");
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Left) => {
                self.commons_mut().current_line.move_left();
                (self, PresenterCommand::Redraw)
            }
            ((false, false, false), SpecialKey::Right) => {
                self.commons_mut().current_line.move_right();
                (self, PresenterCommand::Redraw)
            }

            ((true, false, false), SpecialKey::PageUp) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                if self.commons.last_line_shown > middle {
                    self.commons.last_line_shown -= middle;
                } else {
                    self.commons.last_line_shown = 0;
                }
                (self, PresenterCommand::Redraw)
            }

            ((true, false, false), SpecialKey::PageDown) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                let n = self.line_iter().count();
                self.commons.last_line_shown =
                    ::std::cmp::min(n, self.commons.last_line_shown + middle);
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Home) => {
                self.commons.current_line.move_start();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::End) => {
                self.commons.current_line.move_end();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Delete) => {
                self.commons.current_line.delete_right();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Backspace) => {
                self.commons.current_line.delete_left();
                (self, PresenterCommand::Redraw)
            }

            _ => (self, PresenterCommand::Unknown),
        }

    }

    fn event_normal_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match mod_state.as_tuple() {
            (false, true, false) => {
                // Control-only
                match letter {
                    b'c' => {
                        bash_kill_last();
                        return (self, PresenterCommand::Redraw);
                    }

                    b'd' => {
                        // ODO: Exit bite only if input line is empty.
                        let letter = [0x04; 1];
                        ::model::bash::programm_add_input(unsafe { from_utf8_unchecked(&letter) });
                        return (self, PresenterCommand::Redraw);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        (self, PresenterCommand::Unknown)
    }

    fn event_update_line(self: Box<Self>) -> Box<SubPresenter> {
        self
    }

    fn handle_click(
        mut self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw) {
        match (clicked_line_type(&mut *self, y), button) {
            (Some(LineType::Command(_, pos, _)), 1) => {
                if x < COMMAND_PREFIX_LEN {
                    match pos {
                        CommandPosition::CurrentInteraction => Some(&mut self.current_interaction),
                        p => self.commons_mut().session.find_interaction_from_command(p),
                    }.map(|i| i.cycle_visibility());
                    return (self, NeedRedraw::Yes);
                }
            }
            _ => {
                // Unhandled combination, ignore
            }
        }
        return (self, NeedRedraw::No);
    }
}
