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

//! Provide a rate-limited polling mechanism.
//!
//! Assumes that if one event has been processed, another one will follow immediately. If no event
//! was obtained during that polling cycle, wait for a predetermined time to try again. This
//! ensures that events that come in quick succession can be processed at maximum speed (one per
//! cycle), but the CPU doesn't have to spin at 100% to do the polling.
//!
//! # Errors
//!
//! If the polling reads from a limited buffer and that fills up (e.g. due to a too long polling
//! cycle), the sender might block.

use std::thread::sleep;
use std::time::Duration;

/// State and parameters of the rate-limiting mechanism.
pub struct Gate {
    /// Did we get an event during the last cycle.
    had_event: bool,
    /// If now, how long shall we sleep until the next try.
    time_between_polls: Duration,
}

impl Gate {
    /// Allocate a gate with the giving polling rate
    pub fn new(time_between_polls: Duration) -> Self {
        Self {
            had_event: true,
            time_between_polls,
        }
    }

    /// Wait to test for the next event.
    ///
    /// If we have seen an event in the last cycle, we assume, there's another one. If not, sleep
    /// to free the CPU.
    pub fn wait(&mut self) {
        if self.had_event {
            self.had_event = false;
        } else {
            sleep(self.time_between_polls);
        }
    }

    /// Mark that we had an event in this cycle.
    pub fn mark(&mut self) {
        self.had_event = true;
    }

    /// Return true if we can exit the polling loop.
    ///
    /// If there was no event in the last iteration, we assume we read all pending ones.
    pub fn can_exit(&self) -> bool {
        !self.had_event
    }
}
