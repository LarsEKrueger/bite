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
//!
//! The text that was input is parsed as your type.

use sesd::{CompiledGrammar, CstIterItem, SymbolId};

use model::interpreter::parse_script;
use model::screen::Screen;
use model::session::{OutputVisibility, RunningStatus, Session};
use presenter::{
    check_response_clicked, CursorMapping, DisplayLine, LineItem, LineType, ModifierState,
    NeedRedraw, PresenterCommand, PresenterCommons, SpecialKey, SubPresenter,
};

use presenter::style_sheet::{LookedUp, Style};

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
    /// (start position, input, help)
    completions: Vec<(usize, String, String)>,

    /// Selection mode
    selection_mode: SelectionMode,

    /// Index into cursor map
    cursor_map_index: usize,
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
            cursor_map_index: 0,
        };
        presenter.search_history();
        Box::new(presenter)
    }

    fn execute_input(&mut self) -> PresenterCommand {
        let line = match self.selection_mode {
            SelectionMode::None => self.commons.editor.as_string(),
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
        self.commons.editor.clear();
        self.update_input_screen();
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
                self.commons.editor.enter_iter(line.chars());
            }
        }

        PresenterCommand::Redraw
    }

    fn set_input_from_history(&mut self) {
        let selected_item = self.selected_item;
        let prediction_len = self.commons.history.prediction().len();
        if selected_item < prediction_len {
            let mut line = self.search.clone();
            line.push_str(&self.commons.history.prediction()[selected_item]);
            self.commons.editor.clear();
            self.commons.editor.enter_iter(line.chars());
            self.selection_mode = SelectionMode::None;
            self.search.clear();
            self.update_input_screen();
        }
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
        let selection_rad = std::cmp::max(SELECTION_RAD, self.commons.window_height / 4);
        let selected_item = self.selected_item;
        let from = if selected_item > selection_rad {
            selected_item - selection_rad
        } else {
            0
        };
        // TODO: Handle multi-line entries in history
        let prediction_len = self.selection_screen.height() as usize;
        let to = if selected_item + selection_rad + 1 <= prediction_len {
            selected_item + selection_rad + 1
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
        // TODO: Handle window heights smaller than input_height better
        // If we don't have room to display the session, display everything else
        self.commons
            .window_height
            .saturating_sub(input_height)
            .saturating_sub(to - from)
            .saturating_sub(search_height)
    }

    /// Move cursor up one line, return true if that worked
    fn move_cursor_up(&mut self) -> bool {
        let col = self.commons.text_input.cursor_x() as usize;
        if let Some(this_start) = self
            .commons
            .editor
            .search_backward(self.commons.editor.cursor(), sesd::char::start_of_line)
        {
            if this_start > 0 {
                let prev_end = this_start - 1;
                if let Some(prev_start) = self
                    .commons
                    .editor
                    .search_backward(prev_end, sesd::char::start_of_line)
                {
                    if prev_start <= prev_end && prev_end < self.commons.editor.cursor() {
                        self.commons
                            .editor
                            .set_cursor(if prev_start + col <= prev_end {
                                prev_start + col
                            } else {
                                prev_end
                            });
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Update the cursor position on-screen from the cursor position off-screen after a backwards
    /// move.
    fn update_input_cursor_backwards(&mut self) {
        if self.cursor_map_index < self.commons.cursor_map.len() {
            let c = self.commons.editor.cursor();
            while self.commons.cursor_map[self.cursor_map_index].position > c {
                self.cursor_map_index -= 1;
            }

            let cy = self.commons.cursor_map[self.cursor_map_index].y;
            let cx = self.commons.cursor_map[self.cursor_map_index].x
                + (c - self.commons.cursor_map[self.cursor_map_index].position) as isize;
            self.commons.text_input.move_cursor_to(cx, cy);
        }
    }

    /// Update the cursor position on-screen from the cursor position off-screen after a forwards
    /// move.
    fn update_input_cursor_forwards(&mut self) {
        let c = self.commons.editor.cursor();
        trace!(
            "update_input_cursor_forwards: {:?}, {}, {}",
            self.commons.cursor_map,
            c,
            self.cursor_map_index
        );
        while self.cursor_map_index + 1 < self.commons.cursor_map.len()
            && c >= self.commons.cursor_map[self.cursor_map_index + 1].position
        {
            self.cursor_map_index += 1;
        }
        let cy = self.commons.cursor_map[self.cursor_map_index].y;
        let cx = self.commons.cursor_map[self.cursor_map_index].x
            + (c - self.commons.cursor_map[self.cursor_map_index].position) as isize;
        self.commons.text_input.move_cursor_to(cx, cy);
    }

    /// Move cursor down one line, return true if that worked
    fn move_cursor_down(&mut self) -> bool {
        let col = self.commons.text_input.cursor_x() as usize;
        if let Some(this_end) = self
            .commons
            .editor
            .search_forward(self.commons.editor.cursor(), sesd::char::end_of_line)
        {
            let next_start = this_end + 1;
            if let Some(next_end) = self
                .commons
                .editor
                .search_forward(next_start, sesd::char::end_of_line)
            {
                if next_start <= next_end && self.commons.editor.cursor() < next_start {
                    self.commons
                        .editor
                        .set_cursor(if next_start + col <= next_end {
                            next_start + col
                        } else {
                            next_end
                        });
                    return true;
                }
            }
        }
        false
    }

    fn start_completion(&mut self) {
        // Algo
        //
        // * Get the full predictions, incl. open parent rules that might still match
        // * Compile the list of completions based on the configuration

        let cursor_position = self.commons.editor.cursor();
        debug!(
            "start_completion at {}: {:?}",
            cursor_position,
            self.commons.editor.span_string(0, cursor_position)
        );

        self.commons.editor.parser().trace_cst(cursor_position);

        let predictions = self
            .commons
            .editor
            .parser()
            .full_predictions(cursor_position);

        let mut completions = Vec::new();
        let mut state = self.commons.completions.begin();
        for (sym, start, end) in predictions.iter() {
            let start_str = self.commons.editor.span_string(*start, cursor_position);
            if *end == cursor_position {
                trace!(
                    "lookup {} , {} - {}, {:?}",
                    self.commons.editor.grammar().nt_name(*sym),
                    start,
                    end,
                    start_str
                );
                let mut sym_comp = self
                    .commons
                    .completions
                    .lookup(&mut state, *sym, *start, &start_str);
                for (s, h) in sym_comp.drain(0..) {
                    completions.push((*start, s, h));
                }
            } else {
                trace!(
                    "ignore {} , {} - {}, {:?}",
                    self.commons.editor.grammar().nt_name(*sym),
                    start,
                    end,
                    start_str
                );
            }
        }
        let mut sym_comp =
            self.commons
                .completions
                .end(&state, &self.commons.editor, cursor_position);
        for (start, s, h) in sym_comp.drain(0..) {
            completions.push((start, s, h));
        }

        if !completions.is_empty() {
            let completion_len = completions.len();
            // If there is only one match, insert that
            if completion_len == 1 {
                self.commons.editor.replace(
                    completions[0].0,
                    cursor_position,
                    completions[0].1.chars(),
                );
                self.update_input_screen();
            } else {
                // Otherwise make the user pick
                self.completions = completions;
                self.selection_screen.reset();
                for item in self.completions.iter() {
                    let _ = self.selection_screen.add_bytes(item.1.as_bytes());
                    let _ = self.selection_screen.add_bytes(b" -- \x1b[0;32m");
                    let _ = self.selection_screen.add_bytes(item.2.as_bytes());
                    let _ = self.selection_screen.add_bytes(b"\x1b[39m\n");
                }
                self.selected_item = completion_len - 1;
                self.selection_mode = SelectionMode::Completion;
            }
        }
    }

    fn event_special_key_history(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Escape) => {
                self.selection_mode = SelectionMode::None;
                self.search.clear();
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Left)
            | ((false, false, false), SpecialKey::Right)
            | ((false, false, false), SpecialKey::End) => {
                self.set_input_from_history();
                self.commons.editor.skip_forward(sesd::char::end_of_line);
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Home) => {
                self.set_input_from_history();
                self.commons.editor.skip_backward(sesd::char::start_of_line);
                PresenterCommand::Redraw
            }
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
            ((false, false, false), SpecialKey::Tab) => {
                // Ignore tabs or they will be added to the input
                PresenterCommand::Ignored
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

            ((false, false, false), SpecialKey::PageUp) => {
                // No shift -> select
                let (from, to) = self.selections_from_to();
                let selection_height = to - from;
                let selected_item = &mut self.selected_item;
                *selected_item =
                    selected_item.saturating_sub(std::cmp::max(1, selection_height / 2));
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::PageDown) => {
                // No shift -> select
                let (from, to) = self.selections_from_to();
                let selection_height = to - from;
                let steps = std::cmp::max(1, selection_height / 2);

                let selected_item = &mut self.selected_item;
                if *selected_item + steps < self.commons.history.prediction().len() {
                    *selected_item += steps;
                } else {
                    self.selection_mode = SelectionMode::None;
                    self.search.clear();
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

    fn event_special_key_completion(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Escape) => {
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }
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
                let completion = self.completions.remove(selected_item);
                let cursor_position = self.commons.editor.cursor();
                self.commons
                    .editor
                    .replace(completion.0, cursor_position, completion.1.chars());
                self.update_input_screen();
                // Go back to normal mode
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Tab) => {
                // Ignore tabs or they will be added to the input
                PresenterCommand::Ignored
            }
            ((false, false, false), SpecialKey::Backspace) => {
                if self.commons.editor.move_backward(1) {
                    self.commons.editor.delete(1);
                    self.update_input_screen();
                    self.start_completion();
                }
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Left) => {
                self.commons.editor.move_backward(1);
                self.update_input_cursor_backwards();
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Right) => {
                self.commons.editor.move_forward(1);
                self.update_input_cursor_forwards();
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Home) => {
                self.commons.editor.skip_backward(sesd::char::start_of_line);
                self.update_input_cursor_backwards();
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::End) => {
                self.commons.editor.skip_forward(sesd::char::end_of_line);
                self.update_input_cursor_forwards();
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Delete) => {
                self.commons.editor.delete(1);
                self.update_input_screen();
                self.selection_mode = SelectionMode::None;
                PresenterCommand::Redraw
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

            ((false, false, false), SpecialKey::PageUp) => {
                // No shift -> select
                let (from, to) = self.selections_from_to();
                let selection_height = to - from;
                let selected_item = &mut self.selected_item;
                *selected_item =
                    selected_item.saturating_sub(std::cmp::max(1, selection_height / 2));
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::PageDown) => {
                // No shift -> select
                let (from, to) = self.selections_from_to();
                let selection_height = to - from;
                let steps = std::cmp::max(1, selection_height / 2);

                let selected_item = &mut self.selected_item;
                if *selected_item + steps < (self.selection_screen.height() as usize) {
                    *selected_item += steps;
                } else {
                    *selected_item = (self.selection_screen.height() as usize).saturating_sub(1);
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

            _ => PresenterCommand::Ignored,
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
                // Enter -> Execute command
                // TODO: do nothing if parser didn't accept
                self.execute_input()
            }
            ((true, false, false), SpecialKey::Enter) => {
                // Shift-Enter -> Insert a line break and let the parser re-render it
                self.commons.editor.enter('\n');
                self.update_input_screen();
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Left) => {
                self.commons.editor.move_backward(1);
                self.update_input_cursor_backwards();
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Right) => {
                self.commons.editor.move_forward(1);
                self.update_input_cursor_forwards();
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Up) => {
                if self.move_cursor_up() {
                    self.update_input_cursor_backwards();
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
                if self.move_cursor_down() {
                    self.update_input_cursor_forwards();
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
                self.commons.editor.skip_backward(sesd::char::start_of_line);
                self.update_input_cursor_backwards();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::End) => {
                self.commons.editor.skip_forward(sesd::char::end_of_line);
                self.update_input_cursor_forwards();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Delete) => {
                self.commons.editor.delete(1);
                self.update_input_screen();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Backspace) => {
                if self.commons.editor.move_backward(1) {
                    self.commons.editor.delete(1);
                    self.update_input_screen();
                }
                PresenterCommand::Redraw
            }

            // Tab: Completion
            ((false, false, false), SpecialKey::Tab) => {
                self.start_completion();
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

    /// Render a node of the parse tree.
    ///
    /// Return None, if the cursor is not inside this node. Return the x and y coordinate of the
    /// cursor, then the index into the cursor map.
    fn render_node(
        &self,
        text_input: &mut Screen,
        cursor_map: &mut Vec<CursorMapping>,
        start: usize,
        end: usize,
        cursor_index: usize,
        style: &Style,
    ) -> Option<(isize, isize, usize)> {
        // Print the style's pre string before taking the cursor position. This way, the cursor can
        // be moved to the editable portion correctly.
        let _ = text_input.add_bytes(style.pre.as_bytes());

        let cx = text_input.cursor_x();
        let cy = text_input.cursor_y();
        let cm = cursor_map.len();

        // Print the text as usual
        let text = self.commons.editor.span_string(start, end);
        let _ = text_input.add_bytes(text.as_bytes());

        // Print the style's post string to reset the attributes
        let _ = text_input.add_bytes(style.post.as_bytes());

        // Insert span into element position cache
        cursor_map.push(CursorMapping {
            position: start,
            x: cx,
            y: cy,
        });

        // Check if the cursor is between start and end. Check for position past the end of the
        // string to catch the cursor at the end of the buffer. If two elements touch, the second
        // will overwrite it later.
        if start <= cursor_index && cursor_index <= end {
            let offs = cursor_index - start;
            // If the cursor is below the last line, update the size of the matrix to render it
            // correctly
            text_input.make_room();
            Some((cx + offs as isize, cy, cm))
        } else {
            None
        }
    }

    /// Render the parsed input into the text input screen.
    ///
    /// Modelled after sesd's example program.
    fn update_input_screen(&mut self) {
        let cursor_index = self.commons.editor.cursor();
        trace!("update_input_screen: cursor_index = {}", cursor_index);

        let mut text_input = std::mem::replace(&mut self.commons.text_input, Screen::new());
        let mut cursor_map = std::mem::replace(&mut self.commons.cursor_map, Vec::new());
        text_input.reset();
        text_input.make_room();
        cursor_map.clear();

        let mut rendered_until = 0;
        let mut cursor_pos = (0, 0, 0);
        for cst_node in self.commons.editor.cst_iter() {
            trace!("rendered_until = {:?}", rendered_until);
            trace!("cursor_index = {:?}", cursor_index);
            match cst_node {
                CstIterItem::Parsed(cst_node) => {
                    trace!("end = {:?}", cst_node.end);
                    // If a rule contains a terminal in the middle, and no style has been defined,
                    // it is possible that rendered_until is larger than cst_node.start. Thus, the
                    // buffer needs to be rendered from rendered_until to cst_node.end.
                    if cst_node.end != cst_node.start && cst_node.end > rendered_until {
                        // Convert the path to a list of SymbolIds
                        let mut path: Vec<SymbolId> = cst_node
                            .path
                            .0
                            .iter()
                            .map(|n| {
                                let dr = self.commons.editor.parser().dotted_rule(&n);
                                self.commons.editor.grammar().lhs(dr.rule as usize)
                            })
                            .collect();
                        path.push(
                            self.commons
                                .editor
                                .grammar()
                                .lhs(cst_node.dotted_rule.rule as usize),
                        );
                        {
                            trace!("path:");
                            for n in cst_node.path.0.iter() {
                                trace!("  {:?}", n);
                            }
                            trace!("  {:?}", cst_node.current);
                        }

                        trace!("lookup( {:?})", path);
                        let looked_up = self.commons.style_sheet.lookup(&path);
                        trace!("looked_up = {:?}", looked_up);
                        match looked_up {
                            LookedUp::Parent => {
                                // Do nothing now. Render later.
                            }
                            LookedUp::Found(style) => {
                                // Found an exact match. Render with style.
                                if let Some(xy) = self.render_node(
                                    &mut text_input,
                                    &mut cursor_map,
                                    rendered_until,
                                    cst_node.end,
                                    cursor_index,
                                    style,
                                ) {
                                    cursor_pos = xy;
                                }
                                rendered_until = cst_node.end;
                            }
                            LookedUp::Nothing => {
                                // Found nothing. Render with default style.
                                if let Some(xy) = self.render_node(
                                    &mut text_input,
                                    &mut cursor_map,
                                    rendered_until,
                                    cst_node.end,
                                    cursor_index,
                                    &::presenter::style_sheet::DEFAULT,
                                ) {
                                    cursor_pos = xy;
                                }
                                rendered_until = cst_node.end;
                            }
                        }
                    }
                }
                CstIterItem::Unparsed(_unparsed) => {
                    trace!("unparsed = {:?}", _unparsed);
                    trace!("editor.len = {:?}", self.commons.editor.len());
                    // Render the unparsed part with defualt syle
                    if let Some(xy) = self.render_node(
                        &mut text_input,
                        &mut cursor_map,
                        rendered_until,
                        self.commons.editor.len(),
                        cursor_index,
                        &::presenter::style_sheet::UNPARSED,
                    ) {
                        cursor_pos = xy;
                    }
                    rendered_until = self.commons.editor.len();
                }
            }
        }

        trace!("update_input_screen: Cursor = {:?}", cursor_pos);
        text_input.move_cursor_to(cursor_pos.0, cursor_pos.1);
        self.commons.text_input = text_input;
        self.commons.cursor_map = cursor_map;
        self.cursor_map_index = cursor_pos.2;
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
            ((false, true, false), _) => PresenterCommand::Ignored,
            _ => PresenterCommand::Unknown,
        }
    }

    fn event_text(&mut self, s: &str) -> PresenterCommand {
        match self.selection_mode {
            SelectionMode::None => {
                self.commons.editor.enter_iter(s.chars());
                self.update_input_screen();
            }
            SelectionMode::History => {
                self.search.push_str(s);
                self.search_history();
                self.fix_selected_item();
            }
            SelectionMode::Completion => {
                self.commons.editor.enter_iter(s.chars());
                self.update_input_screen();
                self.start_completion();
            }
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
