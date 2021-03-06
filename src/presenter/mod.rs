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

mod completions;
mod compose_command;
pub mod display_line;
mod execute_command;
mod style_sheet;
mod tui;

#[cfg(test)]
mod test;

use std::fmt::{Display, Formatter};
use term::terminfo::TermInfo;

use sesd::{char::CharMatcher, SynchronousEditor};

use self::compose_command::bubble_above;
use self::compose_command::bubble_exclusive;
use self::compose_command::live_parse;
use self::compose_command::markov_below;
use self::display_line::*;
use self::execute_command::ExecuteCommandPresenter;
use self::tui::TuiExecuteCommandPresenter;
use model::error::*;
use model::history::History;
use model::interpreter::grammar;
use model::interpreter::InteractiveInterpreter;
use model::screen::*;
use model::session::{
    ConversationLocator, InteractionHandle, InteractionLocator, LineItem, LineType,
    MaybeSessionLocator, ResponseLocator, Session, SessionLocator, SharedSession,
};

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
    Space,
}

/// A GUI callback that is called once per line drawn
pub trait DrawLineTrait {
    fn draw_line(&self, usize, &DisplayLine);
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

#[derive(Debug, PartialEq)]
pub enum PresenterCommand {
    /// Unknown key combination, not handled
    Unknown,

    /// Known, but ignored key combination
    Ignored,

    /// Key combination has been dealt with, redraw gui
    Redraw,

    /// Exit bite
    Exit,
}

/// Feature flag
#[derive(Debug)]
pub enum ComposeVariant {
    MarkovBelow,
    BubbleAbove,
    BubbleExclusive,
    LiveParse,
}

/// Trait to split the big presenter into several small ones.
///
/// Each SubPresenter handles a different kind of interaction mode, e.g. command composition or
/// history browsing.
trait SubPresenter {
    /// Destroy the presenter and get back the commons
    fn finish(self: Box<Self>) -> Box<PresenterCommons>;

    /// Provide read access to the data that is common to the presenter in all modi.
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons>;

    /// Provide write access to the data that is common to the presenter in all modi.
    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons>;

    /// Get a single DisplayLine for the given line coordinate
    fn single_display_line<'a, 'b: 'a>(
        &'a self,
        session: &'b Session,
        y: usize,
    ) -> Option<DisplayLine<'a>>;

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand;

    /// Handle scrolling up
    fn event_scroll_up(&mut self, mod_state: &ModifierState) -> PresenterCommand;

    /// Handle scrolling down
    fn event_scroll_down(&mut self, mod_state: &ModifierState) -> PresenterCommand;

    /// Handle the event when a modifier and a letter/number is pressed.
    fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand;

    /// Handle input of normal text
    fn event_text(&mut self, s: &str) -> PresenterCommand;

    /// Handle the event when the mouse was pushed and released at the same position.
    fn handle_click(&mut self, button: usize, x: usize, y: usize) -> NeedRedraw;
}

type Editor = SynchronousEditor<char, CharMatcher, grammar::script2::Grammar>;

/// Link the editor's cursor position to the on-screen position
///
/// Used to convert the cursor movement on-screen back to movements in the buffer.
#[derive(Debug)]
pub struct CursorMapping {
    /// Position in editor buffer
    position: usize,
    /// Horizontal position in text_input
    x: isize,
    /// Vertical position in text_input
    y: isize,
}

/// Data that is common to all presenter views.
pub struct PresenterCommons {
    /// The current and previous commands and the outputs of them.
    session: SharedSession,

    /// Interpreter to run things
    interpreter: InteractiveInterpreter,

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
    /// visible section.
    ///
    /// As the input and history fields will occupy the lowest lines on the screen, they don't need
    /// to be taken into account in the scrolling, i.e. scrolling only affects the session.
    ///
    /// In order to simplify the most common case (show everything), this is marked as None.
    session_end_line: Option<SessionLocator>,

    /// Currently edited input line
    text_input: Screen,

    /// List of all lines that have been entered
    history: History,

    /// TermInfo entry for xterm
    term_info: TermInfo,

    /// Parsing editor for the command line
    editor: Editor,

    /// Style sheet for rendering the command line
    style_sheet: style_sheet::StyleSheet,

    /// Map the on-screen cursor position to the in-buffer position
    cursor_map: Vec<CursorMapping>,

    /// Completion algos
    completions: completions::Completions,
}

