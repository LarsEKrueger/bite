/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2020  Lars Krüger

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

//! Module tests for Session, mostly for the locator code

use model::screen::Screen;
use model::session::*;

pub fn new_test_session(prompt: &[u8]) -> SharedSession {
    SharedSession(Arc::new(Mutex::new(Session::new(Screen::one_line_matrix(
        prompt,
    )))))
}

/// Test locator
///
/// Session:
/// * prompt 1
///     * command 1.1
///         * output 1.1.1
///         * output 1.1.2
///     * command 1.2
///         * output 1.2.1
///         * output 1.2.2
/// * prompt 2a
///   prompt 2b
///     * command 2.1
///         * output 2.1.1
///         * output 2.1.2
///     * command 2.2
///         * output 2.2.1
///         * output 2.2.2
#[test]
fn locator() {
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

    session.new_conversation(Screen::one_line_matrix(b"prompt 2a\nprompt 2b"));
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

    assert_eq!(session.conversations[1].prompt.rows(), 2);

    {
        let mut loc = session.locate_at_last_prompt_end();
        assert_eq!(
            loc,
            Some(SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Prompt(2)
            })
        );
        assert_eq!(loc.as_ref().unwrap().is_start_line(), false);
        let mut lines = 1;
        loc.as_mut().unwrap().dec_line(&mut lines);
        assert_eq!(lines, 0);
        assert_eq!(
            loc,
            Some(SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Prompt(1)
            })
        );
        assert_eq!(loc.as_ref().unwrap().is_start_line(), false);
        lines = 1;
        loc.as_mut().unwrap().dec_line(&mut lines);
        assert_eq!(lines, 0);
        assert_eq!(
            loc,
            Some(SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Prompt(0)
            })
        );
        assert_eq!(loc.as_ref().unwrap().is_start_line(), true);
        lines = 1;
        loc.as_mut().unwrap().dec_line(&mut lines);
        assert_eq!(lines, 1);
        assert_eq!(
            loc,
            Some(SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Prompt(0)
            })
        );
    }
}
