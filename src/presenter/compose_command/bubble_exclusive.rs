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

//! Sub presenter for composing commands. Variant shows history above prompt, based on bubble-up
//! stack. Entry is not used for prediction. Automatic search is performed when typing in the
//! history.

use model::completion;
use model::interpreter::parse_script;
use model::screen::Screen;
use model::session::{OutputVisibility, RunningStatus, Session};
use presenter::{
    check_response_clicked, DisplayLine, LineItem, LineType, ModifierState, NeedRedraw,
    PresenterCommand, PresenterCommons, SpecialKey, SubPresenter,
};

/// Which selection to show
enum SelectionMode {
    None,
    History,
    Completion,
}

/// Presenter to input and run commands.
pub struct ComposeCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Index of selected history or completion item.
    selected_item: usize,

    /// String to search for in the history
    search: String,

    /// Cache of rendered selection
    selection_screen: Screen,

    /// List of completions
    completions: Vec<String>,

    /// Selection mode
    selection_mode: SelectionMode,
}

const SELECTION_RAD: usize = 2;

impl ComposeCommandPresenter {
    /// Allocate a sub-presenter for command composition and input to running programs.
    pub fn new(mut commons: Box<PresenterCommons>) -> Box<Self> {
        commons.to_last_line();
        let mut presenter = ComposeCommandPresenter {
            commons,
            selected_item: 0,
            selection_screen: Screen::new(),
            search: String::new(),
            completions: Vec::new(),
            selection_mode: SelectionMode::None,
        };
        presenter.search_history();
        Box::new(presenter)
    }

    fn is_multi_line(&self) -> bool {
        self.commons.text_input.height() > 1
    }

    fn text_input(&mut self) -> &mut Screen {
        &mut self.commons.text_input
    }

    fn execute_input(&mut self) -> PresenterCommand {
        let line = match self.selection_mode {
            SelectionMode::None => self.commons.text_input.extract_text_without_last_nl(),
            SelectionMode::History => {
                let selected_item = self.selected_item;
                let prediction_len = self.commons.history.prediction().len();
                if selected_item < prediction_len {
                    let mut line = self.search.clone();
                    line.push_str(&self.commons.history.prediction()[selected_item]);
                    self.search_history();
                    line
                } else {
                    return PresenterCommand::Unknown;
                }
            }
            SelectionMode::Completion => {
                return PresenterCommand::Unknown;
            }
        };
        self.commons.text_input.reset();
        self.commons.text_input.make_room();
        self.selection_mode = SelectionMode::None;
        self.search.clear();
        if line.is_empty() {
            return PresenterCommand::Redraw;
        }
        trace!("Execute »{}«", line);
        let mut line_with_nl = line.clone();
        line_with_nl.push('\n');

        // Check if the input parses
        match parse_script(&line_with_nl) {
            Ok(instructions) => {
                // Put the command in the history
                let cwd = self.commons.interpreter.get_cwd();
                self.commons.history.enter(&cwd.to_string_lossy(), &line);
                // Run the compiled instructions
                let _interaction_handle = self.commons.interpreter.run(line_with_nl, instructions);
            }
            Err(msg) => {
                // Create a fake interaction, print the error, set the return code to error
                let interaction_handle = self
                    .commons
                    .session
                    .add_interaction(Screen::one_line_matrix(line.as_bytes()));
                self.commons.session.add_bytes(
                    OutputVisibility::Error,
                    interaction_handle,
                    msg.as_bytes(),
                );
                self.commons
                    .session
                    .set_running_status(interaction_handle, RunningStatus::Exited(1));
                self.commons
                    .session
                    .set_visibility(interaction_handle, OutputVisibility::Error);
                // Put back the input
                self.commons.text_input.replace(&line, false);
            }
        }

        PresenterCommand::Redraw
    }

    /// Fix the selected_item to cope with changes in the number of items
    fn fix_selected_item(&mut self) {
        let selected_item = &mut self.selected_item;
        let prediction_len = self.commons.history.prediction().len();
        if prediction_len == 0 {
            *selected_item = 0;
        } else if *selected_item >= prediction_len {
            *selected_item = prediction_len - 1;
        }
    }

