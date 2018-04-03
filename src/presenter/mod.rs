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

use std::sync::mpsc::{Receiver, Sender};

mod runeline;

use model::session::*;
use model::iterators::*;
use model::interaction::*;
use model::error::*;
use model::bash::*;
use model::bash::history::*;
use model::types::*;

/// GUI agnostic representation of the modifier keys
pub struct ModifierState {
    pub shift_pressed: bool,
    pub control_pressed: bool,
    pub meta_pressed: bool,
}

/// Represent a boolean with the semantics 'does the GUI need to be redrawn'.
#[derive(PartialEq, Eq)]
pub enum NeedRedraw {
    No,
    Yes,
}

/// Item for the output iterator to be shown by the GUI.
///
/// Each line can have its own cursor, but the GUI might render them to blink synchronously.
pub struct DisplayLine {
    pub text: String,
    pub cursor_col: Option<usize>,
}

/// Constant to indicate how long the prefix of Command line items (as output by line_iter) is.
///
/// This is used to check if we clicked the prefix.
const COMMAND_PREFIX_LEN: usize = 4;

/// Trait to split the big presenter into several small ones.
///
/// Each SubPresenter handles a different kind of interaction mode, e.g. command composition or
/// history browsing.
trait SubPresenter {
    /// Provide read access to the data that is common to the presenter in all modi.
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons>;

    /// Provide write access to the data that is common to the presenter in all modi.
    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons>;

    /// Poll anything that needs polling.
    ///
    /// Return true if there was new data.
    fn poll_interaction(self: Box<Self>) -> (Box<SubPresenter>, bool);

    /// Return the lines to be presented.
    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a>;

    /// Handle the event when the return key is pressed.
    fn event_return(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;

    /// Handle the event when the cursor up key is pressed.
    fn event_cursor_up(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;

    /// Handle the event when the cursor down key is pressed.
    fn event_cursor_down(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;

    /// Handle the event when the page up key is pressed.
    fn event_page_up(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;

    /// Handle the event when the page down key is pressed.
    fn event_page_down(self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter>;

    /// Handle the event when a modifier and a letter is pressed.
    fn event_control_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<SubPresenter>, bool);

    /// Handle the event when the input string was changed.
    fn event_update_line(self: Box<Self>) -> Box<SubPresenter>;

    /// Handle the event when the mouse was pushed and released at the same position.
    fn handle_click(
        self: Box<Self>,
        button: usize,
        x: usize,
        y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw);
}

/// Data that is common to all presenter views.
struct PresenterCommons {
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
    current_line: runeline::Runeline,

    /// Bash script interpreter.
    bash: Option<Bash>,
}

/// Presenter to input and run commands.
struct ComposeCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,
}

/// Presenter to run commands and send input to their stdin.
struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: Interaction,

    /// Channel to running command
    cmd_input: Sender<String>,

    /// Channel from running command
    cmd_output: Receiver<execute::CommandOutput>,
}

/// Presenter to select an item from the history.
struct HistoryPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,
    /// Current search result
    search: history::HistorySearchCursor,
}

/// The top-level presenter dispatches events to the sub-presenters.
pub struct Presenter(Option<Box<SubPresenter>>);

impl ModifierState {
    /// Returns true if no modifier key is pressed.
    fn none_pressed(&self) -> bool {
        !(self.shift_pressed || self.control_pressed || self.meta_pressed)
    }

