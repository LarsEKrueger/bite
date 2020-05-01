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

//! Sub presenter for executing programs.

use model::screen::Matrix;
use model::session::{InteractionHandle, Session};
use presenter::{
    check_response_clicked, DisplayLine, DrawLineTrait, LineItem, LineType, ModifierState,
    NeedRedraw, PresenterCommand, PresenterCommons, SpecialKey, SubPresenter,
};

/// Presenter to run commands and send input to their stdin.
#[allow(dead_code)]
pub struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: InteractionHandle,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<Matrix>,
}

#[allow(dead_code)]
impl ExecuteCommandPresenter {
    pub fn new(
        mut commons: Box<PresenterCommons>,
        current_interaction: InteractionHandle,
    ) -> Box<Self> {
        commons.to_last_line();
        let presenter = ExecuteCommandPresenter {
            commons,
            current_interaction,
            next_prompt: None,
        };
        Box::new(presenter)
    }

    fn compute_session_height(&self) -> usize {
        let input_height = self.commons.text_input.height() as usize;
        // TODO: Handle window heights smaller than input_height
        self.commons.window_height - input_height
    }
}

impl SubPresenter for ExecuteCommandPresenter {
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
        if let Some(mut loc) = self.commons.start_line(session, false, session_height) {
            trace!("display_lines: loc={:?}", loc);
            let mut row = 0;
            while (row as usize) < session_height {
                if let Some(display_line) = session.display_line(&loc) {
                    draw_line.draw_line(row, &DisplayLine::from(display_line));
                }
                row += 1;
                if let Some(new_loc) = PresenterCommons::locate_down(session, &loc, false, 1) {
                    loc = new_loc;
                } else {
                    break;
                }
            }
        }

        // Draw the text input
        let input_start = session_height;
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
        _session: &'b Session,
        _y: usize,
    ) -> Option<DisplayLine<'a>> {
        None
    }

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((false, false, false), SpecialKey::Enter) => {
                let line = self.commons.text_input.extract_text();
                self.commons.text_input.reset();
                self.commons.text_input.make_room();
                self.commons
                    .session
                    .write_stdin(self.current_interaction, line.as_bytes());
                PresenterCommand::Redraw
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
                self.commons.scroll_up(false, middle, |session, loc| {
                    PresenterCommons::locate_up(session, loc, session_height).and_then(|loc| {
                        PresenterCommons::locate_down(session, &loc, false, session_height)
                    })
                });
                PresenterCommand::Redraw
            }

            ((true, false, false), SpecialKey::PageDown) => {
                // Shift only -> Scroll
                let middle = self.commons.window_height / 2;
                self.commons.scroll_down(false, middle);
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Home) => {
                self.commons.text_input.move_left_edge();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::End) => {
                self.commons.text_input.move_right_edge();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Delete) => {
                self.commons.text_input.delete_character();
                PresenterCommand::Redraw
            }

            ((false, false, false), SpecialKey::Backspace) => {
                self.commons.text_input.delete_left();
                PresenterCommand::Redraw
            }

            // Ctrl-Space: cycle current interaction's output
            ((false, true, false), SpecialKey::Space) => {
                self.commons
                    .session
                    .cycle_visibility(self.current_interaction);
                PresenterCommand::Redraw
            }
            // Shift-Ctrl-Space: cycle all interaction's output
            ((true, true, false), SpecialKey::Space) => {
                self.commons
                    .session
                    .cycle_visibility(self.current_interaction);
                if let Some(ov) = self
                    .commons
                    .session
                    .get_visibility(self.current_interaction)
                {
                    self.commons.session.set_visibility_all(ov);
                }
                PresenterCommand::Redraw
            }

            _ => PresenterCommand::Unknown,
        }
    }

    fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand {
        match mod_state.as_tuple() {
            (false, true, false) => {
                // Control-only
                match letter {
                    b'c' => {
                        // Kill the last job if it is still running
                        self.commons.session.terminate(self.current_interaction);
                        PresenterCommand::Redraw
                    }

                    b'd' => {
                        // Send to running program
                        self.commons
                            .session
                            .write_stdin(self.current_interaction, b"\x04");
                        PresenterCommand::Redraw
                    }
                    _ => PresenterCommand::Unknown,
                }
            }
            _ => PresenterCommand::Unknown,
        }
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

    fn event_text(&mut self, s: &str) -> PresenterCommand {
        self.commons_mut().text_input_add_characters(s);
        PresenterCommand::Redraw
    }

    fn event_scroll_up(&mut self, mod_state: &ModifierState) -> PresenterCommand {
        if mod_state.none_pressed() {
            self.commons.scroll_up(false, 1, |_, _| None);
            PresenterCommand::Redraw
        } else {
            PresenterCommand::Unknown
        }
    }

    fn event_scroll_down(&mut self, mod_state: &ModifierState) -> PresenterCommand {
        if mod_state.none_pressed() {
            self.commons.scroll_down(false, 1);
            PresenterCommand::Redraw
        } else {
            PresenterCommand::Unknown
        }
    }
}
