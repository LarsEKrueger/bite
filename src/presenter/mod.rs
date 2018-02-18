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

mod runeline;

use model::session::*;
use model::iterators::*;
use model::interaction::*;
use model::bash::*;
use model::bash::history::*;

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

pub struct DisplayLine {
    pub text: String,
    pub cursor_col: Option<usize>,
}

const COMMAND_PREFIX_LEN: usize = 4;

trait SubPresenter {
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons>;

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons>;

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a>;

    fn event_return(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;
    fn event_cursor_up(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;
    fn event_cursor_down(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;
    fn event_page_up(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;
    fn event_page_down(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;

    fn handle_click(
        self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw);
}

struct PresenterCommons {
    session: Session,

    window_width: usize,
    window_height: usize,

    button_down: Option<(usize, usize, usize)>,

    last_line_shown: usize,

    current_line: runeline::Runeline,
}

struct ComposeCommandPresenter {
    commons: Box<PresenterCommons>,
}

struct HistoryPresenter {
    commons: Box<PresenterCommons>,
    search: history::HistorySearchCursor,
}

pub struct Presenter(Option<Box<SubPresenter>>);

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

impl PresenterCommons {
    pub fn new() -> Self {
        PresenterCommons {
            session: Session::new(),
            window_width: 0,
            window_height: 0,
            button_down: None,
            current_line: runeline::Runeline::new(),
            last_line_shown: 0,
        }
    }

    pub fn start_line(&self) -> usize {
        if self.last_line_shown > self.window_height {
            self.last_line_shown - self.window_height
        } else {
            0
        }
    }

    fn current_line_pos(&self) -> usize {
        self.current_line.char_index()
    }
}

impl Presenter {
    pub fn new() -> Self {
        Presenter(Some(ComposeCommandPresenter::new(
            Box::new(PresenterCommons::new()),
        )))
    }

    fn d(&self) -> &Box<SubPresenter> {
        self.0.as_ref().unwrap()
    }

    fn dm(&mut self) -> &mut Box<SubPresenter> {
        self.0.as_mut().unwrap()
    }

    fn c(&self) -> &PresenterCommons {
        self.d().commons().as_ref()
    }

    fn cm(&mut self) -> &mut PresenterCommons {
        self.dm().commons_mut().as_mut()
    }

    fn dispatch<T: Fn(Box<SubPresenter>) -> Box<SubPresenter>>(&mut self, f: T) {
        let sp = ::std::mem::replace(&mut self.0, None);
        let new_sp = f(sp.unwrap());
        self.0 = Some(new_sp);
    }

    fn dispatch_res<R, T: Fn(Box<SubPresenter>) -> (Box<SubPresenter>, R)>(&mut self, f: T) -> R {
        let sp = ::std::mem::replace(&mut self.0, None);
        let (new_sp, res) = f(sp.unwrap());
        self.0 = Some(new_sp);
        res
    }

    fn last_line_visible(&self) -> bool {
        self.d().line_iter().count() == self.c().last_line_shown
    }

    fn to_last_line(&mut self) {
        let len = self.d().line_iter().count();
        self.cm().last_line_shown = len;
    }

    pub fn poll_interaction(&mut self) -> NeedRedraw {
        let last_line_visible_pre = self.last_line_visible();
        let needs_redraw = self.cm().session.poll_interaction();
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
        let commons = self.cm();
        commons.window_width = width;
        commons.window_height = height;
        commons.button_down = None;
    }

    pub fn event_focus_gained(&mut self) {
        self.cm().button_down = None;
    }

    pub fn event_focus_lost(&mut self) {
        self.cm().button_down = None;
    }

    pub fn event_scroll_down(&mut self, mod_state: ModifierState) -> NeedRedraw {
        if mod_state.none_pressed() {
            if self.c().last_line_shown < self.d().line_iter().count() {
                self.cm().last_line_shown += 1;
                return NeedRedraw::Yes;
            }
        }
        NeedRedraw::No
    }

    pub fn event_scroll_up(&mut self, mod_state: ModifierState) -> NeedRedraw {
        if mod_state.none_pressed() {
            if self.c().last_line_shown > self.c().window_height {
                self.cm().last_line_shown -= 1;
                return NeedRedraw::Yes;
            }
        }
        NeedRedraw::No
    }

    pub fn event_cursor_left(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.move_left();
    }

    pub fn event_cursor_right(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.move_right();
    }

    pub fn event_delete_right(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.delete_right();
    }

    pub fn event_backspace(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.delete_left();
    }

    pub fn event_text(&mut self, _mod_state: ModifierState, s: &str) {
        self.cm().current_line.insert_str(s);
        self.to_last_line();
    }

    pub fn event_return(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_return(mod_state));
    }

    pub fn event_button_down(
        &mut self,
        _mod_state: ModifierState,
        btn: usize,
        x: usize,
        y: usize,
    ) -> NeedRedraw {
        self.cm().button_down = Some((btn, x, y));
        NeedRedraw::No
    }

    pub fn event_button_up(
        &mut self,
        _mod_state: ModifierState,
        btn: usize,
        x: usize,
        y: usize,
    ) -> NeedRedraw {
        if let Some((down_btn, down_x, down_y)) = self.c().button_down {
            if down_btn == btn && down_x == x && down_y == y {
                self.cm().button_down = None;
                return self.dispatch_res(|sp| sp.handle_click(btn, x, y));
            }
        }
        NeedRedraw::No
    }

    pub fn event_cursor_up(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_cursor_up(mod_state));
    }

    pub fn event_cursor_down(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_cursor_down(mod_state));
    }

    pub fn event_page_up(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_page_up(mod_state));
    }

    pub fn event_page_down(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_page_down(mod_state));
    }

