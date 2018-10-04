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

//! Pseudo randon number generators
//!
//! Taken from https://en.wikipedia.org/wiki/Linear-feedback_shift_register

use num::{PrimInt, Unsigned};

/// Check if a given bit is set and return as 0/1
pub fn bit<T: PrimInt + Unsigned>(val: T, n: usize) -> T {
    (val >> (n - 1)) & T::one()
}

/// 8 bit LFSR, update state in place
pub fn rng8(state: &mut u8) {
    // x^8+x^6+x^5+x^4+1
    let poly = bit(*state, 8) ^ bit(*state, 6) ^ bit(*state, 5) ^ bit(*state, 4) ^ 1;
    let shifted = *state << 1;
    *state = shifted | poly;
}

/// 8 bit LFSR, limited to ASCII characters
pub fn rng8_char(state:&mut u8) -> char {
    // Wrap around
    let num_states = 128-32;

    while *state >= num_states {
        *state -= num_states;
    }
    let c = (32 + *state ) as char;
    rng8(state);
    c
}

/// 13 bit LFSR, update state in place
pub fn rng13(state:&mut u16) {
     // x^13 + x^12 + x^11 + x^8 + 1
    let poly = bit(*state, 13) ^ bit(*state, 12) ^ bit(*state, 11) ^ bit(*state, 8) ^ 1;
    let shifted = (*state << 1) & 0x1fff;
    *state = shifted | poly;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_rng8() {
        let mut state = 1;
        rng8(&mut state);
        assert_eq!(state, 3);
        rng8(&mut state);
        assert_eq!(state, 7);
        rng8(&mut state);
        assert_eq!(state, 15);
        rng8(&mut state);
        assert_eq!(state, 30);
    }

    #[test]
    fn basic_rng8_char() {
        let mut state = 1;
        assert_eq!(rng8_char(&mut state), 33 as char);
        assert_eq!(rng8_char(&mut state), 35 as char);
        assert_eq!(rng8_char(&mut state), 39 as char);
        assert_eq!(rng8_char(&mut state), 47 as char);
        assert_eq!(rng8_char(&mut state), 62 as char);
    }

    #[test]
    fn basic_rng13() {
        let mut state = 1;
        rng13(&mut state);
        assert_eq!(state, 3);
        rng13(&mut state);
        assert_eq!(state, 7);
        rng13(&mut state);
        assert_eq!(state, 15);
        rng13(&mut state);
        assert_eq!(state, 31);
        rng13(&mut state);
        assert_eq!(state, 63);
        rng13(&mut state);
        assert_eq!(state, 127);
        rng13(&mut state);
        assert_eq!(state, 255);
        rng13(&mut state);
        assert_eq!(state, 510);
    }
}
