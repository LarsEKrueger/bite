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

//! Module tests for Presenter, mostly for the locator code

use std::borrow::Cow;

use model::screen::{Cell, Screen};
use model::session::test::new_test_session;
use model::session::OutputVisibility;
use presenter::{
    ConversationLocator, InteractionLocator, PresenterCommons, ResponseLocator, SessionLocator,
};

fn c2s(cells: Cow<[Cell]>) -> String {
    let mut s = String::new();
    for c in cells.iter().map(|c| c.code_point()) {
        s.push(c);
    }
    s
}

type GroundTruth = (SessionLocator, &'static str);

/// Test locator
///
/// Session:
/// [ 0, 15]     * command 1.1
/// [ 1, 14]         * output 1.1.1
/// [ 2, 13]         * output 1.1.2
/// [ 3, 12]     * command 1.2
/// [ 4, 11]         * output 1.2.1
/// [ 5, 10]         * output 1.2.2
/// [ 6,  9] * prompt 1
/// [ 7,  8]     * command 2.1
/// [ 8,  7]         * output 2.1.1
/// [ 9,  6]         * output 2.1.2
/// [10,  5]     * command 2.2
/// [11,  4]         * output 2.2.1
/// [12,  3]         * output 2.2.2
/// [13,  2] * prompt 2a
/// [14,  1]   prompt 2b
/// [--,  0]
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
        b"output 2.2.1\noutput 2.2.2",
    );

    let session = session.0.lock().unwrap();

    let loc = PresenterCommons::locate_end(&session);
    assert_eq!(
        loc,
        Some(SessionLocator {
            conversation: 1,
            in_conversation: ConversationLocator::Prompt(2)
        })
    );

    // Test the backwards iterator
    let bwd_gt: [GroundTruth; 15] = [
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Prompt(1),
            },
            "prompt 2b",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Prompt(0),
            },
            "prompt 2a",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Interaction(
                    1,
                    InteractionLocator::Response(ResponseLocator::Screen(0)),
                ),
            },
            "output 2.2.2",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Interaction(
                    1,
                    InteractionLocator::Response(ResponseLocator::Lines(0)),
                ),
            },
            "output 2.2.1",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Interaction(
                    1,
                    InteractionLocator::Command(0),
                ),
            },
            "command 2.2",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Interaction(
                    0,
                    InteractionLocator::Response(ResponseLocator::Lines(1)),
                ),
            },
            "output 2.1.2",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Interaction(
                    0,
                    InteractionLocator::Response(ResponseLocator::Lines(0)),
                ),
            },
            "output 2.1.1",
        ),
        (
            SessionLocator {
                conversation: 1,
                in_conversation: ConversationLocator::Interaction(
                    0,
                    InteractionLocator::Command(0),
                ),
            },
            "command 2.1",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Prompt(0),
            },
            "prompt 1",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Interaction(
                    1,
                    InteractionLocator::Response(ResponseLocator::Lines(1)),
                ),
            },
            "output 1.2.2",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Interaction(
                    1,
                    InteractionLocator::Response(ResponseLocator::Lines(0)),
                ),
            },
            "output 1.2.1",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Interaction(
                    1,
                    InteractionLocator::Command(0),
                ),
            },
            "command 1.2",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Interaction(
                    0,
                    InteractionLocator::Response(ResponseLocator::Lines(1)),
                ),
            },
            "output 1.1.2",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Interaction(
                    0,
                    InteractionLocator::Response(ResponseLocator::Lines(0)),
                ),
            },
            "output 1.1.1",
        ),
        (
            SessionLocator {
                conversation: 0,
                in_conversation: ConversationLocator::Interaction(
                    0,
                    InteractionLocator::Command(0),
                ),
            },
            "command 1.1",
        ),
    ];

    for i in 0..bwd_gt.len() {
        println!("  Locator working backwards, step {}", 1 + i);
        let loc = PresenterCommons::locate_up(&session, loc.as_ref().unwrap(), 1 + i);
        assert_eq!(loc, Some(bwd_gt[i].0.clone()));
        assert_eq!(
            c2s(session
                .display_line(loc.as_ref().unwrap())
                .expect("display_line should work")
                .text),
            bwd_gt[i].1
        );
    }

    // Test the forward iterator
    {
        let fwd_loc = PresenterCommons::locate_up(&session, loc.as_ref().unwrap(), bwd_gt.len())
            .expect("going to the start should have worked");
        assert_eq!(
            fwd_loc,
            bwd_gt
                .last()
                .map(|g| g.0.clone())
                .expect("there should be at least one entry in bwd_gt")
        );
        assert_eq!(
            c2s(session
                .display_line(&fwd_loc)
                .expect("display_line should work")
                .text),
            bwd_gt
                .last()
                .expect("there should be at least one entry in bwd_gt")
                .1
        );
        for i in 1..bwd_gt.len() {
            println!("  Locator working forward, step {}", i);
            let loc = PresenterCommons::locate_down(&session, &fwd_loc, i);
            assert_eq!(loc, Some(bwd_gt[bwd_gt.len() - 1 - i].0.clone()));
            assert_eq!(
                c2s(session
                    .display_line(loc.as_ref().unwrap())
                    .expect("display_line should work")
                    .text),
                bwd_gt[bwd_gt.len() - 1 - i].1
            );
        }
    }

    // Test going up more than required
    {
        let new_loc = PresenterCommons::locate_up(&session, loc.as_ref().unwrap(), 1000)
            .expect("going to the start should have worked");
        assert_eq!(
            new_loc,
            bwd_gt
                .last()
                .map(|g| g.0.clone())
                .expect("there should be at least one entry in bwd_gt")
        );
        assert_eq!(
            c2s(session
                .display_line(&new_loc)
                .expect("display_line should work")
                .text),
            bwd_gt
                .last()
                .expect("there should be at least one entry in bwd_gt")
                .1
        );
    }
}
