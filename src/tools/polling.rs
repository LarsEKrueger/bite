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

use std::time::Duration;
use std::thread::sleep;

pub struct Gate {
    had_event: bool,
    time_between_polls: Duration,
}

impl Gate {
    pub fn new(time_between_polls: Duration) -> Self {
        Self {
            had_event: true,
            time_between_polls,
        }
    }

    pub fn wait(&mut self) {
        if self.had_event {
            self.had_event = false;
        } else {
            sleep(self.time_between_polls);
        }
    }

    pub fn mark(&mut self) {
        self.had_event = true;
    }

    pub fn can_exit(&self) -> bool {
        !self.had_event
    }
}
