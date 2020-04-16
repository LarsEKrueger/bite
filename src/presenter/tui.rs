/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2019  Lars Krüger

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

//! Sub presenter for TUIs
//!
//! Same functionality as ExecuteCommandPresenter, but maintains a single, non-resizable screen and
//! does not archive its output.

use super::*;
use model::session::InteractionHandle;

/// Presenter to run commands and send input to their stdin.
pub struct TuiExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,
    /// Current interaction
    current_interaction: InteractionHandle,
}

impl TuiExecuteCommandPresenter {
    pub fn new(
        commons: Box<PresenterCommons>,
        current_interaction: InteractionHandle,
    ) -> Box<Self> {
        let mut presenter = TuiExecuteCommandPresenter {
            commons,
            current_interaction,
        };

        // Show all lines
        presenter.commons.last_line_shown = presenter.commons.window_height;

        Box::new(presenter)
    }

    fn deconstruct(self) -> (Box<PresenterCommons>, InteractionHandle) {
        (self.commons, self.current_interaction)
    }

    fn send_string(&mut self, send: &str) -> PresenterCommand {
        self.commons
            .session
            .write_stdin(self.current_interaction, send.as_bytes());
        PresenterCommand::Redraw
    }

    fn send_term_info(&mut self, cap_name: &str) -> PresenterCommand {
        if let Some(cap_str) = self.commons.term_info.strings.get(cap_name) {
            self.commons
                .session
                .write_stdin(self.current_interaction, cap_str);
            PresenterCommand::Redraw
        } else {
            PresenterCommand::Unknown
        }
    }

    fn send_term_info_shift(
        &mut self,
        shifted: bool,
        cap_normal: &str,
        cap_shifted: &str,
    ) -> PresenterCommand {
        if shifted {
            self.send_term_info(cap_shifted)
        } else {
            self.send_term_info(cap_normal)
        }
    }
}

impl SubPresenter for TuiExecuteCommandPresenter {
    fn finish(self: Box<Self>) -> Box<PresenterCommons> {
        self.commons
    }

    /// Provide read access to the data that is common to the presenter in all modi.
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    /// Provide write access to the data that is common to the presenter in all modi.
    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    /// Return the lines to be presented.
    fn line_iter<'a>(&'a self, session: &'a Session) -> Box<dyn Iterator<Item = LineItem> + 'a> {
        match session.tui_screen(self.current_interaction) {
            Some(s) => Box::new(s.line_iter_full().zip(0..).map(move |(line, nr)| {
                let cursor_x = if s.cursor_y() == nr {
                    Some(s.cursor_x() as usize)
                } else {
                    None
                };
                LineItem::new(line, LineType::Tui, cursor_x, 0)
            })),
            None => {
                error!("Got empty screen for {:?}", self.current_interaction);
                Box::new(std::iter::empty())
            }
        }
    }

    fn get_overlay(&self, _session: &Session) -> Option<(Vec<String>, usize, usize, i32)> {
        None
    }

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        &mut self,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> PresenterCommand {
        match (mod_state.as_tuple(), key) {
            ((_, _, _), SpecialKey::Escape) => {
                // TODO: Send key to program
                PresenterCommand::Redraw
            }
            ((_, _, _), SpecialKey::Enter) => self.send_term_info("kent"),
            ((shifted, _, _), SpecialKey::Left) => {
                self.send_term_info_shift(shifted, "kcub1", "kLFT")
            }
            ((shifted, _, _), SpecialKey::Right) => {
                self.send_term_info_shift(shifted, "kcuf1", "kRIT")
            }
            ((_, _, _), SpecialKey::Up) => self.send_term_info("kcuu1"),
            ((_, _, _), SpecialKey::Down) => self.send_term_info("kcud1"),
            ((shifted, _, _), SpecialKey::Home) => {
                self.send_term_info_shift(shifted, "khome", "kHOM")
            }
            ((shifted, _, _), SpecialKey::End) => {
                self.send_term_info_shift(shifted, "kend", "kEND")
            }
            ((_, _, _), SpecialKey::PageUp) => self.send_term_info("kpp"),
            ((_, _, _), SpecialKey::PageDown) => self.send_term_info("knp"),
            ((shifted, _, _), SpecialKey::Delete) => {
                self.send_term_info_shift(shifted, "kdch1", "kDC")
            }
            ((_, _, _), SpecialKey::Backspace) => self.send_term_info("kbs"),
            ((false, false, false), SpecialKey::Tab) => self.send_term_info("tab"),

            //           // Ctrl-Tab => Switch to next running TUI if there is one
            //           ((false, true, false), SpecialKey::Tab) => {
            //               let next_tui = self
            //                   .commons
            //                   .session
            //                   .next_running_tui(Some(self.current_interaction));
            //               trace!("Ctrl-Tab next_tui: {:?}", next_tui);
            //               if let Some(next_tui) = next_tui {
            //                   self.current_interaction = next_tui;
            //                   PresenterCommand::Redraw
            //               } else {
            //                   // None found, return to the previous presenter
            //                   if let Some(return_interaction) = self.return_interaction {
            //                       // Previous interaction found -> return to ExecuteCommandPresenter
            //                       return (
            //                           ExecuteCommandPresenter::new(self.commons, return_interaction),
            //                           PresenterCommand::Redraw,
            //                       );
            //                   } else {
            //                       // No previous interaction -> return ComposeCommandPresenter
            //                       return (
            //                           ComposeCommandPresenter::new(self.commons),
            //                           PresenterCommand::Redraw,
            //                       );
            //                   }
            //               }
            //           }
            //           // Shift-Ctrl-Tab => Switch to previous running TUI if there is one
            //           ((true, true, false), SpecialKey::Tab) => {
            //               let next_tui = self
            //                   .commons
            //                   .session
            //                   .prev_running_tui(Some(self.current_interaction));
            //               trace!("Shift-Ctrl-Tab: next_tui: {:?}", next_tui);
            //               if let Some(next_tui) = next_tui {
            //                   self.current_interaction = next_tui;
            //                   PresenterCommand::Redraw
            //               } else {
            //                   // No previous interaction -> return ComposeCommandPresenter
            //                   return (
            //                       ComposeCommandPresenter::new(self.commons),
            //                       PresenterCommand::Redraw,
            //                   );
            //               }
            //           }
            ((_, _, _), _) => {
                // For all other keys, do nothing as they can't be represented in a TUI.
                PresenterCommand::Unknown
            }
        }
    }

    /// Handle the event when a modifier and a letter/number is pressed.
    fn event_normal_key(&mut self, mod_state: &ModifierState, letter: u8) -> PresenterCommand {
        match (mod_state.as_tuple(), letter) {
            _ => PresenterCommand::Unknown,
        }
    }

    /// Handle the event when the mouse was pushed and released at the same position.
    fn handle_click(&mut self, button: usize, _x: usize, y: usize) -> NeedRedraw {
        match (clicked_line_type(&mut *self, y), button) {
            _ => NeedRedraw::No,
        }
    }

    fn event_text(&mut self, s: &str) -> PresenterCommand {
        self.send_string(s);
        PresenterCommand::Redraw
    }
}
