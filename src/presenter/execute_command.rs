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
use model::session::{InteractionHandle, OutputVisibility, RunningStatus};

/// Presenter to run commands and send input to their stdin.
#[allow(dead_code)]
pub struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: InteractionHandle,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<Matrix>,
}

#[allow(dead_code)]
impl ExecuteCommandPresenter {
    pub fn new(
        commons: Box<PresenterCommons>,
        current_interaction: InteractionHandle,
    ) -> Box<Self> {
        let mut presenter = ExecuteCommandPresenter {
            commons,
            current_interaction,
            next_prompt: None,
        };
        presenter.to_last_line();
        Box::new(presenter)
    }

    pub fn new_with_interaction(
        commons: Box<PresenterCommons>,
        current_interaction: InteractionHandle,
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

    /// Count the number of items of line_iter would return at most
    fn line_iter_count(&self) -> usize {
        let session = self.commons.session.clone();
        let session = session.0.lock().unwrap();
        let iter = self.line_iter(&session);
        iter.count()
    }

    /// Ensure that the last line is visible, even if the number of lines was changed.
    fn to_last_line(&mut self) {
        let len = self.line_iter_count();
        self.commons.last_line_shown = len;
    }

    fn deconstruct(self) -> (Box<PresenterCommons>, InteractionHandle) {
        (self.commons, self.current_interaction)
    }
}

impl SubPresenter for ExecuteCommandPresenter {
    fn finish(self: Box<Self>) -> Box<PresenterCommons> {
        self.commons
    }

    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn to_commons(self) -> Box<PresenterCommons> {
        self.commons
    }

    fn set_exit_status(self: &mut Self, exit_status: ExitStatus) {
        self.commons
            .session
            .set_running_status(self.current_interaction, RunningStatus::Exited(exit_status));
    }

    fn set_next_prompt(self: &mut Self, bytes: &[u8]) {
        self.next_prompt = Some(Screen::one_line_matrix(bytes));
    }

    fn end_polling(self: Box<Self>, _needs_marking: bool) -> Box<dyn SubPresenter> {
        let is_running = self.commons.session.is_running(self.current_interaction);
        trace!(
            "ExecuteCommandPresenter::end_polling: is_running = {}",
            is_running
        );
        if !is_running {
            return ComposeCommandPresenter::new(self.commons);
        }
        self
    }

    fn line_iter<'a>(&'a self, session: &'a Session) -> Box<dyn Iterator<Item = LineItem> + 'a> {
        Box::new(
            session
                .line_iter(false)
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
                self.commons
                    .interpreter
                    .write_stdin_foreground(line.as_bytes());
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
                let n = self.line_iter_count();
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
                        // TODO: Kill the last job if it is still running
                        return (self, PresenterCommand::Redraw);
                    }

                    b'd' => {
                        // TODO: Send to running program
                        return (self, PresenterCommand::Redraw);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        (self, PresenterCommand::Unknown)
    }

    /// Handle a click.
    ///
    /// If a command was clicked, cycle through the visibility of output and error.
    fn handle_click(
        mut self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<dyn SubPresenter>, NeedRedraw) {
        let redraw = if check_response_clicked(&mut *self, button, x, y) {
            NeedRedraw::Yes
        } else {
            NeedRedraw::No
        };
        (self, redraw)
    }

    fn event_text(mut self: Box<Self>, s: &str) -> (Box<dyn SubPresenter>, PresenterCommand) {
        self.commons_mut().text_input_add_characters(s);
        (self, PresenterCommand::Redraw)
    }
}
