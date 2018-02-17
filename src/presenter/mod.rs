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

pub mod runeline;

use model::session::*;
use model::iterators::*;
use model::interaction::*;
use model::bash::*;

pub struct ModifierState {
    pub shift_pressed: bool,
    pub control_pressed: bool,
    pub meta_pressed: bool,
}

#[derive(PartialEq, Eq)]
pub enum NeedRedraw {
    No,
    Yes,
}

#[derive(PartialEq, Eq)]
pub enum EventHandled {
    No,
    Yes,
}

pub struct DisplayLine {
    pub text: String,
    pub cursor_col: Option<usize>,
}

const COMMAND_PREFIX_LEN: usize = 4;


enum HistorySearchMode {
    None,
    Sequential(history::HistorySeqIter),
    Prefix(history::HistoryPrefIter),
    Interactive(history::HistoryInteractiveSearch),
}

pub struct Presenter {
    session: Session,

    window_width: usize,
    window_height: usize,

    button_down: Option<(usize, usize, usize)>,

    last_line_shown: usize,

    current_line: runeline::Runeline,

    history_search: HistorySearchMode,
}

impl ModifierState {
    fn none_pressed(&self) -> bool {
        !(self.shift_pressed | self.control_pressed | self.meta_pressed)
    }
}

impl DisplayLine {
    fn new(line: LineItem) -> DisplayLine {

        // Depending on the type, choose the offset and draw the decoration
        let deco = match line.is_a {
            LineType::Output => "  ",
            LineType::Prompt => "",
            LineType::Command(ref ov, _) => {
                match ov {
                    &OutputVisibility::None => " » ",
                    &OutputVisibility::Output => "O» ",
                    &OutputVisibility::Error => "E» ",
                }
            }
            LineType::Input => "",
            LineType::MenuDecoration => "",
            LineType::SelectedMenuItem(_) => "==> ",
            LineType::MenuItem(_) => "    ",
        };
        DisplayLine {
            text: deco.to_owned() + line.text,
            cursor_col: line.cursor_col,
        }
    }
}

impl Presenter {
    pub fn new() -> Self {

        let mut presenter = Presenter {
            session: Session::new(),
            window_width: 0,
            window_height: 0,
            button_down: None,
            last_line_shown: 0,
            current_line: runeline::Runeline::new(),
            history_search: HistorySearchMode::None,
        };

        let last_line_shown = presenter.line_iter().count() - 1;
        presenter.last_line_shown = last_line_shown;
        presenter
    }

    pub fn start_line(&self) -> usize {
        if self.last_line_shown > self.window_height {
            self.last_line_shown + 1 - self.window_height
        } else {
            0
        }
    }

    fn current_line_pos(&self) -> usize {
        self.current_line.char_index()
    }

    fn last_line_visible(&self) -> bool {
        self.line_iter().count() == (self.last_line_shown + 1)
    }

    fn to_last_line(&mut self) {
        let last_line_shown = self.line_iter().count();
        self.last_line_shown = last_line_shown - 1;
    }

    pub fn poll_interaction(&mut self) -> NeedRedraw {
        let last_line_visible_pre = self.last_line_visible();
        let needs_redraw = self.session.poll_interaction();
        if last_line_visible_pre {
            self.to_last_line();
        }
        if needs_redraw {
            NeedRedraw::Yes
        } else {
            NeedRedraw::No
        }
    }

    pub fn event_window_resize(&mut self, width: usize, height: usize) {
        self.window_width = width;
        self.window_height = height;
        self.button_down = None;
    }

    pub fn event_focus_gained(&mut self) {
        self.button_down = None;
    }

    pub fn event_focus_lost(&mut self) {
        self.button_down = None;
    }

    pub fn event_scroll_down(&mut self, mod_state: ModifierState) -> NeedRedraw {
        if mod_state.none_pressed() {
            if self.last_line_shown + 1 < self.line_iter().count() {
                self.last_line_shown += 1;
                return NeedRedraw::Yes;
            }
        }
        NeedRedraw::No
    }

    pub fn event_scroll_up(&mut self, mod_state: ModifierState) -> NeedRedraw {
        if mod_state.none_pressed() {
            if self.last_line_shown > self.window_height {
                self.last_line_shown -= 1;
                return NeedRedraw::Yes;
            }
        }
        NeedRedraw::No
    }

