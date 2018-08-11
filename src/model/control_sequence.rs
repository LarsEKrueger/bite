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
use std::mem;
use std::iter;
use std::cmp;

use super::vt_parse_table::*;

/// Parser for control sequences
#[allow(dead_code)]
pub struct Parser {
    /// Incomplete code point being built
    code_point: u32,

    /// Number of bytes already processed
    code_byte: u8,

    /// How many bytes are supposed to follow for code_point
    code_bytes: u8,

    /// First byte of an utf8 string
    first_byte: u8,

    /// Index of last parameter or None if none has been set yet.
    last_parameter_index: Option<u8>,

    /// Parameters
    parameter: [Parameter; PARAMETERS],

    parsestate: &'static CaseTable,
    private_function: bool,
    lastchar: i32,
    nextstate: Case,

    print_area: String,

    string_mode: i32,
    string_area: String,
}

/// Maximal number of parameters
const PARAMETERS: usize = 30;

/// Parameter of a control sequence.
///
/// Prepared for sub-parameters.
type Parameter = u32;

/// Actions to be taken after processing a byte
#[derive(PartialEq)]
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
}

/// State machine cases for control sequence parser.
///
/// Taken from: $XTermId: VTparse.def,v 1.49 2014/04/25 21:36:12 tom Exp $
/// licensed as:
/// Copyright 1996-2013,2014 by Thomas E. Dickey
///
///                         All Rights Reserved
///
/// Permission is hereby granted, free of charge, to any person obtaining a
/// copy of this software and associated documentation files (the
/// "Software"), to deal in the Software without restriction, including
/// without limitation the rights to use, copy, modify, merge, publish,
/// distribute, sublicense, and/or sell copies of the Software, and to
/// permit persons to whom the Software is furnished to do so, subject to
/// the following conditions:
///
/// The above copyright notice and this permission notice shall be included
/// in all copies or substantial portions of the Software.
///
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
/// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
/// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
/// IN NO EVENT SHALL THE ABOVE LISTED COPYRIGHT HOLDER(S) BE LIABLE FOR ANY
/// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
/// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
/// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
///
/// Except as contained in this notice, the name(s) of the above copyright
/// holders shall not be used in advertising or otherwise to promote the
/// sale, use or other dealings in this Software without prior written
/// authorization.
#[allow(non_camel_case_types)]
#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Case {
    Illegal = 0,
    GROUND_STATE,
    IGNORE,
    BELL,
    BS,
    CR,
    ESC,
    VMOT,
    TAB,
    SI,
    SO,
    SCR_STATE,
    SCS0_STATE,
    SCS1_STATE,
    SCS2_STATE,
    SCS3_STATE,
    ESC_IGNORE,
    ESC_DIGIT,
    ESC_SEMI,
    DEC_STATE,
    ICH,
    CUU,
    CUD,
    CUF,
    CUB,
    CUP,
    ED,
    EL,
    IL,
    DL,
    DCH,
    DA1,
    TRACK_MOUSE,
    TBC,
    SET,
    RST,
    SGR,
    CPR,
    DECSTBM,
    DECREQTPARM,
    DECSET,
    DECRST,
    DECALN,
    GSETS,
    DECSC,
    DECRC,
    DECKPAM,
    DECKPNM,
    IND,
    NEL,
    HTS,
    RI,
    SS2,
    SS3,
    CSI_STATE,
    OSC,
    RIS,
    LS2,
    LS3,
    LS3R,
    LS2R,
    LS1R,
    PRINT,
    XTERM_SAVE,
    XTERM_RESTORE,
    XTERM_TITLE,
    DECID,
    HP_MEM_LOCK,
    HP_MEM_UNLOCK,
    HP_BUGGY_LL,
    HPA,
    VPA,
    XTERM_WINOPS,
    ECH,
    CHT,
    CPL,
    CNL,
    CBT,
    SU,
    SD,
    S7C1T,
    S8C1T,
    ESC_SP_STATE,
    ENQ,
    DECSCL,
    DECSCA,
    DECSED,
    DECSEL,
    DCS,
    PM,
    SOS,
    ST,
    APC,
    EPA,
    SPA,
    CSI_QUOTE_STATE,
    DSR,
    ANSI_LEVEL_1,
    ANSI_LEVEL_2,
    ANSI_LEVEL_3,
    MC,
    DEC2_STATE,
    DA2,
    DEC3_STATE,
    DECRPTUI,
    VT52_CUP,
    REP,
    CSI_EX_STATE,
    DECSTR,
    DECDHL,
    DECSWL,
    DECDWL,
    DEC_MC,
    ESC_PERCENT,
    UTF8,
    CSI_TICK_STATE,
    DECELR,
    DECRQLP,
    DECEFR,
    DECSLE,
    CSI_IGNORE,
    VT52_IGNORE,
    VT52_FINISH,
    CSI_DOLLAR_STATE,
    DECCRA,
    DECERA,
    DECFRA,
    DECSERA,
    DECSACE,
    DECCARA,
    DECRARA,
    CSI_STAR_STATE,
    SET_MOD_FKEYS,
    SET_MOD_FKEYS0,
    HIDE_POINTER,
    SCS1A_STATE,
    SCS2A_STATE,
    SCS3A_STATE,
    CSI_SPACE_STATE,
    DECSCUSR,
    SM_TITLE,
    RM_TITLE,
    DECSMBV,
    DECSWBV,
    DECLL,
    DECRQM,
    RQM,
    CSI_DEC_DOLLAR_STATE,
    SL,
    SR,
    DECDC,
    DECIC,
    DECBI,
    DECFI,
    DECRQCRA,
    HPR,
    VPR,
    ANSI_SC,
    ANSI_RC,
    ESC_COLON,
    SCS_PERCENT,
    GSETS_PERCENT,
    GRAPHICS_ATTRIBUTES,

    NUM_CASES,
}

