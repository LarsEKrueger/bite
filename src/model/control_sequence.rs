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

//! Terminal Control Sequences Parser

use std::char;
use std::fmt;

/// Parser for control sequences
pub struct Parser {
    /// Incomplete code point being built
    code_point: u32,

    /// State of the state machine
    state: State,

    /// How many bytes are supposed to follow for code_point
    code_bytes: u8,

    /// First byte of an utf8 string
    first_byte: u8,
}

/// Actions to be taken after processing a byte
#[derive(PartialEq)]
pub enum Action {
    /// Send more input, no output yet
    More,

    /// An error occurred, state was reset
    Error,

    /// A carriage-return has been seen
    Cr,

    /// A line-feed has been seen
    Lf,

    /// A UTF8 character has been completed
    Char(char),
}

/// States of the machine
#[derive(PartialEq, Debug)]
#[repr(u8)]
enum State {
    /// Ready for the next byte
    Ready,

    /// Waiting for UTF8 byte1
    Byte1,

    /// Waiting for UTF8 byte2
    Byte2,

    /// Waiting for UTF8 byte3
    Byte3,
}


// Taken from core::str::mod.rs and std_unicode::lossy, see https://www.rust-lang.org/COPYRIGHT.
// Applies to the following sections between the markers "RUST CODE BEGIN" and "RUST CODE END".

// RUST CODE BEGIN

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
pub fn utf8_char_width(b: u8) -> u8 {
    return UTF8_CHAR_WIDTH[b as usize];
}

/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;
/// Value of the tag bits (tag mask is !CONT_MASK) of a continuation byte.
const TAG_CONT_U8: u8 = 0b1000_0000;

/// Returns the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

/// Returns the value of `ch` updated with continuation byte `byte`.
#[inline]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

/// Checks whether the byte is a UTF-8 continuation byte (i.e. starts with the
/// bits `10`).
#[inline]
fn utf8_is_cont_byte(byte: u8) -> bool {
    (byte & !CONT_MASK) == TAG_CONT_U8
}

// RUST CODE END

impl Parser {
    pub fn new() -> Self {
        Self {
            code_point: 0,
            state: State::Ready,
            code_bytes: 0,
            first_byte: 0,
        }
    }

    /// Process a single-byte character and check for potential escape sequences.
    fn single_byte(&mut self, byte: u8) -> Action {
        // TODO: handle escape sequences

        Action::char_from_u32(byte as u32)
    }

    /// Process a single byte from the input stream, convert from utf8 to chars on the fly.
    ///
    /// This function is the byte-by-byte version of core::str::next_code_point.
    pub fn add_byte(&mut self, byte: u8) -> Action {
        match self.state {
            State::Ready => {
                if byte < TAG_CONT_U8 {
                    return self.single_byte(byte);
                } else {
                    self.first_byte = byte;
                    self.code_bytes = self::utf8_char_width(byte);
                    if 2 <= self.code_bytes && self.code_bytes <= 4 {
                        self.code_point = self::utf8_first_byte(byte, self.code_bytes as u32);
                        self.state = State::Byte1;
                        return Action::More;
                    }
                }
            }

            State::Byte1 => {
                match self.code_bytes {
                    2 => {
                        if utf8_is_cont_byte(byte) {
                            let code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                            self.reset();
                            return Action::char_from_u32(code_point);
                        }
                    }

                    3 => {
                        // RUST CODE BEGIN
                        match (self.first_byte, byte) {
                            (0xE0, 0xA0...0xBF) |
                            (0xE1...0xEC, 0x80...0xBF) |
                            (0xED, 0x80...0x9F) |
                            (0xEE...0xEF, 0x80...0xBF) => {
                                // RUST CODE END
                                self.code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                                self.state = State::Byte2;
                                return Action::More;
                            }
                            _ => {}
                        }
                    }

                    4 => {
                        // RUST CODE BEGIN
                        match (self.first_byte, byte) {
                            (0xF0, 0x90...0xBF) |
                            (0xF1...0xF3, 0x80...0xBF) |
                            (0xF4, 0x80...0x8F) => {
                                // RUST CODE END
                                self.code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                                self.state = State::Byte2;
                                return Action::More;
                            }
                            _ => {}
                        }
                    }

                    _ => {
                        // We should never get here.
                        panic!("Internal error: code_bytes={} ", self.code_bytes);
                    }
                };
            }

            State::Byte2 => {
                match self.code_bytes {
                    3 => {
                        if utf8_is_cont_byte(byte) {
                            let code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                            self.state = State::Ready;
                            return Action::char_from_u32(code_point);
                        }
                    }

                    4 => {
                        if utf8_is_cont_byte(byte) {
                            self.code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                            self.state = State::Byte3;
                            return Action::More;
                        }
                    }

                    _ => {
                        // We should never get here.
                        panic!("Internal error: code_bytes={} ", self.code_bytes);
                    }
                }
            }

            State::Byte3 => {
                match self.code_bytes {
                    4 => {
                        if utf8_is_cont_byte(byte) {
                            let code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                            self.state = State::Ready;
                            return Action::char_from_u32(code_point);
                        }
                    }
                    _ => {
                        // We should never get here.
                        panic!("Internal error: code_bytes={} ", self.code_bytes);
                    }
                }

            }
        }
        self.reset();
        Action::Error
    }