/// The top-level presenter dispatches events to the sub-presenters.
pub struct Presenter {
    /// The interaction that should be shown fullscreen
    ///
    /// If
    /// * None and interpreter is busy: Show ExecuteCommandPresenter
    /// * None and interpreter is free: Show ComposeCommandPresenter
    /// * Some(i) and i is running and i is tui: Show TuiExecuteCommandPresenter
    /// * Some(i) and i is running and i is not tui: Show FocusExecuteCommandPresenter (not implemented yet)
    /// * Some(i) and i is not running: Show InspectOutputCommandPresenter (not implemented yet)
    focused_interaction: Option<InteractionHandle>,

    /// Sub-presenter to handle the current view
    subpresenter: Option<Box<dyn SubPresenter>>,
    /// Current sub-presenter type
    sp_type: SubPresenterType,

    /// Feature flag: ComposeCommandPresenter Variant
    feat_compose_variant: ComposeVariant,
}

/// Enum to fake C++'s typeof
///
/// Each value is named like the struct it represents
///
/// TODO: Implement InspectOutputCommandPresenter and FocusExecuteCommandPresenter
#[derive(PartialEq, Debug)]
enum SubPresenterType {
    ComposeCommandPresenter,
    ExecuteCommandPresenter(InteractionHandle),
    TuiExecuteCommandPresenter(InteractionHandle),
}

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
    pub fn new(
        session: SharedSession,
        interpreter: InteractiveInterpreter,
        history: History,
        term_info: TermInfo,
    ) -> Result<Self> {
        // let history = History::new(bash.get_current_user_home_dir());
        let mut text_input = Screen::new();
        text_input.make_room();

        let compiled_grammar = crate::model::interpreter::grammar::script2::grammar();
        let style_sheet = style_sheet::script();
        let completions = completions::Completions::new();
        Ok(PresenterCommons {
            session,
            interpreter,
            window_width: 0,
            window_height: 0,
            button_down: None,
            text_input,
            session_end_line: None,
            history,
            term_info,
            editor: Editor::new(compiled_grammar),
            style_sheet,
            cursor_map: Vec::new(),
            completions,
        })
    }

    /// Create a locator at the end of the session
    fn locate_end(session: &Session, show_last_prompt: bool) -> MaybeSessionLocator {
        let loc = session.locate_at_last_prompt_end()?;
        if show_last_prompt {
            Some(loc)
        } else {
            let new_loc = session.locate_at_conversation_end(&loc)?;
            session.locate_at_output_end(&new_loc)
        }
    }

    /// Create a new locator that is n lines above the initial one
    ///
    /// Does not wrap around.
    ///
    /// This function encodes the order in which session elements are drawn. It needs to be kept in
    /// sync with locate_down.
    fn locate_up(session: &Session, loc: &SessionLocator, mut lines: usize) -> MaybeSessionLocator {
        // Go step by step to the next border until lines has been reduced to 0.
        let mut loc = loc.clone();
        while lines > 0 {
            let loc_before = loc.clone();
            if loc.is_start_line() {
                // Where we want to go depends on where we are.
                match loc.in_conversation {
                    // Prompt -> End of last interaction in conversation
                    ConversationLocator::Prompt(_) => {
                        if let Some(new_loc) = session.locate_at_conversation_end(&loc) {
                            loc = session.locate_at_output_end(&new_loc)?
                        } else {
                            // The conversion might not have any interactions yet. Try to get to
                            // the prompt of the previous conversation
                            if let Some(new_loc) = session.locate_at_previous_conversation(&loc) {
                                loc = session.locate_at_prompt_end(&new_loc)?;
                            }
                        }
                    }
                    // Interaction command -> Previous interaction / conversation
                    ConversationLocator::Interaction(_, InteractionLocator::Command(_)) => {
                        if let Some(new_loc) = session.locate_at_previous_interaction(&loc) {
                            // There was a previous interaction -> End of output
                            loc = session.locate_at_output_end(&new_loc)?;
                        } else {
                            // No previous interaction -> End of previous conversation
                            if let Some(new_loc) = session.locate_at_previous_conversation(&loc) {
                                loc = session.locate_at_prompt_end(&new_loc)?;
                            }
                        }
                    }
                    // Output -> Last Line of Command
                    ConversationLocator::Interaction(_, InteractionLocator::Tui(_))
                    | ConversationLocator::Interaction(
                        _,
                        InteractionLocator::Response(ResponseLocator::Lines(_)),
                    ) => loc = session.locate_at_command_end(&loc)?,
                    // Screen -> Last line of lines
                    ConversationLocator::Interaction(
                        _,
                        InteractionLocator::Response(ResponseLocator::Screen(_)),
                    ) => {
                        loc = session.locate_at_lines_end(&loc)?;
                    }
                }
            } else {
                loc.dec_line(&mut lines);
            }
            if loc_before == loc {
                break;
            }
        }
        Some(loc)
    }

    /// Create a new locator that is n lines below the initial one
    ///
    /// Does not wrap around.
    ///
    /// This function encodes the order in which session elements are drawn. It needs to be kept in
    /// sync with locate_up.
    fn locate_down(
        session: &Session,
        start_loc: &SessionLocator,
        show_last_prompt: bool,
        mut lines: usize,
    ) -> MaybeSessionLocator {
        // Go step by step to the next border until lines has been reduced to 0.
        let mut loc = start_loc.clone();
        loop {
            session.locator_inc_line(&mut loc, &mut lines);
            if session.locator_is_end_line(&loc)? {
                // Where we want to go depends on where we are.
                match loc.in_conversation {
                    // Prompt -> First command in next conversation
                    ConversationLocator::Prompt(_) => {
                        loc = session.locate_at_next_conversation(&loc)?;
                    }
                    // Interaction command -> First output line
                    ConversationLocator::Interaction(_, InteractionLocator::Command(_)) => {
                        // If the locator can't be set to the start of the output, there is no
                        // output yet. Go to the prompt directly.
                        if let Some(new_loc) = session.locate_at_output_start(&loc) {
                            loc = new_loc;
                        } else {
                            loc = session.locate_at_prompt_start(&loc)?;
                        }
                    }
                    // Output --> First line of command in next interaction or conversation prompt.
                    ConversationLocator::Interaction(
                        _,
                        InteractionLocator::Response(ResponseLocator::Screen(_)),
                    )
                    | ConversationLocator::Interaction(_, InteractionLocator::Tui(_)) => {
                        if let Some(new_loc) = session.locate_at_next_interaction(&loc) {
                            // There was a next interaction
                            loc = new_loc;
                        } else {
                            if show_last_prompt || (!session.locator_is_last_conversation(&loc)) {
                                // Start of conversation prompt
                                loc = session.locate_at_prompt_start(&loc)?;
                            } else {
                                return None;
                            }
                        }
                    }
                    // Lines -> Screen of same interaction
                    ConversationLocator::Interaction(
                        _,
                        InteractionLocator::Response(ResponseLocator::Lines(_)),
                    ) => {
                        loc = session.locate_at_screen_start(&loc)?;
                        // If the screen is empty, the next iteration will fix it.
                    }
                }
            } else {
                break;
            }
        }
        Some(loc)
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
        self.text_input.insert_str(s);
    }

    /// Change session_end_line by going up n lines. This encodes the order of lines.
    pub fn scroll_up<F>(&mut self, show_last_prompt: bool, n: usize, f: F)
    where
        F: FnOnce(&Session, &SessionLocator) -> MaybeSessionLocator,
    {
        if let Some(ref mut loc) = self.session_end_line {
            let session = self.session.0.lock().unwrap();
            if let Some(new_loc) = Self::locate_up(&session, loc, n) {
                if let Some(fix_loc) = f(&session, &new_loc) {
                    *loc = fix_loc;
                } else {
                    *loc = new_loc;
                }
            }
        } else {
            // Initialize past the end of the session, then correct
            let session = self.session.0.lock().unwrap();
            if let Some(loc) = Self::locate_end(&session, show_last_prompt) {
                if let Some(loc) = Self::locate_up(&session, &loc, n) {
                    if let Some(fix_loc) = f(&session, &loc) {
                        self.session_end_line = Some(fix_loc);
                    } else {
                        self.session_end_line = Some(loc);
                    }
                }
            }
        }
        // TOOD: Limit to scrollable area
    }

    pub fn scroll_down(&mut self, show_last_prompt: bool, n: usize) {
        if let Some(ref mut loc) = self.session_end_line {
            let session = self.session.0.lock().unwrap();
            self.session_end_line = Self::locate_down(&session, loc, show_last_prompt, n);
        }
    }

    pub fn to_last_line(&mut self) {
        self.session_end_line = None;
    }

    /// Return a session locator that refers to the first of n lines to draw
    fn start_line(
        &self,
        session: &Session,
        show_last_prompt: bool,
        n: usize,
    ) -> MaybeSessionLocator {
        self.session_end_line
            .clone()
            .or_else(|| Self::locate_end(&session, show_last_prompt))
            .and_then(|loc| Self::locate_up(&session, &loc, n))
    }
}