    pub fn event_cursor_left(&mut self, _mod_state: ModifierState) -> EventHandled {
        self.current_line.move_left();
        EventHandled::Yes
    }

    pub fn event_cursor_right(&mut self, _mod_state: ModifierState) -> EventHandled {
        self.current_line.move_right();
        EventHandled::Yes
    }

    pub fn event_delete_right(&mut self, _mod_state: ModifierState) -> EventHandled {
        self.current_line.delete_right();
        EventHandled::Yes
    }

    pub fn event_backspace(&mut self, _mod_state: ModifierState) -> EventHandled {
        self.current_line.delete_left();
        EventHandled::Yes
    }

    pub fn event_text(&mut self, _mod_state: ModifierState, s: &str) {
        // if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
        //     self.current_line.insert_str(s);
        //     hsi.set_prefix(&self.bash.history, self.current_line.text());
        // } else {
        //     self.clear_history_mode();
        self.current_line.insert_str(s);
        // }
        self.to_last_line();
    }

    pub fn event_return(&mut self, _mod_state: ModifierState) -> EventHandled {
        // if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
        //     if hsi.ind_item < hsi.matching_items.len() {
        //         self.current_line.replace(
        //             self.bash.history.items[hsi.matching_items[hsi.ind_item]].clone(),
        //             false,
        //         );
        //     }
        // };
        let line = self.current_line.clear();
        self.session.add_line(line);
        self.to_last_line();
        EventHandled::Yes
    }

    pub fn event_button_down(
        &mut self,
        _mod_state: ModifierState,
        btn: usize,
        x: usize,
        y: usize,
    ) -> NeedRedraw {
        self.button_down = Some((btn, x, y));
        NeedRedraw::No
    }

    pub fn event_button_up(
        &mut self,
        _mod_state: ModifierState,
        btn: usize,
        x: usize,
        y: usize,
    ) -> NeedRedraw {
        if let Some((down_btn, down_x, down_y)) = self.button_down {
            if down_btn == btn && down_x == x && down_y == y {
                self.button_down = None;
                return self.handle_click(btn, x, y);
            }
        }
        NeedRedraw::No
    }

    pub fn handle_click(&mut self, button: usize, x: usize, y: usize) -> NeedRedraw {
        // Find the item that was clicked
        let click_line_index = self.start_line() + y;
        let is_a = self.line_iter().nth(click_line_index).map(|i| i.is_a);
        match (is_a, button) {
            (Some(LineType::Command(_, pos)), 1) => {
                if x < COMMAND_PREFIX_LEN {
                    // Click on a command
                    let inter = self.session.find_interaction_from_command(pos);
                    let (ov, ev) = match (inter.output.visible, inter.errors.visible) {
                        (true, false) => (false, true),
                        (false, true) => (false, false),
                        _ => (true, false),
                    };
                    inter.output.visible = ov;
                    inter.errors.visible = ev;
                    NeedRedraw::Yes
                } else {
                    NeedRedraw::No
                }
            }
            _ => {
                // Unhandled combination, ignore
                NeedRedraw::No
            }
        }
    }


    pub fn event_cursor_up(&mut self, _mod_state: ModifierState) -> EventHandled {
        EventHandled::No
    }
    pub fn event_cursor_down(&mut self, _mod_state: ModifierState) -> EventHandled {
        EventHandled::No
    }
    pub fn event_page_up(&mut self, _mod_state: ModifierState) -> EventHandled {
        EventHandled::No
    }
    pub fn event_page_down(&mut self, _mod_state: ModifierState) -> EventHandled {
        EventHandled::No
    }

