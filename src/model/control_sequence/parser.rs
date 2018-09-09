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
use std::mem;

use super::vt_parse_table::*;
use super::types::{Case, CaseTable};
use super::action::{Action, CharSet, StringMode, EraseDisplay, EraseLine, GraReg, GraOp,
                    TitleModes, TabClear, SetMode, SetPrivateMode, MediaCopy, CharacterAttribute,
                    Color, FKeys, PointerMode, Terminal, LoadLeds, CursorStyle,
                    CharacterProtection, WindowOp, AttributeChangeExtent, LocatorReportEnable,
                    LocatorReportUnit};
use super::parameter::{Parameter, Parameters};

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

    /// Parameters
    parameter: Parameters,

    parsestate: &'static CaseTable,
    private_function: bool,
    lastchar: i32,
    nextstate: Case,

    scstype: u8,

    print_area: String,

    string_mode: StringMode,
    string_area: String,
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

mod action {
    use super::*;

    macro_rules! action_reset {
        ($name:ident,$action:ident) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action { p.reset(); Action::$action }
        };
        ($name:ident,$action:ident,zero) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                Action::$action( p.parameter.zero_if_default(0))
            }
        };
        ($name:ident,$action:ident,one) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                Action::$action( p.parameter.one_if_default(0))
            }
        };
        ($name:ident,$action:ident,one_minus) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                Action::$action( p.parameter.one_if_default(0)-1)
            }
        };
        ($name:ident,$action:ident,one_minus, one_minus) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                Action::$action( p.parameter.one_if_default(0)-1, p.parameter.one_if_default(1)-1)
            }
        };
        ($name:ident,$action:ident,$const:tt) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                Action::$action( $const)
            }
        };
        ($name:ident,$action:ident,$c1:tt,$c2:tt) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                Action::$action( $c1, $c2)
            }
        };
    }

    macro_rules! action_simple {
        ($i:ident,$a:ident) => {
            pub fn $i(_p:&mut Parser, _byte: u8) -> Action { Action::$a }
        }
    }

    macro_rules! action_state {
        ($i:ident,$a:ident) => {
            pub fn $i(p:&mut Parser, _byte: u8) -> Action {
                p.parsestate = &$a;
                Action::More
            }
        }
    }

    macro_rules! action_string {
        ($i:ident,$a:ident) => {
            pub fn $i(p:&mut Parser, _byte: u8) -> Action {
                p.string_mode = StringMode::$a;
                p.parsestate = &sos_table;
                Action::More
            }
        }
    }

    macro_rules! action_scs {
        ($name:ident,$table:ident, $const:tt) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.scstype = $const;
                p.parsestate = &$table;
                Action::More
            }
        }
    }

    macro_rules! action_switch_param {
        ($name:ident, [$($n:expr => $v:ident),+]) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                match p.parameter.zero_if_default(0) {
                    $($n => Action::$v),+
                    , _ => Action::More,
                }
            }
        };
        ($name:ident, $action:ident, [$($n:expr => $v:ident),+]) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                match p.parameter.zero_if_default(0) {
                    $($n => Action::$action($action::$v)),+
                    , _ => Action::More,
                }
            }
        };
        ($name:ident, $action:ident, [$($n:expr => ($v1:ident,$v2:expr)),+]) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                match p.parameter.zero_if_default(0) {
                    $($n => Action::$action($action::$v1,$v2)),+
                    , _ => Action::More,
                }
            }
        };
        ($name:ident, $action:ident, one, [$($n:expr => $v:ident),+]) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                match p.parameter.one_if_default(0) {
                    $($n => Action::$action($action::$v)),+
                    , _ => Action::More,
                }
            }
        };
        // Generate a helper function to only match the parameter
        ($name:ident, $action:ident, {$($n:expr => $v:ident),+}) => {
            pub fn $name(p0:Parameter) -> Option<$action> {
                match p0 {
                    $($n => Some($action::$v)),+,
                    _ =>  None,
                }
            }
        };
        ($name:ident, $action:ident, $helper:ident) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                match $helper(p.parameter.zero_if_default(0)) {
                    Some(v) => Action::$action(v),
                    None => Action::More,
                }
            }
        };
        ($name:ident, $action:ident, $v2:expr, [$($n:expr => $v:ident),+]) => {
            pub fn $name(p:&mut Parser, _byte: u8) -> Action {
                p.reset();
                match p.parameter.zero_if_default(0) {
                    $($n => Action::$action($action::$v, $v2)),+
                    , _ => Action::More,
                }
            }
        };
    }

    action_simple!(CR, Cr);
    action_simple!(IGNORE, More);

    action_reset!(ANSI_LEVEL_1, AnsiConformanceLevel, 1);
    action_reset!(ANSI_LEVEL_2, AnsiConformanceLevel, 2);
    action_reset!(ANSI_LEVEL_3, AnsiConformanceLevel, 3);
    action_reset!(ANSI_RC, RestoreCursor);
    action_reset!(CHT, CursorForwardTab, one);
    action_reset!(CNL, CursorNextLine, one);
    action_reset!(CPL, CursorPrevLine, one);
    action_reset!(CUB, CursorBackward, one);
    action_reset!(CUD, CursorDown, one);
    action_reset!(CUF, CursorForward, one);
    action_reset!(CUP, CursorAbsolutePosition, one_minus, one_minus);
    action_reset!(CUU, CursorUp, one);
    action_reset!(DA1, DA1, zero);
    action_reset!(DA2, DA2, zero);
    action_reset!(DECALN, DecAlignmentTest);
    action_reset!(DECBI, DecBackIndex);
    action_reset!(DECDWL, DecDoubleWidth, true);
    action_reset!(DECFI, DecForwardIndex);
    action_reset!(DECKPAM, DecApplicationKeypad, true);
    action_reset!(DECKPNM, DecApplicationKeypad, false);
    action_reset!(DECSWL, DecDoubleWidth, false);
    action_reset!(GROUND_STATE, More);
    action_reset!(HPA, CursorAbsoluteColumn, one_minus);
    action_reset!(HPR, HorizontalMove, one);
    action_reset!(HP_BUGGY_LL, CursorLowerLeft);
    action_reset!(HP_MEM_LOCK, LockMemory, true);
    action_reset!(HP_MEM_UNLOCK, LockMemory, false);
    action_reset!(ICH, InsertCharacters, one);
    action_reset!(LS1R, InvokeCharSet, 1, true);
    action_reset!(LS2, InvokeCharSet, 2, false);
    action_reset!(LS2R, InvokeCharSet, 2, true);
    action_reset!(LS3, InvokeCharSet, 3, false);
    action_reset!(LS3R, InvokeCharSet, 3, true);
    action_reset!(RIS, FullReset);
    action_reset!(S7C1T, Show8BitControl, false);
    action_reset!(S8C1T, Show8BitControl, true);
    action_reset!(IL, InsertLines, one);
    action_reset!(DL, DeleteLines, one);
    action_reset!(DCH, DeleteCharacters, one);
    action_reset!(SU, ScrollUp, one);
    action_reset!(ECH, EraseCharacters, one);
    action_reset!(CBT, CursorBackwardTab, one);
    action_reset!(SD, ScrollDown, one);
    action_reset!(REP, RepeatCharacter, one);
    action_reset!(VPA, VerticalPositionAbsolute, one_minus);
    action_reset!(VPR, VerticalPositionRelative, one_minus);
    action_reset!(DECSTR, SoftReset);

    action_scs!(SCS0_STATE, scstable, 0);
    action_scs!(SCS1A_STATE, scs96table, 1);
    action_scs!(SCS1_STATE, scstable, 1);
    action_scs!(SCS2A_STATE, scs96table, 2);
    action_scs!(SCS2_STATE, scstable, 2);
    action_scs!(SCS3A_STATE, scs96table, 3);
    action_scs!(SCS3_STATE, scstable, 3);

    action_state!(CSI_DOLLAR_STATE, csi_dollar_table);
    action_state!(CSI_IGNORE, cigtable);
    action_state!(CSI_SPACE_STATE, csi_sp_table);
    action_state!(DEC_STATE, dec_table);
    action_state!(ESC, esc_table);
    action_state!(ESC_IGNORE, eigtable);
    action_state!(ESC_PERCENT, esc_pct_table);
    action_state!(ESC_SP_STATE, esc_sp_table);
    action_state!(SCR_STATE, scrtable);
    action_state!(SCS_PERCENT, scs_pct_table);
    action_state!(DEC2_STATE, dec2_table);
    action_state!(CSI_EX_STATE, csi_ex_table);
    action_state!(CSI_QUOTE_STATE, csi_quo_table);
    action_state!(CSI_DEC_DOLLAR_STATE, csi_dec_dollar_table);
    action_state!(CSI_TICK_STATE, csi_tick_table);
    action_state!(CSI_STAR_STATE, csi_star_table);

    action_string!(APC, Apc);
    action_string!(DCS, Dcs);

    action_switch_param!(
        param_set_mode, SetMode,
        {2 => KeyboardAction, 4 => Insert, 12 => SendReceive, 20 => AutomaticNewline});
    action_switch_param!(SET, SetMode, param_set_mode);
    action_switch_param!(RESET, ResetMode, param_set_mode);

    action_switch_param!(TBC, TabClear, [ 0 => Column, 3 => All ]);
    action_switch_param!(ED,EraseDisplay,false, [0 => Below, 1 => Above, 2 => All, 3 => Saved]);
    action_switch_param!(DECSED,EraseDisplay,true, [0 => Below, 1 => Above, 2 => All, 3 => Saved]);
    action_switch_param!(EL,EraseLine,false,[ 0 => Right, 1 => Left, 2 => All]);
    action_switch_param!(DECSEL,EraseLine,true,[ 0 => Right, 1 => Left, 2 => All]);

    action_switch_param!(
        param_set_private_mode,SetPrivateMode,
        { 1 => ApplicationCursorKeys, 2 => UsAsciiForG0toG3,
        3 => Hundred32Columns, 4 => SmoothScroll, 5 => ReverseVideo, 6 => OriginMode,
        7 => AutoWrapMode, 8 => AutoRepeatKeys, 9 => SendMousePosOnPress, 10 => ShowToolbar,
        12 => StartBlinkingCursor, 13 => StartBlinkingCursor, 14 => EnableXorBlinkingCursor,
        18 => PrintFormFeed, 19 => PrintFullScreen, 25 => ShowCursor, 30 => ShowScrollbar,
        35 => EnableFontShifting, 38 => TektronixMode, 40 => AllowHundred32Mode, 41 => MoreFix,
        42 => EnableNrc, 44 => MarginBell, 45 => ReverseWrapAroundMode, 46 => StartLogging,
        47 => AlternateScreenBuffer, 66 => ApplicationKeypad, 67 => BackArrowIsBackSspace,
        69 => EnableLeftRightMarginMode, 95 => NoClearScreenOnDECCOLM,
        1000 => SendMousePosOnBoth, 1001 => HiliteMouseTracking, 1002 => CellMouseTracking,
        1003 => AllMouseTracking, 1004 => SendFocusEvents, 1005 => Utf8MouseMode,
        1006 => SgrMouseMode, 1007 => AlternateScrollMode, 1010 => ScrollToBottomOnTty,
        1011 => ScrollToBottomOnKey, 1015 => UrxvtMouseMode, 1034 => InterpretMetaKey,
        1035 => EnableSpecialModifiers, 1036 => SendEscOnMeta, 1037 => SendDelOnKeypad,
        1039 => SendEscOnAlt, 1040 => KeepSelection, 1041 => UseClipboard, 1042 => UrgencyHint,
        1043 => RaiseWindowOnBell, 1044 => KeepClipboard, 1046 => EnableAlternateScreen,
        1047 => UseAlternateScreen, 1048 => SaveCursor, 1049 => SaveCursorAndUseAlternateScreen,
        1050 => TerminfoFnMode, 1051 => SunFnMode, 1052 => HpFnMode, 1053 => ScoFnMode,
        1060 => LegacyKeyboard, 1061 => Vt220Keyboard, 2004 => BracketedPaste});
    action_switch_param!(DECSET,SetPrivateMode,param_set_private_mode);
    action_switch_param!(DECRESET,ResetPrivateMode,param_set_private_mode);
    action_switch_param!(XTERM_RESTORE,RestorePrivateMode,param_set_private_mode);
    action_switch_param!(XTERM_SAVE,SavePrivateMode,param_set_private_mode);
    action_switch_param!(MC,MediaCopy, [ 0 => PrintScreen, 4 => PrinterCtrlOff, 5 => PrinterCtrlOn,
                         10 => HtmlScreenDump, 11 => SvgScreenDump]);
    action_switch_param!(DECMC,MediaCopy, [ 1 => PrintCursorLine, 4 => AutoPrintOff,
                         5 => AutoPrintOn, 10 => PrintComposeDisplay, 11 => PrintAllPages]);
    action_switch_param!(CPR, [5=>StatusReport,6=>ReportCursorPosition]);

    action_switch_param!(HIDE_POINTER, PointerMode, one,
                         [0=>NeverHide,1=>HideNotTracking,2=>HideOutside,3=>AlwaysHide]);
    action_switch_param!(DECLL,LoadLeds,
                         [0=>(All,false),1=>(NumLock,false),2=>(CapsLock,false),
                         3=>(ScrollLock,false),21=>(NumLock,true),22=>(CapsLock,true),
                         23=>(ScrollLock,true)]);
    action_switch_param!(DECSCUSR,CursorStyle,
                         [0=>BlinkBlock,1=>BlinkBlock,2=>SteadyBlock,3=>BlinkUnderline,
                         4=>SteadyUnderline,5=>BlinkBar,6=>SteadyBar]);
    action_switch_param!(DECSCA,CharacterProtection,[0=>CanErase,1=>NoErase,2=>CanErase]);

    action_switch_param!(DECRQPSR,[1=>CursorInformationReport,2=>TabstopReport]);
    action_switch_param!(DECREQTPARM, [0=>RequestTerminalParameters,1=>RequestTerminalParameters]);
    action_switch_param!(DECSACE,AttributeChangeExtent,[0=>Wrapped,1=>Wrapped,2=>Rectangle]);
}