impl ComposeVariant {
    fn new_subpresenter(&self, commons: Box<PresenterCommons>) -> Box<dyn SubPresenter> {
        match self {
            Self::MarkovBelow => markov_below::ComposeCommandPresenter::new(commons),
            Self::BubbleAbove => bubble_above::ComposeCommandPresenter::new(commons),
            Self::BubbleExclusive => bubble_exclusive::ComposeCommandPresenter::new(commons),
            Self::LiveParse => live_parse::ComposeCommandPresenter::new(commons),
        }
    }
}

impl Presenter {
    /// Allocate a new presenter and start presenting in normal mode.
    pub fn new(
        session: SharedSession,
        interpreter: InteractiveInterpreter,
        history: History,
        term_info: TermInfo,
        feat_compose_variant: ComposeVariant,
    ) -> Result<Self> {
        let commons = Box::new(PresenterCommons::new(
            session,
            interpreter,
            history,
            term_info,
        )?);
        let subpresenter = feat_compose_variant.new_subpresenter(commons);
        let presenter = Presenter {
            focused_interaction: None,
            subpresenter: Some(subpresenter),
            sp_type: SubPresenterType::ComposeCommandPresenter,
            feat_compose_variant,
        };
        Ok(presenter)
    }

    /// Clean up and get back the interpreter
    pub fn finish(self) -> (InteractiveInterpreter, History) {
        let commons = self.subpresenter.unwrap().finish();
        (commons.interpreter, commons.history)
    }

