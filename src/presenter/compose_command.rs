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

use super::*;
use super::history::*;
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
}

impl SubPresenter for ComposeCommandPresenter {
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn poll_interaction(self: Box<Self>) -> (Box<SubPresenter>, bool) {
        (self, false)
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(self.commons.session.line_iter().chain(::std::iter::once(
            LineItem::new(
                self.commons.current_line.text(),
                LineType::Input,
                Some(self.commons.current_line_pos()),
            ),
        )))
    }

    fn event_return(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        let line = self.commons.current_line.clear();

        ::model::bash::bash_add_input(line.as_str());
        ::model::bash::bash_add_input("\n");

        ExecuteCommandPresenter::new(self.commons, line)
    }

    fn event_update_line(mut self: Box<Self>) -> Box<SubPresenter> {
        self.to_last_line();
        self
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

    /// Handle pressing modifier + letter.
    ///
    /// If Ctrl-R is pressed, go to history browse mode with search for contained strings.
    /// If Ctrl-D is pressed, quit bite.
    fn event_control_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), letter) {
            ((false, true, false), b'd') => (self, PresenterCommand::Exit),
            ((false, true, false), b'r') => {
                // Control-R -> Start interactive history search
                let prefix = String::from(self.commons.current_line.text_before_cursor());
                self.commons.current_line.clear();
                self.commons.current_line.insert_str(&prefix);
                (
                    HistoryPresenter::new(self.commons, HistorySearchMode::Contained(prefix), true),
                    PresenterCommand::Redraw,
                )
            }
            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle pressing cursor up.
    ///
    /// Go to history browse mode without search.
    fn event_cursor_up(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        HistoryPresenter::new(self.commons, HistorySearchMode::Browse, true)
    }

    /// Handle pressing cursor down.
    ///
    /// Go to history browse mode without search.
    fn event_cursor_down(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        HistoryPresenter::new(self.commons, HistorySearchMode::Browse, false)
    }

    /// Handle pressing page up.
    ///
    /// Scroll page-wise on Shift-PageUp.
    ///
    /// Go to history browse mode with prefix search if no modifiers were pressed.
    fn event_page_up(mut self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        match mod_state.as_tuple() {
            (true, false, false) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                if self.commons.last_line_shown > middle {
                    self.commons.last_line_shown -= middle;
                } else {
                    self.commons.last_line_shown = 0;
                }
                self
            }
            (false, false, false) => {
                // Nothing -> Prefix search
                let prefix = String::from(self.commons.current_line.text_before_cursor());
                self.commons.current_line.clear();
                self.commons.current_line.insert_str(&prefix);
                HistoryPresenter::new(self.commons, HistorySearchMode::Prefix(prefix), true)
            }
            _ => self,
        }
    }

    /// Handle pressing page down.
    ///
    /// Scroll page-wise on Shift-PageDown.
    ///
    /// Go to history browse mode with prefix search if no modifiers were pressed.
    fn event_page_down(mut self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        match mod_state.as_tuple() {
            (true, false, false) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                let n = self.line_iter().count();
                self.commons.last_line_shown =
                    ::std::cmp::min(n, self.commons.last_line_shown + middle);
                self
            }
            (false, false, false) => {
                // Nothing -> Prefix search
                let prefix = String::from(self.commons.current_line.text_before_cursor());
                self.commons.current_line.clear();
                self.commons.current_line.insert_str(&prefix);
                HistoryPresenter::new(self.commons, HistorySearchMode::Prefix(prefix), false)
            }
            _ => self,
        }
    }
}
