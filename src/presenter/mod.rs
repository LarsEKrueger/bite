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

//! Presenter component of the model-view-presenter pattern.
//!
//! The presenter dispatches all events to sub-presenters that handle different views, e.g. command
//! composition or history browsing.

use std::fmt::{Display, Formatter};
use std::process::ExitStatus;
use std::sync::mpsc::Receiver;

use term::terminfo::TermInfo;

mod compose_command;
pub mod display_line;
mod execute_command;
mod history;
mod tui;

use model::bash::BashOutput;
use model::error::*;
use model::iterators::*;
use model::screen::*;
use model::session::*;

use self::compose_command::*;
use self::display_line::*;
use self::execute_command::*;

/// GUI agnostic representation of the modifier keys
#[derive(Debug)]
pub struct ModifierState {
    pub shift_pressed: bool,
    pub control_pressed: bool,
    pub meta_pressed: bool,
}

/// GUI agnostic representation of special keys, e.g. function, cursor
#[derive(Debug)]
pub enum SpecialKey {
    Escape,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Backspace,
    Tab,
}

/// Represent a boolean with the semantics 'does the GUI need to be redrawn'.
#[derive(PartialEq, Eq)]
pub enum NeedRedraw {
    No,
    Yes,
}

/// Constant to indicate how long the prefix of Command line items (as output by line_iter) is.
///
/// This is used to check if we clicked the prefix.
const COMMAND_PREFIX_LEN: usize = 4;

#[derive(Debug)]
pub enum PresenterCommand {
    /// Unknown key combination, not handled
    Unknown,

    /// Key combination has been dealt with, redraw gui
    Redraw,

    /// Exit bite
    Exit,
}

/// Trait to split the big presenter into several small ones.
///
/// Each SubPresenter handles a different kind of interaction mode, e.g. command composition or
/// history browsing.
trait SubPresenter {
    /// Provide read access to the data that is common to the presenter in all modi.
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons>;

    /// Provide write access to the data that is common to the presenter in all modi.
    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons>;

    /// Extract the commons and forget the presenter
    fn to_commons(self) -> Box<PresenterCommons>;

    fn add_output(self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]);
    fn add_error(self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]);
    fn set_exit_status(self: &mut Self, exit_status: ExitStatus);
    fn set_next_prompt(self: &mut Self, bytes: &[u8]);
    fn end_polling(self: Box<Self>, needs_marking: bool) -> Box<dyn SubPresenter>;

    /// Return the lines to be presented.
    fn line_iter<'a>(&'a self) -> Box<dyn Iterator<Item = LineItem> + 'a>;

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<dyn SubPresenter>, PresenterCommand);

    /// Handle the event when a modifier and a letter/number is pressed.
    fn event_normal_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<dyn SubPresenter>, PresenterCommand);

    /// Handle input of normal text
    fn event_text(self: Box<Self>, s: &str) -> (Box<dyn SubPresenter>, PresenterCommand);

    /// Handle the event when the mouse was pushed and released at the same position.
    fn handle_click(
        self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<dyn SubPresenter>, NeedRedraw);
}

/// Data that is common to all presenter views.
pub struct PresenterCommons {
    /// The current and previous commands and the outputs of them.
    session: Session,

    /// Width of the window in characters
    window_width: usize,

    /// Height of the window in characters
    window_height: usize,

    /// Position where a mouse button was pushed.
    ///
    /// Only the first click is remembered.
    button_down: Option<(usize, usize, usize)>,

    /// Index post the lowest line that is displayed.
    ///
    /// This is the index of first line that is not shown, i.e. the one below the end of the
    /// screen.
    last_line_shown: usize,

    /// Currently edited input line
    text_input: Screen,

    // List of all lines we have successfully parsed.
    // pub history: History,
    /// Channel from Bash
    receiver: Receiver<BashOutput>,

    /// TermInfo entry for xterm
    term_info: TermInfo,
}

/// The top-level presenter dispatches events to the sub-presenters.
pub struct Presenter(Option<Box<dyn SubPresenter>>);

impl ModifierState {
    /// Returns true if no modifier key is pressed.
    fn none_pressed(&self) -> bool {
        !(self.shift_pressed || self.control_pressed || self.meta_pressed)
    }

    /// Return the modifier flags as a tuple (shift,control,meta) for pattern matching
    fn as_tuple(&self) -> (bool, bool, bool) {
        (self.shift_pressed, self.control_pressed, self.meta_pressed)
    }

    /// Check if any modifier but shift is pressed.
    pub fn not_only_shift(&self) -> bool {
        self.control_pressed || self.meta_pressed
    }
}