    /// Access sub-presenter read-only for dynamic dispatch
    fn d(&self) -> &Box<dyn SubPresenter> {
        trace!("d(): {:?}", self.subpresenter.is_some());
        self.subpresenter.as_ref().unwrap()
    }

    /// Access sub-presenter read-write for dynamic dispatch
    fn dm(&mut self) -> &mut Box<dyn SubPresenter> {
        trace!("dm(): {:?}", self.subpresenter.is_some());
        self.subpresenter.as_mut().unwrap()
    }

    /// Access the common fields read-only
    fn c(&self) -> &PresenterCommons {
        self.d().commons().as_ref()
    }

    /// Access the common fields read-write
    fn cm(&mut self) -> &mut PresenterCommons {
        self.dm().commons_mut().as_mut()
    }

    /// Create a new prompt string and check if a new conversation needs to be created.
    pub fn update_prompt(&mut self) {
        let user_name = {
            let cuid = nix::unistd::getuid().as_raw();
            unsafe {
                let pw = libc::getpwuid(cuid);
                std::ffi::CStr::from_ptr((*pw).pw_name as *const libc::c_char)
            }
        };
        let cwd = match nix::unistd::getcwd() {
            Ok(cwd) => cwd,
            _ => std::path::PathBuf::new(),
        };
        let prompt_string = format!(
            "{}@{} {}",
            user_name.to_string_lossy(),
            nix::sys::utsname::uname().nodename(),
            cwd.to_string_lossy()
        );

        let screen = Screen::one_line_matrix(prompt_string.as_bytes());
        self.cm().session.new_conversation(screen);
    }

