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
//! stack.

use model::interpreter::parse_script;
use model::screen::Screen;
use model::session::{OutputVisibility, RunningStatus, Session};
use presenter::{
    check_response_clicked, DisplayLine, DrawLineTrait, LineItem, LineType, ModifierState,
    NeedRedraw, PresenterCommand, PresenterCommons, SpecialKey, SubPresenter,
};

/// Presenter to input and run commands.
pub struct ComposeCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Index of selected prediction
    selected_prediction: usize,

    /// Cache of rendered prediction
    prediction_screen: Screen,
}

const PREDICTION_RAD: usize = 2;

impl ComposeCommandPresenter {
    /// Allocate a sub-presenter for command composition and input to running programs.
    pub fn new(mut commons: Box<PresenterCommons>) -> Box<Self> {
        commons.to_last_line();
        let mut presenter = ComposeCommandPresenter {
            commons,
            selected_prediction: 0,
            prediction_screen: Screen::new(),
        };
        presenter.predict();
        let items_len = presenter.prediction().len();
        if items_len == 0 {
            presenter.selected_prediction = 0;
        } else {
            presenter.selected_prediction = items_len - 1;
        }
        Box::new(presenter)
    }

    fn is_multi_line(&self) -> bool {
        self.commons.text_input.height() > 1
    }

    fn text_input(&mut self) -> &mut Screen {
        &mut self.commons.text_input
    }

