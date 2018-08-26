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
#[derive(PartialEq, Debug)]
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

    HorizontalMove(u32),

    VerticalPos(isize),

    DA1(u32),

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
    CursorUp(u32),
    CursorDown(u32),
    CursorForward(u32),
    CursorBackward(u32),
    CursorNextLine(u32),
    CursorPrevLine(u32),
    CursorAbsoluteColumn(u32),
    /// row, column
    CursorAbsolutePosition(u32, u32),
    CursorForwardTab(u32),

    /// Erase in display
    ///
    /// (direction,selective)
    EraseDisplay(EraseDisplay, bool),

    /// Erase in line
    ///
    /// (direction,selective)
    EraseLine(EraseLine, bool),

    FullReset,

    /// true = Lock Memory, false = Unlock Memory
    LockMemory(bool),

    /// (level, is_gr)
    /// level = 1 -> G1
    /// is_gr = true -> invoke as GR
    InvokeCharSet(u8, bool),

    ApplicationProgramCommand(String),

    /// This will currently catch all DCS command in the parameter.
    ///
    /// TODO: Implement string decoding
    DecUserDefinedKeys(String),

    InsertCharacters(u32),
    InsertLines(u32),

    DeleteLines(u32),
    DeleteCharacters(u32),

    ScrollUp(u32),
    ScrollDown(u32),

    GraphicRegister(GraReg, GraOp),

    /// The 5 parameters of the sequence
    MouseTracking(u32, u32, u32, u32, u32),

    ResetTitleModes(TitleModes),
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

#[derive(Debug, PartialEq)]
pub enum EraseDisplay {
    Below,
    Above,
    All,
    Saved,
}

#[derive(Debug, PartialEq)]
pub enum EraseLine {
    Left,
    Right,
    All,
}

/// Graphic register
#[derive(Debug, PartialEq)]
pub enum GraReg {
    Color,
    Sixel,
    Regis,
}

/// Operations on graphics registers
#[derive(Debug, PartialEq)]
pub enum GraOp {
    Read,
    Reset,
    Write(u32),
    GetMax,
}

bitflags! {
    pub struct TitleModes: u8 {
        const SetLabelHex  = 0b0001;
        const GetLabelHex  = 0b0010;
        const SetLabelUtf8 = 0b0100;
        const GetLabelUtf8 = 0b1000;

        const DEFAULT = 0;
        const ALL = Self::SetLabelHex.bits | Self::GetLabelHex.bits | Self::SetLabelUtf8.bits |
            Self::GetLabelUtf8.bits;
    }
}

impl Action {
    pub fn char_from_u32(byte: u32) -> Action {
        Action::Char(unsafe { char::from_u32_unchecked(byte as u32) })
    }
}
