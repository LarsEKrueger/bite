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
        if start_line <= self.search.item_ind &&
            self.search.item_ind < self.commons.last_line_shown
        {
            NeedRedraw::No
        } else {
            let middle = self.commons.window_height / 2;
            let n = self.line_iter().count();
            self.commons.last_line_shown = ::std::cmp::min(n, self.search.item_ind + middle);
            NeedRedraw::Yes
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

    fn poll_interaction(self: Box<Self>) -> (Box<SubPresenter>, bool) {
        (self, false)
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.search
                .matching_items
                .iter()
                .zip(0..)
                .map(move |(hist_ind, match_ind)| {
                    LineItem::new(
                        history::get_line_as_str(*hist_ind),
                        if match_ind == self.search.item_ind {
                            LineType::SelectedMenuItem(*hist_ind as usize)
                        } else {
                            LineType::MenuItem(*hist_ind as usize)
                        },
                        None,
                    )
                })
                .chain(::std::iter::once(LineItem::new(
                    self.commons.current_line.text(),
                    LineType::Input,
                    Some(self.commons.current_line_pos()),
                ))),
        )
    }

    /// Handle pressing the return key.
    ///
    /// Extract the selected line from history, switch state to the normal presenter and make it
    /// handle the line as if it was entered.
    fn event_return(mut self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        let propagate = if self.search.item_ind < self.search.matching_items.len() {
            let hist_ind = self.search.matching_items[self.search.item_ind];
            let item = history::get_line_as_str(hist_ind).to_string();
            self.commons.current_line.replace(item, false);
            true
        } else {
            false
        };
        let next = ComposeCommandPresenter::new(self.commons);
        if propagate {
            next.event_return(mod_state)
        } else {
            next
        }
    }

    /// Handle changes to the input.
    ///
    /// If we are searching, update the search string and try to scroll as little as possible.
    fn event_update_line(mut self: Box<Self>) -> Box<SubPresenter> {
        let prefix = String::from(self.commons.current_line.text());
        let mut search = history::search(HistorySearchMode::Contained(prefix), false);

        // Find the index into matching_items that is closest to search.item_ind to move the
        // highlight only a litte.
        fn abs_diff(a: usize, b: usize) -> usize {
            if a < b { b - a } else { a - b }
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

    fn event_control_key(
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

    /// Handle the event when the cursor left key is pressed.
    fn event_cursor_left(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        if self.search.item_ind < self.search.matching_items.len() {
            let hist_ind = self.search.matching_items[self.search.item_ind];
            let item = history::get_line_as_str(hist_ind).to_string();
            self.commons.current_line.replace(item, false);
        }
        ComposeCommandPresenter::new(self.commons)
    }

    /// Handle the event when the cursor right key is pressed.
    fn event_cursor_right(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        if self.search.item_ind < self.search.matching_items.len() {
            let hist_ind = self.search.matching_items[self.search.item_ind];
            let item = history::get_line_as_str(hist_ind).to_string();
            self.commons.current_line.replace(item, false);
        }
        ComposeCommandPresenter::new(self.commons)
    }

    fn event_cursor_up(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        self.search.prev1();
        self.show_selection();
        self
    }

    fn event_cursor_down(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        self.search.next1();
        self.show_selection();
        self
    }

    fn event_page_up(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        let n = self.commons.window_height;
        self.search.prev(n);
        self.show_selection();
        self
    }

    fn event_page_down(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        let n = self.commons.window_height;
        self.search.next(n);
        self.show_selection();
        self
    }
}