    pub fn display_line_iter<'a>(&'a self) -> Box<Iterator<Item = DisplayLine> + 'a> {
        let iter = self.d().line_iter();
        let start_line = self.c().start_line();
        Box::new(iter.skip(start_line).map(DisplayLine::new))
    }

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

impl ComposeCommandPresenter {
    fn new(commons: Box<PresenterCommons>) -> Box<Self> {
        let mut presenter = ComposeCommandPresenter { commons };
        presenter.to_last_line();
        Box::new(presenter)
    }

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
        self.commons.session.add_line(line);
        self.to_last_line();
        self
    }

    fn handle_click(
        mut self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw) {
        // Find the item that was clicked
        let click_line_index = self.commons.start_line() + y;
        let is_a = self.line_iter().nth(click_line_index).map(|i| i.is_a);
        match (is_a, button) {
            (Some(LineType::Command(_, pos)), 1) => {
                if x < COMMAND_PREFIX_LEN {
                    // Click on a command
                    {
                        let inter = self.commons.session.find_interaction_from_command(pos);
                        let (ov, ev) = match (inter.output.visible, inter.errors.visible) {
                            (true, false) => (false, true),
                            (false, true) => (false, false),
                            _ => (true, false),
                        };
                        inter.output.visible = ov;
                        inter.errors.visible = ev;
                    }
                    return (self, NeedRedraw::Yes);
                }
            }
            _ => {
                // Unhandled combination, ignore
            }
        }
        (self, NeedRedraw::No)
    }

    fn event_cursor_up(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        HistoryPresenter::new(self.commons, HistorySearchMode::Browse, true)
    }

    fn event_cursor_down(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        HistoryPresenter::new(self.commons, HistorySearchMode::Browse, false)
    }

    fn event_page_up(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        self
    }

    fn event_page_down(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        self
    }
}

impl HistoryPresenter {
    fn new(
        commons: Box<PresenterCommons>,
        mode: HistorySearchMode,
        reverse: bool,
    ) -> Box<HistoryPresenter> {
        let search = commons.session.bash.history.search(mode, reverse);
        let mut presenter = HistoryPresenter { commons, search };

        presenter.to_last_line();

        Box::new(presenter)
    }

    fn to_last_line(&mut self) {
        let cnt = self.line_iter().count();
        self.commons.last_line_shown = cnt;
    }

    fn show_selection(&mut self) -> NeedRedraw {
        // If the selection is already visible, do nothing. Otherwise, center it on the screen.
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

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.search
                .matching_items
                .iter()
                .zip(0..)
                .map(move |(hist_ind, match_ind)| {
                    LineItem::new(
                        self.commons.session.bash.history.items[*hist_ind].as_str(),
                        if match_ind == self.search.item_ind {
                            LineType::SelectedMenuItem(*hist_ind)
                        } else {
                            LineType::MenuItem(*hist_ind)
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

    fn event_return(mut self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        let item_ind = self.search.matching_items[self.search.item_ind];
        let item = self.commons.session.bash.history.items[item_ind].clone();
        self.commons.current_line.replace(item, false);

        ComposeCommandPresenter::new(self.commons).event_return(mod_state)
    }

    fn handle_click(
        self: Box<Self>,
        _button: usize,
        _x: usize,
        _y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw) {

        (self, NeedRedraw::No)
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
