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

//! Sub presenter for completion

use super::compose_command::ComposeCommandPresenter;
use super::*;

/// Presenter to input and run commands.
pub struct CompleteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Number of characters in prefix
    prefix_chars: usize,

    /// List of completions
    completions: Vec<String>,

    /// Currently selected completion
    current: usize,
}

impl CompleteCommandPresenter {
    pub fn new(
        commons: Box<PresenterCommons>,
        prefix: String,
        completions: Vec<String>,
    ) -> Box<Self> {
        // Select current completion
        let current = 0;

        let prefix_chars = prefix.chars().count();
        let mut presenter = CompleteCommandPresenter {
            commons,
            prefix_chars,
            completions,
            current,
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

    /// Make the last line visible.
    fn to_last_line(&mut self) {
        let cnt = self.line_iter_count();
        self.commons.last_line_shown = cnt;
    }

    /// Ensure that the selected item is visible on screen.
    ///
    /// If the selection is already visible, do nothing. Otherwise, center it on the screen.
    fn show_selection(&mut self) -> NeedRedraw {
        let start_line = self.commons.start_line();
        if start_line <= self.current && self.current < self.commons.last_line_shown {
            NeedRedraw::No
        } else {
            let middle = self.commons.window_height / 2;
            let n = self.line_iter_count();
            self.commons.last_line_shown = ::std::cmp::min(n, self.current + middle);
            NeedRedraw::Yes
        }
    }
}

impl SubPresenter for CompleteCommandPresenter {
    fn finish(self: Box<Self>) -> Box<PresenterCommons> {
        self.commons
    }

    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn line_iter<'a>(&'a self, _session: &'a Session) -> Box<dyn Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.completions
                .iter()
                .enumerate()
                .map(move |(item_ind, s)| {
                    LineItem::new_owned(
                        Screen::one_line_cell_vec(s.as_bytes()),
                        if item_ind == self.current {
                            LineType::SelectedMenuItem(item_ind)
                        } else {
                            LineType::MenuItem(item_ind)
                        },
                        Some(self.prefix_chars),
                        0,
                    )
                }),
        )
    }

    fn get_overlay(&self, _session: &Session) -> Option<(Vec<String>, usize, usize, i32)> {
        None
    }

    /// Handle a click.
    fn handle_click(
        self: Box<Self>,
        _button: usize,
        _x: usize,
        _y: usize,
    ) -> (Box<dyn SubPresenter>, NeedRedraw) {
        // TODO
        (self, NeedRedraw::Yes)
    }

    /// Handle special keys
    ///
    /// React to cursor and return
    fn event_special_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<dyn SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Enter) => {
                // TODO: Use this completion
                // Delete the beginning
                self.commons
                    .text_input
                    .move_left(self.prefix_chars as isize);
                for _i in 0..self.prefix_chars {
                    self.commons.text_input.delete_character();
                }
                if self.current < self.completions.len() {
                    self.commons
                        .text_input
                        .place_str(&self.completions[self.current]);
                }
                (
                    ComposeCommandPresenter::new(self.commons),
                    PresenterCommand::Redraw,
                )
            }
            ((false, false, false), SpecialKey::Up) => {
                if self.current > 0 {
                    self.current -= 1;
                }
                self.show_selection();
                (self, PresenterCommand::Redraw)
            }
            ((false, false, false), SpecialKey::Down) => {
                if self.current + 1 < self.completions.len() {
                    self.current += 1;
                }
                self.show_selection();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Home) => {
                self.current = 0;
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::End) => {
                self.current = self.completions.len() - 1;
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Escape) => (
                ComposeCommandPresenter::new(self.commons),
                PresenterCommand::Redraw,
            ),

            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle pressing modifier + letter.
    ///
    /// Does nothing
    fn event_normal_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<dyn SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), letter) {
            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle text input
    ///
    /// Does nothing
    fn event_text(self: Box<Self>, _s: &str) -> (Box<dyn SubPresenter>, PresenterCommand) {
        (self, PresenterCommand::Redraw)
    }
}
