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

use super::*;

/// Presenter to run commands and send input to their stdin.
pub struct ExecuteCommandPresenter {
    /// Common data.
    commons: Box<PresenterCommons>,

    /// Current interaction
    current_interaction: Interaction,

    /// Channel to running command
    cmd_input: Sender<String>,

    /// Channel from running command
    cmd_output: Receiver<execute::CommandOutput>,
}

impl ExecuteCommandPresenter {
    pub fn new(
        commons: Box<PresenterCommons>,
        prompt: String,
        cmd_input: Sender<String>,
        cmd_output: Receiver<execute::CommandOutput>,
    ) -> Box<Self> {
        let presenter = ExecuteCommandPresenter {
            commons,
            current_interaction: Interaction::new(prompt),
            cmd_input,
            cmd_output,
        };
        Box::new(presenter)
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
