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

//! Operations on previous command input.

use libc::c_char;
use std::ffi::CStr;
use std::str;

/// What to search for in the history.
pub enum HistorySearchMode {
    /// Everything.
    Browse,
    /// Lines that start with the given string.
    Prefix(String),
    /// Lines that contain the given string.
    Contained(String),
}

/// Search result
pub struct HistorySearchCursor {
    /// Index into History::items that matched the search request.
    pub matching_items: Vec<usize>,

    /// Currently selected item.
    pub item_ind: usize,
}

#[repr(C)]
struct HistEntry {
    line: *const c_char,
    timestamp: *const c_char,
    data: *const c_char,
}

impl HistEntry {
    fn line_as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(CStr::from_ptr(self.line).to_bytes()) }
    }
}

#[link(name = "Bash")]
extern "C" {
    fn history_list() -> *const *const HistEntry;
}

/// Searches the history for matching lines and returns a list of indices into history_list.
///
/// # Safety
///
/// The user must ensure that the search is dropped when the history changes.
pub fn search(mode: HistorySearchMode, reverse: bool) -> HistorySearchCursor {
    let matching_items: Vec<usize> = unsafe {
        let hist_list = history_list();

        (0..)
            .take_while(|&ind| !(*hist_list.offset(ind as isize)).is_null())
            .filter(|&ind| {
                mode.matches((*(*hist_list.offset(ind as isize))).line_as_str())
            })
            .collect()
    };

    let item_ind = if reverse {
        let l = matching_items.len();
        if l == 0 { 0 } else { l - 1 }
    } else {
        0
    };
    HistorySearchCursor {
        matching_items,
        item_ind,
    }
}

/// Get a line as a str
pub fn get_line_as_str(ind: usize) -> &'static str {
    unsafe {
        let hist_list = history_list();
        (*(*hist_list.offset(ind as isize))).line_as_str()
    }
}

impl HistorySearchMode {
    /// Check if the given string matches this search mode
    pub fn matches(&self, other: &str) -> bool {
        match self {
            &HistorySearchMode::Browse => true,
            &HistorySearchMode::Prefix(ref pref) => other.starts_with(pref.as_str()),
            &HistorySearchMode::Contained(ref cont) => other.contains(cont.as_str()),
        }
    }
}

impl HistorySearchCursor {
    /// Go to the previous item and wrap around if necessary.
    pub fn prev1(&mut self) {
        if self.item_ind < self.matching_items.len() {
            if self.item_ind > 0 {
                self.item_ind -= 1;
            } else {
                self.item_ind = self.matching_items.len() - 1;
            }
        }
    }

    /// Go to the next item and wrap around if necessary.
    pub fn next1(&mut self) {
        if self.item_ind + 1 < self.matching_items.len() {
            self.item_ind += 1;
        } else {
            self.item_ind = 0;
        }
    }

    /// Go *n* items back in the search and stop at the beginning.
    pub fn prev(&mut self, n: usize) {
        if self.item_ind < self.matching_items.len() {
            if self.item_ind > n {
                self.item_ind -= n;
            } else {
                self.item_ind = 0;
            }
        }
    }

    /// Go *n* items forward in the search and stop at the end.
    pub fn next(&mut self, n: usize) {
        if self.item_ind + n < self.matching_items.len() {
            self.item_ind += n;
        } else {
            self.item_ind = if self.matching_items.len() > 0 {
                self.matching_items.len() - 1
            } else {
                0
            };
        }
    }
}