pub type CaseTable = [Case; TAG_CONT_U8 as usize];

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

/// Highest byte value without TAG_CONT_U8
const TAG_CONT_U8_1: u8 = TAG_CONT_U8 - 1;

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
            code_byte: 0,
            code_bytes: 0,
            first_byte: 0,
            last_parameter_index: None,
            parameter: unsafe { mem::uninitialized() },

            parsestate: &ansi_table,
            private_function: false,
            lastchar: -1,
            nextstate: Case::Illegal,
            print_area: String::new(),

            string_mode: 0,
            string_area: String::new(),
        }
    }

    /// Process a single-byte character and check for potential escape sequences.
    fn single_byte(&mut self, byte: u8) -> Action {
        debug_assert!(byte < TAG_CONT_U8);
        self.nextstate = self.parsestate[byte as usize];

        if self.nextstate == Case::PRINT {
            return Action::char_from_u32(byte as u32);
        }

        // Accumulate string for APC, DCS, PM, OSC, SOS controls
        // This should always be 8-bit characters.
        if self.parsestate as *const CaseTable == &sos_table as *const CaseTable {
            self.string_area.push(unsafe {
                char::from_u32_unchecked(byte as u32)
            });
        } else if self.parsestate as *const CaseTable != &esc_table as *const CaseTable {
            /* if we were accumulating, we're not any more */
            self.string_mode = 0;
            self.string_area.clear();
        }

        // If the parameter list has subparameters (tokens separated by ":")
        // reject any controls that do not accept subparameters.
        if self.has_subparams() {
            match self.nextstate {
                Case::GROUND_STATE |
                Case::CSI_IGNORE |
                Case::ESC_DIGIT |
                Case::ESC_SEMI |
                Case::ESC_COLON => {
                    // these states are required to parse parameter lists
                }

                Case::SGR => {
                    // ...possible subparam usage
                }

                Case::CSI_DEC_DOLLAR_STATE |
                Case::CSI_DOLLAR_STATE |
                Case::CSI_EX_STATE |
                Case::CSI_QUOTE_STATE |
                Case::CSI_SPACE_STATE |
                Case::CSI_STAR_STATE |
                Case::CSI_TICK_STATE |
                Case::DEC2_STATE |
                Case::DEC3_STATE |
                Case::DEC_STATE => {
                    // use this branch when we do not yet have the final character
                    // ...unexpected subparam usage
                    self.last_parameter_index = None;
                    self.nextstate = Case::CSI_IGNORE;
                }

                _ => {
                    // use this branch for cases where we have the final character
                    // in the table that processed the parameter list.
                    // ... unexpected subparam usage
                    self.reset();

                    // We can safely call recursively because we go back to ground state.
                    return self.single_byte(byte);
                }
            }
        }

        // TODO: Handle repaintWhenPaletteChanged

        // Call the respective method
        dispatch_case[self.nextstate as usize](self, byte)
    }

    fn has_subparams(&self) -> bool {
        // TODO: Add handling of sub parameters
        false
    }

    fn init_params(&mut self) {
        self.last_parameter_index = None;
    }

    fn add_default_param(&mut self) {
        match self.last_parameter_index {
            None => {
                self.last_parameter_index = Some(0);
                self.parameter[0] = 0;
            }
            Some(ref mut n) => {
                *n += 1;
                self.parameter[*n as usize] = 0;
            }
        }
    }

    /// Process a single byte from the input stream, convert from utf8 to chars on the fly.
    ///
    /// This function is the byte-by-byte version of core::str::next_code_point.
    pub fn add_byte(&mut self, byte: u8) -> Action {
        match (self.code_byte, self.code_bytes, self.first_byte, byte) {
            (0, _, _, 0...TAG_CONT_U8_1) => return self.single_byte(byte),
            (0, _, _, _) => {
                self.first_byte = byte;
                self.code_bytes = self::utf8_char_width(byte);
                if 2 <= self.code_bytes && self.code_bytes <= 4 {
                    self.code_point = self::utf8_first_byte(byte, self.code_bytes as u32);
                    self.code_byte += 1;
                    return Action::More;
                }
            }

            // RUST CODE BEGIN
            (1, 3, 0xE0, 0xA0...0xBF) |
            (1, 3, 0xE1...0xEC, 0x80...0xBF) |
            (1, 3, 0xED, 0x80...0x9F) |
            (1, 3, 0xEE...0xEF, 0x80...0xBF) |
            (1, 4, 0xF0, 0x90...0xBF) |
            (1, 4, 0xF1...0xF3, 0x80...0xBF) |
            (1, 4, 0xF4, 0x80...0x8F) |
            // RUST CODE END
            (2, 4, _, _) => {
                if utf8_is_cont_byte(byte) {
                    self.code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                    self.code_byte += 1;
                    return Action::More;
                }
            }

            (1, 2, _, _) | (2, 3, _, _) | (3, 4, _, _) => {
                if utf8_is_cont_byte(byte) {
                    let code_point = self::utf8_acc_cont_byte(self.code_point, byte);
                    self.code_byte = 0;
                    return Action::char_from_u32(code_point);
                }
            }

            (_, _, _, _) => {}
        }
        self.reset();
        Action::Error
    }

    /// Reset to ready state
    pub fn reset(&mut self) {
        self.code_byte = 0;
        self.code_point = 0;
        self.code_bytes = 0;
        self.parsestate = &ansi_table;
    }

    /// Return an iterator on the parameters
    pub fn parameters<'a>(&'a self) -> Box<Iterator<Item = Parameter> + 'a> {
        match self.last_parameter_index {
            None => Box::new(iter::empty()),
            Some(last) => {
                let count = last + 1;
                Box::new((0..count).map(move |i| self.parameter[i as usize]))
            }
        }
    }

    fn param_at_least_if_default(&self, param_index: u8, min_val: Parameter) -> Parameter {
        match self.last_parameter_index {
            None => min_val,
            Some(last) => {
                if param_index <= last {
                    cmp::max(min_val, self.parameter[param_index as usize])
                } else {
                    min_val
                }
            }
        }
    }

    fn param_zero_if_default(&self, param_index: u8) -> Parameter {
        self.param_at_least_if_default(param_index, 0)
    }
    fn param_one_if_default(&self, param_index: u8) -> Parameter {
        self.param_at_least_if_default(param_index, 1)
    }

    fn action_Illegal(&mut self, _byte: u8) -> Action {
        panic!("This should not happen!");
    }

    fn action_GROUND_STATE(&mut self, _byte: u8) -> Action {
        self.reset();
        Action::More
    }
    fn action_IGNORE(&mut self, _byte: u8) -> Action {
        // Ignore this state
        Action::More
    }
    fn action_BELL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_BS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CR(&mut self, _byte: u8) -> Action {
        Action::Cr
    }
    fn action_ESC(&mut self, _byte: u8) -> Action {
        self.parsestate = &esc_table;
        Action::More
    }
    fn action_VMOT(&mut self, byte: u8) -> Action {
        match byte {
            b'\n' => Action::NewLine,
            _ => panic!("Unknown VMOT"),
        }
    }
    fn action_TAB(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SI(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SO(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCR_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS0_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS1_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS2_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS3_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ESC_IGNORE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ESC_DIGIT(&mut self, byte: u8) -> Action {
        if let Some(last) = self.last_parameter_index {
            let last = last as usize;
            self.parameter[last] =
                cmp::min(65535, 10 * self.parameter[last] + ((byte - b'0') as u32));
        }
        if self.parsestate as *const CaseTable == &csi_table as *const CaseTable {
            self.parsestate = &csi2_table;
        }
        Action::More
    }
    fn action_ESC_SEMI(&mut self, _byte: u8) -> Action {
        if let Some(last) = self.last_parameter_index {
            if (last as usize) < PARAMETERS {
                self.add_default_param();
            }
        }
        if self.parsestate as *const CaseTable == &csi_table as *const CaseTable {
            self.parsestate = &csi2_table;
        }
        Action::More
    }
    fn action_DEC_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ICH(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CUU(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CUD(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CUF(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CUB(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CUP(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ED(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_EL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_IL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DCH(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DA1(&mut self, _byte: u8) -> Action {
        let val = self.param_zero_if_default(0);
        self.reset();
        Action::DA1(val as usize)
    }
    fn action_TRACK_MOUSE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_TBC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SET(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_RST(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SGR(&mut self, _byte: u8) -> Action {
        Action::Sgr
    }
    fn action_CPR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSTBM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECREQTPARM(&mut self, _byte: u8) -> Action {
        self.reset();
        Action::DECREQTPARM
    }
    fn action_DECSET(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRST(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECALN(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_GSETS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECKPAM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECKPNM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_IND(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_NEL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HTS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_RI(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SS2(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SS3(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_STATE(&mut self, _byte: u8) -> Action {
        self.init_params();
        self.add_default_param();
        self.parsestate = &csi_table;
        Action::More
    }
    fn action_OSC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_RIS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_LS2(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_LS3(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_LS3R(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_LS2R(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_LS1R(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_PRINT(&mut self, _byte: u8) -> Action {
        panic!("This should not happen: Printable characters have no action.");
    }
    fn action_XTERM_SAVE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_XTERM_RESTORE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_XTERM_TITLE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECID(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HP_MEM_LOCK(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HP_MEM_UNLOCK(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HP_BUGGY_LL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HPA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_VPA(&mut self, _byte: u8) -> Action {
        let val = self.param_one_if_default(0) - 1;
        self.reset();
        Action::VerticalPos(val as isize)
    }
    fn action_XTERM_WINOPS(&mut self, _byte: u8) -> Action {
        let val = self.param_zero_if_default(0);
        let val1 = self.param_zero_if_default(1);
        let val2 = self.param_zero_if_default(2);
        self.reset();
        Action::WindowOps(cmp::max(val, 255) as u8, val1 as usize, val2 as usize)
    }
    fn action_ECH(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CHT(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CPL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CNL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CBT(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SU(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SD(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_S7C1T(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_S8C1T(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ESC_SP_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ENQ(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSCL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSCA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSED(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSEL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DCS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_PM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SOS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ST(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_APC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_EPA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SPA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_QUOTE_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DSR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ANSI_LEVEL_1(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ANSI_LEVEL_2(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ANSI_LEVEL_3(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_MC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DEC2_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DA2(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DEC3_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRPTUI(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_VT52_CUP(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_REP(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_EX_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSTR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECDHL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSWL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECDWL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DEC_MC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ESC_PERCENT(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_UTF8(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_TICK_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECELR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRQLP(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECEFR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSLE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_IGNORE(&mut self, _byte: u8) -> Action {
        self.parsestate = &cigtable;
        Action::More
    }
    fn action_VT52_IGNORE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_VT52_FINISH(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_DOLLAR_STATE(&mut self, _byte: u8) -> Action {
        self.parsestate = &csi_dollar_table;
        Action::More
    }
    fn action_DECCRA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECERA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECFRA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSERA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSACE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECCARA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRARA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_STAR_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SET_MOD_FKEYS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SET_MOD_FKEYS0(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HIDE_POINTER(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS1A_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS2A_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS3A_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_SPACE_STATE(&mut self, _byte: u8) -> Action {
        self.parsestate = &csi_sp_table;
        Action::More
    }
    fn action_DECSCUSR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SM_TITLE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_RM_TITLE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSMBV(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSWBV(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECLL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRQM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_RQM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_CSI_DEC_DOLLAR_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECDC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECIC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECBI(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECFI(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRQCRA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_HPR(&mut self, _byte: u8) -> Action {
        let col = self.param_one_if_default(0);
        self.reset();
        Action::HorizontalMove(col as isize)
    }
    fn action_VPR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ANSI_SC(&mut self, _byte: u8) -> Action {
        self.reset();
        Action::SaveCursor
    }
    fn action_ANSI_RC(&mut self, _byte: u8) -> Action {
        self.reset();
        Action::RestoreCursor
    }
    fn action_ESC_COLON(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SCS_PERCENT(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_GSETS_PERCENT(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_GRAPHICS_ATTRIBUTES(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
}

type CaseDispatch = fn(&mut Parser, byte: u8) -> Action;

static dispatch_case: [CaseDispatch; Case::NUM_CASES as usize] =
    [
        Parser::action_Illegal,
        Parser::action_GROUND_STATE,
        Parser::action_IGNORE,
        Parser::action_BELL,
        Parser::action_BS,
        Parser::action_CR,
        Parser::action_ESC,
        Parser::action_VMOT,
        Parser::action_TAB,
        Parser::action_SI,
        Parser::action_SO,
        Parser::action_SCR_STATE,
        Parser::action_SCS0_STATE,
        Parser::action_SCS1_STATE,
        Parser::action_SCS2_STATE,
        Parser::action_SCS3_STATE,
        Parser::action_ESC_IGNORE,
        Parser::action_ESC_DIGIT,
        Parser::action_ESC_SEMI,
        Parser::action_DEC_STATE,
        Parser::action_ICH,
        Parser::action_CUU,
        Parser::action_CUD,
        Parser::action_CUF,
        Parser::action_CUB,
        Parser::action_CUP,
        Parser::action_ED,
        Parser::action_EL,
        Parser::action_IL,
        Parser::action_DL,
        Parser::action_DCH,
        Parser::action_DA1,
        Parser::action_TRACK_MOUSE,
        Parser::action_TBC,
        Parser::action_SET,
        Parser::action_RST,
        Parser::action_SGR,
        Parser::action_CPR,
        Parser::action_DECSTBM,
        Parser::action_DECREQTPARM,
        Parser::action_DECSET,
        Parser::action_DECRST,
        Parser::action_DECALN,
        Parser::action_GSETS,
        Parser::action_DECSC,
        Parser::action_DECRC,
        Parser::action_DECKPAM,
        Parser::action_DECKPNM,
        Parser::action_IND,
        Parser::action_NEL,
        Parser::action_HTS,
        Parser::action_RI,
        Parser::action_SS2,
        Parser::action_SS3,
        Parser::action_CSI_STATE,
        Parser::action_OSC,
        Parser::action_RIS,
        Parser::action_LS2,
        Parser::action_LS3,
        Parser::action_LS3R,
        Parser::action_LS2R,
        Parser::action_LS1R,
        Parser::action_PRINT,
        Parser::action_XTERM_SAVE,
        Parser::action_XTERM_RESTORE,
        Parser::action_XTERM_TITLE,
        Parser::action_DECID,
        Parser::action_HP_MEM_LOCK,
        Parser::action_HP_MEM_UNLOCK,
        Parser::action_HP_BUGGY_LL,
        Parser::action_HPA,
        Parser::action_VPA,
        Parser::action_XTERM_WINOPS,
        Parser::action_ECH,
        Parser::action_CHT,
        Parser::action_CPL,
        Parser::action_CNL,
        Parser::action_CBT,
        Parser::action_SU,
        Parser::action_SD,
        Parser::action_S7C1T,
        Parser::action_S8C1T,
        Parser::action_ESC_SP_STATE,
        Parser::action_ENQ,
        Parser::action_DECSCL,
        Parser::action_DECSCA,
        Parser::action_DECSED,
        Parser::action_DECSEL,
        Parser::action_DCS,
        Parser::action_PM,
        Parser::action_SOS,
        Parser::action_ST,
        Parser::action_APC,
        Parser::action_EPA,
        Parser::action_SPA,
        Parser::action_CSI_QUOTE_STATE,
        Parser::action_DSR,
        Parser::action_ANSI_LEVEL_1,
        Parser::action_ANSI_LEVEL_2,
        Parser::action_ANSI_LEVEL_3,
        Parser::action_MC,
        Parser::action_DEC2_STATE,
        Parser::action_DA2,
        Parser::action_DEC3_STATE,
        Parser::action_DECRPTUI,
        Parser::action_VT52_CUP,
        Parser::action_REP,
        Parser::action_CSI_EX_STATE,
        Parser::action_DECSTR,
        Parser::action_DECDHL,
        Parser::action_DECSWL,
        Parser::action_DECDWL,
        Parser::action_DEC_MC,
        Parser::action_ESC_PERCENT,
        Parser::action_UTF8,
        Parser::action_CSI_TICK_STATE,
        Parser::action_DECELR,
        Parser::action_DECRQLP,
        Parser::action_DECEFR,
        Parser::action_DECSLE,
        Parser::action_CSI_IGNORE,
        Parser::action_VT52_IGNORE,
        Parser::action_VT52_FINISH,
        Parser::action_CSI_DOLLAR_STATE,
        Parser::action_DECCRA,
        Parser::action_DECERA,
        Parser::action_DECFRA,
        Parser::action_DECSERA,
        Parser::action_DECSACE,
        Parser::action_DECCARA,
        Parser::action_DECRARA,
        Parser::action_CSI_STAR_STATE,
        Parser::action_SET_MOD_FKEYS,
        Parser::action_SET_MOD_FKEYS0,
        Parser::action_HIDE_POINTER,
        Parser::action_SCS1A_STATE,
        Parser::action_SCS2A_STATE,
        Parser::action_SCS3A_STATE,
        Parser::action_CSI_SPACE_STATE,
        Parser::action_DECSCUSR,
        Parser::action_SM_TITLE,
        Parser::action_RM_TITLE,
        Parser::action_DECSMBV,
        Parser::action_DECSWBV,
        Parser::action_DECLL,
        Parser::action_DECRQM,
        Parser::action_RQM,
        Parser::action_CSI_DEC_DOLLAR_STATE,
        Parser::action_SL,
        Parser::action_SR,
        Parser::action_DECDC,
        Parser::action_DECIC,
        Parser::action_DECBI,
        Parser::action_DECFI,
        Parser::action_DECRQCRA,
        Parser::action_HPR,
        Parser::action_VPR,
        Parser::action_ANSI_SC,
        Parser::action_ANSI_RC,
        Parser::action_ESC_COLON,
        Parser::action_SCS_PERCENT,
        Parser::action_GSETS_PERCENT,
        Parser::action_GRAPHICS_ATTRIBUTES,
    ];

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
            Action::NewLine => write!(f, "NewLine"),
            Action::Sgr => write!(f, "Sgr"),
            Action::Char(c) => write!(f, "Char({})", *c as u32),
            Action::DECREQTPARM => write!(f, "DECREQTPARM"),
            Action::HorizontalMove(n) => write!(f, "HorizontalMove({})", n),
            Action::VerticalPos(n) => write!(f, "VerticalPos({})", n),
            Action::DA1(n) => write!(f, "DA1({})", n),
            Action::SaveCursor => write!(f, "SaveCursor"),
            Action::RestoreCursor => write!(f, "RestoreCursor"),
            Action::WindowOps(n0, n1, n2) => write!(f, "WindowOps({},{},{})", n0, n1, n2),

        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Helper function to map a string to the vector of actions that were returned after each byte
    fn emu(bytes: &[u8]) -> Vec<Action> {
        let mut e = Parser::new();
        let actions = bytes.iter().map(|b| e.add_byte(*b)).collect();
        assert_eq!(e.code_byte, 0);
        actions
    }

    /// Helper function to map a vector of strings to the vector of actions that were returned
    /// after each byte
    fn emu2(blocks: &[&[u8]]) -> Vec<Action> {
        let mut e = Parser::new();
        let actions = blocks.iter().fold(Vec::new(), |mut v, bytes| {
            v.append(&mut bytes.iter().map(|b| e.add_byte(*b)).collect());
            v
        });
        assert_eq!(e.code_byte, 0);
        actions
    }

    /// Helper function to map a string to the vector of actions and states that were returned
    /// after each byte
    fn emus(bytes: &[u8]) -> Vec<(Action, u8)> {
        let mut e = Parser::new();
        let mut res = Vec::new();
        for b in bytes {
            let a = e.add_byte(*b);
            let s = e.code_byte;
            res.push((a, s));
        }
        assert_eq!(e.code_byte, 0);
        res
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

    #[test]
    fn cr() {
        assert_eq!(emu(b"he\rwo"), [c('h'), c('e'), Action::Cr, c('w'), c('o')]);
    }

    #[test]
    fn decreqtparm() {
        assert_eq!(
            emu(b"a\x1b[0x\n"),
            [c('a'), m(), m(), m(), Action::DECREQTPARM, Action::NewLine]
        );
    }

    #[test]
    fn sgr() {
        assert_eq!(
            emu(b"a\x1b[32;12;0m"),
            [
                c('a'),
                m(),
                m(),
                m(),
                m(),
                m(),
                m(),
                m(),
                m(),
                m(),
                Action::Sgr,
            ]
        );

        // Non-SGR sequence (no escape)
        assert_eq!(emu(b"a[32m"), [c('a'), c('['), c('3'), c('2'), c('m')]);

        // Check parameter reset
        {
            let mut e = Parser::new();
            {
                let actions: Vec<Action> = b"\x1b[32;12m".iter().map(|b| e.add_byte(*b)).collect();
                assert_eq!(e.code_byte, 0);
                assert_eq!(actions, [m(), m(), m(), m(), m(), m(), m(), Action::Sgr]);
                assert_eq!(e.last_parameter_index, Some(1));
                assert_eq!(e.parameter[0], 32);
                assert_eq!(e.parameter[1], 12);

                let ps: Vec<Parameter> = e.parameters().collect();
                assert_eq!(ps, [32, 12]);
            }
            {
                let actions: Vec<Action> = b"\x1b[45m".iter().map(|b| e.add_byte(*b)).collect();
                assert_eq!(e.code_byte, 0);
                assert_eq!(actions, [m(), m(), m(), m(), Action::Sgr]);
                assert_eq!(e.last_parameter_index, Some(0));
                assert_eq!(e.parameter[0], 45);

                let ps: Vec<Parameter> = e.parameters().collect();
                assert_eq!(ps, [45]);
            }
        }
    }
}