    pub fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(self.session.line_iter().chain(
            ::std::iter::once(LineItem::new(
                self.current_line.text(),
                LineType::Input,
                Some(self.current_line_pos()),
            )),
        ))
    }

    pub fn display_line_iter<'a>(&'a self) -> Box<Iterator<Item = DisplayLine> + 'a> {
        let iter = self.line_iter();
        let start_line = self.start_line();
        Box::new(iter.skip(start_line).map(DisplayLine::new))
    }

    //   pub fn line_iter_history_search<'a>(
    //       &'a self,
    //       hsi: &'a history::HistoryInteractiveSearch,
    //   ) -> Box<Iterator<Item = LineItem> + 'a> {
    //       Box::new(
    //           hsi.matching_items
    //               .iter()
    //               .zip(0..)
    //               .map(move |(hist_ind, match_ind)| {
    //                   LineItem::new(
    //                       self.bash.history.items[*hist_ind].as_str(),
    //                       if match_ind == hsi.ind_item {
    //                           LineType::SelectedMenuItem(*hist_ind)
    //                       } else {
    //                           LineType::MenuItem(*hist_ind)
    //                       },
    //                       None,
    //                   )
    //               })
    //               .chain(::std::iter::once(LineItem::new(
    //                   self.current_line.text(),
    //                   LineType::Input,
    //                   Some(self.current_line_pos()),
    //               ))),
    //       )
    //   }
    //
    //
    //   pub fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
    //       if let HistorySearchMode::Interactive(ref hsi) = self.history_search {
    //           self.line_iter_history_search(hsi)
    //       } else {
    //           self.line_iter_normal()
    //       }
    //   }


    //   fn clear_history_mode(&mut self) {
    //       self.history_search = HistorySearchMode::None;
    //   }
    //
    //   fn history_search_seq(&mut self, reverse: bool) {
    //       match self.history_search {
    //           HistorySearchMode::Sequential(_) => {}
    //           _ => {
    //               self.history_search =
    //                   HistorySearchMode::Sequential(self.bash.history.seq_iter(reverse));
    //           }
    //       }
    //   }
    //
    //   pub fn previous_history(&mut self) {
    //       if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
    //           hsi.prev();
    //           return;
    //       };
    //       self.history_search_seq(true);
    //       let line = match self.history_search {
    //           HistorySearchMode::Sequential(ref mut iter) => iter.prev(&self.bash.history),
    //           _ => None,
    //       };
    //       match line {
    //           Some(s) => {
    //               self.current_line.replace(s, true);
    //               self.to_last_line();
    //               // TODO: Go to end of line
    //           }
    //           None => self.clear_history_mode(),
    //       }
    //   }
    //
    //   pub fn next_history(&mut self) {
    //       if let HistorySearchMode::Interactive(ref mut hsi) = self.history_search {
    //           hsi.next();
    //           return;
    //       };
    //       self.history_search_seq(false);
    //       let line = match self.history_search {
    //           HistorySearchMode::Sequential(ref mut iter) => iter.next(&self.bash.history),
    //           _ => None,
    //       };
    //       match line {
    //           Some(s) => {
    //               self.current_line.replace(s, true);
    //               self.to_last_line();
    //               // TODO: Go to end of line
    //           }
    //           None => self.clear_history_mode(),
    //       }
    //   }
    //
    //   fn history_search_pref(&mut self, reverse: bool) {
    //       match self.history_search {
    //           HistorySearchMode::Prefix(_) => {}
    //           _ => {
    //               let iter = self.bash.history.prefix_iter(
    //                   self.current_line.text_before_cursor(),
    //                   reverse,
    //               );
    //               self.history_search = HistorySearchMode::Prefix(iter);
    //           }
    //       }
    //   }
    //
    //   pub fn history_search_forward(&mut self) {
    //       self.history_search_pref(false);
    //       let line = match self.history_search {
    //           HistorySearchMode::Prefix(ref mut iter) => iter.next(&self.bash.history),
    //           _ => None,
    //       };
    //       match line {
    //           Some(s) => {
    //               self.current_line.replace(s, true);
    //               self.to_last_line();
    //           }
    //           None => self.clear_history_mode(),
    //       }
    //   }
    //
    //   pub fn history_search_backward(&mut self) {
    //       self.history_search_pref(true);
    //
    //       let line = match self.history_search {
    //           HistorySearchMode::Prefix(ref mut iter) => iter.prev(&self.bash.history),
    //           _ => None,
    //       };
    //       match line {
    //           Some(s) => {
    //               self.current_line.replace(s, true);
    //               self.to_last_line();
    //           }
    //           None => self.clear_history_mode(),
    //       }
    //   }
    //
    //   pub fn history_search_interactive(&mut self) {
    //       println!("history_search_interactive");
    //       match self.history_search {
    //           HistorySearchMode::Interactive(_) => {}
    //           _ => {
    //               self.current_line.clear();
    //               self.history_search =
    //                   HistorySearchMode::Interactive(self.bash.history.begin_interactive_search());
    //               self.to_last_line();
    //           }
    //       }
    //   }
}