impl Display for ModifierState {
    /// Show the modifier state as a prefix for a key.
    fn fmt(&self, f: &mut Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        fn b2s(b: bool, s: &str) -> &str {
            if b {
                s
            } else {
                ""
            }
        }

        write!(
            f,
            "{}{}{}",
            b2s(self.shift_pressed, "Shift-"),
            b2s(self.control_pressed, "Ctrl-"),
            b2s(self.meta_pressed, "Meta-")
        )
    }
}

impl PresenterCommons {
    /// Allocate a new data struct.
    ///
    /// This will be passed from sub-presenter to sub-presenter on state changes.
    pub fn new(receiver: Receiver<BashOutput>, term_info: TermInfo) -> Result<Self> {
        // let history = History::new(bash.get_current_user_home_dir());
        let mut prompt = Screen::new();
        let _ = prompt.add_bytes(b"System");
        let mut text_input = Screen::new();
        text_input.make_room();
        Ok(PresenterCommons {
            session: Session::new(prompt.freeze()),
            window_width: 0,
            window_height: 0,
            button_down: None,
            text_input,
            last_line_shown: 0,
            // history,
            receiver,
            term_info,
        })
    }

    /// Compute the index of the first line to be shown.
    pub fn start_line(&self) -> usize {
        if self.last_line_shown > self.window_height {
            self.last_line_shown - self.window_height
        } else {
            0
        }
    }

    pub fn input_line_iter(&self) -> impl Iterator<Item = LineItem> {
        self.text_input
            .line_iter()
            .zip(0..)
            .map(move |(cells, row)| {
                let cursor_col = if row == self.text_input.cursor_y() {
                    Some(self.text_input.cursor_x() as usize)
                } else {
                    None
                };
                LineItem::new(cells, LineType::Input, cursor_col, 0)
            })
    }

    fn text_input_add_characters(&mut self, s: &str) {
        let ref mut screen = self.text_input;
        for c in s.chars() {
            screen.insert_character();
            screen.place_char(c);
        }
    }
}

impl Presenter {
    /// Allocate a new presenter and start presenting in normal mode.
    pub fn new(receiver: Receiver<BashOutput>, term_info: TermInfo) -> Result<Self> {
        Ok(Presenter(Some(ExecuteCommandPresenter::new(
            Box::new(PresenterCommons::new(receiver, term_info)?),
            Screen::one_line_matrix(b"Startup"),
        ))))
    }

    /// Access sub-presenter read-only for dynamic dispatch
    fn d(&self) -> &Box<dyn SubPresenter> {
        self.0.as_ref().unwrap()
    }

    /// Access sub-presenter read-write for dynamic dispatch
    fn dm(&mut self) -> &mut Box<dyn SubPresenter> {
        self.0.as_mut().unwrap()
    }

    /// Access the common fields read-only
    fn c(&self) -> &PresenterCommons {
        self.d().commons().as_ref()
    }

    /// Access the common fields read-write
    fn cm(&mut self) -> &mut PresenterCommons {
        self.dm().commons_mut().as_mut()
    }

    // /// Call an event handler in the sub-presenter.
    // ///
    // /// Update the sub-presenter if it was changed.
    // fn dispatch<T: Fn(Box<SubPresenter>) -> Box<SubPresenter>>(&mut self, f: T) {
    //     let sp = ::std::mem::replace(&mut self.0, None);
    //     let new_sp = f(sp.unwrap());
    //     self.0 = Some(new_sp);
    // }

    /// Call an event handler with an additional return value in the sub-presenter.
    ///
    /// Update the sub-presenter if it was changed.
    fn dispatch_res<R, T: Fn(Box<dyn SubPresenter>) -> (Box<dyn SubPresenter>, R)>(
        &mut self,
        f: T,
    ) -> R {
        let sp = ::std::mem::replace(&mut self.0, None);
        let (new_sp, res) = f(sp.unwrap());
        self.0 = Some(new_sp);
        res
    }

    /// Check if the view is scrolled down to the bottom to facilitate auto-scrolling.
    fn last_line_visible(&self) -> bool {
        self.d().line_iter().count() == self.c().last_line_shown
    }

    /// Ensure that the last line is visible, even if the number of lines was changed.
    fn to_last_line(&mut self) {
        let len = self.d().line_iter().count();
        self.cm().last_line_shown = len;
    }