    /// Prepare the presenter for the new cycle.
    ///
    /// Return true if a redraw is required.
    pub fn prepare_cycle(&mut self) -> bool {
        // Determine which sub-presenter show be used
        // If focused_interaction is
        // * None and interpreter is busy: Show ExecuteCommandPresenter or
        //   TuiExecuteCommandPresenter
        // * None and interpreter is free: Show ComposeCommandPresenter
        // * Some(i) and i is running and i is tui: Show TuiExecuteCommandPresenter
        // * Some(i) and i is running and i is not tui: Show FocusExecuteCommandPresenter (not implemented yet)
        // * Some(i) and i is not running: Show InspectOutputCommandPresenter (not implemented yet)

        let sp_type = match self.focused_interaction {
            None => self.c().interpreter.is_busy().map_or(
                SubPresenterType::ComposeCommandPresenter,
                |h| {
                    if self.c().session.is_tui(h) {
                        SubPresenterType::TuiExecuteCommandPresenter(h)
                    } else {
                        SubPresenterType::ExecuteCommandPresenter(h)
                    }
                },
            ),
            Some(handle) => {
                if self.c().session.has_exited(handle) {
                    // TODO: Implement InspectOutputCommandPresenter
                    self.focused_interaction = None;
                    self.c()
                        .interpreter
                        .is_busy()
                        .map_or(SubPresenterType::ComposeCommandPresenter, |h| {
                            SubPresenterType::ExecuteCommandPresenter(h)
                        })
                } else {
                    if self.c().session.is_tui(handle) {
                        SubPresenterType::TuiExecuteCommandPresenter(handle)
                    } else {
                        // TODO: Implement FocusExecuteCommandPresenter
                        self.focused_interaction = None;
                        self.c()
                            .interpreter
                            .is_busy()
                            .map_or(SubPresenterType::ComposeCommandPresenter, |h| {
                                SubPresenterType::ExecuteCommandPresenter(h)
                            })
                    }
                }
            }
        };

        // The GUI needs to be redrawn if the session has been changed.
        let mut redraw = self.dm().commons_mut().session.check_redraw();
        // If the new sp_type is different from the old one, transfer ownership from one to the
        // other.
        if sp_type != self.sp_type {
            trace!("Switching to subpresenter {:?}", sp_type);
            self.sp_type = sp_type;
            // The GUI also needs to be redrawn if the presenter was changed.
            redraw = true;
            let old_sp = std::mem::replace(&mut self.subpresenter, None);
            let commons = old_sp.unwrap().finish();
            let mut update_prompt = false;
            self.subpresenter = Some(match self.sp_type {
                SubPresenterType::ComposeCommandPresenter => {
                    update_prompt = true;
                    self.feat_compose_variant.new_subpresenter(commons)
                }
                SubPresenterType::ExecuteCommandPresenter(handle) => {
                    ExecuteCommandPresenter::new(commons, handle)
                }
                SubPresenterType::TuiExecuteCommandPresenter(handle) => {
                    TuiExecuteCommandPresenter::new(commons, handle)
                }
            });
            trace!("Switched to subpresenter {:?}", self.sp_type);
            if update_prompt {
                self.update_prompt();
            }
        }
        redraw
    }

    /// Handle the View event when the window size changes.
    pub fn event_window_resize(&mut self, width: usize, height: usize) {
        let commons = self.cm();
        commons.window_width = width;
        commons.window_height = height;
        commons.button_down = None;
        commons.session.set_window_size(width, height);
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
    pub fn event_scroll_down(&mut self, mod_state: &ModifierState) -> PresenterCommand {
        self.dm().event_scroll_down(mod_state)
    }

    /// Handle the event that the window was scrolled up.
    pub fn event_scroll_up(&mut self, mod_state: &ModifierState) -> PresenterCommand {
        self.dm().event_scroll_up(mod_state)
    }

    pub fn event_special_key(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            // Ctrl-Tab => Switch to next running TUI if there is one
            ((false, true, false), SpecialKey::Tab) => {
                let current_focus = self.focused_interaction;
                let next_focus = self.c().session.next_running_tui(current_focus);
                trace!("Ctrl-Tab next_focus: {:?}", next_focus);
                self.focused_interaction = next_focus;
                return PresenterCommand::Redraw;
            }
            _ => {}
        }
        self.dm().event_special_key(mod_state, key)
    }

    /// Dispatch the event that Modifier+Letter was pressed.
    pub fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand {
        self.dm().event_normal_key(mod_state, letter)
    }

    /// Handle the event that some text was entered.
    ///
    /// TODO: Handle escape sequences
    pub fn event_text(&mut self, s: &str) -> PresenterCommand {
        self.dm().event_text(s)
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
                return self.dm().handle_click(btn, x, y);
            }
        }
        NeedRedraw::No
    }

    /// Call the drawing function for the given screen rows
    ///
    /// This is required as the session mutex must be locked and thus an iterator cannot be
    /// returned.
    pub fn display_lines(&self, draw_line: &dyn DrawLineTrait) {
        let session = self.c().session.clone();
        let session = session.0.lock().unwrap();
        for row in 0..self.c().window_height {
            if let Some(l) = self.d().single_display_line(&session, row) {
                draw_line.draw_line(row, &l);
            }
        }
    }
}

/// Get the line type of the line clicked.
fn clicked_line_type<T: SubPresenter>(pres: &mut T, y: usize) -> Option<LineType> {
    // Find the item that was clicked
    let session = pres.commons_mut().session.clone();
    let session = session.0.lock().unwrap();
    pres.single_display_line(&session, y).map(|i| i.is_a)
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
        (Some(LineType::Command(_, handle, _)), 1) => {
            if x < COMMAND_PREFIX_LEN {
                pres.commons_mut().session.cycle_visibility(handle);
                return true;
            }
        }
        _ => {
            // Unhandled combination, ignore
        }
    }
    false
}