    /// Compute history selection based on the current input.
    fn search_history(&mut self) {
        self.commons.history.predict_bubble_up(&self.search);
        self.selection_screen.reset();

        for item in self.commons.history.prediction() {
            let _ = self.selection_screen.add_bytes(self.search.as_bytes());
            let _ = self.selection_screen.add_bytes(item.as_bytes());
            let _ = self.selection_screen.add_bytes(b"\n");
        }
    }

    /// Determine which elements are visible
    ///
    /// Returns (show_input, show_input_cursor, show_selection, show_search)
    fn visible_elements(&self) -> (bool, bool, bool, bool) {
        match self.selection_mode {
            SelectionMode::None => (true, true, false, false),
            SelectionMode::History => (false, false, true, !self.search.is_empty()),
            SelectionMode::Completion => (true, false, true, false),
        }
    }

    fn selections_from_to(&self) -> (usize, usize) {
        let selected_item = self.selected_item;
        let from = if selected_item > SELECTION_RAD {
            selected_item - SELECTION_RAD
        } else {
            0
        };
        // TODO: Handle multi-line entries in history
        let prediction_len = self.selection_screen.height() as usize;
        let to = if selected_item + SELECTION_RAD + 1 <= prediction_len {
            selected_item + SELECTION_RAD + 1
        } else {
            prediction_len
        };
        (from, to)
    }

    fn compute_session_height(&self) -> usize {
        let (show_input, _, show_selection, show_search) = self.visible_elements();
        let input_height = if show_input {
            self.commons.text_input.height() as usize
        } else {
            0
        };
        let (from, to) = if show_selection {
            self.selections_from_to()
        } else {
            (0, 0)
        };
        let search_height = if show_search { 1 } else { 0 };
        // TODO: Handle window heights smaller than input_height
        self.commons.window_height - input_height - (to - from) - search_height
    }

    fn event_special_key_history(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Up) => {
                let selected_item = &mut self.selected_item;
                *selected_item = selected_item.saturating_sub(1);
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Down) => {
                let selected_item = &mut self.selected_item;
                if *selected_item + 1 < self.commons.history.prediction().len() {
                    *selected_item += 1;
                } else {
                    self.selection_mode = SelectionMode::None;
                    self.search.clear();
                }
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Backspace) => {
                // Shorten the search string by one char
                let _ = self.search.pop();
                self.search_history();
                self.fix_selected_item();
                PresenterCommand::Redraw
            }
            ((_, _, _), SpecialKey::Enter) => self.execute_input(),

