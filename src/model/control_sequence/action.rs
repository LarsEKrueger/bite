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

    DECREQTPARM,

    SaveCursor,
    RestoreCursor,

    HorizontalMove(u32),

    VerticalPositionAbsolute(u32),
    VerticalPositionRelative(u32),

    DA1(u32),
    DA2(u32),

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
    CursorBackwardTab(u32),

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

    EraseCharacters(u32),
    RepeatCharacter(u32),

    ScrollUp(u32),
    ScrollDown(u32),

    GraphicRegister(GraReg, GraOp),

    /// The 5 parameters of the sequence
    MouseTracking(u32, u32, u32, u32, u32),

    SetTitleModes(TitleModes),
    ResetTitleModes(TitleModes),

    TabClear(TabClear),
    SetMode(SetMode),
    ResetMode(SetMode),
    RequestAnsiMode(SetMode),
    SetPrivateMode(SetPrivateMode),
    ResetPrivateMode(SetPrivateMode),
    RequestPrivateMode(SetPrivateMode),
    SavePrivateMode(SetPrivateMode),
    RestorePrivateMode(SetPrivateMode),
    MediaCopy(MediaCopy),

    CharacterAttributes(Vec<CharacterAttribute>),
    ForegroundColorRgb(u8, u8, u8),
    ForegroundColorIndex(u8),
    BackgroundColorRgb(u8, u8, u8),
    BackgroundColorIndex(u8),

    SetModFKeys(FKeys, u32),
    DisableModFKeys(FKeys),
    StatusReport,
    ReportCursorPosition,

    DecDeviceStatusReport,
    PrinterStatusReport,
    UdkStatusReport,
    KeyboardStatusReport,
    LocatorStatusReport,
    LocatorTypeReport,
    MacroStatusReport,
    MemoryStatusReport(u32),
    DataIntegrityReport,
    MultiSessionReport,

    PointerMode(PointerMode),
    SoftReset,

    /// Conformance Level
    ///
    /// Terminal, 8 bit
    ConformanceLevel(Terminal, bool),

    /// Led, true=on
    LoadLeds(LoadLeds, bool),

    CursorStyle(CursorStyle),
    CharacterProtection(CharacterProtection),

    /// Scroll region.
    ///
    /// top, bottom. Scroll region is exclusive, i.e. if bottom is one more than top, the region is
    /// one line.
    /// (0,0) means region is the full window.
    ScrollRegion(u32, u32),

    /// Change Attributes in area
    ///
    /// top, left, bottom, right, attribute. Range is exclusive.
    ChangeAttributesArea(u32, u32, u32, u32, CharacterAttribute),

    /// Set left and right margins
    ///
    /// left, right. Range in exclusive.
    SetMargins(u32, u32),

    WindowOp(WindowOp),
    SetBellVolume(u8),
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

#[derive(Debug, PartialEq)]
pub enum TabClear {
    All,
    Column,
}

