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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use super::interpreter::jobs::Job;
use super::iterators::*;
use super::response::*;
use super::screen::{AddBytesResult, Matrix, Screen};

use tools::shared_item;

/// Which output is visible.
///
/// The GUI concept dictates that at most one output (stdout or stderr) is visible.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OutputVisibility {
    None,
    Output,
    Error,
}

/// Running status of an interaction
#[derive(PartialEq, Debug, Clone)]
pub enum RunningStatus {
    Running,
    Unknown,
    Exited(i32),
}

/// A command and its output.
///
/// This is just a visual representation of a command and not connected to a running process in any
/// way.
struct Interaction {
    /// Visual representation of the command that was called to create these responses
    command: Matrix,
    /// Collected stdout lines
    output: Response,
    /// Collected stderr lines
    errors: Response,
    /// Which response to show
    visible: OutputVisibility,
    /// status of the command
    running_status: RunningStatus,
    /// True if TUI is running
    tui_mode: bool,
    /// Screen used for TUI mode
    pub tui_screen: Screen,
    /// Number of threads that feed data into the interaction.
    threads: usize,
    /// Job currently writing output to this interaction
    job: Option<Job>,
}

/// A number of commands that are executed with the same prompt string.
struct Conversation {
    /// List of programs and their outputs for this prompt.
    interactions: Vec<InteractionHandle>,
    /// The prompt for this conversation.
    prompt: Matrix,
    /// Hash value of the prompt for displaying a color
    prompt_hash: u64,
}

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
}

/// Session that can be shared between threads
#[derive(Clone)]
pub struct SharedSession(pub Arc<Mutex<Session>>);

/// Index of an interaction in a session.
///
/// While there will be usually less than 2^64 interactions in a session, this is a usize to avoid
/// error handling now. Opening too many interactions will eat up all the memory before the program
/// runs out of indices.
#[derive(PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub struct InteractionHandle(usize);

impl Interaction {
    /// Create a new command without any output yet.
    ///
    /// The command is a vector of cells as to support syntax coloring later.
    pub fn new(command: Matrix) -> Self {
        let mut tui_screen = Screen::new();
        tui_screen.make_room_for(79, 24);
        tui_screen.fixed_size();
        Self {
            command,
            output: Response::new(),
            errors: Response::new(),
            visible: OutputVisibility::Output,
            running_status: RunningStatus::Unknown,
            tui_mode: false,
            tui_screen,
            threads: 0,
            job: None,
        }
    }

    /// Get the visible response, if any.
    fn visible_response(&self) -> Option<&Response> {
        match self.visible {
            OutputVisibility::None => None,
            OutputVisibility::Output => Some(&self.output),
            OutputVisibility::Error => Some(&self.errors),
        }
    }

    /// Get the iterator over the items in this interaction.
    fn line_iter<'a>(
        &'a self,
        handle: InteractionHandle,
        prompt_hash: u64,
    ) -> impl Iterator<Item = LineItem<'a>> {
        // We always have the command, regardless if there is any output to show.
        let resp_lines = self
            .visible_response()
            .map(|r| r.line_iter(prompt_hash))
            .into_iter()
            .flat_map(|i| i);

        let visible = self.visible;
        let lt = LineType::Command(visible, handle, self.running_status.clone());

        self.command
            .line_iter()
            .map(move |r| LineItem::new(r, lt.clone(), None, prompt_hash))
            .chain(resp_lines)
    }

    /// Check if there are any error lines.
    fn has_errors(&self) -> bool {
        !self.errors.lines.is_empty()
    }

    /// Make the error lines visible
    pub fn show_errors(&mut self) {
        self.visible = OutputVisibility::Error;
    }

    /// If there are errors, show them.
    pub fn show_potential_errors(&mut self) {
        let failure = match self.running_status {
            RunningStatus::Exited(es) => es != 0,
            _ => false,
        };
        if !failure {
            self.visible = OutputVisibility::Output;
        } else if self.has_errors() {
            self.show_errors();
        }
    }

    /// If there is data in the TUI screen, add it to the end of output
    pub fn exit_cleanup(&mut self) {
        trace!("exit cleanup on interaction");
        if self.tui_mode {
            for l in self.tui_screen.line_iter() {
                self.output.lines.push(l.to_vec());
            }
            self.tui_mode = false;
            self.tui_screen.reset();
        }
    }
}

