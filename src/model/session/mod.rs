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

//! Organizes the past and current programs and their outputs.

mod conversation;
mod interaction;
mod lineitem;
mod locator;
mod response;
#[cfg(test)]
pub mod test;

use std::sync::{Arc, Mutex};

use model::interpreter::jobs::Job;
use model::screen::{AddBytesResult, Matrix, Screen};
use tools::shared_item;

pub use self::interaction::{OutputVisibility, RunningStatus};
pub use self::lineitem::{LineItem, LineType};
pub use self::locator::{ConversationLocator, InteractionLocator, ResponseLocator};
pub use self::locator::{MaybeSessionLocator, SessionLocator};

use self::conversation::Conversation;
use self::interaction::Interaction;

pub const DEFAULT_TUI_WIDTH: usize = 80;
pub const DEFAULT_TUI_HEIGHT: usize = 25;

/// Session that can be shared between threads
#[derive(Clone)]
pub struct SharedSession(pub Arc<Mutex<Session>>);

/// An ordered list of conversations.
///
/// Conversations and Interactions are only supposed to be accessed through a session as their
/// indices need to be consistent.
pub struct Session {
    /// History of conversations, oldest first.
    conversations: Vec<Conversation>,

    /// History of interactions, oldest first.
    interactions: Vec<Interaction>,

    /// Marker if the session has been changed since the last redraw
    needs_redraw: bool,

    /// Width of window in characters
    window_width: usize,

    /// Height of window in characters
    window_height: usize,
}

/// Index of an interaction in a session.
///
/// While there will be usually less than 2^64 interactions in a session, this is a usize to avoid
/// error handling now. Opening too many interactions will eat up all the memory before the program
/// runs out of indices.
#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct InteractionHandle(usize);

impl InteractionHandle {
    pub const INVALID: Self = InteractionHandle(std::usize::MAX);
}

impl Session {
    /// Create a new session.
    pub fn new(prompt: Matrix) -> Self {
        Session {
            conversations: vec![Conversation::new(prompt)],
            interactions: vec![],
            needs_redraw: true,
            window_width: DEFAULT_TUI_WIDTH,
            window_height: DEFAULT_TUI_HEIGHT,
        }
    }

    /// Return a locator at the end of the prompt of the last conversation.
    ///
    /// Operates on a session to force locking the SharedSession in order to stay consistent.
    ///
    /// Returns None if session is empty.
    pub fn locate_at_last_prompt_end(&self) -> MaybeSessionLocator {
        let conversations = self.conversations.len();
        if conversations > 0 {
            let conversation = conversations - 1;
            let prompt_lines = self.conversations[conversation].prompt.rows() as usize;
            let in_conversation = ConversationLocator::Prompt(prompt_lines);
            return Some(SessionLocator {
                conversation,
                in_conversation,
            });
        }
        None
    }

