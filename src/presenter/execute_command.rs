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
use model::bash::bash_kill_last;

/// Presenter to run commands and send input to their stdin.
#[allow(dead_code)]
pub struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: Interaction,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<String>,
}

#[allow(dead_code)]
impl ExecuteCommandPresenter {
    pub fn new(commons: Box<PresenterCommons>, prompt: String) -> Box<Self> {
        let presenter = ExecuteCommandPresenter {
            commons,
            current_interaction: Interaction::new(prompt),
            next_prompt: None,
        };
        Box::new(presenter)
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
                    self.current_interaction.add_output(line);
                }
                BashOutput::FromError(line) => {
                    self.current_interaction.add_error(line);
                }
                BashOutput::Terminated(exit_code) => {
                    self.current_interaction.set_exit_status(exit_code);
                }
                BashOutput::Prompt(prompt) => {
                    self.next_prompt = Some(prompt);
                }
            }
        }
        if !needs_marking {
            let next_prompt = ::std::mem::replace(&mut self.next_prompt, None);
            if let Some(prompt) = next_prompt {
                self.current_interaction.prepare_archiving();
                let ci = ::std::mem::replace(
                    &mut self.current_interaction,
                    Interaction::new(String::from("")),
                );
                self.commons.session.archive_interaction(ci);
                if self.commons.session.current_conversation.prompt != prompt {
                    self.commons.session.new_conversation(prompt);
                }
                return (ComposeCommandPresenter::new(self.commons), needs_marking);
            }
        }
        (self, needs_marking)
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.commons.session.line_iter().chain(
                self.current_interaction
                    .line_iter(CommandPosition::CurrentInteraction)
                    .chain(::std::iter::once(LineItem::new(
                        self.commons.current_line.text(),
                        LineType::Input,
                        Some(self.commons.current_line_pos()),
                    ))),
            ),
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
                        // TODO: Exit bite if input line is empty.
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
        let redraw = if check_response_clicked(&mut *self, button, x, y) {
            NeedRedraw::Yes
        } else {
            NeedRedraw::No
        };
        (self, redraw)
    }
}