    /// Poll the session if new data arrived from a running command.
    ///
    /// Tell the view that is should it redraw itself soon.
    pub fn poll_interaction(&mut self) -> NeedRedraw {
        let last_line_visible_pre = self.last_line_visible();
        let needs_redraw = self.dispatch_res(|sp| {
            let mut needs_marking = false;
            let mut presenter = sp;
            if let Ok(output) = presenter.commons_mut().receiver.try_recv() {
                needs_marking = true;
                match output {
                    BashOutput::FromOutput(full_line) => {
                        let mut line = &full_line[..];
                        while line.len() != 0 {
                            let (pres, rest) = presenter.add_output(line);
                            presenter = pres;
                            line = rest;
                        }
                    }
                    BashOutput::FromError(full_line) => {
                        let mut line = &full_line[..];
                        while line.len() != 0 {
                            let (pres, rest) = presenter.add_error(line);
                            presenter = pres;
                            line = rest;
                        }
                    }
                    BashOutput::Terminated(exit_code) => {
                        presenter.set_exit_status(exit_code);
                    }
                    BashOutput::Prompt(prompt) => {
                        presenter.set_next_prompt(&prompt);
                    }
                }
            }

            presenter = presenter.end_polling(needs_marking);
            (presenter, needs_marking)
        });
        if last_line_visible_pre {
            self.to_last_line();
        }
        if needs_redraw {
            NeedRedraw::Yes
        } else {
            NeedRedraw::No
        }
    }

    /// Handle the View event when the window size changes.
    pub fn event_window_resize(&mut self, width: usize, height: usize) {
        let commons = self.cm();
        commons.window_width = width;
        commons.window_height = height;
        commons.button_down = None;
    }

    /// Handle the view event when the window regained focus.
    pub fn event_focus_gained(&mut self) {
        self.cm().button_down = None;
    }

    /// Handle the view event when the window lost focus.
    pub fn event_focus_lost(&mut self) {
        self.cm().button_down = None;
    }

    /// Handle the event that the window was scrolled down.
    pub fn event_scroll_down(&mut self, mod_state: ModifierState) -> NeedRedraw {
        if mod_state.none_pressed() {
            if self.c().last_line_shown < self.d().line_iter().count() {
                self.cm().last_line_shown += 1;
                return NeedRedraw::Yes;
            }
        }
        NeedRedraw::No
    }

    /// Handle the event that the window was scrolled up.
    pub fn event_scroll_up(&mut self, mod_state: ModifierState) -> NeedRedraw {
        if mod_state.none_pressed() {
            if self.c().last_line_shown > self.c().window_height {
                self.cm().last_line_shown -= 1;
                return NeedRedraw::Yes;
            }
        }
        NeedRedraw::No
    }

    pub fn event_special_key(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        self.dispatch_res(|sp| sp.event_special_key(mod_state, key))
    }

    /// Dispatch the event that Modifier+Letter was pressed.
    pub fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand {
        self.dispatch_res(|sp| sp.event_normal_key(mod_state, letter))
    }

    /// Handle the event that some text was entered.
    ///
    /// TODO: Handle escape sequences
    pub fn event_text(&mut self, s: &str) -> PresenterCommand {
        self.dispatch_res(|sp| sp.event_text(s))
        //       {
        //       }
        //       self.event_update_line();
    }

    /// Handle the event that a mouse button was pressed.
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

    /// Handle the event that a mouse button was released.
    ///
    /// If the same button was released at the position where it was pressed, dispatch the click
    /// event to the sub-presenter.
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

    /// Yield an iterator that provides the currently visible lines for display.
    pub fn display_line_iter<'a>(&'a self) -> impl Iterator<Item = DisplayLine<'a>> {
        let iter = self.d().line_iter();
        let start_line = self.c().start_line();
        iter.skip(start_line).into_iter().map(DisplayLine::from)
    }
}

/// Get the line type of the line clicked.
fn clicked_line_type<T: SubPresenter>(pres: &mut T, y: usize) -> Option<LineType> {
    // Find the item that was clicked
    let click_line_index = pres.commons().start_line() + y;
    pres.line_iter().nth(click_line_index).map(|i| i.is_a)
}

/// Check if the response selector has been clicked and update the visibility flags
/// accordingly.
///
/// This is used by ComposeCommandPresenter and ExecuteCommandPresenter.
fn check_response_clicked<T: SubPresenter>(
    pres: &mut T,
    button: usize,
    x: usize,
    y: usize,
) -> bool {
    let is_a = clicked_line_type(pres, y);
    match (is_a, button) {
        (Some(LineType::Command(_, pos, _)), 1) => {
            if x < COMMAND_PREFIX_LEN {
                // Click on a command
                pres.commons_mut()
                    .session
                    .find_interaction_from_command(pos)
                    .map(|i| i.cycle_visibility());
                return true;
            }
        }
        _ => {
            // Unhandled combination, ignore
        }
    }
    false
}
