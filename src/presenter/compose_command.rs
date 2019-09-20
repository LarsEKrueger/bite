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

//! Sub presenter for composing commands.

use super::execute_command::ExecuteCommandPresenter;
use super::history::*;
use super::*;
use model::history::*;

/// Presenter to input and run commands.
pub struct ComposeCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,
}

impl ComposeCommandPresenter {
    /// Allocate a sub-presenter for command composition and input to running programs.
    pub fn new(commons: Box<PresenterCommons>) -> Box<Self> {
        let mut presenter = ComposeCommandPresenter { commons };
        presenter.to_last_line();
        Box::new(presenter)
    }

    /// Make the last line visible.
    fn to_last_line(&mut self) {
        let cnt = self.line_iter().count();
        self.commons.last_line_shown = cnt;
    }

    fn is_multi_line(&self) -> bool {
        self.commons.text_input.height() > 1
    }

    fn text_input(&mut self) -> &mut Screen {
        &mut self.commons.text_input
    }

    fn execute_input(mut self) -> (Box<SubPresenter>, PresenterCommand) {
        let line = self.commons.text_input.extract_text_without_last_nl();
        self.commons.text_input.reset();
        self.commons.text_input.make_room();
        trace!("Execute »{}«", line);
        let mut line_with_nl = line.clone();
        line_with_nl.push('\n');
        ::model::bash::bash_add_input(line_with_nl.as_str());
        (
            ExecuteCommandPresenter::new(self.commons, Screen::one_line_matrix(line.as_bytes())),
            PresenterCommand::Redraw,
        )
    }
}

impl SubPresenter for ComposeCommandPresenter {
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn to_commons(self) -> Box<PresenterCommons> {
        self.commons
    }

    fn add_output(self: Box<Self>, _bytes: &[u8]) -> (Box<SubPresenter>, &[u8]) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        // TODO: Log this occurance.
        (self, b"")
    }

    fn add_error(self: Box<Self>, _bytes: &[u8]) -> (Box<SubPresenter>, &[u8]) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        // TODO: Log this occurance.
        (self, b"")
    }

    fn set_exit_status(self: &mut Self, _exit_status: ExitStatus) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        // TODO: Log this occurance.
    }

    fn set_next_prompt(self: &mut Self, _bytes: &[u8]) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        // TODO: Log this occurance.
    }

    fn end_polling(self: Box<Self>, _needs_marking: bool) -> Box<SubPresenter> {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        // TODO: Log this occurance.
        self
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.commons
                .session
                .line_iter()
                .chain(self.commons.input_line_iter()),
        )
    }

    /// Handle a click.
    ///
    /// If a command was clicked, cycle through the visibility of output and error.
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

    fn event_special_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Enter) => {
                if self.is_multi_line() {
                    self.commons_mut().text_input.break_line();
                    (self, PresenterCommand::Redraw)
                } else {
                    self.execute_input()
                }
            }
            ((true, false, false), SpecialKey::Enter) => {
                // Shift-Enter -> Break the line and thereby start multi-line editing
                self.commons_mut().text_input.break_line();
                (self, PresenterCommand::Redraw)
            }
            ((false, true, false), SpecialKey::Enter) => {
                // Ctrl-Enter -> Start the command in multi-line mode
                if self.is_multi_line() {
                    self.execute_input()
                } else {
                    (self, PresenterCommand::Unknown)
                }
            }
            ((false, false, false), SpecialKey::Left) => {
                self.commons_mut().text_input.move_left(1);
                (self, PresenterCommand::Redraw)
            }
            ((false, false, false), SpecialKey::Right) => {
                self.commons_mut().text_input.move_right(1);
                (self, PresenterCommand::Redraw)
            }
            ((false, false, false), SpecialKey::Up) => {
                if self.is_multi_line() {
                    self.text_input().move_up(1);
                    (self, PresenterCommand::Redraw)
                } else {
                    // Go to history browse mode without search.
                    (
                        HistoryPresenter::new(self.commons, HistorySearchMode::Browse, true),
                        PresenterCommand::Redraw,
                    )
                }
            }
            ((false, false, false), SpecialKey::Down) => {
                if self.is_multi_line() {
                    self.text_input().move_down(1);
                    (self, PresenterCommand::Redraw)
                } else {
                    (
                        // Go to history browse mode without search.
                        HistoryPresenter::new(self.commons, HistorySearchMode::Browse, false),
                        PresenterCommand::Redraw,
                    )
                }
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

            ((false, false, false), SpecialKey::PageUp) => {
                // Nothing -> Prefix search
                let prefix = {
                    let ref mut text_input = self.commons.text_input;
                    let prefix = text_input.text_before_cursor();
                    text_input.reset();
                    text_input.place_str(&prefix);
                    prefix
                };
                (
                    HistoryPresenter::new(self.commons, HistorySearchMode::Prefix(prefix), true),
                    PresenterCommand::Redraw,
                )
            }
            ((true, false, false), SpecialKey::PageDown) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                let n = self.line_iter().count();
                self.commons.last_line_shown =
                    ::std::cmp::min(n, self.commons.last_line_shown + middle);
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::PageDown) => {
                // Nothing -> Prefix search
                let prefix = {
                    let ref mut text_input = self.commons.text_input;
                    let prefix = text_input.text_before_cursor();
                    text_input.reset();
                    text_input.place_str(&prefix);
                    prefix
                };
                (
                    HistoryPresenter::new(self.commons, HistorySearchMode::Prefix(prefix), false),
                    PresenterCommand::Redraw,
                )
            }
            ((false, false, false), SpecialKey::Home) => {
                self.commons.text_input.move_left_edge();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::End) => {
                self.text_input().move_end_of_line();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Delete) => {
                if self.text_input().cursor_at_end_of_line() {
                    self.text_input().join_next_line();
                } else {
                    self.commons.text_input.delete_character();
                }
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Backspace) => {
                if self.text_input().cursor_x() == 0 {
                    if self.text_input().cursor_y() > 0 {
                        self.text_input().move_up(1);
                        self.text_input().move_end_of_line();
                        self.text_input().join_next_line();
                    }
                } else {
                    self.commons.text_input.delete_left();
                }
                (self, PresenterCommand::Redraw)
            }

            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle pressing modifier + letter.
    ///
    /// If Ctrl-R is pressed, go to history browse mode with search for contained strings.
    /// If Ctrl-D is pressed, quit bite.
    fn event_normal_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), letter) {
            ((false, true, false), b'd') => (self, PresenterCommand::Exit),
            ((false, true, false), b'r') => {
                // Control-R -> Start interactive history search
                let prefix = {
                    let ref mut text_input = self.commons.text_input;
                    let prefix = text_input.text_before_cursor();
                    text_input.reset();
                    text_input.place_str(&prefix);
                    prefix
                };
                (
                    HistoryPresenter::new(self.commons, HistorySearchMode::Contained(prefix), true),
                    PresenterCommand::Redraw,
                )
            }
            _ => (self, PresenterCommand::Unknown),
        }
    }

    fn event_text(mut self: Box<Self>, s: &str) -> (Box<SubPresenter>, PresenterCommand) {
        self.commons_mut().text_input_add_characters(s);
        self.to_last_line();
        (self, PresenterCommand::Redraw)
    }
}
