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

//! Sub presenter for searching the history.

use super::*;
use model::history::*;

/// Presenter to select an item from the history.
pub struct HistoryPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,
    /// Current search result
    search: history::HistorySearchCursor,
}

impl HistoryPresenter {
    /// Allocate a new sub-presenter for history browsing.
    ///
    /// The filter for determining which items to show is passed in mode.
    pub fn new(
        commons: Box<PresenterCommons>,
        mode: HistorySearchMode,
        reverse: bool,
    ) -> Box<HistoryPresenter> {
        let search = history::search(mode, reverse);
        let mut presenter = HistoryPresenter { commons, search };

        presenter.to_last_line();

        Box::new(presenter)
    }

    /// Scroll to last line
    fn to_last_line(&mut self) {
        let cnt = self.line_iter().count();
        self.commons.last_line_shown = cnt;
    }

    /// Ensure that the selected item is visible on screen.
    ///
    /// If the selection is already visible, do nothing. Otherwise, center it on the screen.
    fn show_selection(&mut self) -> NeedRedraw {
        let start_line = self.commons.start_line();
        if start_line <= self.search.item_ind && self.search.item_ind < self.commons.last_line_shown
        {
            NeedRedraw::No
        } else {
            let middle = self.commons.window_height / 2;
            let n = self.line_iter().count();
            self.commons.last_line_shown = ::std::cmp::min(n, self.search.item_ind + middle);
            NeedRedraw::Yes
        }
    }

    /// Set the input line from the current selection
    fn replace_text_input(&mut self) -> bool {
        if self.search.item_ind < self.search.matching_items.len() {
            let hist_ind = self.search.matching_items[self.search.item_ind];
            let item = history::get_line_as_str(hist_ind).to_string();
            self.commons.text_input.replace(&item, true);
            true
        } else {
            false
        }
    }
}

impl SubPresenter for HistoryPresenter {
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
        error!("HistoryPresenter::add_output called. Internal error.");
        (self, b"")
    }

    fn add_error(self: Box<Self>, _bytes: &[u8]) -> (Box<SubPresenter>, &[u8]) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        error!("HistoryPresenter::add_error called. Internal error.");
        (self, b"")
    }

    fn set_exit_status(self: &mut Self, _exit_status: ExitStatus) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        error!("HistoryPresenter::set_exit_status called. Internal error.");
    }

    fn set_next_prompt(self: &mut Self, _bytes: &[u8]) {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        error!("HistoryPresenter::set_next_prompt called. Internal error.");
    }

    fn end_polling(self: Box<Self>, _needs_marking: bool) -> Box<SubPresenter> {
        // This should not happen. If it does happen, someone is generating output while the shell
        // is waiting for commands.
        error!("HistoryPresenter::end_polling called. Internal error.");
        self
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.search
                .matching_items
                .iter()
                .zip(0..)
                .map(move |(hist_ind, match_ind)| {
                    LineItem::new_owned(
                        Screen::one_line_cell_vec(history::get_line_as_str(*hist_ind).as_bytes()),
                        if match_ind == self.search.item_ind {
                            LineType::SelectedMenuItem(*hist_ind as usize)
                        } else {
                            LineType::MenuItem(*hist_ind as usize)
                        },
                        None,
                        0,
                    )
                })
                .chain(self.commons.input_line_iter()),
        )
    }

    fn event_special_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Enter) => {
                let propagate = self.replace_text_input();
                let next = ComposeCommandPresenter::new(self.commons);
                if propagate {
                    next.event_special_key(mod_state, &SpecialKey::Enter)
                } else {
                    (next, PresenterCommand::Redraw)
                }
            }

            ((false, false, false), SpecialKey::Left)
            | ((false, false, false), SpecialKey::Right)
            | ((true, false, false), SpecialKey::Enter) => {
                self.replace_text_input();
                (
                    ComposeCommandPresenter::new(self.commons),
                    PresenterCommand::Redraw,
                )
            }

            ((false, false, false), SpecialKey::Up) => {
                self.search.prev1();
                self.show_selection();
                (self, PresenterCommand::Redraw)
            }
            ((false, false, false), SpecialKey::Down) => {
                self.search.next1();
                self.show_selection();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::PageUp) => {
                let n = self.commons.window_height;
                self.search.prev(n);
                self.show_selection();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::PageDown) => {
                let n = self.commons.window_height;
                self.search.next(n);
                self.show_selection();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::Home) => {
                self.search.first();
                (self, PresenterCommand::Redraw)
            }

            ((false, false, false), SpecialKey::End) => {
                self.search.last();
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

    /// Handle changes to the input.
    ///
    /// If we are searching, update the search string and try to scroll as little as possible.
    fn event_update_line(mut self: Box<Self>) -> Box<SubPresenter> {
        let prefix = String::from(self.commons.text_input.extract_text());
        let mut search = history::search(HistorySearchMode::Contained(prefix), false);

        // Find the index into matching_items that is closest to search.item_ind to move the
        // highlight only a litte.
        fn abs_diff(a: usize, b: usize) -> usize {
            if a < b {
                b - a
            } else {
                a - b
            }
        }

        let last_history_ind = if self.search.item_ind < self.search.matching_items.len() {
            self.search.matching_items[self.search.item_ind]
        } else {
            0
        };
        let mut ind_item = 0;
        let mut dist = None;
        for i in 0..search.matching_items.len() {
            let history_ind = search.matching_items[i];
            let d = abs_diff(last_history_ind, history_ind);
            dist = match dist {
                None => {
                    ind_item = i;
                    Some(d)
                }
                Some(dist) => {
                    if d < dist {
                        ind_item = i;
                        Some(d)
                    } else {
                        Some(dist)
                    }
                }
            };
        }
        search.item_ind = ind_item;
        self.search = search;
        self.show_selection();
        self
    }

    fn event_normal_key(
        self: Box<Self>,
        _mod_state: &ModifierState,
        _letter: u8,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        (self, PresenterCommand::Unknown)
    }

    fn handle_click(
        self: Box<Self>,
        _button: usize,
        _x: usize,
        _y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw) {
        (self, NeedRedraw::No)
    }
}
