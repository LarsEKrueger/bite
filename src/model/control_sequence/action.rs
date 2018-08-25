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

//! Parsing result, action to be taken from seeing this sequence.

use std::char;

/// Actions to be taken after processing a byte
#[derive(PartialEq,Debug)]
pub enum Action {
    /// Send more input, no output yet
    More,

    /// An error occurred, state was reset
    Error,

    /// A carriage-return has been seen
    Cr,

    /// A new line character has been seen
    NewLine,

    /// A UTF8 character has been completed
    Char(char),

    /// An SGR sequence has been found.
    ///
    /// Process the parameters outside and then reset the state
    Sgr,

    DECREQTPARM,

    SaveCursor,
    RestoreCursor,

    HorizontalMove(isize),

    VerticalPos(isize),

    DA1(usize),

    WindowOps(u8, usize, usize),

    Show8BitControl(bool),

    AnsiConformanceLevel(u8),

    /// DECDHL (top half = true, bottom half = false)
    DecDoubleHeight(bool),

    /// DECSWL/DESDWL (single width = false)
    DecDoubleWidth(bool),

    DecAlignmentTest,

    /// Charset(level,CharSet)
    DesignateCharacterSet(u8, CharSet),

    DecBackIndex,
    DecForwardIndex,

    /// true = Application, false = normal
    DecApplicationKeypad(bool),

    CursorLowerLeft,

    FullReset,

    /// true = Lock Memory, false = Unlock Memory
    LockMemory(bool),

    /// (level, is_gr)
    /// level = 1 -> G1
    /// is_gr = true -> invoke as GR
    InvokeCharSet(u8, bool),

    ApplicationProgramCommand(String),
}

/// Character set
#[derive(PartialEq, Debug)]
pub enum CharSet {
    DefaultSet,
    Utf8,
    DecSpecial,
    DecSupplemental,
    DecSupplementalGraphics,
    DecTechnical,
    Uk,
    UsAscii,
    Dutch,
    Finnish,
    Finnish2,
    French,
    French2,
    FrenchCanadian,
    FrenchCanadian2,
    German,
    Italian,
    Norwegian,
    Norwegian2,
    Norwegian3,
    Portugese,
    Spanish,
    Swedish,
    Swedish2,
    Swiss,
}

#[derive(Debug, PartialEq)]
pub enum StringMode {
    None,
    Apc,
    Pm,
    Dcs,
}

impl Action {
    pub fn char_from_u32(byte: u32) -> Action {
        Action::Char(unsafe { char::from_u32_unchecked(byte as u32) })
    }
}

