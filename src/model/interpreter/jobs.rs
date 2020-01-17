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

//! Manage a number of jobs

use super::super::job::Job;
use std::sync::{Arc, Condvar, Mutex};

use tools::shared_item;

struct Jobs {
    foreground: Option<Job>,
}

#[derive(Clone)]
pub struct SharedJobs(Arc<Mutex<Jobs>>);

impl Jobs {}

impl SharedJobs {
    pub fn new() -> Self {
        Self(shared_item::new(Jobs { foreground: None }))
    }

    fn jobs_mut<F, R>(&mut self, default: R, f: F) -> R
    where
        F: FnOnce(&mut Jobs) -> R,
    {
        shared_item::item_mut(&mut self.0, default, f)
    }

    fn jobs<F, R>(&self, default: R, f: F) -> R
    where
        F: FnOnce(&Jobs) -> R,
    {
        shared_item::item(&self.0, default, f)
    }

    pub fn has_foreground(&self) -> bool {
        self.jobs(false, |j| j.foreground.is_some())
    }

    /// Set the foreground job to the given job.
    ///
    /// TODO: Handle an already existing foreground job. Move to background?
    pub fn set_foreground(&mut self, job: Job) {
        self.jobs_mut((), |j| j.foreground = Some(job))
    }
}
