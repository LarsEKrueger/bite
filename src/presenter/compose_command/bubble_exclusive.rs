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

use std::borrow::Cow;

use model::interpreter::parse_script;
use model::screen::Screen;
use model::session::{OutputVisibility, RunningStatus, Session};
use presenter::{
    check_response_clicked, DisplayLine, LineItem, LineType, ModifierState, NeedRedraw,
    PresenterCommand, PresenterCommons, SpecialKey, SubPresenter,
};

/// Presenter to input and run commands.
pub struct ComposeCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Index of selected prediction, If None, no option is selected
    selected_prediction: Option<usize>,

    /// String to search for
    search: String,

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
            selected_prediction: None,
            prediction_screen: Screen::new(),
            search: String::new(),
        };
        presenter.predict();
        Box::new(presenter)
    }

    fn is_multi_line(&self) -> bool {
        self.commons.text_input.height() > 1
    }

    fn text_input(&mut self) -> &mut Screen {
        &mut self.commons.text_input
    }

    fn execute_input(&mut self) -> PresenterCommand {
        let line = if let Some(selected_prediction) = self.selected_prediction {
            let mut line = self.search.clone();
            line.push_str(&self.commons.history.prediction()[selected_prediction]);
            line
        } else {
            self.commons.text_input.extract_text_without_last_nl()
        };
        self.commons.text_input.reset();
        self.commons.text_input.make_room();
        self.search.clear();
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
    fn fix_selected_prediction(&mut self) {
        if let Some(ref mut selected_prediction) = self.selected_prediction {
            let prediction_len = self.commons.history.prediction().len();
            if prediction_len == 0 {
                *selected_prediction = 0;
            } else if *selected_prediction >= prediction_len {
                *selected_prediction = prediction_len - 1;
            }
        }
    }

    /// Compute prediction based on the current input.
    fn predict(&mut self) {
        self.commons.history.predict_bubble_up(&self.search);
        self.prediction_screen.reset();

        trace!("bubble_exclusive::predict(»{}«): >>>>> ", self.search);
        for item in self.commons.history.prediction() {
            trace!("bubble_exclusive::predict: »{}«", item);
            let _ = self.prediction_screen.add_bytes(self.search.as_bytes());
            let _ = self.prediction_screen.add_bytes(item.as_bytes());
            let _ = self.prediction_screen.add_bytes(b"\n");
        }
        trace!("bubble_exclusive::predict: <<<<< ");
    }

    fn compute_predictions_from_to(&self) -> (usize, usize) {
        if let Some(selected_prediction) = self.selected_prediction {
            let from = if selected_prediction > PREDICTION_RAD {
                selected_prediction - PREDICTION_RAD
            } else {
                0
            };
            let prediction_len = self.commons.history.prediction().len();
            let to = if selected_prediction + PREDICTION_RAD + 1 <= prediction_len {
                selected_prediction + PREDICTION_RAD + 1
            } else {
                prediction_len
            };
            (from, to)
        } else {
            // Nothing selected, draw the last PREDICTION_RAD+1 items
            let prediction_len = self.commons.history.prediction().len();
            let from = if prediction_len > PREDICTION_RAD {
                prediction_len - 1 - PREDICTION_RAD
            } else {
                0
            };
            let to = prediction_len;
            (from, to)
        }
    }

    fn compute_session_height(&self) -> usize {
        let input_height = self.commons.text_input.height() as usize;
        let (from, to) = self.compute_predictions_from_to();
        let search_height = if self.search.is_empty() { 0 } else { 1 };
        // TODO: Handle window heights smaller than input_height
        self.commons.window_height - input_height - (to - from) - search_height
    }

    fn event_special_key_prediction(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Up) => {
                if let Some(ref mut selected_prediction) = self.selected_prediction {
                    *selected_prediction = selected_prediction.saturating_sub(1);
                }
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Down) => {
                if let Some(ref mut selected_prediction) = self.selected_prediction {
                    if *selected_prediction + 1 < self.commons.history.prediction().len() {
                        *selected_prediction += 1;
                    } else {
                        self.selected_prediction = None;
                        self.search.clear();
                    }
                }
                PresenterCommand::Redraw
            }
            ((false, false, false), SpecialKey::Backspace) => {
                // Shorten the search string by one char
                let _ = self.search.pop();
                self.predict();
                self.fix_selected_prediction();
                PresenterCommand::Redraw
            }
            ((_, _, _), SpecialKey::Enter) => self.execute_input(),

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
                        self.selected_prediction = Some(prediction_len - 1);
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

    fn single_display_line<'a, 'b: 'a>(
        &'a self,
        session: &'b Session,
        y: usize,
    ) -> Option<DisplayLine<'a>> {
        let mut offs = y;
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
        let (from, to) = self.compute_predictions_from_to();
        let prediction_height = to - from;
        if offs < prediction_height {
            let cursor_col = {
                if let Some(selected_prediction) = self.selected_prediction {
                    if offs + from == selected_prediction {
                        Some(self.search.len())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            let cells = self
                .prediction_screen
                .compacted_row_slice((from + offs) as isize);
            return Some(DisplayLine::from(LineItem::new(
                &cells,
                LineType::HistoryItem,
                cursor_col,
                0,
            )));
        }
        offs -= prediction_height;

        if !self.search.is_empty() {
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

        let input_height = self.commons.text_input.height() as usize;
        if offs < input_height {
            return self.commons.text_input.line_iter().nth(offs).map(|cells| {
                let cursor_col = if self.selected_prediction.is_none() {
                    if offs == (self.commons.text_input.cursor_y() as usize) {
                        Some(self.commons.text_input.cursor_x() as usize)
                    } else {
                        None
                    }
                } else {
                    None
                };
                return DisplayLine::from(LineItem::new(cells, LineType::Input, cursor_col, 0));
            });
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
        if self.selected_prediction.is_some() {
            return self.event_special_key_prediction(mod_state, key);
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
            ((false, true, false), b'r') => {
                // Control-R -> Start interactive history search
                let prediction_len = self.commons.history.prediction().len();
                if prediction_len > 0 {
                    self.selected_prediction = Some(prediction_len - 1);
                }
                PresenterCommand::Redraw
            }
            _ => PresenterCommand::Unknown,
        }
    }

    fn event_text(&mut self, s: &str) -> PresenterCommand {
        if self.selected_prediction.is_none() {
            self.commons_mut().text_input_add_characters(s);
        } else {
            self.search.push_str(s);
            self.predict();
            self.fix_selected_prediction();
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
