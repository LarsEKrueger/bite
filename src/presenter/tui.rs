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
use model::bash::is_bash_waiting;
use model::interaction::CurrentInteraction;

/// Presenter to run commands and send input to their stdin.
pub struct TuiExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Terminal screen
    screen: Screen,

    /// Current interaction
    current_interaction: CurrentInteraction,

    /// Prompt to set. If None, we didn't receive one yet
    next_prompt: Option<Matrix>,
}

impl TuiExecuteCommandPresenter {
    pub fn new(
        commons: Box<PresenterCommons>,
        current_interaction: CurrentInteraction,
    ) -> Box<Self> {
        let presenter = TuiExecuteCommandPresenter {
            commons,
            screen: Screen::new(),
            current_interaction,
            next_prompt: None,
        };

        Box::new(presenter)
    }

    fn deconstruct(self) -> (Box<PresenterCommons>, CurrentInteraction, Option<Matrix>) {
        (self.commons, self.current_interaction, self.next_prompt)
    }

    fn add_bytes_to_screen(mut self: Box<Self>, bytes: &[u8]) -> (Box<SubPresenter>, &[u8]) {
        for (i, b) in bytes.iter().enumerate() {
            match self.screen.add_byte(*b) {
                // TODO: Handle TUI Switch
                _ => {}

            };
        }
        (self, b"")
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

    fn add_output(self: Box<Self>, bytes: &[u8]) -> (Box<SubPresenter>, &[u8]) {
        self.add_bytes_to_screen(bytes)
    }

    fn add_error(self: Box<Self>, bytes: &[u8]) -> (Box<SubPresenter>, &[u8]) {
        self.add_bytes_to_screen(bytes)
    }

    fn set_exit_status(self: &mut Self, exit_status: ExitStatus) {
        self.current_interaction.set_exit_status(exit_status);
    }
    fn set_next_prompt(self: &mut Self, bytes: &[u8]) {
        self.next_prompt = Some(Screen::one_line_matrix(bytes));
    }

    fn end_polling(self: Box<Self>, needs_marking: bool) -> Box<SubPresenter> {
        if !needs_marking && is_bash_waiting() {
            let (mut commons, current_interaction, next_prompt) = self.deconstruct();
            if let Some(prompt) = next_prompt {
                commons.session.archive_interaction(
                    current_interaction.prepare_archiving(),
                );

                if prompt != commons.session.current_conversation.prompt {
                    commons.session.new_conversation(prompt);
                }
            }
            return ComposeCommandPresenter::new(commons);
        }
        self
    }

    /// Return the lines to be presented.
    fn line_iter<'a>(&'a self) -> Box<Iterator<Item = LineItem> + 'a> {
        let ref s = self.screen;
        Box::new(s.line_iter().zip(0..).map(move |(line, nr)| {
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
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), key) {

            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle the event when a modifier and a letter/number is pressed.
    fn event_normal_key(
        self: Box<Self>,
        mod_state: &ModifierState,
        letter: u8,
    ) -> (Box<SubPresenter>, PresenterCommand) {
        match (mod_state.as_tuple(), letter) {

            _ => (self, PresenterCommand::Unknown),
        }
    }

    /// Handle the event when the input string was changed.
    fn event_update_line(self: Box<Self>) -> Box<SubPresenter> {
        self
    }

    /// Handle the event when the mouse was pushed and released at the same position.
    fn handle_click(
        mut self: Box<Self>,
        button: usize,
        _x: usize,
        y: usize,
    ) -> (Box<SubPresenter>, NeedRedraw) {
        match (clicked_line_type(&mut *self, y), button) {
            _ => (self, NeedRedraw::No),
        }
    }
}