#[derive(Debug, PartialEq)]
pub enum SetMode {
    KeyboardAction,
    Insert,
    SendReceive,
    AutomaticNewline,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum SetPrivateMode {
    ApplicationCursorKeys,
    UsAsciiForG0toG3,
    Hundred32Columns,
    SmoothScroll,
    ReverseVideo,
    OriginMode,
    AutoWrapMode,
    AutoRepeatKeys,
    SendMousePosOnPress,
    ShowToolbar,
    StartBlinkingCursor,
    EnableXorBlinkingCursor,
    PrintFormFeed,
    PrintFullScreen,
    ShowCursor,
    ShowScrollbar,
    EnableFontShifting,
    TektronixMode,
    AllowHundred32Mode,
    MoreFix,
    EnableNrc,
    MarginBell,
    ReverseWrapAroundMode,
    StartLogging,
    AlternateScreenBuffer,
    ApplicationKeypad,
    BackArrowIsBackSspace,
    EnableLeftRightMarginMode,
    NoClearScreenOnDECCOLM,
    SendMousePosOnBoth,
    HiliteMouseTracking,
    CellMouseTracking,
    AllMouseTracking,
    SendFocusEvents,
    Utf8MouseMode,
    SgrMouseMode,
    AlternateScrollMode,
    ScrollToBottomOnTty,
    ScrollToBottomOnKey,
    UrxvtMouseMode,
    InterpretMetaKey,
    EnableSpecialModifiers,
    SendEscOnMeta,
    SendDelOnKeypad,
    SendEscOnAlt,
    KeepSelection,
    UseClipboard,
    UrgencyHint,
    RaiseWindowOnBell,
    KeepClipboard,
    EnableAlternateScreen,
    UseAlternateScreen,
    SaveCursor,
    SaveCursorAndUseAlternateScreen,
    TerminfoFnMode,
    SunFnMode,
    HpFnMode,
    ScoFnMode,
    LegacyKeyboard,
    Vt220Keyboard,
    BracketedPaste,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum MediaCopy {
    PrintScreen,
    PrinterCtrlOff,
    PrinterCtrlOn,
    HtmlScreenDump,
    SvgScreenDump,
    PrintCursorLine,
    AutoPrintOff,
    AutoPrintOn,
    PrintComposeDisplay,
    PrintAllPages,
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

#[derive(Debug, PartialEq)]
pub enum CharacterAttribute {
    Normal,
    Bold,
    Faint,
    Italicized,
    Underlined,
    Blink,
    Inverse,
    Invisible,
    CrossedOut,
    DoublyUnderlined,
    NotBoldFaint,
    NotItalicized,
    NotUnderlined,
    Steady,
    Positive,
    Visible,
    NotCrossedOut,
    Foreground(Color),
    Background(Color),
}

#[derive(Debug, PartialEq)]
pub enum Color {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Grey,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

#[derive(Debug, PartialEq)]
pub enum FKeys {
    Keyboard,
    Cursor,
    Function,
    Other,
}

#[derive(Debug, PartialEq)]
pub enum PointerMode {
    NeverHide,
    HideNotTracking,
    HideOutside,
    AlwaysHide,
}

#[derive(Debug, PartialEq)]
pub enum Terminal {
    Vt100,
    Vt200,
    Vt300,
}

#[derive(Debug, PartialEq)]
pub enum LoadLeds {
    All,
    NumLock,
    CapsLock,
    ScrollLock,
}

#[derive(Debug, PartialEq)]
pub enum CursorStyle {
    BlinkBlock,
    SteadyBlock,
    BlinkUnderline,
    SteadyUnderline,
    BlinkBar,
    SteadyBar,
}

#[derive(Debug, PartialEq)]
pub enum CharacterProtection {
    CanErase,
    NoErase,
}

#[derive(Debug, PartialEq)]
pub enum WindowOp {
    DeIconify,
    Iconify,
    Move(u32, u32),
    /// If None, don't change. If zero, use display size.
    ResizeWindow(Option<u32>, Option<u32>),
    Raise,
    Lower,
    Refresh,
    /// If None, don't change. If zero, use display size.
    ResizeTextArea(Option<u32>, Option<u32>),
    RestoreMaximized,
    MaximizeWindow,
    MaximizeVertically,
    MaximizeHorizontally,
    UndoFullscreen,
    Fullscreen,
    ToggleFullscreen,
    ReportWindowState,
    ReportWindowPosition,
    ReportTextAreaPosition,
    ReportTextAreaSize,
    ReportWindowSize,
    ReportScreenSize,
    ReportCharacterSize,
    ReportTextAreaSizeChar,
    ReportScreenSizeChar,
    ReportIconLabel,
    ReportWindowTitle,
    PushIconAndWindowTitle,
    PushIconTitle,
    PushWindowTitle,
    PopIconAndWindowTitle,
    PopIconTitle,
    PopWindowTitle,
    ResizeLines(u32),
}

impl Action {
    pub fn char_from_u32(byte: u32) -> Action {
        Action::Char(unsafe { char::from_u32_unchecked(byte as u32) })
    }
}