            _ => PresenterCommand::Unknown,
        }
    }

    fn event_special_key_completion(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Up) => {
                let selected_item = &mut self.selected_item;
                *selected_item = selected_item.saturating_sub(1);
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Down) => {
                let selected_item = &mut self.selected_item;
                if *selected_item + 1 < self.selection_screen.height() as usize {
                    *selected_item += 1;
                }
                PresenterCommand::Redraw
            }
            ((_, _, _), SpecialKey::Enter) => {
                let selected_item = self.selected_item;
                // Insert the selected completion
                let word = self.text_input().word_before_cursor();
                let word_chars = word.chars().count();
                // Delete the beginning
                self.text_input().move_left(word_chars as isize);
                for _i in 0..word_chars {
                    self.text_input().delete_character();
                }
                // Put the match there
                let completion = self.completions.remove(selected_item);
                self.text_input().place_str(&completion);
                // Go back to normal mode
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }
            _ => PresenterCommand::Unknown,
        }
    }

    fn event_special_key_normal(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            // (shift,control,meta)
            ((false, false, false), SpecialKey::Enter) => {
                if self.is_multi_line() {
                    self.commons_mut().text_input.break_line();
                    PresenterCommand::Redraw
                } else {
                    self.execute_input()
                }
            }
            ((true, false, false), SpecialKey::Enter) => {
                // Shift-Enter -> Break the line and thereby start multi-line editing
                self.commons_mut().text_input.break_line();
                PresenterCommand::Redraw
            }
            ((false, true, false), SpecialKey::Enter) => {
                // Ctrl-Enter -> Start the command in multi-line mode
                if self.is_multi_line() {
                    self.execute_input()
                } else {
                    PresenterCommand::Unknown
                }
            }
            ((false, false, false), SpecialKey::Left) => {
                self.commons_mut().text_input.move_left(1);
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Right) => {
                self.commons_mut().text_input.move_right(1);
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Up) => {
                if self.commons.text_input.cursor_y() > 0 {
                    self.commons_mut().text_input.move_up(1);
                } else {
                    // Go to last history entry
                    let prediction_len = self.commons.history.prediction().len();
                    if prediction_len > 0 {
                        self.selected_item = prediction_len - 1;
                        self.selection_mode = SelectionMode::History;
                    }
                }
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Down) => {
                if self.commons.text_input.cursor_y() + 1 < self.commons.text_input.height() {
                    self.commons_mut().text_input.move_down(1);
                    PresenterCommand::Redraw
                } else {
                    PresenterCommand::Unknown
                }
            }
            ((true, false, false), SpecialKey::PageUp) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                let session_height = self.compute_session_height();
                self.commons.scroll_up(true, middle, |session, loc| {
                    PresenterCommons::locate_up(session, loc, session_height).and_then(|loc| {
                        PresenterCommons::locate_down(session, &loc, true, session_height)
                    })
                });
                PresenterCommand::Redraw
            }

            ((true, false, false), SpecialKey::PageDown) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                self.commons.scroll_down(true, middle);
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Home) => {
                self.commons.text_input.move_left_edge();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::End) => {
                self.text_input().move_end_of_line();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Delete) => {
                if self.text_input().cursor_at_end_of_line() {
                    self.text_input().join_next_line();
                } else {
                    self.commons.text_input.delete_character();
                }
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Backspace) => {
                if self.text_input().cursor_x() == 0 {
                    if self.text_input().cursor_y() > 0 {
                        self.text_input().move_up(1);
                        self.text_input().move_end_of_line();
                        self.text_input().join_next_line();
                    }
                } else {
                    self.commons.text_input.delete_left();
                }
                PresenterCommand::Redraw
            }

            // Tab: Completion
            ((false, false, false), SpecialKey::Tab) => {
                let word = self.text_input().word_before_cursor();

                let completions = completion::file_completion(&word);
                let completion_len = completions.len();
                // If there is only one match, insert that
                if completion_len == 1 {
                    let word_chars = word.chars().count();
                    // Delete the beginning
                    self.text_input().move_left(word_chars as isize);
                    for _i in 0..word_chars {
                        self.text_input().delete_character();
                    }
                    // Put the match there
                    self.text_input().place_str(&completions[0]);
                } else {
                    // Otherwise make the user pick
                    self.completions = completions;
                    self.selection_screen.reset();
                    for item in self.completions.iter() {
                        let _ = self.selection_screen.add_bytes(item.as_bytes());
                        let _ = self.selection_screen.add_bytes(b"\n");
                    }
                    self.selected_item = completion_len - 1;
                    self.selection_mode = SelectionMode::Completion;
                }
                PresenterCommand::Redraw
            }

            // Ctrl-Space: cycle last interaction's output
            ((false, true, false), SpecialKey::Space) => {
                if let Some(interaction_handle) = self.commons.session.last_interaction() {
                    self.commons.session.cycle_visibility(interaction_handle);
                    PresenterCommand::Redraw
                } else {
                    PresenterCommand::Unknown
                }
            }
            // Shift-Ctrl-Space: cycle all interaction's output
            ((true, true, false), SpecialKey::Space) => {
                if let Some(interaction_handle) = self.commons.session.last_interaction() {
                    self.commons.session.cycle_visibility(interaction_handle);
                    if let Some(ov) = self.commons.session.get_visibility(interaction_handle) {
                        self.commons.session.set_visibility_all(ov);
                    }
                    PresenterCommand::Redraw
                } else {
                    PresenterCommand::Unknown
                }
            }

            _ => PresenterCommand::Unknown,
        }
    }
}

