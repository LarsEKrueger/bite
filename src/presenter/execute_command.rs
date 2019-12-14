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

use super::tui::TuiExecuteCommandPresenter;
use super::*;
use model::bash::{bash_kill_last, is_bash_waiting};
use model::interaction::{CommandPosition, CurrentInteraction};
use std::str::from_utf8_unchecked;

/// Presenter to run commands and send input to their stdin.
#[allow(dead_code)]
pub struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: CurrentInteraction,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<Matrix>,
}

#[allow(dead_code)]
impl ExecuteCommandPresenter {
    pub fn new(commons: Box<PresenterCommons>, prompt: Matrix) -> Box<Self> {
        let mut presenter = ExecuteCommandPresenter {
            commons,
            current_interaction: CurrentInteraction::new(prompt),
            next_prompt: None,
        };
        presenter.to_last_line();
        Box::new(presenter)
    }

    pub fn new_with_interaction(
        commons: Box<PresenterCommons>,
        current_interaction: CurrentInteraction,
        next_prompt: Option<Matrix>,
    ) -> Box<Self> {
        let mut presenter = ExecuteCommandPresenter {
            commons,
            current_interaction,
            next_prompt,
        };
        presenter.to_last_line();
        Box::new(presenter)
    }

    /// Ensure that the last line is visible, even if the number of lines was changed.
    fn to_last_line(&mut self) {
        let len = self.line_iter().count();
        self.commons.last_line_shown = len;
    }

    fn deconstruct(self) -> (Box<PresenterCommons>, CurrentInteraction) {
        (self.commons, self.current_interaction)
    }
}

impl SubPresenter for ExecuteCommandPresenter {
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn to_commons(self) -> Box<PresenterCommons> {
        self.commons
    }

    fn add_output(mut self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]) {
        match self.current_interaction.add_output(&bytes) {
            AddBytesResult::ShowStream(rest) => {
                self.current_interaction.show_output();
                (self, rest)
            }
            AddBytesResult::AllDone => (self, b""),
            AddBytesResult::StartTui(rest) => {
                let (c, i) = self.deconstruct();
                let presenter = TuiExecuteCommandPresenter::new(c, i);
                (presenter, rest)
            }
        }
    }

    fn add_error(mut self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]) {
        match self.current_interaction.add_error(&bytes) {
            AddBytesResult::ShowStream(rest) => {
                self.current_interaction.show_errors();
                (self, rest)
            }
            AddBytesResult::AllDone => (self, b""),
            AddBytesResult::StartTui(rest) => {
                let (c, i) = self.deconstruct();
                let presenter = TuiExecuteCommandPresenter::new(c, i);
                (presenter, rest)
            }
        }
    }

    fn set_exit_status(self: &mut Self, exit_status: ExitStatus) {
        self.current_interaction.set_exit_status(exit_status);
    }

    fn set_next_prompt(self: &mut Self, bytes: &[u8]) {
        self.next_prompt = Some(Screen::one_line_matrix(bytes));
    }

    fn end_polling(mut self: Box<Self>, needs_marking: bool) -> Box<dyn SubPresenter> {
        if !needs_marking && is_bash_waiting() {
            let next_prompt = ::std::mem::replace(&mut self.next_prompt, None);
            if let Some(prompt) = next_prompt {
                let ci = ::std::mem::replace(
                    &mut self.current_interaction,
                    CurrentInteraction::new(Matrix::new()),
                );
                self.commons
                    .session
                    .archive_interaction(ci.prepare_archiving());

                if prompt != self.commons.session.current_conversation.prompt {
                    self.commons.session.new_conversation(prompt);
                }
                trace!("Done executing");
                return ComposeCommandPresenter::new(self.commons);
            }
        }
        self
    }

    fn line_iter<'a>(&'a self) -> Box<dyn Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.commons
                .session
                .line_iter()
                .chain(
                    self.current_interaction
                        .line_iter(CommandPosition::CurrentInteraction, 0),
                )
                .chain(self.commons.input_line_iter()),
        )
    }

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<dyn SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Enter) => {
                let line = self.commons.text_input.extract_text();
                self.commons.text_input.reset();
                self.commons.text_input.make_room();
                // TODO: disable write-back in bash and mark this line as input
                // self.current_interaction.add_output(line.clone());
                ::model::bash::program_add_input(line.as_str());
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Left) => {
                self.commons_mut().text_input.move_left(1);
                (self, PresenterCommand::Redraw)
            }
            ((false, false, false), SpecialKey::Right) => {
                self.commons_mut().text_input.move_right(1);
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
                self.commons.text_input.move_left_edge();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::End) => {
                self.commons.text_input.move_right_edge();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Delete) => {
                self.commons.text_input.delete_character();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Backspace) => {
                self.commons.text_input.delete_left();
                (self, PresenterCommand::Redraw)
            }

            _ => (self, PresenterCommand::Unknown),
        }
    }

    fn event_normal_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<dyn SubPresenter>, PresenterCommand) {
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
                        ::model::bash::program_add_input(unsafe { from_utf8_unchecked(&letter) });
                        return (self, PresenterCommand::Redraw);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        (self, PresenterCommand::Unknown)
    }

    fn handle_click(
        mut self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<dyn SubPresenter>, NeedRedraw) {
        match (clicked_line_type(&mut *self, y), button) {
            (Some(LineType::Command(_, pos, _)), 1) => {
                if x < COMMAND_PREFIX_LEN {
                    match pos {
                        CommandPosition::CurrentInteraction => {
                            Some(self.current_interaction.get_archive())
                        }
                        p => self.commons_mut().session.find_interaction_from_command(p),
                    }
                    .map(|i| i.cycle_visibility());
                    return (self, NeedRedraw::Yes);
                }
            }
            _ => {
                // Unhandled combination, ignore
            }
        }
        return (self, NeedRedraw::No);
    }

    fn event_text(mut self: Box<Self>, s: &str) -> (Box<dyn SubPresenter>, PresenterCommand) {
        self.commons_mut().text_input_add_characters(s);
        (self, PresenterCommand::Redraw)
    }
}