impl Parser {
    pub fn new() -> Self {
        Self {
            code_point: 0,
            code_byte: 0,
            code_bytes: 0,
            first_byte: 0,
            parameter: Parameters::new(),

            parsestate: &ansi_table,
            private_function: false,
            lastchar: -1,
            nextstate: Case::Illegal,
            print_area: String::new(),

            scstype: 0,

            string_mode: StringMode::None,
            string_area: String::new(),
        }
    }

    pub fn parameters<'a>(&'a self) -> impl Iterator<Item = Parameter> + 'a {
        self.parameter.iter()
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
        // TODO: Support utf-8 characters
        if self.parsestate as *const CaseTable == &sos_table as *const CaseTable {
            self.string_area.push(unsafe {
                char::from_u32_unchecked(byte as u32)
            });
        } else if self.parsestate as *const CaseTable != &esc_table as *const CaseTable {
            /* if we were accumulating, we're not any more */
            self.string_mode = StringMode::None;
            self.string_area.clear();
        }

        // If the parameter list has subparameters (tokens separated by ":")
        // reject any controls that do not accept subparameters.
        if self.parameter.has_subparams() {
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
                    self.parameter.reset();
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

    fn action_Illegal(&mut self, _byte: u8) -> Action {
        panic!("This should not happen!");
    }

    fn action_BELL(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_BS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
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
    fn action_ESC_DIGIT(&mut self, byte: u8) -> Action {
        self.parameter.add_digit(byte);
        if self.parsestate as *const CaseTable == &csi_table as *const CaseTable {
            self.parsestate = &csi2_table;
        }
        Action::More
    }
    fn action_ESC_SEMI(&mut self, _byte: u8) -> Action {
        self.parameter.add_default();
        if self.parsestate as *const CaseTable == &csi_table as *const CaseTable {
            self.parsestate = &csi2_table;
        }
        Action::More
    }

    fn action_TRACK_MOUSE(&mut self, _byte: u8) -> Action {
        self.reset();
        // One non-zero parameter is scroll down. Everything else is mouse tracking.
        let func = self.parameter.zero_if_default(0);
        if self.parameter.count() == 1 && func != 0 {
            Action::ScrollDown(func)
        } else {
            let ref p = self.parameter;
            Action::MouseTracking(
                func,
                p.zero_if_default(1),
                p.zero_if_default(2),
                p.zero_if_default(3),
                p.zero_if_default(4),
            )
        }
    }

    fn action_SGR(&mut self, _byte: u8) -> Action {
        self.reset();
        let p0 = self.parameter.zero_if_default(0);
        match p0 {
            38 | 48 => {
                let p1 = self.parameter.zero_if_default(1);
                match p1 {
                    2 => {
                        // Set RGB
                        match self.parameter.iter().count() {
                            5 => {
                                let r = self.parameter.clip8(2);
                                let g = self.parameter.clip8(3);
                                let b = self.parameter.clip8(4);
                                if p0 == 38 {
                                    Action::ForegroundColorRgb(r, g, b)
                                } else {
                                    Action::BackgroundColorRgb(r, g, b)
                                }
                            }
                            6 => {
                                let r = self.parameter.clip8(3);
                                let g = self.parameter.clip8(4);
                                let b = self.parameter.clip8(5);
                                if p0 == 38 {
                                    Action::ForegroundColorRgb(r, g, b)
                                } else {
                                    Action::BackgroundColorRgb(r, g, b)
                                }
                            }
                            _ => Action::More,
                        }
                    }
                    5 => {
                        // Set index
                        let p2 = self.parameter.clip8(2);
                        if p0 == 38 {
                            Action::ForegroundColorIndex(p2)
                        } else {
                            Action::BackgroundColorIndex(p2)
                        }
                    }
                    _ => Action::More,
                }
            }
            _ => {
                let attrs: Vec<CharacterAttribute> = self.parameters()
                    .filter_map(|attr| match attr {
                        0 => Some(CharacterAttribute::Normal),
                        1 => Some(CharacterAttribute::Bold),
                        2 => Some(CharacterAttribute::Faint),
                        3 => Some(CharacterAttribute::Italicized),
                        4 => Some(CharacterAttribute::Underlined),
                        5 => Some(CharacterAttribute::Blink),
                        7 => Some(CharacterAttribute::Inverse),
                        8 => Some(CharacterAttribute::Invisible),
                        9 => Some(CharacterAttribute::CrossedOut),
                        21 => Some(CharacterAttribute::DoublyUnderlined),
                        22 => Some(CharacterAttribute::NotBoldFaint),
                        23 => Some(CharacterAttribute::NotItalicized),
                        24 => Some(CharacterAttribute::NotUnderlined),
                        25 => Some(CharacterAttribute::Steady),
                        27 => Some(CharacterAttribute::Positive),
                        28 => Some(CharacterAttribute::Visible),
                        29 => Some(CharacterAttribute::NotCrossedOut),
                        30 => Some(CharacterAttribute::Foreground(Color::Black)),
                        31 => Some(CharacterAttribute::Foreground(Color::Red)),
                        32 => Some(CharacterAttribute::Foreground(Color::Green)),
                        33 => Some(CharacterAttribute::Foreground(Color::Yellow)),
                        34 => Some(CharacterAttribute::Foreground(Color::Blue)),
                        35 => Some(CharacterAttribute::Foreground(Color::Magenta)),
                        36 => Some(CharacterAttribute::Foreground(Color::Cyan)),
                        37 => Some(CharacterAttribute::Foreground(Color::White)),
                        39 => Some(CharacterAttribute::Foreground(Color::Default)),
                        40 => Some(CharacterAttribute::Background(Color::Black)),
                        41 => Some(CharacterAttribute::Background(Color::Red)),
                        42 => Some(CharacterAttribute::Background(Color::Green)),
                        43 => Some(CharacterAttribute::Background(Color::Yellow)),
                        44 => Some(CharacterAttribute::Background(Color::Blue)),
                        45 => Some(CharacterAttribute::Background(Color::Magenta)),
                        46 => Some(CharacterAttribute::Background(Color::Cyan)),
                        47 => Some(CharacterAttribute::Background(Color::White)),
                        49 => Some(CharacterAttribute::Background(Color::Default)),
                        90 => Some(CharacterAttribute::Foreground(Color::Grey)),
                        91 => Some(CharacterAttribute::Foreground(Color::BrightRed)),
                        92 => Some(CharacterAttribute::Foreground(Color::BrightGreen)),
                        93 => Some(CharacterAttribute::Foreground(Color::BrightYellow)),
                        94 => Some(CharacterAttribute::Foreground(Color::BrightBlue)),
                        95 => Some(CharacterAttribute::Foreground(Color::BrightMagenta)),
                        96 => Some(CharacterAttribute::Foreground(Color::BrightCyan)),
                        97 => Some(CharacterAttribute::Foreground(Color::BrightWhite)),
                        100 => Some(CharacterAttribute::Background(Color::Grey)),
                        101 => Some(CharacterAttribute::Background(Color::BrightRed)),
                        102 => Some(CharacterAttribute::Background(Color::BrightGreen)),
                        103 => Some(CharacterAttribute::Background(Color::BrightYellow)),
                        104 => Some(CharacterAttribute::Background(Color::BrightBlue)),
                        105 => Some(CharacterAttribute::Background(Color::BrightMagenta)),
                        106 => Some(CharacterAttribute::Background(Color::BrightCyan)),
                        107 => Some(CharacterAttribute::Background(Color::BrightWhite)),
                        _ => None,
                    })
                    .collect();
                if attrs.is_empty() {
                    Action::More
                } else {
                    Action::CharacterAttributes(attrs)
                }
            }
        }
    }

    fn action_DECSTBM(&mut self, _byte: u8) -> Action {
        self.reset();
        let p0 = self.parameter.zero_if_default(0);
        let p1 = self.parameter.zero_if_default(1);
        if p0 != 0 && p1 != 0 && p1 > p0 {
            Action::ScrollRegion(p0 - 1, p1 - 1)
        } else if p0 == 0 && p1 == 0 {
            Action::ScrollRegion(0, 0)
        } else {
            Action::More
        }
    }
    fn action_GSETS(&mut self, byte: u8) -> Action {
        let cs = match byte {
            b'B' => CharSet::UsAscii,
            b'A' => CharSet::Uk,
            b'0' => CharSet::DecSpecial,
            b'1' => CharSet::DecSupplemental,
            b'2' => CharSet::DecSupplementalGraphics,
            b'<' => CharSet::DecSupplemental,
            b'4' => CharSet::Dutch,
            b'5' => CharSet::Finnish,
            b'C' => CharSet::Finnish2,
            b'R' => CharSet::French,
            b'f' => CharSet::French2,
            b'Q' => CharSet::FrenchCanadian,
            b'K' => CharSet::German,
            b'Y' => CharSet::Italian,
            b'E' => CharSet::Norwegian2,
            b'6' => CharSet::Norwegian3,
            b'Z' => CharSet::Spanish,
            b'7' => CharSet::Swedish,
            b'H' => CharSet::Swedish2,
            b'=' => CharSet::Swiss,
            b'>' => CharSet::DecTechnical,
            b'9' => CharSet::FrenchCanadian2,
            b'`' => CharSet::Norwegian,
            _ => CharSet::DefaultSet,
        };
        self.reset();
        if cs != CharSet::DefaultSet {
            Action::DesignateCharacterSet(self.scstype, cs)
        } else {
            Action::More
        }
    }
    fn action_DECSC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECRC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_IND(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ANSI_SC(&mut self, _byte: u8) -> Action {
        self.reset();
        if self.parameter.is_empty() {
            Action::SaveCursor
        } else {
            let p0 = self.parameter.one_if_default(0);
            let p1 = self.parameter.one_if_default(1);
            if p0 != 0 && p1 != 0 && p0 < p1 {
                Action::SetMargins(p0 - 1, p1 - 1)
            } else {
                Action::More
            }
        }
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
        self.parameter.reset();
        self.parsestate = &csi_table;
        Action::More
    }
    fn action_OSC(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_PRINT(&mut self, _byte: u8) -> Action {
        panic!("This should not happen: Printable characters have no action.");
    }
    fn action_XTERM_TITLE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECID(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_XTERM_WINOPS(&mut self, _byte: u8) -> Action {
        self.reset();
        let p0 = self.parameter.zero_if_default(0);
        match p0 {
            0 => Action::More,
            1 => Action::WindowOp(WindowOp::DeIconify),
            2 => Action::WindowOp(WindowOp::Iconify),
            3 => Action::WindowOp(WindowOp::Move(
                self.parameter.zero_if_default(1),
                self.parameter.zero_if_default(2),
            )),
            4 => Action::WindowOp(WindowOp::ResizeWindow(
                self.parameter.maybe(1),
                self.parameter.maybe(2),
            )),
            5 => Action::WindowOp(WindowOp::Raise),
            6 => Action::WindowOp(WindowOp::Lower),
            7 => Action::WindowOp(WindowOp::Refresh),
            8 => Action::WindowOp(WindowOp::ResizeTextArea(
                self.parameter.maybe(1),
                self.parameter.maybe(2),
            )),
            9 => {
                match self.parameter.zero_if_default(1) {
                    0 => Action::WindowOp(WindowOp::RestoreMaximized),
                    1 => Action::WindowOp(WindowOp::MaximizeWindow),
                    2 => Action::WindowOp(WindowOp::MaximizeVertically),
                    3 => Action::WindowOp(WindowOp::MaximizeHorizontally),
                    _ => Action::More,
                }
            }
            10 => {
                match self.parameter.zero_if_default(1) {
                    0 => Action::WindowOp(WindowOp::UndoFullscreen),
                    1 => Action::WindowOp(WindowOp::Fullscreen),
                    2 => Action::WindowOp(WindowOp::ToggleFullscreen),
                    _ => Action::More,
                }
            }
            11 => Action::WindowOp(WindowOp::ReportWindowState),
            12 => Action::More,
            13 => {
                if self.parameter.count() == 1 {
                    Action::WindowOp(WindowOp::ReportWindowPosition)
                } else if self.parameter.zero_if_default(1) == 2 {
                    Action::WindowOp(WindowOp::ReportTextAreaPosition)
                } else {
                    Action::More
                }
            }
            14 => {
                if self.parameter.count() == 1 {
                    Action::WindowOp(WindowOp::ReportTextAreaSize)
                } else if self.parameter.zero_if_default(1) == 2 {
                    Action::WindowOp(WindowOp::ReportWindowSize)
                } else {
                    Action::More
                }
            }
            15 => Action::WindowOp(WindowOp::ReportScreenSize),
            16 => Action::WindowOp(WindowOp::ReportCharacterSize),
            17 => Action::More,
            18 => Action::WindowOp(WindowOp::ReportTextAreaSizeChar),
            19 => Action::WindowOp(WindowOp::ReportScreenSizeChar),
            20 => Action::WindowOp(WindowOp::ReportIconLabel),
            21 => Action::WindowOp(WindowOp::ReportWindowTitle),
            22 => {
                match self.parameter.zero_if_default(1) {
                    0 => Action::WindowOp(WindowOp::PushIconAndWindowTitle),
                    1 => Action::WindowOp(WindowOp::PushIconTitle),
                    2 => Action::WindowOp(WindowOp::PushWindowTitle),
                    _ => Action::More,
                }
            }
            23 => {
                match self.parameter.zero_if_default(1) {
                    0 => Action::WindowOp(WindowOp::PopIconAndWindowTitle),
                    1 => Action::WindowOp(WindowOp::PopIconTitle),
                    2 => Action::WindowOp(WindowOp::PopWindowTitle),
                    _ => Action::More,
                }
            }
            _ => Action::WindowOp(WindowOp::ResizeLines(p0)),
        }
    }
    fn action_ENQ(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSCL(&mut self, _byte: u8) -> Action {
        self.reset();
        let p0 = self.parameter.zero_if_default(0);
        let p1 = self.parameter.zero_if_default(1);
        match (p0, p1) {
            (61, _) => Action::ConformanceLevel(Terminal::Vt100, false),
            (62, 0) => Action::ConformanceLevel(Terminal::Vt200, true),
            (62, 1) => Action::ConformanceLevel(Terminal::Vt200, false),
            (62, 2) => Action::ConformanceLevel(Terminal::Vt200, true),
            (63, 0) => Action::ConformanceLevel(Terminal::Vt300, true),
            (63, 1) => Action::ConformanceLevel(Terminal::Vt300, false),
            (63, 2) => Action::ConformanceLevel(Terminal::Vt300, true),
            _ => Action::More,
        }
    }
    fn action_PM(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SOS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_ST(&mut self, _byte: u8) -> Action {
        self.reset();
        let res = match self.string_mode {
            StringMode::Apc => {
                let mut s = mem::replace(&mut self.string_area, String::new());
                let _ = s.pop();
                Action::ApplicationProgramCommand(s)
            }
            StringMode::Dcs => {
                let mut s = mem::replace(&mut self.string_area, String::new());
                let _ = s.pop();
                Action::DecUserDefinedKeys(s)
            }
            _ => {
                self.string_area.clear();
                Action::More
            }
        };
        res
    }
    fn action_EPA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_SPA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DSR(&mut self, _byte: u8) -> Action {
        self.reset();
        match self.parameter.zero_if_default(0) {
            6 => Action::DecDeviceStatusReport,
            15 => Action::PrinterStatusReport,
            25 => Action::UdkStatusReport,
            26 => Action::KeyboardStatusReport,
            53 => Action::LocatorStatusReport,
            55 => Action::LocatorStatusReport,
            56 => Action::LocatorTypeReport,
            62 => Action::MacroStatusReport,
            63 => Action::MemoryStatusReport(self.parameter.zero_if_default(1)),
            75 => Action::DataIntegrityReport,
            85 => Action::MultiSessionReport,
            _ => Action::More,
        }
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
    fn action_DECDHL(&mut self, byte: u8) -> Action {
        self.reset();
        Action::DecDoubleHeight(byte == b'3')
    }
    fn action_UTF8(&mut self, byte: u8) -> Action {
        self.reset();
        match byte {
            b'@' => Action::DesignateCharacterSet(0, CharSet::DefaultSet),
            b'G' => Action::DesignateCharacterSet(0, CharSet::Utf8),
            _ => Action::More,
        }
    }
    fn action_DECELR(&mut self, _byte: u8) -> Action {
        self.reset();
        let enable = match self.parameter.zero_if_default(0) {
            0 => Some(LocatorReportEnable::Off),
            1 => Some(LocatorReportEnable::On),
            2 => Some(LocatorReportEnable::Once),
            _ => None,
        };
        let unit = match self.parameter.zero_if_default(1) {
            0 => Some(LocatorReportUnit::Character),
            1 => Some(LocatorReportUnit::Device),
            2 => Some(LocatorReportUnit::Character),
            _ => None,
        };
        match (enable, unit) {
            (Some(e), Some(u)) => Action::LocatorReport(e, u),
            _ => Action::More,
        }
    }
    fn action_DECRQLP(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECEFR(&mut self, _byte: u8) -> Action {
        self.reset();
        let top = self.parameter.one_if_default(0);
        let left = self.parameter.one_if_default(1);
        let bottom = self.parameter.one_if_default(2);
        let right = self.parameter.one_if_default(3);
        if top < bottom && left < right {
            Action::EnableFilterArea(top, left, bottom, right)
        } else {
            Action::More
        }
    }
    fn action_DECSLE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_VT52_IGNORE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_VT52_FINISH(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECCRA(&mut self, _byte: u8) -> Action {
        self.reset();
        let top = self.parameter.one_if_default(0);
        let left = self.parameter.one_if_default(1);
        let bottom = self.parameter.one_if_default(2);
        let right = self.parameter.one_if_default(3);
        let from_page = self.parameter.one_if_default(4);
        let to_top = self.parameter.one_if_default(5);
        let to_left = self.parameter.one_if_default(6);
        let to_page = self.parameter.one_if_default(7);
        if top < bottom && left < right {
            Action::CopyArea(
                top,
                left,
                bottom,
                right,
                from_page,
                to_top,
                to_left,
                to_page,
            )
        } else {
            Action::More
        }
    }
    fn action_DECERA(&mut self, _byte: u8) -> Action {
        self.reset();
        let top = self.parameter.one_if_default(0);
        let left = self.parameter.one_if_default(1);
        let bottom = self.parameter.one_if_default(2);
        let right = self.parameter.one_if_default(3);
        if top < bottom && left < right {
            Action::EraseArea(top, left, bottom, right)
        } else {
            Action::More
        }
    }
    fn action_DECFRA(&mut self, _byte: u8) -> Action {
        self.reset();
        let c = self.parameter.zero_if_default(0);
        let top = self.parameter.one_if_default(1);
        let left = self.parameter.one_if_default(2);
        let bottom = self.parameter.one_if_default(3);
        let right = self.parameter.one_if_default(4);
        if top < bottom && left < right {
            Action::FillArea(c, top, left, bottom, right)
        } else {
            Action::More
        }
    }
    fn action_DECSERA(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECCARA(&mut self, _byte: u8) -> Action {
        self.reset();
        let top = self.parameter.one_if_default(0);
        let left = self.parameter.one_if_default(1);
        let bottom = self.parameter.one_if_default(2);
        let right = self.parameter.one_if_default(3);
        let attr = self.parameter.zero_if_default(4);
        if top < bottom && left < right {
            match attr {
                0 => {
                    Action::ChangeAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Normal,
                    )
                }
                1 => {
                    Action::ChangeAttributesArea(top, left, bottom, right, CharacterAttribute::Bold)
                }
                4 => {
                    Action::ChangeAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Underlined,
                    )
                }
                5 => {
                    Action::ChangeAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Blink,
                    )
                }
                7 => {
                    Action::ChangeAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Inverse,
                    )
                }
                _ => Action::More,
            }
        } else {
            Action::More
        }
    }
    fn action_DECRARA(&mut self, _byte: u8) -> Action {
        self.reset();
        let top = self.parameter.one_if_default(0);
        let left = self.parameter.one_if_default(1);
        let bottom = self.parameter.one_if_default(2);
        let right = self.parameter.one_if_default(3);
        let attr = self.parameter.zero_if_default(4);
        if top < bottom && left < right {
            match attr {
                1 => {
                    Action::ReverseAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Bold,
                    )
                }
                4 => {
                    Action::ReverseAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Underlined,
                    )
                }
                5 => {
                    Action::ReverseAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Blink,
                    )
                }
                7 => {
                    Action::ReverseAttributesArea(
                        top,
                        left,
                        bottom,
                        right,
                        CharacterAttribute::Inverse,
                    )
                }
                _ => Action::More,
            }
        } else {
            Action::More
        }
    }
    fn action_SET_MOD_FKEYS(&mut self, _byte: u8) -> Action {
        self.reset();
        let p1 = self.parameter.zero_if_default(1);
        match self.parameter.zero_if_default(0) {
            0 => Action::SetModFKeys(FKeys::Keyboard, p1),
            1 => Action::SetModFKeys(FKeys::Cursor, p1),
            2 => Action::SetModFKeys(FKeys::Function, p1),
            4 => Action::SetModFKeys(FKeys::Other, p1),
            _ => Action::More,
        }
    }
    fn action_SET_MOD_FKEYS0(&mut self, _byte: u8) -> Action {
        self.reset();
        if self.parameter.is_empty() {
            Action::DisableModFKeys(FKeys::Function)
        } else {
            match self.parameter.zero_if_default(0) {
                0 => Action::DisableModFKeys(FKeys::Keyboard),
                1 => Action::DisableModFKeys(FKeys::Cursor),
                2 => Action::DisableModFKeys(FKeys::Function),
                4 => Action::DisableModFKeys(FKeys::Other),
                _ => Action::More,
            }
        }
    }

    fn param_title_modes(&self) -> TitleModes {
        let mut tm = TitleModes::empty();
        for p in self.parameter.iter() {
            match p {
                0 => tm.insert(TitleModes::SetLabelHex),
                1 => tm.insert(TitleModes::GetLabelHex),
                2 => tm.insert(TitleModes::SetLabelUtf8),
                3 => tm.insert(TitleModes::GetLabelUtf8),
                _ => {}
            }
        }
        tm
    }

    fn action_SM_TITLE(&mut self, _byte: u8) -> Action {
        self.reset();
        Action::SetTitleModes(self.param_title_modes())
    }
    fn action_RM_TITLE(&mut self, _byte: u8) -> Action {
        self.reset();
        Action::ResetTitleModes(self.param_title_modes())
    }
    fn action_DECSMBV(&mut self, _byte: u8) -> Action {
        self.reset();
        let p0 = self.parameter.zero_if_default(0);
        match p0 {
            0...8 => Action::SetMarginBellVolume(p0 as u8),
            _ => Action::More,
        }
    }
    fn action_DECSWBV(&mut self, _byte: u8) -> Action {
        self.reset();
        let p0 = self.parameter.zero_if_default(0);
        match p0 {
            0...8 => Action::SetWarningBellVolume(p0 as u8),
            _ => Action::More,
        }
    }
    fn action_DECRQM(&mut self, _byte: u8) -> Action {
        self.reset();
        match action::param_set_private_mode(self.parameter.zero_if_default(0)) {
            Some(v) => Action::RequestPrivateMode(v),
            None => Action::RequestPrivateMode(SetPrivateMode::Unknown),
        }
    }
    fn action_RQM(&mut self, _byte: u8) -> Action {
        self.reset();
        match action::param_set_mode(self.parameter.zero_if_default(0)) {
            Some(v) => Action::RequestAnsiMode(v),
            None => Action::RequestAnsiMode(SetMode::Unknown),
        }
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
    fn action_DECRQCRA(&mut self, _byte: u8) -> Action {
        self.reset();
        let code = self.parameter.zero_if_default(0);
        let page = self.parameter.zero_if_default(1);
        let top = self.parameter.one_if_default(2);
        let left = self.parameter.one_if_default(3);
        let bottom = self.parameter.one_if_default(4);
        let right = self.parameter.one_if_default(5);
        if top < bottom && left < right {
            Action::ChecksumArea(code, page, top, left, bottom, right)
        } else {
            Action::More
        }
    }
    fn action_ESC_COLON(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_GSETS_PERCENT(&mut self, byte: u8) -> Action {
        let cs = match byte {
            b'5' => CharSet::DecSupplementalGraphics,
            b'6' => CharSet::Portugese,
            _ => CharSet::DefaultSet,
        };
        self.reset();
        if cs != CharSet::DefaultSet {
            Action::DesignateCharacterSet(self.scstype, cs)
        } else {
            Action::More
        }
    }
    fn action_GRAPHICS_ATTRIBUTES(&mut self, _byte: u8) -> Action {
        let register = match self.parameter.zero_if_default(0) {
            1 => Some(GraReg::Color),
            2 => Some(GraReg::Sixel),
            3 => Some(GraReg::Regis),
            _ => None,
        };
        let op = match self.parameter.zero_if_default(1) {
            1 => Some(GraOp::Read),
            2 => Some(GraOp::Reset),
            3 => Some(GraOp::Write(self.parameter.zero_if_default(2))),
            4 => Some(GraOp::GetMax),
            _ => None,
        };
        self.reset();
        match (register, op) {
            (Some(r), Some(o)) => Action::GraphicRegister(r, o),
            _ => Action::More,
        }
    }
    fn action_CSI_HASH_STATE(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_XTERM_PUSH_SGR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_XTERM_REPORT_SGR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_XTERM_POP_SGR(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSCPP(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
    fn action_DECSNLS(&mut self, _byte: u8) -> Action {
        panic!("Not implemented");
    }
}

type CaseDispatch = fn(&mut Parser, byte: u8) -> Action;

static dispatch_case: [CaseDispatch; Case::NUM_CASES as usize] = [
    Parser::action_Illegal,
    action::GROUND_STATE,
    action::IGNORE,
    Parser::action_BELL,
    Parser::action_BS,
    action::CR,
    action::ESC,
    Parser::action_VMOT,
    Parser::action_TAB,
    Parser::action_SI,
    Parser::action_SO,
    action::SCR_STATE,
    action::SCS0_STATE,
    action::SCS1_STATE,
    action::SCS2_STATE,
    action::SCS3_STATE,
    action::ESC_IGNORE,
    Parser::action_ESC_DIGIT,
    Parser::action_ESC_SEMI,
    action::DEC_STATE,
    action::ICH,
    action::CUU,
    action::CUD,
    action::CUF,
    action::CUB,
    action::CUP,
    action::ED,
    action::EL,
    action::IL,
    action::DL,
    action::DCH,
    action::DA1,
    Parser::action_TRACK_MOUSE,
    action::TBC,
    action::SET,
    action::RESET,
    Parser::action_SGR,
    action::CPR,
    Parser::action_DECSTBM,
    action::DECREQTPARM,
    action::DECSET,
    action::DECRESET,
    action::DECALN,
    Parser::action_GSETS,
    Parser::action_DECSC,
    Parser::action_DECRC,
    action::DECKPAM,
    action::DECKPNM,
    Parser::action_IND,
    Parser::action_NEL,
    Parser::action_HTS,
    Parser::action_RI,
    Parser::action_SS2,
    Parser::action_SS3,
    Parser::action_CSI_STATE,
    Parser::action_OSC,
    action::RIS,
    action::LS2,
    action::LS3,
    action::LS3R,
    action::LS2R,
    action::LS1R,
    Parser::action_PRINT,
    action::XTERM_SAVE,
    action::XTERM_RESTORE,
    Parser::action_XTERM_TITLE,
    Parser::action_DECID,
    action::HP_MEM_LOCK,
    action::HP_MEM_UNLOCK,
    action::HP_BUGGY_LL,
    action::HPA,
    action::VPA,
    Parser::action_XTERM_WINOPS,
    action::ECH,
    action::CHT,
    action::CPL,
    action::CNL,
    action::CBT,
    action::SU,
    action::SD,
    action::S7C1T,
    action::S8C1T,
    action::ESC_SP_STATE,
    Parser::action_ENQ,
    Parser::action_DECSCL,
    action::DECSCA,
    action::DECSED,
    action::DECSEL,
    action::DCS,
    Parser::action_PM,
    Parser::action_SOS,
    Parser::action_ST,
    action::APC,
    Parser::action_EPA,
    Parser::action_SPA,
    action::CSI_QUOTE_STATE,
    Parser::action_DSR,
    action::ANSI_LEVEL_1,
    action::ANSI_LEVEL_2,
    action::ANSI_LEVEL_3,
    action::MC,
    action::DEC2_STATE,
    action::DA2,
    Parser::action_DEC3_STATE,
    Parser::action_DECRPTUI,
    Parser::action_VT52_CUP,
    action::REP,
    action::CSI_EX_STATE,
    action::DECSTR,
    Parser::action_DECDHL,
    action::DECSWL,
    action::DECDWL,
    action::DECMC,
    action::ESC_PERCENT,
    Parser::action_UTF8,
    action::CSI_TICK_STATE,
    Parser::action_DECELR,
    Parser::action_DECRQLP,
    Parser::action_DECEFR,
    Parser::action_DECSLE,
    action::CSI_IGNORE,
    Parser::action_VT52_IGNORE,
    Parser::action_VT52_FINISH,
    action::CSI_DOLLAR_STATE,
    Parser::action_DECCRA,
    Parser::action_DECERA,
    Parser::action_DECFRA,
    Parser::action_DECSERA,
    action::DECSACE,
    Parser::action_DECCARA,
    Parser::action_DECRARA,
    action::CSI_STAR_STATE,
    Parser::action_SET_MOD_FKEYS,
    Parser::action_SET_MOD_FKEYS0,
    action::HIDE_POINTER,
    action::SCS1A_STATE,
    action::SCS2A_STATE,
    action::SCS3A_STATE,
    action::CSI_SPACE_STATE,
    action::DECSCUSR,
    Parser::action_SM_TITLE,
    Parser::action_RM_TITLE,
    Parser::action_DECSMBV,
    Parser::action_DECSWBV,
    action::DECLL,
    Parser::action_DECRQM,
    Parser::action_RQM,
    action::CSI_DEC_DOLLAR_STATE,
    Parser::action_SL,
    Parser::action_SR,
    Parser::action_DECDC,
    Parser::action_DECIC,
    action::DECBI,
    action::DECFI,
    Parser::action_DECRQCRA,
    action::HPR,
    action::VPR,
    Parser::action_ANSI_SC,
    action::ANSI_RC,
    Parser::action_ESC_COLON,
    action::SCS_PERCENT,
    Parser::action_GSETS_PERCENT,
    Parser::action_GRAPHICS_ATTRIBUTES,
    Parser::action_CSI_HASH_STATE,
    Parser::action_XTERM_PUSH_SGR,
    Parser::action_XTERM_REPORT_SGR,
    Parser::action_XTERM_POP_SGR,
    action::DECRQPSR,
    Parser::action_DECSCPP,
    Parser::action_DECSNLS,
];


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

    fn c(ch: char) -> Action {
        Action::Char(ch)
    }

    fn m() -> Action {
        Action::More
    }

    fn e() -> Action {
        Action::Error
    }

    macro_rules! pt {
        (@accu $str:tt, () -> ($($body:tt)*)) => {
            assert_eq!(
                emu($str),
                vec![$($body)*]);
        };
        (@accu $str:tt, (c $c:tt $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::Char($c),));
        };
        (@accu $str:tt, (m $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::More,));
        };
        (@accu $str:tt, (e $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::Error,));
        };
        (@accu $str:tt, (s $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::Char(' '),));
        };
        (@accu $str:tt, (DCS $n:tt $i:ident $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) ->
                ($($body)* Action::DesignateCharacterSet($n,CharSet::$i),))
        };
        (@accu $str:tt, ($i:ident ($v1:expr ) $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i($v1),));
        };
        (@accu $str:tt, ($i:ident ($v1:expr, $v2:expr) $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i($v1,$v2),));
        };
        (@accu $str:tt, ($i:ident ($v1:expr, $v2:expr, $v3:expr) $($rest:tt)*) ->
         ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i($v1,$v2,$v3),));
        };
        (@accu $str:tt, ($i:ident ($v1:expr, $v2:expr, $v3:expr, $v4:expr)
                         $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i($v1,$v2,$v3,$v4),));
        };
        (@accu $str:tt, ($i:ident ($v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr)
                         $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i($v1,$v2,$v3,$v4,$v5),));
        };
        (@accu $str:tt, ($i:ident ($v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr)
                         $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i($v1,$v2,$v3,$v4,$v5,$v6),));
        };
        (@accu $str:tt, ($i:ident ($v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr,
                                   $v7:expr, $v8:expr) $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)*
                                            Action::$i($v1,$v2,$v3,$v4,$v5,$v6,$v7,$v8),));
        };
        (@accu $str:tt, ($i:ident $($rest:tt)*) -> ($($body:tt)*)) => {
            pt!(@accu $str, ($($rest)*) -> ($($body)* Action::$i,))
        };
        ($str:expr, $($rest:tt)+ ) => {pt!(@accu $str, ($($rest)*) -> ());};
    }


    #[test]
    fn unicode() {
        pt!["\u{0080}".as_bytes(), m c '\u{0080}'];
        pt!["\u{07FF}".as_bytes(), m c '\u{07FF}'];

        pt!("\u{0800}".as_bytes(), m m c '\u{0800}');
        pt!("\u{FFFF}".as_bytes(), m m c '\u{FFFF}');

        pt!("\u{100CC}".as_bytes(), m m m c'\u{100CC}');

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
        pt!(b"hello", c'h' c'e' c'l' c'l' c'o');
        pt!("ศไทย中华Việt Nam".as_bytes(),
        m  m  c 'ศ' m  m  c 'ไ' m  m  c 'ท' m  m  c 'ย' m  m  c '中'
        m  m  c '华' c 'V' c 'i' m  m  c 'ệ' c 't' c ' ' c 'N' c 'a' c 'm');
        pt!("Hä".as_bytes(), c'H' m c'ä');

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

        pt!(b"H\xC0\x80T\xE6\x83e", c'H' e e c'T' m m e);
        pt!(b"\xF5f\xF5\x80b", e c'f' e e c'b');
        pt!(b"\xF1f\xF1\x80b\xF1\x80\x80ba", m e m m e m m m e c'a');
        pt!(b"\xF4f\xF4\x80b\xF4\xBFb", m e m m e m e c'b');
        pt!(b"\xF0\x80\x80\x80f\xF0\x90\x80\x80b", m e e e c'f' m m m c'\u{10000}' c'b');
        pt!(b"\xED\xA0\x80f\xED\xBF\xBFb", m e e c'f' m e e c'b');
    }
    // RUST CODE END

    #[test]
    fn character_sets() {
        pt!(b"\x1b(f", m m DCS 0 French2);

        pt!(b"\x1b(0 \x1b(< \x1b(%5 \x1b(> \x1b(A \x1b(B \x1b(4 \x1b(C \x1b(5 \x1b(R \
            \x1b(f \x1b(Q \x1b(9 \x1b(K \x1b(Y \x1b(` \x1b(E \x1b(6 \x1b(%6 \x1b(Z \
            \x1b(H \x1b(7 \x1b(=",
            m m DCS 0 DecSpecial s m m DCS 0 DecSupplemental s m m m DCS 0 DecSupplementalGraphics
            s m m DCS 0 DecTechnical s m m DCS 0 Uk s m m DCS 0 UsAscii s m m DCS 0 Dutch s m m DCS
            0 Finnish2 s m m DCS 0 Finnish s m m DCS 0 French s m m DCS 0 French2 s m m DCS 0
            FrenchCanadian s m m DCS 0 FrenchCanadian2 s m m DCS 0 German s m m DCS 0 Italian s m m
            DCS 0 Norwegian s m m DCS 0 Norwegian2 s m m DCS 0 Norwegian3 s m m m DCS 0 Portugese s
            m m DCS 0 Spanish s m m DCS 0 Swedish2 s m m DCS 0 Swedish s m m DCS 0 Swiss);

        pt!( b"\x1b)0 \x1b)< \x1b)%5 \x1b)> \x1b)A \x1b)B \x1b)4 \x1b)C \x1b)5 \x1b)R \
            \x1b)f \x1b)Q \x1b)9 \x1b)K \x1b)Y \x1b)` \x1b)E \x1b)6 \x1b)%6 \x1b)Z \
            \x1b)H \x1b)7 \x1b)=",
            m m DCS 1 DecSpecial s m m DCS 1 DecSupplemental s m m m DCS 1 DecSupplementalGraphics
            s m m DCS 1 DecTechnical s m m DCS 1 Uk s m m DCS 1 UsAscii s m m DCS 1 Dutch s m m DCS
            1 Finnish2 s m m DCS 1 Finnish s m m DCS 1 French s m m DCS 1 French2 s m m DCS 1
            FrenchCanadian s m m DCS 1 FrenchCanadian2 s m m DCS 1 German s m m DCS 1 Italian s m m
            DCS 1 Norwegian s m m DCS 1 Norwegian2 s m m DCS 1 Norwegian3 s m m m DCS 1 Portugese s
            m m DCS 1 Spanish s m m DCS 1 Swedish2 s m m DCS 1 Swedish s m m DCS 1 Swiss);

        pt!( b"\x1b*0 \x1b*< \x1b*%5 \x1b*> \x1b*A \x1b*B \x1b*4 \x1b*C \x1b*5 \x1b*R \
            \x1b*f \x1b*Q \x1b*9 \x1b*K \x1b*Y \x1b*` \x1b*E \x1b*6 \x1b*%6 \x1b*Z \
            \x1b*H \x1b*7 \x1b*=",
            m m DCS 2 DecSpecial s m m DCS 2 DecSupplemental s m m m DCS 2 DecSupplementalGraphics
            s m m DCS 2 DecTechnical s m m DCS 2 Uk s m m DCS 2 UsAscii s m m DCS 2 Dutch s m m DCS
            2 Finnish2 s m m DCS 2 Finnish s m m DCS 2 French s m m DCS 2 French2 s m m DCS 2
            FrenchCanadian s m m DCS 2 FrenchCanadian2 s m m DCS 2 German s m m DCS 2 Italian s m m
            DCS 2 Norwegian s m m DCS 2 Norwegian2 s m m DCS 2 Norwegian3 s m m m DCS 2 Portugese s
            m m DCS 2 Spanish s m m DCS 2 Swedish2 s m m DCS 2 Swedish s m m DCS 2 Swiss);

        pt!( b"\x1b+0 \x1b+< \x1b+%5 \x1b+> \x1b+A \x1b+B \x1b+4 \x1b+C \x1b+5 \x1b+R \
            \x1b+f \x1b+Q \x1b+9 \x1b+K \x1b+Y \x1b+` \x1b+E \x1b+6 \x1b+%6 \x1b+Z \
            \x1b+H \x1b+7 \x1b+=",
            m m DCS 3 DecSpecial s m m DCS 3 DecSupplemental s m m m DCS 3 DecSupplementalGraphics
            s m m DCS 3 DecTechnical s m m DCS 3 Uk s m m DCS 3 UsAscii s m m DCS 3 Dutch s m m DCS
            3 Finnish2 s m m DCS 3 Finnish s m m DCS 3 French s m m DCS 3 French2 s m m DCS 3
            FrenchCanadian s m m DCS 3 FrenchCanadian2 s m m DCS 3 German s m m DCS 3 Italian s m m
            DCS 3 Norwegian s m m DCS 3 Norwegian2 s m m DCS 3 Norwegian3 s m m m DCS 3 Portugese s
            m m DCS 3 Spanish s m m DCS 3 Swedish2 s m m DCS 3 Swedish s m m DCS 3 Swiss);

        // For the next three block, the specification of XTerm 335 is misleading. We test for
        // identical implementation with XTerm.
        pt!(b"\x1b-0 \x1b-< \x1b-%5 \x1b-> \x1b-A \x1b-B \x1b-4 \x1b-C \x1b-5 \x1b-R \
            \x1b-f \x1b-Q \x1b-9 \x1b-K \x1b-Y \x1b-` \x1b-E \x1b-6 \x1b-%6 \x1b-Z \
            \x1b-H \x1b-7 \x1b-=",
            m m m s m m m s m m m m s m m m s m m DCS 1 Uk s m m m s m m m s m m m s m m m s m m m
            s m m m s m m m s m m m s m m m s m m m s m m m s m m m s m m m s m m m m s m m m s m m
            m s m m m s m m m);

        pt!(b"\x1b.0 \x1b.< \x1b.%5 \x1b.> \x1b.A \x1b.B \x1b.4 \x1b.C \x1b.5 \x1b.R \
            \x1b.f \x1b.Q \x1b.9 \x1b.K \x1b.Y \x1b.` \x1b.E \x1b.6 \x1b.%6 \x1b.Z \
            \x1b.H \x1b.7 \x1b.=",
            m m m s m m m s m m m m s m m m s m m DCS 2 Uk s m m m s m m m s m m m s m m m s m m m
            s m m m s m m m s m m m s m m m s m m m s m m m s m m m s m m m s m m m m s m m m s m m
            m s m m m s m m m);

        pt!(b"\x1b/0 \x1b/< \x1b/%5 \x1b/> \x1b/A \x1b/B \x1b/4 \x1b/C \x1b/5 \x1b/R \
            \x1b/f \x1b/Q \x1b/9 \x1b/K \x1b/Y \x1b/` \x1b/E \x1b/6 \x1b/%6 \x1b/Z \
            \x1b/H \x1b/7 \x1b/=",
            m m m s m m m s m m m m s m m m s m m DCS 3 Uk s m m m s m m m s m m m s m m m s m m m
            s m m m s m m m s m m m s m m m s m m m s m m m s m m m s m m m s m m m m s m m m s m m
            m s m m m s m m m);

        pt!(b"\x1b%@\x1b%G", m m DCS 0 DefaultSet m m DCS 0 Utf8);
    }

    #[test]
    fn actions() {
        pt!(b"he\rwo", c'h' c'e' Cr c'w' c'o');
        pt!(b"a\nx", c'a' NewLine c'x');

        // Non-SGR sequence (no escape)
        pt!(b"a[32m", c'a' c'[' c'3' c'2' c'm');

        // Check parameter reset
        {
            let mut e = Parser::new();
            {
                let actions: Vec<Action> = b"\x1b[32;12m".iter().map(|b| e.add_byte(*b)).collect();
                assert_eq!(e.code_byte, 0);
                assert_eq!(
                    actions,
                    [
                        m(),
                        m(),
                        m(),
                        m(),
                        m(),
                        m(),
                        m(),
                        Action::CharacterAttributes(vec![CharacterAttribute::Foreground(Color::Green)]),
                    ]
                );
                assert_eq!(e.parameter.count(), (2));
                assert_eq!(e.parameter.zero_if_default(0), 32);
                assert_eq!(e.parameter.zero_if_default(1), 12);

                let ps: Vec<Parameter> = e.parameters().collect();
                assert_eq!(ps, [32, 12]);
            }
            {
                let actions: Vec<Action> = b"\x1b[45m".iter().map(|b| e.add_byte(*b)).collect();
                assert_eq!(e.code_byte, 0);
                assert_eq!(
                    actions,
                    [
                        m(),
                        m(),
                        m(),
                        m(),
                        Action::CharacterAttributes(vec![CharacterAttribute::Background(Color::Magenta)]),
                    ]
                );
                assert_eq!(e.parameter.count(), (1));
                assert_eq!(e.parameter.zero_if_default(0), 45);

                let ps: Vec<Parameter> = e.parameters().collect();
                assert_eq!(ps, [45]);
            }
        }

        pt!(b"a\x1b Fy", c'a' m m Show8BitControl(false) c'y');
        pt!(b"a\x1b Gy", c'a' m m Show8BitControl(true) c'y');
        pt!(b"a\x1b Ly\x1b M\x1b Nz",
            c'a' m m AnsiConformanceLevel(1) c'y' m m AnsiConformanceLevel(2)
            m m AnsiConformanceLevel(3) c'z');
        pt!(b"a\x1b#3\x1b#4\x1b#5\x1b#6\x1b#8z",
            c'a' m m DecDoubleHeight(true) m m DecDoubleHeight(false) m m DecDoubleWidth(false)
            m m DecDoubleWidth(true) m m DecAlignmentTest c'z');
        pt!(b"a\x1b[12@b", c 'a' m m m m InsertCharacters(12) c 'b');

        pt!(b"a\x1b6b\x1b9c", c'a' m DecBackIndex c'b' m DecForwardIndex c'c');
        pt!(b"a\x1b=b\x1b>c",
            c'a' m DecApplicationKeypad(true) c'b'
            m DecApplicationKeypad(false) c'c');
        pt!(b"a\x1bFc", c'a' m CursorLowerLeft c'c');
        pt!(b"a\x1bcc", c'a' m FullReset c'c');
        pt!(b"a\x1blb\x1bmc", c'a' m LockMemory(true) c'b' m LockMemory(false) c'c');
        pt!(b"a\x1bn\x1bo\x1b|\x1b}\x1b~b", c'a' m InvokeCharSet(2, false) m
            InvokeCharSet(3, false) m InvokeCharSet(3, true) m InvokeCharSet(2, true) m
            InvokeCharSet(1, true) c'b');
        pt!(b"a\x1b_stuff\x1b\\b", c'a' m m m m m m m m
            ApplicationProgramCommand("stuff".to_string()) c'b');
        pt!(b"a\x1bP0;0|17/17;15/15\x1b\\b", c'a' m m m m m m m m m m m m m m m m m m
            DecUserDefinedKeys("0;0|17/17;15/15".to_string()) c'b');
        pt!(b"a\x1b[12Ab", c'a' m m m m CursorUp(12) c'b');
        pt!(b"a\x1b[12Bb", c'a' m m m m CursorDown(12) c'b');
        pt!(b"a\x1b[12Cb", c'a' m m m m CursorForward(12) c'b');
        pt!(b"a\x1b[12Db", c'a' m m m m CursorBackward(12) c'b');
        pt!(b"a\x1b[12Eb", c'a' m m m m CursorNextLine(12) c'b');
        pt!(b"a\x1b[12Fb", c'a' m m m m CursorPrevLine(12) c'b');
        pt!(b"a\x1b[12Gb", c'a' m m m m CursorAbsoluteColumn(11) c'b');
        pt!(b"a\x1b[12;13Hb", c'a' m m m m m m m CursorAbsolutePosition(11,12) c'b');
        pt!(b"a\x1b[12Ib", c'a' m m m m CursorForwardTab(12) c'b');
        pt!(b"a\x1b[0Jb", c'a' m m m EraseDisplay(EraseDisplay::Below,false) c'b');
        pt!(b"a\x1b[1Jb", c'a' m m m EraseDisplay(EraseDisplay::Above,false) c'b');
        pt!(b"a\x1b[2Jb", c'a' m m m EraseDisplay(EraseDisplay::All,false) c'b');
        pt!(b"a\x1b[3Jb", c'a' m m m EraseDisplay(EraseDisplay::Saved,false) c'b');
        pt!(b"a\x1b[12Jb", c'a' m m m m m c'b');
        pt!(b"a\x1b[?0Jb", c'a' m m m m EraseDisplay(EraseDisplay::Below,true) c'b');
        pt!(b"a\x1b[?1Jb", c'a' m m m m EraseDisplay(EraseDisplay::Above,true) c'b');
        pt!(b"a\x1b[?2Jb", c'a' m m m m EraseDisplay(EraseDisplay::All,true) c'b');
        pt!(b"a\x1b[?3Jb", c'a' m m m m EraseDisplay(EraseDisplay::Saved,true) c'b');
        pt!(b"a\x1b[?12Jb", c'a' m m m m m m c'b');
        pt!(b"a\x1b[0Kb", c'a' m m m EraseLine(EraseLine::Right,false) c'b');
        pt!(b"a\x1b[1Kb", c'a' m m m EraseLine(EraseLine::Left,false) c'b');
        pt!(b"a\x1b[2Kb", c'a' m m m EraseLine(EraseLine::All,false) c'b');
        pt!(b"a\x1b[23Kb", c'a' m m m m m c'b');
        pt!(b"a\x1b[?0Kb", c'a' m m m m EraseLine(EraseLine::Right,true) c'b');
        pt!(b"a\x1b[?1Kb", c'a' m m m m EraseLine(EraseLine::Left,true) c'b');
        pt!(b"a\x1b[?2Kb", c'a' m m m m EraseLine(EraseLine::All,true) c'b');
        pt!(b"a\x1b[?23Kb", c'a' m m m m m m c'b');
        pt!(b"a\x1b[23Lb", c'a' m m m m InsertLines(23) c'b');
        pt!(b"a\x1b[23Mb", c'a' m m m m DeleteLines(23) c'b');
        pt!(b"a\x1b[23Pb", c'a' m m m m DeleteCharacters(23) c'b');
        pt!(b"a\x1b[23Sb", c'a' m m m m ScrollUp(23) c'b');
        pt!(b"a\x1b[?1;1;1Sb", c'a' m m m m m m m m
            GraphicRegister(GraReg::Color,GraOp::Read) c'b');
        pt!(b"a\x1b[?2;1;1Sb", c'a' m m m m m m m m
            GraphicRegister(GraReg::Sixel,GraOp::Read) c'b');
        pt!(b"a\x1b[?3;1;1Sb", c'a' m m m m m m m m
            GraphicRegister(GraReg::Regis,GraOp::Read) c'b');
        pt!(b"a\x1b[?1;2;1Sb", c'a' m m m m m m m m
            GraphicRegister(GraReg::Color,GraOp::Reset) c'b');
        pt!(b"a\x1b[?1;3;5Sb", c'a' m m m m m m m m
            GraphicRegister(GraReg::Color,GraOp::Write(5)) c'b');
        pt!(b"a\x1b[?1;4;5Sb", c'a' m m m m m m m m
            GraphicRegister(GraReg::Color,GraOp::GetMax) c'b');
        pt!(b"a\x1b[?1;5;1Sb", c'a' m m m m m m m m m c'b');
        pt!(b"a\x1b[?4;1;1Sb", c'a' m m m m m m m m m c'b');
        pt!(b"a\x1b[23Tb", c'a' m m m m ScrollDown(23) c'b');
        pt!(b"a\x1b[23;1;2;3;4Tb", c'a' m m m m m m m m m m m m MouseTracking(23,1,2,3,4) c'b');

        pt!(b"a\x1b[>0Tb", c'a' m m m m ResetTitleModes(TitleModes::SetLabelHex) c'b');
        pt!(b"a\x1b[>1Tb", c'a' m m m m ResetTitleModes(TitleModes::GetLabelHex) c'b');
        pt!(b"a\x1b[>2Tb", c'a' m m m m ResetTitleModes(TitleModes::SetLabelUtf8) c'b');
        pt!(b"a\x1b[>3Tb", c'a' m m m m ResetTitleModes(TitleModes::GetLabelUtf8) c'b');
        pt!(b"a\x1b[>0;1Tb", c'a' m m m m m m
            ResetTitleModes(TitleModes::SetLabelHex | TitleModes::GetLabelHex) c'b');
        pt!(b"a\x1b[>12;14Tb", c'a' m m m m m m m m ResetTitleModes(TitleModes::empty()) c'b');
        pt!(b"a\x1b[12Xc", c'a' m m m m EraseCharacters(12) c'c');
        pt!(b"a\x1b[12Zc", c'a' m m m m CursorBackwardTab(12) c'c');
        pt!(b"a\x1b[12^c", c'a' m m m m ScrollDown(12) c'c');
        pt!(b"a\x1b[12`c", c'a' m m m m CursorAbsoluteColumn(11) c'c');
        pt!(b"a\x1b[12ax", c'a' m m m m HorizontalMove(12) c'x');
        pt!(b"a\x1b[12bx", c'a' m m m m RepeatCharacter(12) c'x');
        pt!(b"a\x1b[12cx", c'a' m m m m DA1(12) c'x');
        pt!(b"a\x1b[>12cx", c'a' m m m m m DA2(12) c'x');
        pt!(b"a\x1b[12dy", c'a' m m m m VerticalPositionAbsolute(11) c'y');
        pt!(b"a\x1b[12ey", c'a' m m m m VerticalPositionRelative(11) c'y');
        pt!(b"a\x1b[12;13fy", c'a' m m m m m m m CursorAbsolutePosition(11,12) c'y');
        pt!(b"a\x1b[0gy", c'a' m m m TabClear(TabClear::Column) c'y');
        pt!(b"a\x1b[3gy", c'a' m m m TabClear(TabClear::All) c'y');
        pt!(b"a\x1b[13gy", c'a' m m m m m c'y');
        pt!(b"a\x1b[2hy", c'a' m m m SetMode(SetMode::KeyboardAction) c'y');
        pt!(b"a\x1b[4hy", c'a' m m m SetMode(SetMode::Insert) c'y');
        pt!(b"a\x1b[12hy", c'a' m m m m SetMode(SetMode::SendReceive) c'y');
        pt!(b"a\x1b[20hy", c'a' m m m m SetMode(SetMode::AutomaticNewline) c'y');
        pt!(b"a\x1b[21hy", c'a' m m m m m c'y');
        pt!(b"a\x1b[?1hz", c'a' m m m m SetPrivateMode(SetPrivateMode::ApplicationCursorKeys) c'z');
        pt!(b"a\x1b[?2hz", c'a' m m m m SetPrivateMode(SetPrivateMode::UsAsciiForG0toG3) c'z');
        pt!(b"a\x1b[?3hz", c'a' m m m m SetPrivateMode(SetPrivateMode::Hundred32Columns) c'z');
        pt!(b"a\x1b[?4hz", c'a' m m m m SetPrivateMode(SetPrivateMode::SmoothScroll) c'z');
        pt!(b"a\x1b[?5hz", c'a' m m m m SetPrivateMode(SetPrivateMode::ReverseVideo) c'z');
        pt!(b"a\x1b[?6hz", c'a' m m m m SetPrivateMode(SetPrivateMode::OriginMode) c'z');
        pt!(b"a\x1b[?7hz", c'a' m m m m SetPrivateMode(SetPrivateMode::AutoWrapMode) c'z');
        pt!(b"a\x1b[?8hz", c'a' m m m m SetPrivateMode(SetPrivateMode::AutoRepeatKeys) c'z');
        pt!(b"a\x1b[?9hz", c'a' m m m m SetPrivateMode(SetPrivateMode::SendMousePosOnPress) c'z');
        pt!(b"a\x1b[?10hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::ShowToolbar) c'z');
        pt!(b"a\x1b[?12hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::StartBlinkingCursor) c'z');
        pt!(b"a\x1b[?13hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::StartBlinkingCursor) c'z');
        pt!(b"a\x1b[?14hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::EnableXorBlinkingCursor) c'z');
        pt!(b"a\x1b[?18hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::PrintFormFeed) c'z');
        pt!(b"a\x1b[?19hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::PrintFullScreen) c'z');
        pt!(b"a\x1b[?25hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::ShowCursor) c'z');
        pt!(b"a\x1b[?30hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::ShowScrollbar) c'z');
        pt!(b"a\x1b[?35hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::EnableFontShifting) c'z');
        pt!(b"a\x1b[?38hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::TektronixMode) c'z');
        pt!(b"a\x1b[?40hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::AllowHundred32Mode) c'z');
        pt!(b"a\x1b[?41hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::MoreFix) c'z');
        pt!(b"a\x1b[?42hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::EnableNrc) c'z');
        pt!(b"a\x1b[?44hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::MarginBell) c'z');
        pt!(b"a\x1b[?45hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::ReverseWrapAroundMode) c'z');
        pt!(b"a\x1b[?46hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::StartLogging) c'z');
        pt!(b"a\x1b[?47hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::AlternateScreenBuffer) c'z');
        pt!(b"a\x1b[?66hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::ApplicationKeypad) c'z');
        pt!(b"a\x1b[?67hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::BackArrowIsBackSspace) c'z');
        pt!(b"a\x1b[?69hz", c'a' m m m m m
            SetPrivateMode(SetPrivateMode::EnableLeftRightMarginMode) c'z');
        pt!(b"a\x1b[?95hz", c'a' m m m m m SetPrivateMode(SetPrivateMode::NoClearScreenOnDECCOLM)
            c'z');
        pt!(b"a\x1b[?1000hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SendMousePosOnBoth)
            c'z');
        pt!(b"a\x1b[?1001hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::HiliteMouseTracking) c'z');
        pt!(b"a\x1b[?1002hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::CellMouseTracking)
            c'z');
        pt!(b"a\x1b[?1003hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::AllMouseTracking)
            c'z');
        pt!(b"a\x1b[?1004hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SendFocusEvents)
            c'z');
        pt!(b"a\x1b[?1005hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::Utf8MouseMode)
            c'z');
        pt!(b"a\x1b[?1006hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SgrMouseMode) c'z');
        pt!(b"a\x1b[?1007hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::AlternateScrollMode) c'z');
        pt!(b"a\x1b[?1010hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::ScrollToBottomOnTty) c'z');
        pt!(b"a\x1b[?1011hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::ScrollToBottomOnKey) c'z');
        pt!(b"a\x1b[?1015hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::UrxvtMouseMode)
            c'z');
        pt!(b"a\x1b[?1034hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::InterpretMetaKey)
            c'z');
        pt!(b"a\x1b[?1035hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::EnableSpecialModifiers) c'z');
        pt!(b"a\x1b[?1036hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SendEscOnMeta)
            c'z');
        pt!(b"a\x1b[?1037hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SendDelOnKeypad)
            c'z');
        pt!(b"a\x1b[?1039hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SendEscOnAlt)
            c'z');
        pt!(b"a\x1b[?1040hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::KeepSelection)
            c'z');
        pt!(b"a\x1b[?1041hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::UseClipboard)
            c'z');
        pt!(b"a\x1b[?1042hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::UrgencyHint) c'z');
        pt!(b"a\x1b[?1043hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::RaiseWindowOnBell)
            c'z');
        pt!(b"a\x1b[?1044hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::KeepClipboard)
            c'z');
        pt!(b"a\x1b[?1046hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::EnableAlternateScreen) c'z');
        pt!(b"a\x1b[?1047hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::UseAlternateScreen)
            c'z');
        pt!(b"a\x1b[?1048hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SaveCursor) c'z');
        pt!(b"a\x1b[?1049hz", c'a' m m m m m m m
            SetPrivateMode(SetPrivateMode::SaveCursorAndUseAlternateScreen) c'z');
        pt!(b"a\x1b[?1050hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::TerminfoFnMode)
            c'z');
        pt!(b"a\x1b[?1051hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::SunFnMode) c'z');
        pt!(b"a\x1b[?1052hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::HpFnMode) c'z');
        pt!(b"a\x1b[?1053hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::ScoFnMode) c'z');
        pt!(b"a\x1b[?1060hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::LegacyKeyboard)
            c'z');
        pt!(b"a\x1b[?1061hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::Vt220Keyboard)
            c'z');
        pt!(b"a\x1b[?2004hz", c'a' m m m m m m m SetPrivateMode(SetPrivateMode::BracketedPaste)
            c'z');
        pt!(b"a\x1b[?0hz", c'a' m m m m m c'z');
        pt!(b"a\x1b[0ik", c'a' m m m MediaCopy(MediaCopy::PrintScreen) c'k');
        pt!(b"a\x1b[4ik", c'a' m m m MediaCopy(MediaCopy::PrinterCtrlOff) c'k');
        pt!(b"a\x1b[5ik", c'a' m m m MediaCopy(MediaCopy::PrinterCtrlOn) c'k');
        pt!(b"a\x1b[10ik", c'a' m m m m MediaCopy(MediaCopy::HtmlScreenDump) c'k');
        pt!(b"a\x1b[11ik", c'a' m m m m MediaCopy(MediaCopy::SvgScreenDump) c'k');
        pt!(b"a\x1b[6ik", c'a' m m m m c'k');
        pt!(b"a\x1b[?1ik", c'a' m m m m MediaCopy(MediaCopy::PrintCursorLine) c'k');
        pt!(b"a\x1b[?4ik", c'a' m m m m MediaCopy(MediaCopy::AutoPrintOff) c'k');
        pt!(b"a\x1b[?5ik", c'a' m m m m MediaCopy(MediaCopy::AutoPrintOn) c'k');
        pt!(b"a\x1b[?10ik", c'a' m m m m m MediaCopy(MediaCopy::PrintComposeDisplay) c'k');
        pt!(b"a\x1b[?11ik", c'a' m m m m m MediaCopy(MediaCopy::PrintAllPages) c'k');
        pt!(b"a\x1b[?12ik", c'a' m m m m m m c'k');
        pt!(b"a\x1b[2ly", c'a' m m m ResetMode(SetMode::KeyboardAction) c'y');
        pt!(b"a\x1b[4ly", c'a' m m m ResetMode(SetMode::Insert) c'y');
        pt!(b"a\x1b[12ly", c'a' m m m m ResetMode(SetMode::SendReceive) c'y');
        pt!(b"a\x1b[20ly", c'a' m m m m ResetMode(SetMode::AutomaticNewline) c'y');
        pt!(b"a\x1b[21ly", c'a' m m m m m c'y');
        pt!(b"a\x1b[?1lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::ApplicationCursorKeys)
            c'z');
        pt!(b"a\x1b[?2lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::UsAsciiForG0toG3) c'z');
        pt!(b"a\x1b[?3lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::Hundred32Columns) c'z');
        pt!(b"a\x1b[?4lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::SmoothScroll) c'z');
        pt!(b"a\x1b[?5lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::ReverseVideo) c'z');
        pt!(b"a\x1b[?6lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::OriginMode) c'z');
        pt!(b"a\x1b[?7lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::AutoWrapMode) c'z');
        pt!(b"a\x1b[?8lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::AutoRepeatKeys) c'z');
        pt!(b"a\x1b[?9lz", c'a' m m m m ResetPrivateMode(SetPrivateMode::SendMousePosOnPress)
            c'z');
        pt!(b"a\x1b[?10lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::ShowToolbar) c'z');
        pt!(b"a\x1b[?12lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::StartBlinkingCursor)
            c'z');
        pt!(b"a\x1b[?13lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::StartBlinkingCursor)
            c'z');
        pt!(b"a\x1b[?14lz", c'a' m m m m m
            ResetPrivateMode(SetPrivateMode::EnableXorBlinkingCursor) c'z');
        pt!(b"a\x1b[?18lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::PrintFormFeed) c'z');
        pt!(b"a\x1b[?19lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::PrintFullScreen) c'z');
        pt!(b"a\x1b[?25lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::ShowCursor) c'z');
        pt!(b"a\x1b[?30lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::ShowScrollbar) c'z');
        pt!(b"a\x1b[?35lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::EnableFontShifting)
            c'z');
        pt!(b"a\x1b[?38lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::TektronixMode) c'z');
        pt!(b"a\x1b[?40lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::AllowHundred32Mode)
            c'z');
        pt!(b"a\x1b[?41lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::MoreFix) c'z');
        pt!(b"a\x1b[?42lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::EnableNrc) c'z');
        pt!(b"a\x1b[?44lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::MarginBell) c'z');
        pt!(b"a\x1b[?45lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::ReverseWrapAroundMode)
            c'z');
        pt!(b"a\x1b[?46lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::StartLogging) c'z');
        pt!(b"a\x1b[?47lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::AlternateScreenBuffer)
            c'z');
        pt!(b"a\x1b[?66lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::ApplicationKeypad)
            c'z');
        pt!(b"a\x1b[?67lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::BackArrowIsBackSspace)
            c'z');
        pt!(b"a\x1b[?69lz", c'a' m m m m m
            ResetPrivateMode(SetPrivateMode::EnableLeftRightMarginMode) c'z');
        pt!(b"a\x1b[?95lz", c'a' m m m m m ResetPrivateMode(SetPrivateMode::NoClearScreenOnDECCOLM)
            c'z');
        pt!(b"a\x1b[?1000lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::SendMousePosOnBoth) c'z');
        pt!(b"a\x1b[?1001lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::HiliteMouseTracking) c'z');
        pt!(b"a\x1b[?1002lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::CellMouseTracking) c'z');
        pt!(b"a\x1b[?1003lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::AllMouseTracking)
            c'z');
        pt!(b"a\x1b[?1004lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SendFocusEvents)
            c'z');
        pt!(b"a\x1b[?1005lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::Utf8MouseMode)
            c'z');
        pt!(b"a\x1b[?1006lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SgrMouseMode)
            c'z');
        pt!(b"a\x1b[?1007lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::AlternateScrollMode) c'z');
        pt!(b"a\x1b[?1010lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::ScrollToBottomOnTty) c'z');
        pt!(b"a\x1b[?1011lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::ScrollToBottomOnKey) c'z');
        pt!(b"a\x1b[?1015lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::UrxvtMouseMode)
            c'z');
        pt!(b"a\x1b[?1034lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::InterpretMetaKey)
            c'z');
        pt!(b"a\x1b[?1035lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::EnableSpecialModifiers) c'z');
        pt!(b"a\x1b[?1036lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SendEscOnMeta)
            c'z');
        pt!(b"a\x1b[?1037lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SendDelOnKeypad)
            c'z');
        pt!(b"a\x1b[?1039lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SendEscOnAlt)
            c'z');
        pt!(b"a\x1b[?1040lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::KeepSelection)
            c'z');
        pt!(b"a\x1b[?1041lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::UseClipboard)
            c'z');
        pt!(b"a\x1b[?1042lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::UrgencyHint)
            c'z');
        pt!(b"a\x1b[?1043lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::RaiseWindowOnBell) c'z');
        pt!(b"a\x1b[?1044lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::KeepClipboard)
            c'z');
        pt!(b"a\x1b[?1046lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::EnableAlternateScreen) c'z');
        pt!(b"a\x1b[?1047lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::UseAlternateScreen) c'z');
        pt!(b"a\x1b[?1048lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SaveCursor)
            c'z');
        pt!(b"a\x1b[?1049lz", c'a' m m m m m m m
            ResetPrivateMode(SetPrivateMode::SaveCursorAndUseAlternateScreen) c'z');
        pt!(b"a\x1b[?1050lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::TerminfoFnMode)
            c'z');
        pt!(b"a\x1b[?1051lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::SunFnMode) c'z');
        pt!(b"a\x1b[?1052lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::HpFnMode) c'z');
        pt!(b"a\x1b[?1053lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::ScoFnMode) c'z');
        pt!(b"a\x1b[?1060lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::LegacyKeyboard)
            c'z');
        pt!(b"a\x1b[?1061lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::Vt220Keyboard)
            c'z');
        pt!(b"a\x1b[?2004lz", c'a' m m m m m m m ResetPrivateMode(SetPrivateMode::BracketedPaste)
            c'z');
        pt!(b"a\x1b[?0lz", c'a' m m m m m c'z');
        pt!(b"a\x1b[38;2;0;12;13;14mx", c'a' m m m m m m m m m m m m m m m m m
            ForegroundColorRgb(12,13,14) c'x');
        pt!(b"a\x1b[38;5;12mx", c'a' m m m m m m m m m ForegroundColorIndex(12) c'x');
        pt!(b"a\x1b[48;2;0;12;13;14mx", c'a' m m m m m m m m m m m m m m m m m
            BackgroundColorRgb(12,13,14) c'x');
        pt!(b"a\x1b[48;5;12mx", c'a' m m m m m m m m m BackgroundColorIndex(12) c'x');
        pt!(b"a\x1b[38;2;12;13;14mx", c'a' m m m m m m m m m m m m m m m
            ForegroundColorRgb(12,13,14) c'x');
        pt!(b"a\x1b[48;2;12;13;14mx", c'a' m m m m m m m m m m m m m m m
            BackgroundColorRgb(12,13,14) c'x');

        pt!(b"a\x1b[0;1;2;3;50;4;5mx", c'a' m m m m m m m m m m m m m m m m
            CharacterAttributes(
                vec![
                CharacterAttribute::Normal,
                CharacterAttribute::Bold,
                CharacterAttribute::Faint,
                CharacterAttribute::Italicized,
                CharacterAttribute::Underlined,
                CharacterAttribute::Blink
                ]) c'x');
        pt!(b"a\x1b[6;7;8;9;21;22;23;24;25;27;28;29mx", c'a' m m m m m m m m m m m m m m m m m m m
            m m m m m m m m m m m m m m
            CharacterAttributes(
                vec![
                CharacterAttribute::Inverse,
                CharacterAttribute::Invisible,
                CharacterAttribute::CrossedOut,
                CharacterAttribute::DoublyUnderlined,
                CharacterAttribute::NotBoldFaint,
                CharacterAttribute::NotItalicized,
                CharacterAttribute::NotUnderlined,
                CharacterAttribute::Steady,
                CharacterAttribute::Positive,
                CharacterAttribute::Visible,
                CharacterAttribute::NotCrossedOut,
                ]) c'x');
        pt!(b"a\x1b[30;31;32;33;34;35;36;37;39mx", c'a' m m m m m m m m m m m m m m
            m m m m m m m m m m m m m m
            CharacterAttributes(
                vec![
                CharacterAttribute::Foreground(Color::Black),
                CharacterAttribute::Foreground(Color::Red),
                CharacterAttribute::Foreground(Color::Green),
                CharacterAttribute::Foreground(Color::Yellow),
                CharacterAttribute::Foreground(Color::Blue),
                CharacterAttribute::Foreground(Color::Magenta),
                CharacterAttribute::Foreground(Color::Cyan),
                CharacterAttribute::Foreground(Color::White),
                CharacterAttribute::Foreground(Color::Default),
                ]) c'x');
        pt!(b"a\x1b[40;41;42;43;44;45;46;47;49mx", c'a' m m m m m m m m m m m m m m
            m m m m m m m m m m m m m m
            CharacterAttributes(
                vec![
                CharacterAttribute::Background(Color::Black),
                CharacterAttribute::Background(Color::Red),
                CharacterAttribute::Background(Color::Green),
                CharacterAttribute::Background(Color::Yellow),
                CharacterAttribute::Background(Color::Blue),
                CharacterAttribute::Background(Color::Magenta),
                CharacterAttribute::Background(Color::Cyan),
                CharacterAttribute::Background(Color::White),
                CharacterAttribute::Background(Color::Default),
                ]) c'x');
        pt!(b"a\x1b[90;91;92;93;94;95;96;97mx", c'a' m m m m m m m m m m m
            m m m m m m m m m m m m m m
            CharacterAttributes(
                vec![
                CharacterAttribute::Foreground(Color::Grey),
                CharacterAttribute::Foreground(Color::BrightRed),
                CharacterAttribute::Foreground(Color::BrightGreen),
                CharacterAttribute::Foreground(Color::BrightYellow),
                CharacterAttribute::Foreground(Color::BrightBlue),
                CharacterAttribute::Foreground(Color::BrightMagenta),
                CharacterAttribute::Foreground(Color::BrightCyan),
                CharacterAttribute::Foreground(Color::BrightWhite),
                ]) c'x');
        pt!(b"a\x1b[100;101;102;103;104;105;106;107mx", c'a' m m m m m m m m m m m m m m m m m m m
            m m m m m m m m m m m m m m
            CharacterAttributes(
                vec![
                CharacterAttribute::Background(Color::Grey),
                CharacterAttribute::Background(Color::BrightRed),
                CharacterAttribute::Background(Color::BrightGreen),
                CharacterAttribute::Background(Color::BrightYellow),
                CharacterAttribute::Background(Color::BrightBlue),
                CharacterAttribute::Background(Color::BrightMagenta),
                CharacterAttribute::Background(Color::BrightCyan),
                CharacterAttribute::Background(Color::BrightWhite),
                ]) c'x');
        pt!(b"a\x1b[>0;12mx", c'a' m m m m m m m SetModFKeys(FKeys::Keyboard,12) c'x');
        pt!(b"a\x1b[>1;12mx", c'a' m m m m m m m SetModFKeys(FKeys::Cursor,12) c'x');
        pt!(b"a\x1b[>2;12mx", c'a' m m m m m m m SetModFKeys(FKeys::Function,12) c'x');
        pt!(b"a\x1b[>4;12mx", c'a' m m m m m m m SetModFKeys(FKeys::Other,12) c'x');
        pt!(b"a\x1b[>3;12mx", c'a' m m m m m m m m c'x');
        pt!(b"a\x1b[5nx", c'a' m m m StatusReport c'x');
        pt!(b"a\x1b[6nx", c'a' m m m ReportCursorPosition c'x');
        pt!(b"a\x1b[0nx", c'a' m m m m c'x');
        pt!(b"a\x1b[>0nx", c'a' m m m m DisableModFKeys(FKeys::Keyboard) c'x');
        pt!(b"a\x1b[>1nx", c'a' m m m m DisableModFKeys(FKeys::Cursor) c'x');
        pt!(b"a\x1b[>2nx", c'a' m m m m DisableModFKeys(FKeys::Function) c'x');
        pt!(b"a\x1b[>4nx", c'a' m m m m DisableModFKeys(FKeys::Other) c'x');
        pt!(b"a\x1b[>nx", c'a' m m m DisableModFKeys(FKeys::Function) c'x');
        pt!(b"a\x1b[>3nx", c'a' m m m m m c'x');
        pt!(b"a\x1b[?6nx", c'a' m m m m DecDeviceStatusReport c'x');
        pt!(b"a\x1b[?15nx", c'a' m m m m m PrinterStatusReport c'x');
        pt!(b"a\x1b[?25nx", c'a' m m m m m UdkStatusReport c'x');
        pt!(b"a\x1b[?26nx", c'a' m m m m m KeyboardStatusReport c'x');
        pt!(b"a\x1b[?53nx", c'a' m m m m m LocatorStatusReport c'x');
        pt!(b"a\x1b[?55nx", c'a' m m m m m LocatorStatusReport c'x');
        pt!(b"a\x1b[?56nx", c'a' m m m m m LocatorTypeReport c'x');
        pt!(b"a\x1b[?62nx", c'a' m m m m m MacroStatusReport c'x');
        pt!(b"a\x1b[?63;12nx", c'a' m m m m m m m m MemoryStatusReport(12) c'x');
        pt!(b"a\x1b[?75nx", c'a' m m m m m DataIntegrityReport c'x');
        pt!(b"a\x1b[?85nx", c'a' m m m m m MultiSessionReport c'x');
        pt!(b"a\x1b[?86nx", c'a' m m m m m m c'x');
        pt!(b"a\x1b[>0px", c'a' m m m m PointerMode(PointerMode::NeverHide) c'x');
        pt!(b"a\x1b[>1px", c'a' m m m m PointerMode(PointerMode::HideNotTracking) c'x');
        pt!(b"a\x1b[>2px", c'a' m m m m PointerMode(PointerMode::HideOutside) c'x');
        pt!(b"a\x1b[>3px", c'a' m m m m PointerMode(PointerMode::AlwaysHide) c'x');
        pt!(b"a\x1b[>px", c'a' m m m PointerMode(PointerMode::HideNotTracking) c'x');
        pt!(b"a\x1b[>4px", c'a' m m m m m c'x');
        pt!(b"a\x1b[!px", c'a' m m m SoftReset c'x');
        pt!(b"a\x1b[61;0\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt100,false) c'x');
        pt!(b"a\x1b[61;1\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt100,false) c'x');
        pt!(b"a\x1b[61;2\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt100,false) c'x');
        pt!(b"a\x1b[62;0\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt200,true) c'x');
        pt!(b"a\x1b[62;1\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt200,false) c'x');
        pt!(b"a\x1b[62;2\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt200,true) c'x');
        pt!(b"a\x1b[63;2\"px", c'a' m m m m m m m ConformanceLevel(Terminal::Vt300,true) c'x');
        pt!(b"a\x1b[12;2\"px", c'a' m m m m m m m m c'x');
        pt!(b"a\x1b[63;4\"px", c'a' m m m m m m m m c'x');
        pt!(b"a\x1b[2$px", c'a' m m m m RequestAnsiMode(SetMode::KeyboardAction) c'x');
        pt!(b"a\x1b[4$px", c'a' m m m m RequestAnsiMode(SetMode::Insert) c'x');
        pt!(b"a\x1b[12$px", c'a' m m m m m RequestAnsiMode(SetMode::SendReceive) c'x');
        pt!(b"a\x1b[20$px", c'a' m m m m m RequestAnsiMode(SetMode::AutomaticNewline) c'x');
        pt!(b"a\x1b[0$px", c'a' m m m m RequestAnsiMode(SetMode::Unknown) c'x');

        pt!(b"a\x1b[?1$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::ApplicationCursorKeys)
            c'z');
        pt!(b"a\x1b[?2$pz", c'a' m m m m m
            RequestPrivateMode(SetPrivateMode::UsAsciiForG0toG3) c'z');
        pt!(b"a\x1b[?3$pz", c'a' m m m m m
            RequestPrivateMode(SetPrivateMode::Hundred32Columns) c'z');
        pt!(b"a\x1b[?4$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::SmoothScroll) c'z');
        pt!(b"a\x1b[?5$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::ReverseVideo) c'z');
        pt!(b"a\x1b[?6$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::OriginMode) c'z');
        pt!(b"a\x1b[?7$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::AutoWrapMode) c'z');
        pt!(b"a\x1b[?8$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::AutoRepeatKeys) c'z');
        pt!(b"a\x1b[?9$pz", c'a' m m m m m RequestPrivateMode(SetPrivateMode::SendMousePosOnPress)
            c'z');
        pt!(b"a\x1b[?10$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::ShowToolbar) c'z');
        pt!(b"a\x1b[?12$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::StartBlinkingCursor) c'z');
        pt!(b"a\x1b[?13$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::StartBlinkingCursor) c'z');
        pt!(b"a\x1b[?14$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::EnableXorBlinkingCursor) c'z');
        pt!(b"a\x1b[?18$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::PrintFormFeed)
            c'z');
        pt!(b"a\x1b[?19$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::PrintFullScreen)
            c'z');
        pt!(b"a\x1b[?25$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::ShowCursor) c'z');
        pt!(b"a\x1b[?30$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::ShowScrollbar)
            c'z');
        pt!(b"a\x1b[?35$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::EnableFontShifting)
            c'z');
        pt!(b"a\x1b[?38$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::TektronixMode)
            c'z');
        pt!(b"a\x1b[?40$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::AllowHundred32Mode)
            c'z');
        pt!(b"a\x1b[?41$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::MoreFix) c'z');
        pt!(b"a\x1b[?42$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::EnableNrc) c'z');
        pt!(b"a\x1b[?44$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::MarginBell) c'z');
        pt!(b"a\x1b[?45$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::ReverseWrapAroundMode) c'z');
        pt!(b"a\x1b[?46$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::StartLogging)
            c'z');
        pt!(b"a\x1b[?47$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::AlternateScreenBuffer) c'z');
        pt!(b"a\x1b[?66$pz", c'a' m m m m m m RequestPrivateMode(SetPrivateMode::ApplicationKeypad)
            c'z');
        pt!(b"a\x1b[?67$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::BackArrowIsBackSspace) c'z');
        pt!(b"a\x1b[?69$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::EnableLeftRightMarginMode) c'z');
        pt!(b"a\x1b[?95$pz", c'a' m m m m m m
            RequestPrivateMode(SetPrivateMode::NoClearScreenOnDECCOLM) c'z');
        pt!(b"a\x1b[?1000$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SendMousePosOnBoth) c'z');
        pt!(b"a\x1b[?1001$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::HiliteMouseTracking) c'z');
        pt!(b"a\x1b[?1002$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::CellMouseTracking) c'z');
        pt!(b"a\x1b[?1003$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::AllMouseTracking) c'z');
        pt!(b"a\x1b[?1004$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SendFocusEvents) c'z');
        pt!(b"a\x1b[?1005$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::Utf8MouseMode) c'z');
        pt!(b"a\x1b[?1006$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SgrMouseMode) c'z');
        pt!(b"a\x1b[?1007$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::AlternateScrollMode) c'z');
        pt!(b"a\x1b[?1010$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::ScrollToBottomOnTty) c'z');
        pt!(b"a\x1b[?1011$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::ScrollToBottomOnKey) c'z');
        pt!(b"a\x1b[?1015$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::UrxvtMouseMode) c'z');
        pt!(b"a\x1b[?1034$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::InterpretMetaKey) c'z');
        pt!(b"a\x1b[?1035$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::EnableSpecialModifiers) c'z');
        pt!(b"a\x1b[?1036$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SendEscOnMeta) c'z');
        pt!(b"a\x1b[?1037$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SendDelOnKeypad) c'z');
        pt!(b"a\x1b[?1039$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SendEscOnAlt) c'z');
        pt!(b"a\x1b[?1040$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::KeepSelection) c'z');
        pt!(b"a\x1b[?1041$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::UseClipboard) c'z');
        pt!(b"a\x1b[?1042$pz", c'a' m m m m m m m m RequestPrivateMode(SetPrivateMode::UrgencyHint)
            c'z');
        pt!(b"a\x1b[?1043$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::RaiseWindowOnBell) c'z');
        pt!(b"a\x1b[?1044$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::KeepClipboard) c'z');
        pt!(b"a\x1b[?1046$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::EnableAlternateScreen) c'z');
        pt!(b"a\x1b[?1047$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::UseAlternateScreen) c'z');
        pt!(b"a\x1b[?1048$pz", c'a' m m m m m m m m RequestPrivateMode(SetPrivateMode::SaveCursor)
            c'z');
        pt!(b"a\x1b[?1049$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::SaveCursorAndUseAlternateScreen) c'z');
        pt!(b"a\x1b[?1050$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::TerminfoFnMode) c'z');
        pt!(b"a\x1b[?1051$pz", c'a' m m m m m m m m RequestPrivateMode(SetPrivateMode::SunFnMode)
            c'z');
        pt!(b"a\x1b[?1052$pz", c'a' m m m m m m m m RequestPrivateMode(SetPrivateMode::HpFnMode)
            c'z');
        pt!(b"a\x1b[?1053$pz", c'a' m m m m m m m m RequestPrivateMode(SetPrivateMode::ScoFnMode)
            c'z');
        pt!(b"a\x1b[?1060$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::LegacyKeyboard) c'z');
        pt!(b"a\x1b[?1061$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::Vt220Keyboard) c'z');
        pt!(b"a\x1b[?2004$pz", c'a' m m m m m m m m
            RequestPrivateMode(SetPrivateMode::BracketedPaste) c'z');
        pt!(b"a\x1b[?2005$pz", c'a' m m m m m m m m RequestPrivateMode(SetPrivateMode::Unknown)
            c'z');
        pt!(b"a\x1b[0qc", c'a' m m m LoadLeds(LoadLeds::All,false) c'c');
        pt!(b"a\x1b[1qc", c'a' m m m LoadLeds(LoadLeds::NumLock,false) c'c');
        pt!(b"a\x1b[2qc", c'a' m m m LoadLeds(LoadLeds::CapsLock,false) c'c');
        pt!(b"a\x1b[3qc", c'a' m m m LoadLeds(LoadLeds::ScrollLock,false) c'c');
        pt!(b"a\x1b[20qc", c'a' m m m m m c'c');
        pt!(b"a\x1b[21qc", c'a' m m m m LoadLeds(LoadLeds::NumLock,true) c'c');
        pt!(b"a\x1b[22qc", c'a' m m m m LoadLeds(LoadLeds::CapsLock,true) c'c');
        pt!(b"a\x1b[23qc", c'a' m m m m LoadLeds(LoadLeds::ScrollLock,true) c'c');
        pt!(b"a\x1b[0 qx", c'a' m m m m CursorStyle(CursorStyle::BlinkBlock) c'x');
        pt!(b"a\x1b[1 qx", c'a' m m m m CursorStyle(CursorStyle::BlinkBlock) c'x');
        pt!(b"a\x1b[2 qx", c'a' m m m m CursorStyle(CursorStyle::SteadyBlock) c'x');
        pt!(b"a\x1b[3 qx", c'a' m m m m CursorStyle(CursorStyle::BlinkUnderline) c'x');
        pt!(b"a\x1b[4 qx", c'a' m m m m CursorStyle(CursorStyle::SteadyUnderline) c'x');
        pt!(b"a\x1b[5 qx", c'a' m m m m CursorStyle(CursorStyle::BlinkBar) c'x');
        pt!(b"a\x1b[6 qx", c'a' m m m m CursorStyle(CursorStyle::SteadyBar) c'x');
        pt!(b"a\x1b[7 qx", c'a' m m m m m c'x');
        pt!(b"a\x1b[0\"qx", c'a' m m m m CharacterProtection(CharacterProtection::CanErase) c'x');
        pt!(b"a\x1b[1\"qx", c'a' m m m m CharacterProtection(CharacterProtection::NoErase) c'x');
        pt!(b"a\x1b[2\"qx", c'a' m m m m CharacterProtection(CharacterProtection::CanErase) c'x');
        pt!(b"a\x1b[3\"qx", c'a' m m m m m c'x');
        pt!(b"a\x1b[12;13rx", c'a' m m m m m m m ScrollRegion(11,12) c'x');
        pt!(b"a\x1b[14;13rx", c'a' m m m m m m m m c'x');
        pt!(b"a\x1b[rx", c'a' m m ScrollRegion(0,0) c'x');
        pt!(b"a\x1b[?1041rz", c'a' m m m m m m m RestorePrivateMode(SetPrivateMode::UseClipboard)
            c'z');
        pt!(b"a\x1b[0;1;2;3;0$rx", c'a' m m m m m m m m m m m m
            ChangeAttributesArea(0,1,2,3,CharacterAttribute::Normal) c'x');
        pt!(b"a\x1b[0;1;2;3;1$rx", c'a' m m m m m m m m m m m m
            ChangeAttributesArea(0,1,2,3,CharacterAttribute::Bold) c'x');
        pt!(b"a\x1b[0;1;2;3;4$rx", c'a' m m m m m m m m m m m m
            ChangeAttributesArea(0,1,2,3,CharacterAttribute::Underlined) c'x');
        pt!(b"a\x1b[0;1;2;3;5$rx", c'a' m m m m m m m m m m m m
            ChangeAttributesArea(0,1,2,3,CharacterAttribute::Blink) c'x');
        pt!(b"a\x1b[0;1;2;3;7$rx", c'a' m m m m m m m m m m m m
            ChangeAttributesArea(0,1,2,3,CharacterAttribute::Inverse) c'x');
        pt!(b"a\x1b[0;1;2;3;2$rx", c'a' m m m m m m m m m m m m m c'x');
        pt!(b"a\x1b[0;1;0;3;0$rx", c'a' m m m m m m m m m m m m m c'x');
        pt!(b"a\x1b[0;1;2;1;0$rx", c'a' m m m m m m m m m m m m m c'x');
        pt!(b"a\x1b[sx", c'a' m m SaveCursor c'x');
        pt!(b"a\x1b[12;13sx", c'a' m m m m m m m SetMargins(11,12) c'x');
        pt!(b"a\x1b[14;13sx", c'a' m m m m m m m m c'x');
        pt!(b"a\x1b[?1041sz", c'a' m m m m m m m SavePrivateMode(SetPrivateMode::UseClipboard)
            c'z');
        pt!(b"a\x1b[1tx", c'a' m m m WindowOp(WindowOp::DeIconify) c'x');
        pt!(b"a\x1b[2tx", c'a' m m m WindowOp(WindowOp::Iconify) c'x');
        pt!(b"a\x1b[3;12;13tx", c'a' m m m m m m m m m WindowOp(WindowOp::Move(12,13)) c'x');
        pt!(b"a\x1b[4;12;13tx", c'a' m m m m m m m m m
            WindowOp(WindowOp::ResizeWindow(Some(12),Some(13))) c'x');
        pt!(b"a\x1b[4;;13tx", c'a' m m m m m m m
            WindowOp(WindowOp::ResizeWindow(None,Some(13))) c'x');
        pt!(b"a\x1b[4;13;tx", c'a' m m m m m m m
            WindowOp(WindowOp::ResizeWindow(Some(13),None)) c'x');
        pt!(b"a\x1b[5tx", c'a' m m m WindowOp(WindowOp::Raise) c'x');
        pt!(b"a\x1b[6tx", c'a' m m m WindowOp(WindowOp::Lower) c'x');
        pt!(b"a\x1b[7tx", c'a' m m m WindowOp(WindowOp::Refresh) c'x');
        pt!(b"a\x1b[8;12;13tx", c'a' m m m m m m m m m
            WindowOp(WindowOp::ResizeTextArea(Some(12),Some(13))) c'x');
        pt!(b"a\x1b[9;0tx", c'a' m m m m m WindowOp(WindowOp::RestoreMaximized) c'x');
        pt!(b"a\x1b[9;1tx", c'a' m m m m m WindowOp(WindowOp::MaximizeWindow) c'x');
        pt!(b"a\x1b[9;2tx", c'a' m m m m m WindowOp(WindowOp::MaximizeVertically) c'x');
        pt!(b"a\x1b[9;3tx", c'a' m m m m m WindowOp(WindowOp::MaximizeHorizontally) c'x');
        pt!(b"a\x1b[10;0tx", c'a' m m m m m m WindowOp(WindowOp::UndoFullscreen) c'x');
        pt!(b"a\x1b[10;1tx", c'a' m m m m m m WindowOp(WindowOp::Fullscreen) c'x');
        pt!(b"a\x1b[10;2tx", c'a' m m m m m m WindowOp(WindowOp::ToggleFullscreen) c'x');
        pt!(b"a\x1b[11tx", c'a' m m m m WindowOp(WindowOp::ReportWindowState) c'x');
        pt!(b"a\x1b[13tx", c'a' m m m m WindowOp(WindowOp::ReportWindowPosition) c'x');
        pt!(b"a\x1b[13;2tx", c'a' m m m m m m WindowOp(WindowOp::ReportTextAreaPosition) c'x');
        pt!(b"a\x1b[14tx", c'a' m m m m WindowOp(WindowOp::ReportTextAreaSize) c'x');
        pt!(b"a\x1b[14;2tx", c'a' m m m m m m WindowOp(WindowOp::ReportWindowSize) c'x');
        pt!(b"a\x1b[15tx", c'a' m m m m WindowOp(WindowOp::ReportScreenSize) c'x');
        pt!(b"a\x1b[16tx", c'a' m m m m WindowOp(WindowOp::ReportCharacterSize) c'x');
        pt!(b"a\x1b[18tx", c'a' m m m m WindowOp(WindowOp::ReportTextAreaSizeChar) c'x');
        pt!(b"a\x1b[19tx", c'a' m m m m WindowOp(WindowOp::ReportScreenSizeChar) c'x');
        pt!(b"a\x1b[20tx", c'a' m m m m WindowOp(WindowOp::ReportIconLabel) c'x');
        pt!(b"a\x1b[21tx", c'a' m m m m WindowOp(WindowOp::ReportWindowTitle) c'x');
        pt!(b"a\x1b[22;0tx", c'a' m m m m m m WindowOp(WindowOp::PushIconAndWindowTitle) c'x');
        pt!(b"a\x1b[22;1tx", c'a' m m m m m m WindowOp(WindowOp::PushIconTitle) c'x');
        pt!(b"a\x1b[22;2tx", c'a' m m m m m m WindowOp(WindowOp::PushWindowTitle) c'x');
        pt!(b"a\x1b[23;0tx", c'a' m m m m m m WindowOp(WindowOp::PopIconAndWindowTitle) c'x');
        pt!(b"a\x1b[23;1tx", c'a' m m m m m m WindowOp(WindowOp::PopIconTitle) c'x');
        pt!(b"a\x1b[23;2tx", c'a' m m m m m m WindowOp(WindowOp::PopWindowTitle) c'x');
        pt!(b"a\x1b[24tx", c'a' m m m m WindowOp(WindowOp::ResizeLines(24)) c'x');
        pt!(b"a\x1b[>0tb", c'a' m m m m SetTitleModes(TitleModes::SetLabelHex) c'b');
        pt!(b"a\x1b[>1tb", c'a' m m m m SetTitleModes(TitleModes::GetLabelHex) c'b');
        pt!(b"a\x1b[>2tb", c'a' m m m m SetTitleModes(TitleModes::SetLabelUtf8) c'b');
        pt!(b"a\x1b[>3tb", c'a' m m m m SetTitleModes(TitleModes::GetLabelUtf8) c'b');
        pt!(b"a\x1b[>0;1tb", c'a' m m m m m m
            SetTitleModes(TitleModes::SetLabelHex | TitleModes::GetLabelHex) c'b');
        pt!(b"a\x1b[>12;14tb", c'a' m m m m m m m m SetTitleModes(TitleModes::empty()) c'b');
        pt!(b"a\x1b[0 tx", c'a' m m m m SetWarningBellVolume(0) c'x');
        pt!(b"a\x1b[8 tx", c'a' m m m m SetWarningBellVolume(8) c'x');
        pt!(b"a\x1b[9 tx", c'a' m m m m m c'x');
        pt!(b"a\x1b[0;1;2;3;0$tx", c'a' m m m m m m m m m m m m
            m c'x');
        pt!(b"a\x1b[0;1;2;3;1$tx", c'a' m m m m m m m m m m m m
            ReverseAttributesArea(0,1,2,3,CharacterAttribute::Bold) c'x');
        pt!(b"a\x1b[0;1;2;3;4$tx", c'a' m m m m m m m m m m m m
            ReverseAttributesArea(0,1,2,3,CharacterAttribute::Underlined) c'x');
        pt!(b"a\x1b[0;1;2;3;5$tx", c'a' m m m m m m m m m m m m
            ReverseAttributesArea(0,1,2,3,CharacterAttribute::Blink) c'x');
        pt!(b"a\x1b[0;1;2;3;7$tx", c'a' m m m m m m m m m m m m
            ReverseAttributesArea(0,1,2,3,CharacterAttribute::Inverse) c'x');
        pt!(b"a\x1b[ux", c'a' m m RestoreCursor c'x');
        pt!(b"a\x1b[0 ux", c'a' m m m m SetMarginBellVolume(0) c'x');
        pt!(b"a\x1b[8 ux", c'a' m m m m SetMarginBellVolume(8) c'x');
        pt!(b"a\x1b[9 ux", c'a' m m m m m c'x');
        pt!(b"a\x1b[0;1;2;3;4;5;6;7$vx", c'a' m m m m m m m m m m m m m m m m m m
            CopyArea(0,1,2,3,4,5,6,7) c'x');
        pt!(b"a\x1b[0$wx", c'a' m m m m m c'x');
        pt!(b"a\x1b[1$wx", c'a' m m m m CursorInformationReport c'x');
        pt!(b"a\x1b[2$wx", c'a' m m m m TabstopReport c'x');
        pt!(b"a\x1b[0;1;2;3'wx", c'a' m m m m m m m m m m EnableFilterArea(0,1,2,3) c'x');
        pt!(b"a\x1b[0xw", c'a' m m m RequestTerminalParameters c'w');
        pt!(b"a\x1b[1xw", c'a' m m m RequestTerminalParameters c'w');
        pt!(b"a\x1b[2xw", c'a' m m m m c'w');
        pt!(b"a\x1b[0*xw", c'a' m m m m AttributeChangeExtent(AttributeChangeExtent::Wrapped) c'w');
        pt!(b"a\x1b[1*xw", c'a' m m m m AttributeChangeExtent(AttributeChangeExtent::Wrapped) c'w');
        pt!(b"a\x1b[2*xw", c'a' m m m m AttributeChangeExtent(AttributeChangeExtent::Rectangle)
            c'w');
        pt!(b"a\x1b[3*xw", c'a' m m m m m c'w');
        pt!(b"a\x1b[0;1;2;3;4$xy", c'a' m m m m m m m m m m m m FillArea(0,1,2,3,4) c'y');
        pt!(b"a\x1b[12;0;1;2;3;4*yx", c'a' m m m m m m m m m m m m m m m ChecksumArea(12,0,1,2,3,4)
            c'x');
        pt!(b"a\x1b[0;0'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::Off,LocatorReportUnit::Character) c'b');
        pt!(b"a\x1b[1;0'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::On,LocatorReportUnit::Character) c'b');
        pt!(b"a\x1b[2;0'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::Once,LocatorReportUnit::Character) c'b');
        pt!(b"a\x1b[0;1'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::Off,LocatorReportUnit::Device) c'b');
        pt!(b"a\x1b[1;1'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::On,LocatorReportUnit::Device) c'b');
        pt!(b"a\x1b[2;1'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::Once,LocatorReportUnit::Device) c'b');
        pt!(b"a\x1b[0;2'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::Off,LocatorReportUnit::Character) c'b');
        pt!(b"a\x1b[1;2'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::On,LocatorReportUnit::Character) c'b');
        pt!(b"a\x1b[2;2'zb", c'a' m m m m m m
            LocatorReport(LocatorReportEnable::Once,LocatorReportUnit::Character) c'b');
        pt!(b"a\x1b[3;2'zb", c'a' m m m m m m m c'b');
        pt!(b"a\x1b[2;3'zb", c'a' m m m m m m m c'b');

        pt!(b"a\x1b[0;1;2;3$zc", c'a' m m m m m m m m m m EraseArea(0,1,2,3) c'c');
    }
}