impl Conversation {
    /// Creates a new conversation without any interactions.
    pub fn new(prompt: Matrix) -> Conversation {
        let mut h = DefaultHasher::new();
        prompt.hash(&mut h);
        let prompt_hash = h.finish();

        Conversation {
            prompt,
            interactions: vec![],
            prompt_hash,
        }
    }
}

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
        }
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

    /// Return an iterator over the currently visible items.
    pub fn line_iter<'a>(
        &'a self,
        show_last_prompt: bool,
    ) -> Box<dyn Iterator<Item = LineItem<'a>> + 'a> {
        let num_conversations = self.conversations.len();
        Box::new(self.conversations.iter().enumerate().flat_map(
            move |(conversation_index, conversation)| {
                let prompt_hash = conversation.prompt_hash;
                let is_last_conv = (conversation_index + 1) >= num_conversations;
                let show_this_prompt = !is_last_conv || show_last_prompt;

                conversation
                    .interactions
                    .iter()
                    .flat_map(move |interHandle| {
                        self.interactions[interHandle.0].line_iter(*interHandle, prompt_hash)
                    })
                    .chain(
                        conversation
                            .prompt
                            .line_iter()
                            .map(move |r| LineItem::new(r, LineType::Prompt, None, prompt_hash))
                            .take_while(move |_| show_this_prompt),
                    )
            },
        ))
    }

    pub fn tui_screen<'a>(&'a self, handle: InteractionHandle) -> Option<&Screen> {
        if handle.0 < self.interactions.len() {
            Some(&self.interactions[handle.0].tui_screen)
        } else {
            None
        }
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
                            interaction.tui_mode = true;
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
}

#[cfg(test)]
mod tests {
    use super::super::response::tests::check;
    use super::super::screen::Screen;
    use super::*;

    fn new_test_session(prompt: &[u8]) -> SharedSession {
        SharedSession(Arc::new(Mutex::new(Session::new(Screen::one_line_matrix(
            prompt,
        )))))
    }

    #[test]
    fn line_iter() {
        let mut session = new_test_session(b"prompt 1");

        let inter_1_1 = session.add_interaction(Screen::one_line_matrix(b"command 1.1"));
        session.add_bytes(
            OutputVisibility::Output,
            inter_1_1,
            b"output 1.1.1\noutput 1.1.2\n",
        );
        let inter_1_2 = session.add_interaction(Screen::one_line_matrix(b"command 1.2"));
        session.add_bytes(
            OutputVisibility::Output,
            inter_1_2,
            b"output 1.2.1\noutput 1.2.2\n",
        );

        session.new_conversation(Screen::one_line_matrix(b"prompt 2"));
        let inter_2_1 = session.add_interaction(Screen::one_line_matrix(b"command 2.1"));
        session.add_bytes(
            OutputVisibility::Output,
            inter_2_1,
            b"output 2.1.1\noutput 2.1.2\n",
        );
        let inter_2_2 = session.add_interaction(Screen::one_line_matrix(b"command 2.2"));
        session.add_bytes(
            OutputVisibility::Output,
            inter_2_2,
            b"output 2.2.1\noutput 2.2.2\n",
        );

        session.session_mut((), |s| {
            assert_eq!(s.conversations.len(), 2);
            assert_eq!(s.conversations[0].interactions.len(), 2);
            assert_eq!(s.conversations[1].interactions.len(), 2);
        });

        let session = session.0.lock().unwrap();
        let mut li = session.line_iter(true);
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 0 },
                RunningStatus::Unknown,
            ),
            None,
            "command 1.1",
        );
        check(li.next(), LineType::Output, None, "output 1.1.1");
        check(li.next(), LineType::Output, None, "output 1.1.2");
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 1 },
                RunningStatus::Unknown,
            ),
            None,
            "command 1.2",
        );
        check(li.next(), LineType::Output, None, "output 1.2.1");
        check(li.next(), LineType::Output, None, "output 1.2.2");
        check(li.next(), LineType::Prompt, None, "prompt 1");

        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 2 },
                RunningStatus::Unknown,
            ),
            None,
            "command 2.1",
        );
        check(li.next(), LineType::Output, None, "output 2.1.1");
        check(li.next(), LineType::Output, None, "output 2.1.2");
        check(
            li.next(),
            LineType::Command(
                OutputVisibility::Output,
                InteractionHandle { 0: 3 },
                RunningStatus::Unknown,
            ),
            None,
            "command 2.2",
        );
        check(li.next(), LineType::Output, None, "output 2.2.1");
        check(li.next(), LineType::Output, None, "output 2.2.2");
        check(li.next(), LineType::Prompt, None, "prompt 2");
        assert_eq!(li.next(), None);
    }

}