    /// Return a locator at the last interaction in the given conversation
    ///
    /// Operates on a session to force locking the SharedSession in order to stay consistent.
    ///
    /// Returns None if locator doesn't have a valid conversation index.
    pub fn locate_at_conversation_end(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            let interactions = self.conversations[loc.conversation].interactions.len();
            if interactions > 0 {
                return Some(SessionLocator {
                    conversation: loc.conversation,
                    in_conversation: ConversationLocator::Interaction(
                        interactions - 1,
                        InteractionLocator::Command(0),
                    ),
                });
            }
        }
        None
    }

    pub fn locate_at_previous_interaction(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if let ConversationLocator::Interaction(interaction, _) = loc.in_conversation {
            if interaction > 0 {
                return Some(SessionLocator {
                    conversation: loc.conversation,
                    in_conversation: ConversationLocator::Interaction(
                        interaction - 1,
                        InteractionLocator::Command(0),
                    ),
                });
            }
        }
        None
    }

    /// Return a locator at the end of the output of the current interaction
    pub fn locate_at_output_end(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            if let ConversationLocator::Interaction(interaction_index, _) = loc.in_conversation {
                if interaction_index < self.conversations[loc.conversation].interactions.len() {
                    let interaction_handle =
                        self.conversations[loc.conversation].interactions[interaction_index];
                    // If an interaction is a TUI, refer to the screen. Otherwise refer to the
                    // response. This is an invariant that is independent of the display order.
                    let interaction = &self.interactions[interaction_handle.0];
                    let in_interaction = if interaction.tui_mode {
                        let tui_height = interaction.tui_screen.height() as usize;
                        InteractionLocator::Tui(tui_height)
                    } else {
                        // Use the visible response. If there is none, go to the last line of the
                        // command.
                        if let Some(response) = interaction.visible_response() {
                            // This is another invariant: If the screen is height = 0, use the lines.
                            let screen_height = response.screen.height() as usize;
                            let in_response = if screen_height == 0 {
                                ResponseLocator::Lines(response.lines.len())
                            } else {
                                ResponseLocator::Screen(screen_height)
                            };
                            InteractionLocator::Response(in_response)
                        } else {
                            let command_height = interaction.command.rows() as usize;
                            // No response visible: Go to last line of command
                            InteractionLocator::Command(command_height)
                        }
                    };
                    let in_conversation =
                        ConversationLocator::Interaction(interaction_index, in_interaction);
                    return Some(SessionLocator {
                        conversation: loc.conversation,
                        in_conversation,
                    });
                }
            }
        }
        None
    }

    /// Return a locator at the end of the archived lines of the current interaction
    pub fn locate_at_lines_end(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            if let ConversationLocator::Interaction(interaction_index, _) = loc.in_conversation {
                if interaction_index < self.conversations[loc.conversation].interactions.len() {
                    let interaction_handle =
                        self.conversations[loc.conversation].interactions[interaction_index];
                    // If an interaction is a TUI, refer to the screen. Otherwise refer to the
                    // response. This is an invariant that is independent of the display order.
                    let interaction = &self.interactions[interaction_handle.0];
                    let in_interaction = if interaction.tui_mode {
                        let tui_height = interaction.tui_screen.height() as usize;
                        InteractionLocator::Tui(tui_height)
                    } else {
                        // Use the visible response. If there is none, go to the last line of the
                        // command.
                        if let Some(response) = interaction.visible_response() {
                            InteractionLocator::Response(ResponseLocator::Lines(
                                response.lines.len(),
                            ))
                        } else {
                            let command_height = interaction.command.rows() as usize;
                            // No response visible: Go to last line of command
                            InteractionLocator::Command(command_height)
                        }
                    };
                    let in_conversation =
                        ConversationLocator::Interaction(interaction_index, in_interaction);
                    return Some(SessionLocator {
                        conversation: loc.conversation,
                        in_conversation,
                    });
                }
            }
        }
        None
    }

    /// Return a locator at the end of the command of the current interaction
    pub fn locate_at_command_end(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            if let ConversationLocator::Interaction(interaction_index, _) = loc.in_conversation {
                if interaction_index < self.conversations[loc.conversation].interactions.len() {
                    let interaction_handle =
                        self.conversations[loc.conversation].interactions[interaction_index];
                    // If an interaction is a TUI, refer to the screen. Otherwise refer to the
                    // response. This is an invariant that is independent of the display order.
                    let interaction = &self.interactions[interaction_handle.0];
                    let command_height = interaction.command.rows() as usize;
                    let in_interaction = InteractionLocator::Command(command_height);
                    let in_conversation =
                        ConversationLocator::Interaction(interaction_index, in_interaction);
                    return Some(SessionLocator {
                        conversation: loc.conversation,
                        in_conversation,
                    });
                }
            }
        }
        None
    }

    /// Return the beginning of the prompt of the previous conversation
    pub fn locate_at_previous_conversation(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation > 0 {
            return Some(SessionLocator {
                conversation: loc.conversation - 1,
                in_conversation: ConversationLocator::Prompt(0),
            });
        }
        None
    }

    /// Return a locator at the end of the prompt of a conversation
    pub fn locate_at_prompt_end(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            let prompt_lines = self.conversations[loc.conversation].prompt.rows() as usize;
            let in_conversation = ConversationLocator::Prompt(prompt_lines);
            return Some(SessionLocator {
                conversation: loc.conversation,
                in_conversation,
            });
        }
        None
    }

    /// Return a locator at the first line of the command of the first interaction of the next conversation
    pub fn locate_at_next_conversation(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        let conversation_index = loc.conversation + 1;
        if conversation_index < self.conversations.len() {
            let in_conversation =
                ConversationLocator::Interaction(0, InteractionLocator::Command(0));
            return Some(SessionLocator {
                conversation: conversation_index,
                in_conversation,
            });
        }
        None
    }

    /// Return a locator at the start of the output of the current interaction
    pub fn locate_at_output_start(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            let conversation = &self.conversations[loc.conversation];
            if let ConversationLocator::Interaction(interaction_index, _) = loc.in_conversation {
                if interaction_index < conversation.interactions.len() {
                    let interaction_handle =
                        self.conversations[loc.conversation].interactions[interaction_index];
                    if interaction_handle.0 < self.interactions.len() {
                        let interaction = &self.interactions[interaction_handle.0];
                        let interaction_locator = if interaction.tui_mode {
                            InteractionLocator::Tui(0)
                        } else {
                            InteractionLocator::Response(ResponseLocator::Lines(0))
                        };
                        let in_conversation = ConversationLocator::Interaction(
                            interaction_index,
                            interaction_locator,
                        );
                        return Some(SessionLocator {
                            conversation: loc.conversation,
                            in_conversation,
                        });
                    }
                }
            }
        }
        None
    }

    /// Return a locator at the start of the next interaction
    pub fn locate_at_next_interaction(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            let conversation = &self.conversations[loc.conversation];
            if let ConversationLocator::Interaction(interaction_index, _) = loc.in_conversation {
                let interaction_index = interaction_index + 1;
                if interaction_index < conversation.interactions.len() {
                    let in_conversation = ConversationLocator::Interaction(
                        interaction_index,
                        InteractionLocator::Command(0),
                    );
                    return Some(SessionLocator {
                        conversation: loc.conversation,
                        in_conversation,
                    });
                }
            }
        }
        None
    }

    /// Return a locator at the start of the prompt of the current conversation
    pub fn locate_at_prompt_start(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            let in_conversation = ConversationLocator::Prompt(0);
            return Some(SessionLocator {
                conversation: loc.conversation,
                in_conversation,
            });
        }
        None
    }

    /// Return a locator at the start of the screen of the current interaction
    pub fn locate_at_screen_start(&self, loc: &SessionLocator) -> MaybeSessionLocator {
        if loc.conversation < self.conversations.len() {
            let conversation = &self.conversations[loc.conversation];
            if let ConversationLocator::Interaction(interaction_index, _) = loc.in_conversation {
                if interaction_index < conversation.interactions.len() {
                    let in_conversation = ConversationLocator::Interaction(
                        interaction_index,
                        InteractionLocator::Response(ResponseLocator::Screen(0)),
                    );
                    return Some(SessionLocator {
                        conversation: loc.conversation,
                        in_conversation,
                    });
                }
            }
        }
        None
    }

    /// Check if the locator is past the element
    ///
    /// Returns None if invalid
    pub fn locator_is_end_line(&self, loc: &SessionLocator) -> Option<bool> {
        if loc.conversation < self.conversations.len() {
            let conversation = &self.conversations[loc.conversation];
            match &loc.in_conversation {
                ConversationLocator::Prompt(line) => {
                    return Some(
                        *line >= (self.conversations[loc.conversation].prompt.rows() as usize),
                    )
                }
                ConversationLocator::Interaction(interaction_index, interaction_locator) => {
                    if *interaction_index < conversation.interactions.len() {
                        let interaction_handle = &conversation.interactions[*interaction_index];
                        if interaction_handle.0 < self.interactions.len() {
                            let interaction = &self.interactions[interaction_handle.0];
                            match &interaction_locator {
                                InteractionLocator::Command(line) => {
                                    return Some(*line >= (interaction.command.rows() as usize))
                                }
                                InteractionLocator::Tui(line) => {
                                    return Some(
                                        *line >= (interaction.tui_screen.height() as usize),
                                    )
                                }
                                InteractionLocator::Response(ResponseLocator::Lines(line)) => {
                                    return interaction
                                        .visible_response()
                                        .and_then(|r| Some(*line >= r.lines.len()))
                                        .or(Some(true))
                                }
                                InteractionLocator::Response(ResponseLocator::Screen(line)) => {
                                    return interaction
                                        .visible_response()
                                        .and_then(|r| Some(*line >= (r.screen.height() as usize)))
                                        .or(Some(true))
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Increment the line field in a locator
    pub fn locator_inc_line(&self, loc: &mut SessionLocator, lines: &mut usize) {
        if loc.conversation < self.conversations.len() {
            let conversation = &self.conversations[loc.conversation];
            let maybe_line_count: Option<(&mut usize, usize)> = match &mut loc.in_conversation {
                ConversationLocator::Prompt(line) => Some((
                    line,
                    (self.conversations[loc.conversation].prompt.rows() as usize),
                )),
                ConversationLocator::Interaction(interaction_index, interaction_locator) => {
                    if *interaction_index < conversation.interactions.len() {
                        let interaction_handle = &conversation.interactions[*interaction_index];
                        if interaction_handle.0 < self.interactions.len() {
                            let interaction = &self.interactions[interaction_handle.0];
                            match interaction_locator {
                                InteractionLocator::Command(line) => {
                                    Some((line, (interaction.command.rows() as usize)))
                                }
                                InteractionLocator::Tui(line) => {
                                    Some((line, (interaction.tui_screen.height() as usize)))
                                }
                                InteractionLocator::Response(ResponseLocator::Lines(line)) => {
                                    interaction
                                        .visible_response()
                                        .and_then(|r| Some((line, r.lines.len())))
                                }
                                InteractionLocator::Response(ResponseLocator::Screen(line)) => {
                                    interaction
                                        .visible_response()
                                        .and_then(|r| Some((line, (r.screen.height() as usize))))
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };
            if let Some((line, count)) = maybe_line_count {
                if *line + *lines <= count {
                    // The desired position is inside the range that the locator describes.
                    *line += *lines;
                    *lines = 0;
                } else {
                    // The desired position is past the range that the locator describes
                    // Reduce the number of lines by the distance to the end of the locator
                    // range. At the same time, advance the locator to the end of of its range.
                    // This can only work if the line is smaller than the range described by the
                    // locator.
                    //
                    // If the locator describes a position past the end of the range, update
                    // the locator to the end, but leave the count as-is. This can happen if the
                    // locator's position isn't updated, but it's length, e.g. by changes in
                    // visibility.
                    if *line <= count {
                        *lines -= count - *line;
                    }
                    *line = count;
                }
            }
        }
    }

    /// Determine if locator is at last conversation
    pub fn locator_is_last_conversation(&self, loc: &SessionLocator) -> bool {
        (loc.conversation + 1) == self.conversations.len()
    }

    /// Quick access to an interaction by handle.
    ///
    /// Returns the default for illegal handles.
    fn interaction_mut<F, R>(&mut self, handle: InteractionHandle, default: R, f: F) -> R
    where
        F: FnOnce(&mut Interaction) -> R,
    {
        if handle.0 < self.interactions.len() {
            f(&mut self.interactions[handle.0])
        } else {
            default
        }
    }

    /// Quick access to an interaction by handle.
    ///
    /// Returns the default for illegal handles.
    fn interaction<F, R>(&self, handle: InteractionHandle, default: R, f: F) -> R
    where
        F: FnOnce(&Interaction) -> R,
    {
        if handle.0 < self.interactions.len() {
            f(&self.interactions[handle.0])
        } else {
            default
        }
    }

    /// Show the output of a given interaction
    pub fn show_output(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| i.visible = OutputVisibility::Output)
    }

    /// Show the errors of a given interaction
    pub fn show_errors(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), Interaction::show_errors)
    }

    /// Archive the given interaction
    pub fn archive_interaction(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            i.output.archive_screen();
            i.errors.archive_screen();
        })
    }

    /// Return a LineItem for the given locator position
    pub fn display_line<'a>(&'a self, loc: &SessionLocator) -> Option<LineItem<'a>> {
        if loc.conversation < self.conversations.len() {
            let conversation = &self.conversations[loc.conversation];
            let prompt_hash = conversation.prompt_hash;
            match &loc.in_conversation {
                ConversationLocator::Prompt(line) => {
                    if *line < (conversation.prompt.rows() as usize) {
                        return Some(LineItem::new(
                            conversation.prompt.compacted_row_slice(*line as isize),
                            LineType::Prompt,
                            None,
                            prompt_hash,
                        ));
                    }
                }
                ConversationLocator::Interaction(interaction_index, interaction_locator) => {
                    if *interaction_index < conversation.interactions.len() {
                        let interaction_handle = &conversation.interactions[*interaction_index];
                        if interaction_handle.0 < self.interactions.len() {
                            let interaction = &self.interactions[interaction_handle.0];
                            match interaction_locator {
                                InteractionLocator::Command(line) => {
                                    if *line < (interaction.command.rows() as usize) {
                                        let lt = LineType::Command(
                                            interaction.visible,
                                            *interaction_handle,
                                            interaction.running_status.clone(),
                                        );
                                        return Some(LineItem::new(
                                            interaction.command.compacted_row_slice(*line as isize),
                                            lt,
                                            None,
                                            prompt_hash,
                                        ));
                                    }
                                }
                                InteractionLocator::Tui(line) => {
                                    if *line < (interaction.tui_screen.height() as usize) {
                                        return Some(LineItem::new(
                                            interaction.tui_screen.row_slice(*line as isize),
                                            LineType::Output,
                                            None,
                                            prompt_hash,
                                        ));
                                    }
                                }
                                InteractionLocator::Response(ResponseLocator::Lines(line)) => {
                                    return interaction.visible_response().and_then(|r| {
                                        if *line < r.lines.len() {
                                            Some(LineItem::new(
                                                &r.lines[*line][..],
                                                LineType::Output,
                                                None,
                                                conversation.prompt_hash,
                                            ))
                                        } else {
                                            None
                                        }
                                    });
                                }
                                InteractionLocator::Response(ResponseLocator::Screen(line)) => {
                                    return interaction.visible_response().and_then(|r| {
                                        if *line < r.screen.height() as usize {
                                            Some(LineItem::new(
                                                &r.screen.compacted_row_slice(*line as isize),
                                                LineType::Output,
                                                None,
                                                conversation.prompt_hash,
                                            ))
                                        } else {
                                            None
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Add a new interaction to the latest conversation.
    fn add_interaction_to_last(&mut self, command: Matrix) -> InteractionHandle {
        let handle = InteractionHandle(self.interactions.len());
        self.interactions.push(Interaction::new(command));
        if let Some(current) = self.conversations.last_mut() {
            current.interactions.push(handle);
        }
        handle
    }

    pub fn tui_screen<'a>(&'a self, handle: &InteractionHandle) -> Option<&'a Screen> {
        if handle.0 < self.interactions.len() {
            let interaction = &self.interactions[handle.0];
            if interaction.tui_mode {
                return Some(&self.interactions[handle.0].tui_screen);
            }
        }
        None
    }
}

impl SharedSession {
    /// Create a new session.
    pub fn new(prompt: Matrix) -> Self {
        SharedSession(shared_item::new(Session::new(prompt)))
    }

    /// Quick access to the underlying session
    ///
    /// Does nothing if something goes wrong
    fn session_mut<F, R>(&mut self, default: R, f: F) -> R
    where
        F: FnOnce(&mut Session) -> R,
    {
        shared_item::item_mut(&mut self.0, default, f)
    }

    /// Quick access to the underlying session
    ///
    /// Does nothing if something goes wrong
    fn session<F, R>(&self, default: R, f: F) -> R
    where
        F: FnOnce(&Session) -> R,
    {
        shared_item::item(&self.0, default, f)
    }

    /// Quick access to an interaction by handle.
    ///
    /// Returns the default if something goes wrong.
    fn interaction_mut<F, R>(&mut self, handle: InteractionHandle, default: R, f: F) -> R
    where
        F: FnOnce(&mut Interaction) -> R,
        R: Copy,
    {
        self.session_mut(default, |s| s.interaction_mut(handle, default, f))
    }

    /// Quick access to an interaction by handle.
    ///
    /// Returns the default if something goes wrong.
    fn interaction<F, R>(&self, handle: InteractionHandle, default: R, f: F) -> R
    where
        F: FnOnce(&Interaction) -> R,
        R: Copy,
    {
        self.session(default, |s| s.interaction(handle, default, f))
    }

    /// Add a new interaction to the latest conversation.
    pub fn add_interaction(&mut self, command: Matrix) -> InteractionHandle {
        self.session_mut(InteractionHandle(std::usize::MAX), |s| {
            s.add_interaction_to_last(command)
        })
    }

    /// Create an interaction in the same conversation as the given one
    pub fn create_sub_interaction(
        &mut self,
        parent_handle: InteractionHandle,
    ) -> InteractionHandle {
        self.session_mut(InteractionHandle(std::usize::MAX), |s| {
            // Find the conversation that contains the parent interaction
            // Iterative over a conversations and their index
            let maybe_conversation = s.conversations.iter_mut().find(|c| {
                // If the index of the parent interaction is found in this conversation, return its index.
                c.interactions
                    .iter()
                    .position(|inter| *inter == parent_handle)
                    .is_some()
            });
            if let Some(conversation) = maybe_conversation {
                // Found a conversation, add an interaction
                let handle = InteractionHandle(s.interactions.len());
                // Derive the command from the parent interaction
                let mut command =
                    Screen::new_from_matrix(s.interactions[parent_handle.0].command.clone());
                command.move_right_edge();
                let _ = command.add_bytes(format!(" # [{}]", handle.0).as_bytes());

                s.interactions.push(Interaction::new(command.freeze()));
                conversation.interactions.push(handle);
                handle
            } else {
                // No conversation found, add one to the last
                error!(
                    "No conversation found containing interaction {:?}",
                    parent_handle
                );
                s.add_interaction_to_last(Screen::one_line_matrix(b"Unknown background program"))
            }
        })
    }

    /// Open a new conversation if the prompts are different
    pub fn new_conversation(&mut self, prompt: Matrix) {
        self.session_mut((), |s| {
            if let Some(current) = s.conversations.last_mut() {
                if current.prompt == prompt {
                    return;
                }
            }
            s.conversations.push(Conversation::new(prompt));
        });
    }

    /// Add bytes to selected stream of interaction
    ///
    /// If the interaction is already in TUI mode, use that response instead.
    pub fn add_bytes(&mut self, stream: OutputVisibility, handle: InteractionHandle, bytes: &[u8]) {
        let mut needs_redraw = false;
        self.interaction_mut(handle, (), |interaction| {
            // TUI mode overrides stream
            let mut work = bytes;
            while work.len() != 0 {
                if interaction.tui_mode {
                    // Add the bytes to the screen
                    for b in work {
                        // TODO: Handle the events correctly.
                        let _ = interaction.tui_screen.add_byte(*b);
                    }
                    needs_redraw = true;
                    return;
                } else {
                    let response = match stream {
                        OutputVisibility::None => return,
                        OutputVisibility::Output => &mut interaction.output,
                        OutputVisibility::Error => &mut interaction.errors,
                    };
                    // Process the bytes
                    match response.add_bytes(work) {
                        AddBytesResult::AllDone => break,
                        AddBytesResult::ShowStream(new_work) => {
                            needs_redraw = true;
                            work = new_work;
                        }
                        AddBytesResult::StartTui(new_work) => {
                            interaction.set_tui_size(DEFAULT_TUI_WIDTH, DEFAULT_TUI_HEIGHT);
                            work = new_work;
                        }
                    }
                }
            }
            // Make new output show up
            interaction.visible = stream;
        });
        self.session_mut((), |s| s.needs_redraw |= needs_redraw);
    }

    /// Set the running status of an interaction
    pub fn set_running_status(&mut self, handle: InteractionHandle, status: RunningStatus) {
        trace!("Set Running Status of {:?} to {:?}", handle, status);
        self.interaction_mut(handle, (), |i| {
            i.running_status = status;
            i.show_potential_errors();
        });
    }

    /// Check if the given interaction is still running
    pub fn has_exited(&self, handle: InteractionHandle) -> bool {
        self.interaction(handle, false, |i| {
            if let RunningStatus::Exited(_) = i.running_status {
                true
            } else {
                false
            }
        })
    }

    /// Mark the session as redrawn
    pub fn mark_drawn(&mut self) {
        self.session_mut((), |s| s.needs_redraw = false)
    }

    /// Check if the session needs redrawing and reset that
    pub fn check_redraw(&mut self) -> bool {
        self.session_mut(true, |s| {
            let res = s.needs_redraw;
            s.needs_redraw = false;
            res
        })
    }

    /// Check if the given interaction is in TUI mode
    pub fn is_tui(&self, handle: InteractionHandle) -> bool {
        self.interaction(handle, false, |i| i.tui_mode)
    }

    /// Cycle the visibility of an interaction
    pub fn cycle_visibility(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            let v = match i.visible {
                OutputVisibility::Output => OutputVisibility::Error,
                OutputVisibility::Error => OutputVisibility::None,
                OutputVisibility::None => OutputVisibility::Output,
            };
            i.visible = v;
        })
    }

    /// Set the visibility
    pub fn set_visibility(&mut self, handle: InteractionHandle, visible: OutputVisibility) {
        self.interaction_mut(handle, (), |i| i.visible = visible)
    }

    /// Get the visibility
    pub fn get_visibility(&mut self, handle: InteractionHandle) -> Option<OutputVisibility> {
        self.interaction(handle, None, |i| Some(i.visible))
    }

    /// Find the last interaction
    pub fn last_interaction(&self) -> Option<InteractionHandle> {
        self.session(None, |session| {
            if session.interactions.len() == 0 {
                None
            } else {
                Some(InteractionHandle(session.interactions.len() - 1))
            }
        })
    }

    /// Set the visibility of all interactions
    pub fn set_visibility_all(&mut self, ov: OutputVisibility) {
        self.session_mut((), |session| {
            for inter in session.interactions.iter_mut() {
                inter.visible = ov.clone();
            }
        })
    }

    /// Increment the number of threads that feed data into an interaction
    pub fn register_thread(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            i.threads = i.threads.saturating_add(1);
            trace!("Register thread on {:?}: {} threads", handle, i.threads);
        });
    }

    /// Decrement the number of threads that feed data into an interaction. If the number becomes
    /// zero, do exit clean up on the interaction.
    pub fn unregister_thread(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            i.threads = i.threads.saturating_sub(1);
            trace!("Unregister thread on {:?}: {} threads", handle, i.threads);
            if i.threads == 0 {
                i.exit_cleanup();
            }
        });
    }

    /// Print the interaction to the respective streams
    pub fn print_interaction(&mut self, handle: InteractionHandle) {
        self.interaction(handle, (), |interaction| {
            use std::io::Write;
            let mut b = [0; 4];
            {
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                let _ = stdout.write(b"BiTE startup stdout output\n");
                for l in interaction.output.lines.iter() {
                    for c in l {
                        let _ = stdout.write(c.code_point().encode_utf8(&mut b).as_bytes());
                    }
                    let _ = stdout.write(b"\n");
                }
            }
            {
                let stderr = std::io::stderr();
                let mut stderr = stderr.lock();
                let _ = stderr.write(b"BiTE startup stderr output\n");
                for l in interaction.errors.lines.iter() {
                    for c in l {
                        let _ = stderr.write(c.code_point().encode_utf8(&mut b).as_bytes());
                    }
                    let _ = stderr.write(b"\n");
                }
            }
        });
    }

    /// Set the current job of an interaction
    pub fn set_job(&mut self, handle: InteractionHandle, job: Option<Job>) {
        self.interaction_mut(handle, (), |i| i.job = job)
    }

    /// Terminate the current job of an interaction
    pub fn terminate(&mut self, handle: InteractionHandle) {
        self.interaction_mut(handle, (), |i| {
            if let Some(ref mut job) = i.job {
                job.terminate();
            }
        })
    }

    /// Send some bytes to the current job of an interaction
    ///
    /// Does nothing if there is no job
    pub fn write_stdin(&mut self, handle: InteractionHandle, bytes: &[u8]) {
        self.interaction_mut(handle, (), |i| {
            if let Some(ref mut job) = i.job {
                job.write_stdin(bytes);
            }
        })
    }

    /// Find the next newer interaction that is a TUI and still running.
    ///
    /// handle: None => Start at the oldest.
    ///
    /// Return None if none such interaction can be found. This will cause all interactions to be
    /// shown.
    pub fn next_running_tui(&self, handle: Option<InteractionHandle>) -> Option<InteractionHandle> {
        self.session(None, |s| {
            // If the start index was invalid, quit right now. This covers the case when there are
            // no interactions.
            let mut index = handle.map(|h| h.0).unwrap_or(0);
            if index >= s.interactions.len() {
                return None;
            }
            loop {
                // Go to the next interaction
                index += 1;
                // Reached the end, show all interactions
                if index >= s.interactions.len() {
                    return None;
                }
                // If the interaction at the index is a TUI and still running, return the index.
                if s.interactions[index].tui_mode
                    && s.interactions[index].running_status.is_running()
                {
                    return Some(InteractionHandle(index));
                }
            }
        })
    }

    pub fn set_window_size(&mut self, w: usize, h: usize) {
        self.session_mut((), |s| {
            s.window_width = w;
            s.window_height = h;
        });
    }

    pub fn window_size(&self) -> (usize, usize) {
        self.session((80, 25), |s| (s.window_width, s.window_height))
    }

    pub fn set_tui_size(&mut self, handle: InteractionHandle, w: usize, h: usize) {
        self.interaction_mut(handle, (), |interaction| interaction.set_tui_size(w, h))
    }

    pub fn set_tui_size_default(&mut self, handle: InteractionHandle) {
        self.set_tui_size(handle, DEFAULT_TUI_WIDTH, DEFAULT_TUI_HEIGHT);
    }
}