impl SubPresenter for ComposeCommandPresenter {
    fn finish(self: Box<Self>) -> Box<PresenterCommons> {
        self.commons
    }

    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn single_display_line<'a, 'b: 'a>(
        &'a self,
        session: &'b Session,
        y: usize,
    ) -> Option<DisplayLine<'a>> {
        let mut offs = y;

        // Always show the session
        let session_height = self.compute_session_height();
        if y < session_height {
            if let Some(loc) = self.commons.start_line(session, true, session_height) {
                if let Some(loc) = PresenterCommons::locate_down(session, &loc, true, offs) {
                    if let Some(display_line) = session.display_line(&loc) {
                        return Some(DisplayLine::from(display_line));
                    }
                }
            }
            return None;
        }
        offs -= session_height;

        let (show_input, show_input_cursor, show_selection, show_search) = self.visible_elements();

        if show_selection {
            let (from, to) = self.selections_from_to();
            let selection_height = to - from;
            if offs < selection_height {
                let (cursor_col, line_type) = {
                    let selected_item = self.selected_item;
                    if offs + from == selected_item {
                        (
                            Some(self.search.len()),
                            LineType::SelectedMenuItem(selected_item),
                        )
                    } else {
                        (None, LineType::MenuItem(selected_item))
                    }
                };
                let cells = self
                    .selection_screen
                    .compacted_row_slice((from + offs) as isize);
                return Some(DisplayLine::from(LineItem::new(
                    &cells, line_type, cursor_col, 0,
                )));
            }
            offs -= selection_height;
        }

        if show_search {
            if offs == 0 {
                // Draw search string
                let cells = Screen::one_line_cell_vec(self.search.as_bytes());
                return Some(DisplayLine::from(LineItem::new_owned(
                    cells,
                    LineType::Search,
                    None,
                    0,
                )));
            }
            offs -= 1;
        }

        if show_input {
            let input_height = self.commons.text_input.height() as usize;
            if offs < input_height {
                return self.commons.text_input.line_iter().nth(offs).map(|cells| {
                    let cursor_col = if show_input_cursor
                        && offs == (self.commons.text_input.cursor_y() as usize)
                    {
                        Some(self.commons.text_input.cursor_x() as usize)
                    } else {
                        None
                    };
                    return DisplayLine::from(LineItem::new(cells, LineType::Input, cursor_col, 0));
                });
            }
        }
        None
    }

    /// Handle a click.
    ///
    /// If a command was clicked, cycle through the visibility of output and error.
    fn handle_click(&mut self, button: usize, x: usize, y: usize) -> NeedRedraw {
        if check_response_clicked(&mut *self, button, x, y) {
            NeedRedraw::Yes
        } else {
            NeedRedraw::No
        }
    }

    fn event_special_key(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match self.selection_mode {
            SelectionMode::None => self.event_special_key_normal(mod_state, key),
            SelectionMode::History => self.event_special_key_history(mod_state, key),
            SelectionMode::Completion => self.event_special_key_completion(mod_state, key),
        }
    }

    /// Handle pressing modifier + letter.
    ///
    /// If Ctrl-R is pressed, go to history browse mode with search for contained strings.
    /// If Ctrl-D is pressed, quit bite.
    fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand {
        match (mod_state.as_tuple(), letter) {
            ((false, true, false), b'd') => PresenterCommand::Exit,
            ((false, true, false), b'r') => {
                // Control-R -> Start interactive history search
                let prediction_len = self.commons.history.prediction().len();
                if prediction_len > 0 {
                    self.selected_item = prediction_len - 1;
                    self.selection_mode = SelectionMode::History;
                }
                PresenterCommand::Redraw
            }
            _ => PresenterCommand::Unknown,
        }
    }

    fn event_text(&mut self, s: &str) -> PresenterCommand {
        match self.selection_mode {
            SelectionMode::None => {
                self.commons_mut().text_input_add_characters(s);
            }
            SelectionMode::History => {
                self.search.push_str(s);
                self.search_history();
                self.fix_selected_item();
            }
            SelectionMode::Completion => {}
        }
        PresenterCommand::Redraw
    }

    fn event_scroll_up(&mut self, mod_state: &ModifierState) -> PresenterCommand {
        if mod_state.none_pressed() {
            self.commons.scroll_up(true, 1, |_, _| None);
            PresenterCommand::Redraw
        } else {
            PresenterCommand::Unknown
        }
    }

    fn event_scroll_down(&mut self, mod_state: &ModifierState) -> PresenterCommand {
        if mod_state.none_pressed() {
            self.commons.scroll_down(true, 1);
            PresenterCommand::Redraw
        } else {
            PresenterCommand::Unknown
        }
    }
}
