/*
    BiTE - Bash-integrated Terminal Parser
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

//! Control sequence parameters

use std::cmp;

/// Parameter of a control sequence.
///
/// Prepared for sub-parameters.
pub type Parameter = u32;

/// Maximal number of parameters
const NUM_PARAMETERS: usize = 30;

/// Magic number to indicate a default value.
///
/// As the value of parameter is clipped to 16 bit, we can use the maximum value of an u32 as magic
/// number.
const DEFAULT: Parameter = Parameter::max_value();

pub struct Parameters {
    /// Number of parameters used
    count: u8,

    /// Values of parameters
    values: [Parameter; NUM_PARAMETERS],
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            count: 0,
            values: [DEFAULT; NUM_PARAMETERS],
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn count(&self) -> usize {
        self.count as usize
    }

    pub fn has_subparams(&self) -> bool {
        // TODO: Add handling of sub parameters
        false
    }

    pub fn add_default(&mut self) {
        if (self.count as usize) < NUM_PARAMETERS {
            self.count += 1;
        }
        let cm = self.current_mut();
        *cm = DEFAULT;
    }

    pub fn current_mut(&mut self) -> &mut Parameter {
        debug_assert!(self.count != 0);
        &mut self.values[(self.count - 1) as usize]
    }

    /// Return an iterator on the parameters
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Parameter> + 'a {
        let c = self.count as usize;
        self.values[0..c].into_iter().map(|v| *v)
    }

    fn if_default(&self, param_index: u8, min_val: Parameter) -> Parameter {
        if param_index < self.count {
            let v = self.values[param_index as usize];
            if v == DEFAULT { min_val } else { v }
        } else {
            min_val
        }
    }

    pub fn zero_if_default(&self, param_index: u8) -> Parameter {
        self.if_default(param_index, 0)
    }

    pub fn clip8(&self, param_index: u8) -> u8 {
        cmp::min(255, self.zero_if_default(param_index)) as u8
    }

    pub fn one_if_default(&self, param_index: u8) -> Parameter {
        self.if_default(param_index, 1)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn add_digit(&mut self, byte: u8) {
        debug_assert!(b'0' <= byte && byte <= b'9');
        if self.is_empty() {
            self.add_default();
        }
        let cm = self.current_mut();
        let v = (byte - b'0') as u32;
        if *cm == DEFAULT {
            *cm = v;
        } else {
            *cm = cmp::min(65535, 10 * (*cm) + v);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut p = Parameters::new();
        assert_eq!(p.count(), 0);
        assert_eq!(p.is_empty(), true);
        p.add_default();
        assert_eq!(p.count(), 1);

        // Add one parameter more than we can handle
        for _i in 0..NUM_PARAMETERS {
            p.add_default();
        }
        assert_eq!(p.count(), NUM_PARAMETERS);
    }

    #[test]
    fn add_to_empty() {
        let mut p = Parameters::new();
        p.add_digit(b'5');
        assert_eq!(p.count(), 1);
        assert_eq!(p.zero_if_default(0), 5);
    }

    #[test]
    fn zero_vs_default() {
        let mut p = Parameters::new();
        p.add_default();
        assert_eq!(p.one_if_default(0), 1);
        p.add_digit(b'0');
        assert_eq!(p.one_if_default(0), 0);
    }

}