    /// Return the modifier flags as a tuple for pattern matching
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
            if b { s } else { "" }
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

impl DisplayLine {
    /// Create a line to be displayed from an session item.
    ///
    /// Decorate the line according to its type and update the cursor position.
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
    /// Allocate a new data struct.
    ///
    /// This will be passed from sub-presenter to sub-presenter on state changes.
    pub fn new() -> Result<Self> {
        let bash = Bash::new()?;
        let prompt = bash.expand_ps1();
        Ok(PresenterCommons {
            session: Session::new(prompt),
            window_width: 0,
            window_height: 0,
            button_down: None,
            current_line: runeline::Runeline::new(),
            last_line_shown: 0,
            bash: Some(bash),
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

    /// Return the index of the character where the cursor is in the current input line.
    fn current_line_pos(&self) -> usize {
        self.current_line.char_index()
    }
}

impl Presenter {
    /// Allocate a new presenter and start presenting in normal mode.
    pub fn new() -> Result<Self> {
        Ok(Presenter(Some(ComposeCommandPresenter::new(
            Box::new(PresenterCommons::new()?),
        ))))
    }

    /// Access sub-presenter read-only for dynamic dispatch
    fn d(&self) -> &Box<SubPresenter> {
        self.0.as_ref().unwrap()
    }

    /// Access sub-presenter read-write for dynamic dispatch
    fn dm(&mut self) -> &mut Box<SubPresenter> {
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

    /// Call an event handler in the sub-presenter.
    ///
    /// Update the sub-presenter if it was changed.
    fn dispatch<T: Fn(Box<SubPresenter>) -> Box<SubPresenter>>(&mut self, f: T) {
        let sp = ::std::mem::replace(&mut self.0, None);
        let new_sp = f(sp.unwrap());
        self.0 = Some(new_sp);
    }

    /// Call an event handler with an additional return value in the sub-presenter.
    ///
    /// Update the sub-presenter if it was changed.
    fn dispatch_res<R, T: Fn(Box<SubPresenter>) -> (Box<SubPresenter>, R)>(&mut self, f: T) -> R {
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
        let needs_redraw = self.dispatch_res(|sp| sp.poll_interaction());
        if last_line_visible_pre {
            self.to_last_line();
        }
        if needs_redraw {
            NeedRedraw::Yes
        } else {
            NeedRedraw::No
        }
    }

    /// Dispatch the event when the input was changed.
    fn event_update_line(&mut self) {
        self.dispatch(|sp| sp.event_update_line());
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

    /// Handle the event that the cursor left key was pressed.
    pub fn event_cursor_left(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.move_left();
    }

    /// Handle the event that the cursor right key was pressed.
    pub fn event_cursor_right(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.move_right();
    }

    /// Handle the event that the delete key was pressed.
    pub fn event_delete_right(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.delete_right();
        self.event_update_line();
    }

    /// Handle the event that the backspace key was pressed.
    pub fn event_backspace(&mut self, _mod_state: ModifierState) {
        self.cm().current_line.delete_left();
        self.event_update_line();
    }

    /// Dispatch the event that Modifier+Letter was pressed.
    pub fn event_control_key(&mut self, mod_state: &ModifierState, letter: u8) -> bool {
        self.dispatch_res(|sp| sp.event_control_key(mod_state, letter))
    }

    /// Handle the event that some text was entered.
    pub fn event_text(&mut self, s: &str) {
        self.cm().current_line.insert_str(s);
        self.event_update_line();
    }

    /// Dispatch the event that the return key was pressed.
    pub fn event_return(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_return(mod_state));
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

    /// Dispatch the event that the cursor up key was pressed.
    pub fn event_cursor_up(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_cursor_up(mod_state));
    }

    /// Dispatch the event that the cursor down key was pressed.
    pub fn event_cursor_down(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_cursor_down(mod_state));
    }

    /// Dispatch the event that the page up key was pressed.
    pub fn event_page_up(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_page_up(mod_state));
    }

    /// Dispatch the event that the page down key was pressed.
    pub fn event_page_down(&mut self, mod_state: &ModifierState) {
        self.dispatch(|sp| sp.event_page_down(mod_state));
    }

    /// Yield an iterator that provides the currently visible lines for display.
    pub fn display_line_iter<'a>(&'a self) -> Box<Iterator<Item = DisplayLine> + 'a> {
        let iter = self.d().line_iter();
        let start_line = self.c().start_line();
        Box::new(iter.skip(start_line).map(DisplayLine::new))
    }
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
    // Find the item that was clicked
    let click_line_index = pres.commons().start_line() + y;
    let is_a = pres.line_iter().nth(click_line_index).map(|i| i.is_a);
    match (is_a, button) {
        (Some(LineType::Command(_, pos)), 1) => {
            if x < COMMAND_PREFIX_LEN {
                // Click on a command
                {
                    let inter = pres.commons_mut().session.find_interaction_from_command(
                        pos,
                    );
                    let (ov, ev) = match (inter.output.visible, inter.errors.visible) {
                        (true, false) => (false, true),
                        (false, true) => (false, false),
                        _ => (true, false),
                    };
                    inter.output.visible = ov;
                    inter.errors.visible = ev;
                }
                return true;
            }
        }
        _ => {
            // Unhandled combination, ignore
        }
    }
    false
}

impl ComposeCommandPresenter {
    /// Allocate a sub-presenter for command composition and input to running programs.
    fn new(commons: Box<PresenterCommons>) -> Box<Self> {
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
        let mut line_ret = line.clone();
        line_ret.push_str("\n");
        let bash = ::std::mem::replace(&mut self.commons.bash, None);
        let mut bash = bash.expect(format!("Internal error! {}:{}", file!(), line!()).as_str());
        let cmd = bash.add_line(line_ret.as_str());
        match cmd {
            Command::Incomplete => self,
            Command::Error(err) => {
                // Parser error. Create a fake interaction with the bad command line and
                // the error message
                let mut inter = Interaction::new(line);
                for l in err.into_iter() {
                    inter.add_error(l);
                }
                inter.prepare_archiving();
                self.commons.session.archive_interaction(inter);
                self
            }
            _ => {
                // Add to history
                bash.history.add_command(line.clone());

                // Execute
                match bash.execute(cmd) {
                    ExecutionResult::Ignore => self,
                    ExecutionResult::Spawned((tx, rx)) => {
                        Box::new(ExecuteCommandPresenter {
                            commons: self.commons,
                            cmd_input: tx,
                            cmd_output: rx,
                            current_interaction: Interaction::new(line.clone()),
                        })
                    }
                    ExecutionResult::Builtin(bi) => {
                        let mut inter = Interaction::new(line);
                        inter.add_output_vec(bi.output);
                        inter.add_errors_vec(bi.errors);
                        inter.prepare_archiving();
                        self.commons.session.archive_interaction(inter);
                        self
                    }
                    ExecutionResult::Err(msg) => {
                        // Something happened during program start
                        let mut inter = Interaction::new(line);
                        inter.add_errors_lines(msg);
                        inter.prepare_archiving();
                        self.commons.session.archive_interaction(inter);
                        self
                    }
                }
            }
        }
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
    fn event_control_key(
        mut self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<SubPresenter>, bool) {
        match (mod_state.as_tuple(), letter) {
            ((false, true, false), b'r') => {
                // Control-R -> Start interactive history search
                let prefix = String::from(self.commons.current_line.text_before_cursor());
                self.commons.current_line.clear();
                self.commons.current_line.insert_str(&prefix);
                (
                    HistoryPresenter::new(self.commons, HistorySearchMode::Contained(prefix), true),
                    true,
                )
            }
            _ => (self, false),
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

impl SubPresenter for ExecuteCommandPresenter {
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn poll_interaction(mut self: Box<Self>) -> (Box<SubPresenter>, bool) {
        let mut clear_spawned = false;
        let mut needs_marking = false;
        if let Ok(line) = self.cmd_output.try_recv() {
            needs_marking = true;
            match line {
                execute::CommandOutput::FromOutput(line) => {
                    self.current_interaction.add_output(line);
                }
                execute::CommandOutput::FromError(line) => {
                    self.current_interaction.add_error(line);
                }
                execute::CommandOutput::Terminated(_exit_code, bash) => {
                    // TODO: show the exit code if there is an error
                    self.commons.bash = Some(bash);
                    clear_spawned = true;
                }
            }
        }
        if clear_spawned {
            self.current_interaction.prepare_archiving();
            let ci = ::std::mem::replace(
                &mut self.current_interaction,
                Interaction::new(String::from("")),
            );
            self.commons.session.archive_interaction(ci);
            (ComposeCommandPresenter::new(self.commons), needs_marking)
        } else {
            (self, needs_marking)
        }
    }

    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        Box::new(
            self.commons.session.line_iter().chain(
                self.current_interaction
                    .line_iter(CommandPosition::CurrentInteraction)
                    .chain(::std::iter::once(LineItem::new(
                        self.commons.current_line.text(),
                        LineType::Input,
                        Some(self.commons.current_line_pos()),
                    ))),
            ),
        )
    }

    fn event_return(mut self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        let line = self.commons.current_line.clear();
        self.current_interaction.add_output(line.clone());
        self.cmd_input.send(line + "\n").unwrap();
        self
    }

    fn event_cursor_up(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        self
    }

    fn event_cursor_down(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        self
    }

    fn event_page_up(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        self
    }

    fn event_page_down(self: Box<Self>, _mod_state: &ModifierState) -> Box<SubPresenter> {
        self
    }

    fn event_control_key(
        self: Box<Self>,
        _mod_state: &ModifierState,
        _letter: u8,
    ) -> (Box<SubPresenter>, bool) {
        (self, false)
    }

    fn event_update_line(self: Box<Self>) -> Box<SubPresenter> {
        self
    }

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
}


impl HistoryPresenter {
    /// Allocate a new sub-presenter for history browsing.
    ///
    /// The filter for determining which items to show is passed in mode.
    fn new(
        commons: Box<PresenterCommons>,
        mode: HistorySearchMode,
        reverse: bool,
    ) -> Box<HistoryPresenter> {
        let search = commons
            .bash
            .as_ref()
            .expect(format!("Internal error! {}:{}", file!(), line!()).as_str())
            .history
            .search(mode, reverse);
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
                        self.commons
                            .bash
                            .as_ref()
                            .expect(format!("Internal error! {}:{}", file!(), line!()).as_str())
                            .history
                            .items
                            [*hist_ind]
                            .as_str(),
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

    /// Handle pressing the return key.
    ///
    /// Extract the selected line from history, switch state to the normal presenter and make it
    /// handle the line as if it was entered.
    fn event_return(mut self: Box<Self>, mod_state: &ModifierState) -> Box<SubPresenter> {
        let propagate = if self.search.item_ind < self.search.matching_items.len() {
            let hist_ind = self.search.matching_items[self.search.item_ind];
            let item = self.commons
                .bash
                .as_ref()
                .expect(format!("Internal error! {}:{}", file!(), line!()).as_str())
                .history
                .items
                [hist_ind]
                .clone();
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
        let mut search = self.commons
            .bash
            .as_ref()
            .expect(format!("Internal error! {}:{}", file!(), line!()).as_str())
            .history
            .search(HistorySearchMode::Contained(prefix), false);

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
        let mut dist = self.commons
            .bash
            .as_ref()
            .expect(format!("Internal error! {}:{}", file!(), line!()).as_str())
            .history
            .items
            .len();
        for i in 0..search.matching_items.len() {
            let history_ind = search.matching_items[i];
            let d = abs_diff(last_history_ind, history_ind);
            if d < dist {
                dist = d;
                ind_item = i;
            }
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
    ) -> (Box<SubPresenter>, bool) {

        (self, false)
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