    /// Reset to ready state
    fn reset(&mut self) {
        self.state = State::Ready;
        self.code_point = 0;
        self.code_bytes = 0;
    }
}

impl Action {
    fn char_from_u32(byte: u32) -> Action {
        Action::Char(unsafe { char::from_u32_unchecked(byte as u32) })
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::More => write!(f, "More"),
            Action::Error => write!(f, "Error"),
            Action::Cr => write!(f, "Cr"),
            Action::Lf => write!(f, "Lf"),
            Action::Char(c) => write!(f, "Char({})", *c as u32),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn emu(bytes: &[u8]) -> Vec<Action> {
        let mut e = Parser::new();
        let actions = bytes.iter().map(|b| e.add_byte(*b)).collect();
        assert_eq!(e.state, State::Ready);
        actions
    }

    fn emu2(blocks: &[&[u8]]) -> Vec<Action> {
        let mut e = Parser::new();
        let actions = blocks.iter().fold(Vec::new(), |mut v, bytes| {
            v.append(&mut bytes.iter().map(|b| e.add_byte(*b)).collect());
            v
        });
        assert_eq!(e.state, State::Ready);
        actions
    }

    fn c(ch: char) -> Action {
        Action::Char(ch)
    }

    fn m() -> Action {
        Action::More
    }

    fn e() -> Action {
        Action::Error
    }

    #[test]
    fn two_bytes() {
        assert_eq!(emu("\u{0080}".as_bytes()), [m(), c('\u{0080}')]);
        assert_eq!(emu("\u{07FF}".as_bytes()), [m(), c('\u{07FF}')]);
    }

    #[test]
    fn three_bytes() {
        assert_eq!(emu("\u{0800}".as_bytes()), [m(), m(), c('\u{0800}')]);
        assert_eq!(emu("\u{FFFF}".as_bytes()), [m(), m(), c('\u{FFFF}')]);
    }

    #[test]
    fn four_bytes() {
        assert_eq!(emu("\u{100CC}".as_bytes()), [m(), m(), m(), c('\u{100CC}')]);
        assert_eq!(
            emu2(&["\u{10000}".as_bytes(), "\u{10FFFF}".as_bytes()]),
            [
                m(),
                m(),
                m(),
                c('\u{10000}'),
                m(),
                m(),
                m(),
                c('\u{10FFFF}'),
            ]
        );
    }


    // Tests adapted from std_unicode/tests/lossy.rs.
    // RUST CODE BEGIN
    #[test]
    fn rust_tests() {
        assert_eq!(emu(b"hello"), [c('h'), c('e'), c('l'), c('l'), c('o')]);

        assert_eq!(
            emu("ศไทย中华Việt Nam".as_bytes()),
            [
                m(),
                m(),
                c('ศ'),
                m(),
                m(),
                c('ไ'),
                m(),
                m(),
                c('ท'),
                m(),
                m(),
                c('ย'),
                m(),
                m(),
                c('中'),
                m(),
                m(),
                c('华'),
                c('V'),
                c('i'),
                m(),
                m(),
                c('ệ'),
                c('t'),
                c(' '),
                c('N'),
                c('a'),
                c('m'),
            ]
        );

        assert_eq!(emu2(&["Hä".as_bytes()]), [c('H'), m(), c('ä')]);
        assert_eq!(
            emu2(
                &["Hä".as_bytes(), b"\xC2l", "ä".as_bytes(), b"\xC2e\xFFe"],
            ),
            [
                c('H'),
                m(),
                c('ä'),
                m(),
                e(),
                m(),
                c('ä'),
                m(),
                e(),
                e(),
                c('e'),
            ]
        );

        assert_eq!(
            emu(b"H\xC0\x80T\xE6\x83e"),
            [c('H'), e(), e(), c('T'), m(), m(), e()]
        );

        assert_eq!(emu(b"\xF5f\xF5\x80b"), [e(), c('f'), e(), e(), c('b')]);

        assert_eq!(
            emu(b"\xF1f\xF1\x80b\xF1\x80\x80ba"),
            [m(), e(), m(), m(), e(), m(), m(), m(), e(), c('a')]
        );

        assert_eq!(
            emu(b"\xF4f\xF4\x80b\xF4\xBFb"),
            [m(), e(), m(), m(), e(), m(), e(), c('b')]
        );

        assert_eq!(
            emu(b"\xF0\x80\x80\x80f\xF0\x90\x80\x80b"),
            [
                m(),
                e(),
                e(),
                e(),
                c('f'),
                m(),
                m(),
                m(),
                c('\u{10000}'),
                c('b'),
            ]
        );

        assert_eq!(
            emu(b"\xED\xA0\x80f\xED\xBF\xBFb"),
            [m(), e(), e(), c('f'), m(), e(), e(), c('b')]
        );
    }
    // RUST CODE END
}
