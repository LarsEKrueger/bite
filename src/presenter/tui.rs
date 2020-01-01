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
use model::bash::{is_bash_waiting, program_add_input};
use model::session::InteractionHandle;
use std::cmp;

/// Presenter to run commands and send input to their stdin.
pub struct TuiExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Terminal screen
    screen: Screen,

    /// Current interaction
    current_interaction: InteractionHandle,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<Matrix>,
}

impl TuiExecuteCommandPresenter {
    pub fn new(
        commons: Box<PresenterCommons>,
        current_interaction: InteractionHandle,
    ) -> Box<Self> {
        let mut s = Screen::new();
        s.make_room_for(
            (cmp::max(commons.window_width, 1) - 1) as isize,
            (cmp::max(commons.window_height, 1) - 1) as isize,
        );
        s.fixed_size();
        let presenter = TuiExecuteCommandPresenter {
            commons,
            screen: s,
            current_interaction,
            next_prompt: None,
        };

        Box::new(presenter)
    }

    fn deconstruct(self) -> (Box<PresenterCommons>, InteractionHandle, Option<Matrix>) {
        (self.commons, self.current_interaction, self.next_prompt)
    }

    fn add_bytes_to_screen(mut self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]) {
        for (_i, b) in bytes.iter().enumerate() {
            match self.screen.add_byte(*b) {
                // TODO: Handle end of TUI mode
                _ => {}
            };
        }
        (self, b"")
    }

    fn send_string(&self, send: &str) -> PresenterCommand {
        program_add_input(send);
        PresenterCommand::Redraw
    }

    fn send_term_info(&self, cap_name: &str) -> PresenterCommand {
        if let Some(cap_str) = self.commons.term_info.strings.get(cap_name) {
            program_add_input(&String::from_utf8_lossy(cap_str));
            PresenterCommand::Redraw
        } else {
            PresenterCommand::Unknown
        }
    }

    fn send_term_info_shift(
        &self,
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
    /// Provide read access to the data that is common to the presenter in all modi.
    fn commons<'a>(&'a self) -> &'a Box<PresenterCommons> {
        &self.commons
    }

    /// Provide write access to the data that is common to the presenter in all modi.
    fn commons_mut<'a>(&'a mut self) -> &'a mut Box<PresenterCommons> {
        &mut self.commons
    }

    fn to_commons(self) -> Box<PresenterCommons> {
        self.commons
    }

    fn add_output(self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]) {
        self.add_bytes_to_screen(bytes)
    }

    fn add_error(self: Box<Self>, bytes: &[u8]) -> (Box<dyn SubPresenter>, &[u8]) {
        self.add_bytes_to_screen(bytes)
    }

    fn set_exit_status(self: &mut Self, exit_status: ExitStatus) {
        self.commons
            .session
            .set_exit_status(self.current_interaction, exit_status);
    }

    fn set_next_prompt(self: &mut Self, bytes: &[u8]) {
        self.next_prompt = Some(Screen::one_line_matrix(bytes));
    }

    fn end_polling(self: Box<Self>, needs_marking: bool) -> (Box<dyn SubPresenter>, bool) {
        if !needs_marking && is_bash_waiting() {
            let (mut commons, current_interaction, next_prompt) = self.deconstruct();
            if let Some(prompt) = next_prompt {
                commons.session.archive_interaction(current_interaction);
                commons.session.new_conversation(prompt);
            }
            return (ComposeCommandPresenter::new(commons), true);
        }
        (self, false)
    }

    /// Return the lines to be presented.
    fn line_iter<'a>(&'a self) -> Box<dyn Iterator<Item = LineItem> + 'a> {
        let ref s = self.screen;
        Box::new(s.line_iter_full().zip(0..).map(move |(line, nr)| {
            let cursor_x = if s.cursor_y() == nr {
                Some(s.cursor_x() as usize)
            } else {
                None
            };
            LineItem::new(line, LineType::Tui, cursor_x, 0)
        }))
    }

    /// Handle the event when a modifier and a special key is pressed.
    fn event_special_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        key: &SpecialKey,
    ) -> (Box<dyn SubPresenter>, PresenterCommand) {
        let cmd = match (mod_state.as_tuple(), key) {
            ((_, _, _), SpecialKey::Escape) => {
                program_add_input("\x1b");
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
            ((_, _, _), SpecialKey::Tab) => self.send_string("\x08"),

            ((_, _, _), _) => {
                // For all other keys, do nothing as they can't be represented in a TUI.
                PresenterCommand::Unknown
            }
        };
        (self, cmd)
    }

    /// Handle the event when a modifier and a letter/number is pressed.
    fn event_normal_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<dyn SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), letter) {
            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle the event when the mouse was pushed and released at the same position.
    fn handle_click(
        mut self: Box<Self>,
        button: usize,
        _x: usize,
        y: usize,
    ) -> (Box<dyn SubPresenter>, NeedRedraw) {
        match (clicked_line_type(&mut *self, y), button) {
            _ => (self, NeedRedraw::No),
        }
    }

    fn event_text(self: Box<Self>, s: &str) -> (Box<dyn SubPresenter>, PresenterCommand) {
        self.send_string(s);
        (self, PresenterCommand::Redraw)
    }
}