    fn execute_input(&mut self) -> PresenterCommand {
        let line = self.commons.text_input.extract_text_without_last_nl();
        self.commons.text_input.reset();
        self.commons.text_input.make_room();
        self.predict();
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

    /// Fix the selected_prediction to cope with changes in the number of items
    fn fix_selected_prediction(&mut self, item_cnt: usize) {
        if item_cnt == 0 {
            self.selected_prediction = 0;
        } else if self.selected_prediction >= item_cnt {
            self.selected_prediction = item_cnt - 1;
        }
    }

    /// Compute prediction based on the current input.
    fn predict(&mut self) {
        let line = self.commons.text_input.extract_text_without_last_nl();
        self.commons.history.predict_bubble_up(&line);
        self.prediction_screen.reset();

        let line = self.commons.text_input.extract_text_without_last_nl();
        for item in self.commons.history.prediction() {
            let _ = self.prediction_screen.add_bytes(line.as_bytes());
            let _ = self.prediction_screen.add_bytes(item.as_bytes());
            let _ = self.prediction_screen.add_bytes(b"\n");
        }
    }

    /// Get latest prediction
    fn prediction<'a>(&'a self) -> &'a Vec<String> {
        self.commons.history.prediction()
    }

    fn compute_predictions_from_to(&self) -> (usize, usize) {
        let from = if self.selected_prediction > PREDICTION_RAD {
            self.selected_prediction - PREDICTION_RAD
        } else {
            0
        };
        let prediction_len = self.commons.history.prediction().len();
        let to = if self.selected_prediction + PREDICTION_RAD + 1 <= prediction_len {
            self.selected_prediction + PREDICTION_RAD + 1
        } else {
            prediction_len
        };
        (from, to)
    }

    fn compute_session_height(&self) -> usize {
        let input_height = self.commons.text_input.height() as usize;
        let (from, to) = self.compute_predictions_from_to();
        // TODO: Handle window heights smaller than input_height
        self.commons.window_height - input_height - (to - from)
    }

    fn event_special_key_prediction(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            // (shift,control,meta)
            ((false, false, false), SpecialKey::Enter) => {
                // Take remaining prediction and execute it
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                let items = &self.commons.history.prediction();
                let item = &items[self.selected_prediction];
                self.commons.text_input.insert_str(item);
                self.commons.to_last_line();
                self.execute_input()
            }
            ((false, false, false), SpecialKey::Left) => {
                // Delete the last character
                self.commons.text_input.delete_left();
                self.predict();
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                PresenterCommand::Redraw
            }
            ((false, true, false), SpecialKey::Left) => {
                self.commons.text_input.delete_word_before_cursor();
                self.predict();
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Right) => {
                // Take the first character from the current prediction
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                let items = &self.commons.history.prediction();
                let item = &items[self.selected_prediction];
                if let Some(c) = item.chars().next() {
                    self.commons_mut().text_input_add_characters(&c.to_string());
                    self.predict();
                    PresenterCommand::Redraw
                } else {
                    PresenterCommand::Unknown
                }
            }
            ((false, true, false), SpecialKey::Right) => {
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                let items = &self.commons.history.prediction();
                let item = &items[self.selected_prediction];
                let mut cs = item.chars();
                // Skip any initial spaces
                while let Some(c) = cs.next() {
                    self.commons.text_input.insert_str(&c.to_string());
                    if c != ' ' {
                        break;
                    }
                }
                // Take non-spaces
                while let Some(c) = cs.next() {
                    if c == ' ' {
                        break;
                    }
                    self.commons.text_input.insert_str(&c.to_string());
                }
                self.predict();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Up) => {
                // Decrement the selection index
                if self.selected_prediction > 0 {
                    self.selected_prediction -= 1;
                }
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Down) => {
                // Increment the selection index
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                if self.selected_prediction + 1 < items_len {
                    self.selected_prediction += 1;
                }
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Home) => {
                // Delete the whole line
                self.commons.text_input.reset();
                self.commons.text_input.make_room();
                self.predict();
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::End) => {
                // Take the rest of the prediction
                let items_len = self.prediction().len();
                self.fix_selected_prediction(items_len);
                let items = &self.commons.history.prediction();
                let item = &items[self.selected_prediction];
                self.commons.text_input.insert_str(item);
                self.predict();
                PresenterCommand::Redraw
            }

            _ => self.event_special_key_normal(mod_state, key),
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
                    self.predict();
                    PresenterCommand::Redraw
                } else {
                    self.execute_input()
                }
            }
            ((true, false, false), SpecialKey::Enter) => {
                // Shift-Enter -> Break the line and thereby start multi-line editing
                self.commons_mut().text_input.break_line();
                self.predict();
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
                self.predict();
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
                self.predict();
                PresenterCommand::Redraw
            }

            //           ((false, false, false), SpecialKey::Tab) => {
            //               // TODO: This needs to be cleaned up.
            //               let word = self.text_input().word_before_cursor();
            //               let word_chars = word.chars().count();
            //
            //               // Remember if the word started with ./ because glob removes that.
            //               let dot_slash = if word.starts_with("./") { "./" } else { "" };
            //
            //               // Find all files and folders that match '<word>*'
            //               match glob::glob(&(word.clone() + "*")) {
            //                   Err(_) => (self, PresenterCommand::Unknown),
            //                   Ok(g) => {
            //                       // Get the matches after word
            //                       let matches: Vec<String> = g
            //                           .filter_map(std::result::Result::ok)
            //                           .map(|path| {
            //                               let mut p = dot_slash.to_string();
            //                               p.push_str(&path.display().to_string());
            //                               // If the path is a directory, add a slash.
            //                               if path.is_dir() {
            //                                   p.push_str("/");
            //                               }
            //                               p
            //                           })
            //                           .collect();
            //
            //                       // If there is only one match, insert that
            //                       if matches.len() == 1 {
            //                           // Delete the beginning
            //                           self.text_input().move_left(word_chars as isize);
            //                           for _i in 0..word_chars {
            //                               self.text_input().delete_character();
            //                           }
            //                           // Put the match there
            //                           self.text_input().place_str(&matches[0]);
            //                           (self, PresenterCommand::Redraw)
            //                       } else {
            //                           // Otherwise make the user pick
            //                           (
            //                               CompleteCommandPresenter::new(self.commons, word, matches),
            //                               PresenterCommand::Redraw,
            //                           )
            //                       }
            //                   }
            //               }
            //           }

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

    fn display_lines(&self, session: &Session, draw_line: &dyn DrawLineTrait) {
        // Draw the session
        let session_height = self.compute_session_height();
        if let Some(mut loc) = self.commons.start_line(session, true, session_height) {
            trace!("display_lines: loc={:?}", loc);
            let mut row = 0;
            while (row as usize) < session_height {
                if let Some(display_line) = session.display_line(&loc) {
                    draw_line.draw_line(row, &DisplayLine::from(display_line));
                }
                row += 1;
                if let Some(new_loc) = PresenterCommons::locate_down(session, &loc, true, 1) {
                    loc = new_loc;
                } else {
                    break;
                }
            }
        }

        // Draw the predictions
        let (from, to) = self.compute_predictions_from_to();
        let prediction_height = to - from;
        {
            let prediction_start = session_height;
            let line = self.commons.text_input.extract_text_without_last_nl();
            let mut offs = 0;
            for index_p in from..to {
                let cursor_col = if index_p == self.selected_prediction {
                    Some(line.len())
                } else {
                    None
                };
                let cells = self.prediction_screen.compacted_row_slice(index_p as isize);
                draw_line.draw_line(
                    prediction_start + offs,
                    &DisplayLine::from(LineItem::new(&cells, LineType::InputInfo, cursor_col, 0)),
                );
                offs += 1;
            }
        }

        // Draw the text input
        let input_start = session_height + prediction_height;
        for (offs, cells) in self.commons.text_input.line_iter().enumerate() {
            let cursor_col = if offs == (self.commons.text_input.cursor_y() as usize) {
                Some(self.commons.text_input.cursor_x() as usize)
            } else {
                None
            };
            draw_line.draw_line(
                input_start + offs,
                &DisplayLine::from(LineItem::new(cells, LineType::Input, cursor_col, 0)),
            );
        }
    }

    fn single_display_line<'a, 'b: 'a>(
        &'a self,
        session: &'b Session,
        y: usize,
    ) -> Option<DisplayLine<'a>> {
        let session_height = self.compute_session_height();
        if y < session_height {
            if let Some(loc) = self.commons.start_line(session, true, session_height) {
                if let Some(loc) = PresenterCommons::locate_down(session, &loc, true, y) {
                    if let Some(display_line) = session.display_line(&loc) {
                        return Some(DisplayLine::from(display_line));
                    }
                }
            }
        } else {
            let (from, to) = self.compute_predictions_from_to();
            let prediction_height = to - from;
            if y < session_height + prediction_height {
                let index_p = y - session_height + from;
                let line = self.commons.text_input.extract_text_without_last_nl();
                let cursor_col = if index_p == self.selected_prediction {
                    Some(line.len())
                } else {
                    None
                };
                let cells = self.prediction_screen.compacted_row_slice(index_p as isize);
                return Some(DisplayLine::from(LineItem::new(
                    &cells,
                    LineType::InputInfo,
                    cursor_col,
                    0,
                )));
            } else {
                let input_height = self.commons.text_input.height() as usize;
                if y < session_height + prediction_height + input_height {
                    let offs = y - session_height - prediction_height;
                    return self.commons.text_input.line_iter().nth(offs).map(|cells| {
                        let cursor_col = if offs == (self.commons.text_input.cursor_y() as usize) {
                            Some(self.commons.text_input.cursor_x() as usize)
                        } else {
                            None
                        };
                        DisplayLine::from(LineItem::new(cells, LineType::Input, cursor_col, 0))
                    });
                }
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
        if self.commons.text_input.cursor_at_end() {
            if !self.prediction().is_empty() {
                return self.event_special_key_prediction(mod_state, key);
            }
        }
        self.event_special_key_normal(mod_state, key)
    }

    /// Handle pressing modifier + letter.
    ///
    /// If Ctrl-R is pressed, go to history browse mode with search for contained strings.
    /// If Ctrl-D is pressed, quit bite.
    fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand {
        match (mod_state.as_tuple(), letter) {
            ((false, true, false), b'd') => PresenterCommand::Exit,
            //           ((false, true, false), b'r') => {
            //               // Control-R -> Start interactive history search
            //               let prefix = {
            //                   let ref mut text_input = self.commons.text_input;
            //                   let prefix = text_input.text_before_cursor();
            //                   text_input.reset();
            //                   text_input.place_str(&prefix);
            //                   prefix
            //               };
            //                   PresenterCommand::Redraw,
            //           }
            _ => PresenterCommand::Unknown,
        }
    }

    fn event_text(&mut self, s: &str) -> PresenterCommand {
        self.commons_mut().text_input_add_characters(s);
        self.predict();
        let items_len = self.prediction().len();
        self.fix_selected_prediction(items_len);
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
