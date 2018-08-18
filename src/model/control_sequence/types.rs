/*
    BiTE - Bash-integrated Terminal
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
#[allow(dead_code)]
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
    CSI_HASH_STATE,
    XTERM_PUSH_SGR,
    XTERM_REPORT_SGR,
    XTERM_POP_SGR,
    DECRQPSR,
    DECSCPP,
    DECSNLS,

    NUM_CASES,
}

const MAX_CONTROL_VALUE: u8 = 128;

pub type CaseTable = [Case; MAX_CONTROL_VALUE as usize];
