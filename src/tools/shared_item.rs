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

//! Struct containing an ArcMutex of some struct
//!
//! Provides some helper methods to access the contained data

use std::sync::{Arc, Mutex};

/// Create the shared item
pub fn new<T>(item: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(item))
}

/// Quick write access to the underlying item
///
/// Does nothing if something goes wrong
pub fn item_mut<T, F, R>(shared_item: &mut Arc<Mutex<T>>, default: R, f: F) -> R
where
    F: FnOnce(&mut T) -> R,
{
    if let Ok(mut s) = shared_item.lock() {
        f(&mut s)
    } else {
        default
    }
}

/// Quick read access to the underlying item
///
/// Does nothing if something goes wrong
pub fn item<T, F, R>(shared_item: &Arc<Mutex<T>>, default: R, f: F) -> R
where
    F: FnOnce(&T) -> R,
{
    if let Ok(s) = shared_item.lock() {
        f(&s)
    } else {
        default
    }
}
