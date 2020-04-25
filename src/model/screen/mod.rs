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

//! Data structure to hold a part of the screen.
//!
//! This stores a matrix of cells, which are colored characters.

use std::cmp;
use std::hash::{Hash, Hasher};

use super::control_sequence::action::{
    Action, CharSet, CharacterAttribute, Color, EraseDisplay, EraseLine, ScrollRegion, ScsType,
};
use super::control_sequence::parser::Parser;
use super::control_sequence::types::Rectangle;

mod charset;
mod test;

/// Colors are pairs of foreground/background indices into the same palette.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Colors {
    /// Foreground color, index into a 256-entry color table
    foreground: u8,

    /// Background color, index into a 256-entry color table
    background: u8,
}

impl PartialEq for Colors {
    fn eq(&self, other: &Colors) -> bool {
        self.foreground == other.foreground && self.background == other.background
    }
}

impl Colors {
    fn fromColor(c: Color) -> u8 {
        match c {
            Color::Default => 0,
            Color::Black => 0,
            Color::Red => 1,
            Color::Green => 2,
            Color::Yellow => 3,
            Color::Blue => 4,
            Color::Magenta => 5,
            Color::Cyan => 6,
            Color::White => 7,
            Color::Grey => 8,
            Color::BrightRed => 9,
            Color::BrightGreen => 10,
            Color::BrightYellow => 11,
            Color::BrightBlue => 12,
            Color::BrightMagenta => 13,
            Color::BrightCyan => 14,
            Color::BrightWhite => 15,
        }
    }
}

/// A cell is a character and its colors and attributes.
///
/// TODO: Pack data more tightly
#[derive(Clone, Copy, Debug, Hash)]
pub struct Cell {
    /// The unicode character to show
    code_point: char,

    /// Attributes as a bit field
    attributes: Attributes,

    /// Colors of this cell
    colors: Colors,
}

impl Cell {
    pub fn new(colors: Colors) -> Self {
        Self {
            code_point: ' ',
            attributes: Attributes::empty(),
            colors,
        }
    }

    pub fn with_attr(colors: Colors, attributes: Attributes) -> Self {
        Self {
            code_point: ' ',
            attributes,
            colors,
        }
    }

    /// Return the color index of the foreground color of the cell.
    ///
    /// If the cell is bold and the color is < 8, return the brighter version.
    pub fn foreground_color(&self) -> Option<u8> {
        if self.attributes.contains(Attributes::FG_COLOR) {
            if self.attributes.contains(Attributes::BOLD) && self.colors.foreground < 8 {
                Some(self.colors.foreground + 8)
            } else {
                Some(self.colors.foreground)
            }
        } else {
            None
        }
    }

    pub fn background_color(&self) -> Option<u8> {
        if self.attributes.contains(Attributes::BG_COLOR) {
            Some(self.colors.background)
        } else {
            None
        }
    }

    pub fn encode_utf8<'a>(&self, buf: &'a mut [u8]) -> &'a mut str {
        self.code_point.encode_utf8(buf)
    }

    pub fn code_point(&self) -> char {
        self.code_point
    }

    pub fn drawn(&self) -> bool {
        self.attributes.contains(Attributes::CHARDRAWN)
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Cell) -> bool {
        if self.code_point != other.code_point {
            return false;
        }
        if self.attributes != other.attributes {
            return false;
        }

        // Colors only matter if they have been set
        if self.foreground_color() != other.foreground_color() {
            return false;
        }
        if self.background_color() != other.background_color() {
            return false;
        }
        true
    }
}

// Attributes as bitflags
bitflags! {
    pub struct Attributes: u16 {
        const INVERSE       = 0b0000000000001;
        const UNDERLINE     = 0b0000000000010;
        const BOLD          = 0b0000000000100;
        const BLINK         = 0b0000000001000;
        /// true if background set
        const BG_COLOR      = 0b0000000010000;
        /// true if foreground set
        const FG_COLOR      = 0b0000000100000;
        /// a character that cannot be erased
        const PROTECTED     = 0b0000001000000;
        /// a character has been drawn here on the screen.  Used to distinguish blanks from empty
        /// parts of the screen when selecting
        const CHARDRAWN     = 0b0000010000000;

        const ATR_FAINT     = 0b0000100000000;
        const ATR_ITALIC    = 0b0001000000000;
        const ATR_STRIKEOUT = 0b0010000000000;
        const ATR_DBL_UNDER = 0b0100000000000;
        const INVISIBLE     = 0b1000000000000;

        const SGR_MASK2     = Self::ATR_FAINT.bits | Self::ATR_ITALIC.bits |
                              Self::ATR_STRIKEOUT.bits | Self::ATR_DBL_UNDER.bits;

        /// mask for video-attributes only
        const SGR_MASK      = Self::BOLD.bits | Self::BLINK.bits | Self::UNDERLINE.bits |
                              Self::INVERSE.bits;

        /// mask: user-visible attributes
        const ATTRIBUTES    = Self::SGR_MASK.bits | Self::SGR_MASK2.bits | Self::BG_COLOR.bits |
                              Self::FG_COLOR.bits | Self::PROTECTED.bits | Self::INVISIBLE.bits;

        /// The toplevel-call to drawXtermText() should have text-attributes guarded:
        const DRAWX_MASK    = Self::ATTRIBUTES.bits | Self::CHARDRAWN.bits;
    }
}

/// A matrix is a rectangular area of cells.
///
/// A matrix is meant to be stored, but not modified.
///
/// Be aware that a matrix can have width of 0, but a non-zero height. This is caused by adding
/// newlines to an empty screen.
#[derive(Clone)]
pub struct Matrix {
    /// The cells of the screen, stored in a row-major ordering.
    cells: Vec<Cell>,

    /// Width of screen fragment in cells. This refers to the allocated size.
    width: isize,

    /// Height of screen fragment in cells. This refers to the allocated size.
    height: isize,
}

impl Matrix {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn rows(&self) -> isize {
        self.height
    }

    pub fn columns(&self) -> isize {
        self.width
    }

    /// Compute the index into cells given valid x and y coordinates.
    ///
    /// Must not be called for matrices of width==0.
    fn cell_index(&self, x: isize, y: isize) -> isize {
        debug_assert!(0 <= x);
        debug_assert!(x < self.width);
        debug_assert!(0 <= y);
        debug_assert!(y < self.height);

        (x + y * self.width)
    }

    #[allow(dead_code)]
    fn cell_at(&self, x: isize, y: isize) -> Cell {
        self.cells[self.cell_index(x, y) as usize]
    }

    #[allow(dead_code)]
    fn cell_at_mut(&mut self, x: isize, y: isize) -> &mut Cell {
        let index = self.cell_index(x, y) as usize;
        &mut self.cells[index]
    }

    pub fn compacted_row_slice(&self, row: isize) -> &[Cell] {
        if self.width == 0 {
            // Return an empty slice
            &self.cells[0..0]
        } else {
            let row_start = self.cell_index(0, row);
            let mut row_end = self.cell_index(self.width - 1, row);
            while row_end >= row_start {
                if self.cells[row_end as usize].drawn() {
                    break;
                }
                row_end -= 1;
            }

            // If we have seen an empty row, row_end < row_start. If this happens in the first row, we
            // would underflow when casting to usize for slicing, thus we update now and then cast.
            row_end += 1;
            let row_start = row_start as usize;
            let row_end = row_end as usize;

            &self.cells[row_start..row_end]
        }
    }

    pub fn row_slice(&self, row: isize) -> &[Cell] {
        if self.width == 0 {
            // Return an empty slice
            &self.cells[0..0]
        } else {
            let row_start = self.cell_index(0, row);
            let row_end = self.cell_index(self.width - 1, row);
            let row_start = row_start as usize;
            let row_end = row_end as usize;

            &self.cells[row_start..row_end]
        }
    }

    pub fn compacted_row(&self, row: isize) -> Vec<Cell> {
        // Special case for matrix of width 0
        if self.width == 0 {
            Vec::new()
        } else {
            self.compacted_row_slice(row).to_vec()
        }
    }

    pub fn line_iter(&self) -> impl Iterator<Item = &[Cell]> {
        (0..self.height).map(move |r| self.compacted_row_slice(r))
    }

    pub fn line_iter_full(&self) -> impl Iterator<Item = &[Cell]> {
        (0..self.height).map(move |r| self.row_slice(r))
    }

    pub fn reset(&mut self) {
        self.cells.clear();
        self.width = 0;
        self.height = 0;
    }

    pub fn rectangle(&self) -> Rectangle {
        Rectangle::new_isize(0, 0, self.width - 1, self.height - 1)
    }

    pub fn first_row_cell_vec(&self) -> Vec<Cell> {
        if self.height == 0 {
            Vec::new()
        } else {
            self.compacted_row(0)
        }
    }
}

impl PartialEq for Matrix {
    /// Visual equality. If it looks the same, it's the same.
    fn eq(&self, other: &Matrix) -> bool {
        self.width == other.width && self.height == other.height && self.cells == other.cells
    }
}

impl Hash for Matrix {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.line_iter().for_each(|r| Hash::hash_slice(r, state));
    }
}

/// Events that happen during adding bytes.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// Nothing to do.
    Ignore,

    /// Newline was seen.
    NewLine,

    /// Carriage-return was seen.
    Cr,

    /// Ring the bell (or make the screen flash)
    Bell,

    /// Start TUI mode
    StartTui,
}

#[derive(Copy, Clone)]
struct Cursor {
    /// Horizontal cursor position. Might be negative.
    x: isize,

    /// Vertical cursor position. Might be negative.
    y: isize,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum AddBytesResult<'a> {
    /// All bytes have been added
    AllDone,

    /// This stream needs to be shown before the rest of the bytes can be processed.
    ShowStream(&'a [u8]),

    /// Switch to TUI mode before the rest of the bytes can be processed.
    StartTui(&'a [u8]),
}

/// A screen is rectangular area of cells and the position of the cursor.
///
/// The cursor can be outside the allocated screen. If a visible character is inserted there, the
/// screen is reallocated. Coordinate system origin is top-left with x increasing to the right and
/// y down.
pub struct Screen {
    /// A matrix of cells
    matrix: Matrix,

    /// Cursor position
    cursor: Cursor,

    /// Saved cursor position
    saved_cursor: Cursor,

    /// Attributes for next character
    attributes: Attributes,

    /// Colors for next character
    colors: Colors,

    /// State for the state machine to interpret the byte stream as a terminal.
    parser: Parser,

    /// Shall the screen keep it size?
    fixed_size: bool,

    /// Last printed character
    last_char: char,

    /// Scroll region.
    ///
    /// The values will be checked every time as non-fixed_size screens might change them.
    scroll_region: ScrollRegion,

    /// Character set for G0 - G3
    gsets: [CharSet; ScsType::NUM as usize],

    /// Character set for characters < 128
    curgl: ScsType,

    /// Character set for characters >= 128
    curgr: ScsType,
}

const INITIAL_COLORS: Colors = Colors {
    foreground: 1,
    background: 0,
};

impl Cursor {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Screen {
    /// Create a new, empty screen
    pub fn new() -> Self {
        Self {
            matrix: Matrix::new(),
            cursor: Cursor::new(),
            saved_cursor: Cursor::new(),
            attributes: Attributes::empty(),
            colors: INITIAL_COLORS,
            parser: Parser::new(),
            fixed_size: false,
            last_char: ' ',
            scroll_region: None,
            gsets: [
                CharSet::UsAscii,
                CharSet::UsAscii,
                CharSet::Latin1,
                CharSet::UsAscii,
            ],
            curgl: ScsType::G0,
            curgr: ScsType::G2,
        }
    }

    /// Create a new screen from the given matrix
    pub fn new_from_matrix(matrix: Matrix) -> Self {
        Self {
            matrix,
            cursor: Cursor::new(),
            saved_cursor: Cursor::new(),
            attributes: Attributes::empty(),
            colors: INITIAL_COLORS,
            parser: Parser::new(),
            fixed_size: false,
            last_char: ' ',
            scroll_region: None,
            gsets: [
                CharSet::UsAscii,
                CharSet::UsAscii,
                CharSet::Latin1,
                CharSet::UsAscii,
            ],
            curgl: ScsType::G0,
            curgr: ScsType::G2,
        }
    }

    /// Direct conversion to one-line vector of cells
    pub fn one_line_cell_vec(line: &[u8]) -> Vec<Cell> {
        if line.is_empty() {
            Vec::new()
        } else {
            Self::one_line_matrix(line).first_row_cell_vec()
        }
    }

    /// Direct conversion to one-line matrix
    pub fn one_line_matrix(bytes: &[u8]) -> Matrix {
        let mut s = Screen::new();
        let _ = s.add_bytes(bytes);
        s.freeze()
    }

    /// Reset the screen to initial values.
    pub fn reset(&mut self) {
        self.matrix.reset();
        self.cursor = Cursor::new();
        self.attributes = Attributes::empty();
        self.colors = INITIAL_COLORS;
        self.parser.reset();
        self.fixed_size = false;
    }

    /// Mark screen as fixed-size
    pub fn fixed_size(&mut self, nx: usize, ny: usize) {
        self.fixed_size = true;
        self.make_room_for((nx - 1) as isize, (ny - 1) as isize);
        trace!("fixed_size: {}x{}", nx, ny);
    }

    /// Get width of matrix
    pub fn width(&self) -> isize {
        self.matrix.width
    }

    /// Get height of matrix
    pub fn height(&self) -> isize {
        self.matrix.height
    }

    /// Cursor position, x coordinate
    pub fn cursor_x(&self) -> isize {
        self.cursor.x
    }

    /// Cursor position, y coordinate
    pub fn cursor_y(&self) -> isize {
        self.cursor.y
    }

    /// Place the cursor, accounting for margins later
    fn move_cursor_to(&mut self, x: isize, y: isize) {
        if self.fixed_size {
            self.cursor.x = cmp::min(self.width() - 1, cmp::max(0, x));
            self.cursor.y = cmp::min(self.height() - 1, cmp::max(0, y));
        } else {
            self.cursor.x = x;
            self.cursor.y = y;
        }
    }

    pub fn line_iter(&self) -> impl Iterator<Item = &[Cell]> {
        self.matrix.line_iter()
    }

    pub fn line_iter_full(&self) -> impl Iterator<Item = &[Cell]> {
        self.matrix.line_iter_full()
    }

    pub fn row_slice(&self, row: isize) -> &[Cell] {
        self.matrix.row_slice(row)
    }
    pub fn compacted_row_slice(&self, row: isize) -> &[Cell] {
        self.matrix.compacted_row_slice(row)
    }

    /// Check if the cursor is at the end of the line
    pub fn cursor_at_end_of_line(&self) -> bool {
        if 0 <= self.cursor.y && self.cursor.y < self.height() {
            let line = self.matrix.compacted_row_slice(self.cursor.y);
            self.cursor.x == line.len() as isize
        } else {
            false
        }
    }

    /// Check if the cursor is at the end of the last line
    pub fn cursor_at_end(&self) -> bool {
        if self.cursor.y + 1 == self.height() {
            let line = self.matrix.compacted_row_slice(self.cursor.y);
            self.cursor.x == line.len() as isize
        } else {
            false
        }
    }

    /// Check if the frozen representation of the screen looks different that the given matrix
    pub fn looks_different(&self, other: &Matrix) -> bool {
        self.matrix != *other
    }

    /// Return the whole text on screen as a string with new lines.
    pub fn extract_text(&self) -> String {
        let mut text = String::new();
        for l in self.line_iter() {
            for c in l {
                text.push(c.code_point);
            }
            text.push('\n');
        }
        text
    }

    /// Return the whole text on screen as a string with new lines.
    ///
    /// The last line does not have a new line.
    pub fn extract_text_without_last_nl(&self) -> String {
        let mut text = String::new();
        let mut place_nl = false;
        for l in self.line_iter() {
            if place_nl {
                text.push('\n');
            }
            place_nl = true;
            for c in l {
                text.push(c.code_point);
            }
        }
        text
    }

    fn collect_text(&self, start_index: usize, end_index: usize) -> String {
        let mut text = String::new();
        let mut current_index = start_index;
        while current_index < end_index {
            text.push(self.matrix.cells[current_index].code_point);
            current_index += 1;
        }
        text
    }

    pub fn text_before_cursor(&mut self) -> String {
        self.make_room();
        self.collect_text(
            self.matrix.cell_index(0, self.cursor.y) as usize,
            self.cursor_index() as usize,
        )
    }

    fn index_word_before_cursor(&mut self) -> usize {
        self.make_room();
        // End at the beginning of the line
        let start_index = self.matrix.cell_index(0, self.cursor.y) as usize;
        // Start at the cursor
        let cursor_index = self.cursor_index() as usize;
        let mut current_index = cursor_index;
        // If we can move one character backwards
        while current_index > start_index {
            current_index -= 1;
            if self.matrix.cells[current_index]
                .code_point
                .is_ascii_whitespace()
            {
                // White space found, go to the character after that, then leave
                current_index += 1;
                break;
            }
        }
        current_index
    }

    /// Scan backwards from cursor to the first whitespace
    pub fn word_before_cursor(&mut self) -> String {
        let current_index = self.index_word_before_cursor();
        let cursor_index = self.cursor_index() as usize;
        self.collect_text(current_index, cursor_index)
    }

    pub fn delete_word_before_cursor(&mut self) {
        let current_index = self.index_word_before_cursor();
        let cell = self.clone_cell(' ');
        let cursor_index = self.cursor_index() as usize;
        for index in current_index..cursor_index {
            self.matrix.cells[index] = cell;
        }
        self.cursor.x -= (cursor_index - current_index) as isize;
    }

    pub fn replace(&mut self, s: &str, stay_there: bool) {
        let x = self.cursor.x;
        self.reset();
        self.place_str(s);
        if stay_there {
            self.cursor.x = x;
        }
    }

    /// Place a character at the current position and advance the cursor
    pub fn place_char(&mut self, c: char) {
        self.make_room();
        let idx = self.cursor_index();
        self.matrix.cells[idx] = Cell {
            code_point: c,
            attributes: self.attributes | Attributes::CHARDRAWN,
            colors: self.colors,
        };
        self.cursor.x += 1;
        if self.fixed_size {
            if self.cursor.x == self.width() {
                self.new_line();
            }
        }
    }

    /// Return a new cell with current colors and attributes
    fn clone_cell(&self, c: char) -> Cell {
        Cell {
            code_point: c,
            colors: self.colors,
            attributes: self.attributes,
        }
    }

    /// Find start and end row of region to scroll.
    ///
    /// Both rows are inside the screen. We assume make_room has been called before.
    /// Also indicate if there is a region (either due to manual request or fixed_size).
    fn determine_scroll_region(&self) -> (isize, isize, bool) {
        match self.scroll_region {
            None => (0, self.height() - 1, self.fixed_size),
            Some((start, end)) => {
                let start = start as isize;
                let end = end as isize;
                if 0 <= start
                    && start < self.height()
                    && 0 <= end
                    && end < self.height()
                    && start < end
                {
                    (start, end, true)
                } else {
                    (0, self.height() - 1, true)
                }
            }
        }
    }

    /// Scroll the character matrix up by n rows and fill the last rows with fresh cells.
    /// Everything below and including at_row will scroll up.
    fn scroll_up(&mut self, at_row: isize, scroll_rows: isize) {
        let (start_row, end_row, _) = self.determine_scroll_region();
        let scroll_rows = cmp::min(scroll_rows, end_row + 1 - start_row);
        debug_assert!(start_row <= at_row && at_row <= end_row);
        if scroll_rows >= 1 {
            // Scroll up
            let w = self.width() as usize;
            for src_row in (at_row + scroll_rows)..(end_row + 1) {
                let dst_index = self.matrix.cell_index(0, src_row - scroll_rows) as usize;
                let src_index = self.matrix.cell_index(0, src_row) as usize;
                for col in 0..w {
                    self.matrix.cells[dst_index + col] = self.matrix.cells[src_index + col];
                }
            }
            for dst_row in (end_row + 1 - scroll_rows)..(end_row + 1) {
                let dst_index = self.matrix.cell_index(0, dst_row) as usize;
                for col in 0..w {
                    self.matrix.cells[dst_index + col] = Cell::new(self.colors);
                }
            }
        }
    }

    /// Scroll the line left by n columns and fill the last columns with fresh cells.
    /// Everything right of column at_col is scrolled left
    ///
    /// We assume the parameters are valid.
    fn scroll_left_line(&mut self, at_row: isize, at_col: isize, scroll_cols: isize) {
        let row_index = self.matrix.cell_index(0, at_row);
        let mut dst_index = row_index + at_col;
        let mut n_to_move = self.width() - scroll_cols - at_col;
        while n_to_move > 0 {
            self.matrix.cells[dst_index as usize] =
                self.matrix.cells[(dst_index + scroll_cols) as usize];
            dst_index += 1;
            n_to_move -= 1;
        }
        // Clear last cols
        let mut n_to_clear = scroll_cols;
        while n_to_clear > 0 {
            self.matrix.cells[dst_index as usize] = Cell::new(self.colors);
            dst_index += 1;
            n_to_clear -= 1;
        }
    }

    /// Scroll the character matrix left by n columns and fill the last columns with fresh cells.
    /// Everything right of column at_col is scrolled left
    fn scroll_left(&mut self, at_col: isize, scroll_cols: isize) {
        let scroll_cols = cmp::min(scroll_cols, self.width());
        let at_col = cmp::max(at_col, 0);
        if scroll_cols >= 1 && at_col < self.width() {
            let (start_row, end_row, _) = self.determine_scroll_region();
            for row in start_row..(end_row + 1) {
                self.scroll_left_line(row, at_col, scroll_cols);
            }
        }
    }

    /// Scroll the character matrix down by n rows and fill the first rows with fresh cells.
    /// Every below of and including at_row is scrolled down.
    fn scroll_down(&mut self, at_row: isize, scroll_rows: isize) {
        let (start_row, end_row, _) = self.determine_scroll_region();
        let scroll_rows = cmp::min(scroll_rows, end_row + 1 - start_row);
        debug_assert!(start_row <= at_row && at_row <= end_row);
        if scroll_rows >= 1 {
            // Scroll down
            let w = self.width() as usize;
            let mut dst_row = end_row;
            while dst_row >= at_row + scroll_rows {
                let src_index = self.matrix.cell_index(0, dst_row - scroll_rows) as usize;
                let dst_index = self.matrix.cell_index(0, dst_row) as usize;
                for col in 0..w {
                    self.matrix.cells[dst_index + col] = self.matrix.cells[src_index + col];
                }
                dst_row -= 1;
            }
            for dst_row in at_row..(at_row + scroll_rows) {
                let dst_index = self.matrix.cell_index(0, dst_row) as usize;
                for col in 0..w {
                    self.matrix.cells[dst_index + col] = Cell::new(self.colors);
                }
            }
        }
    }

    /// Scroll one line of the character matrix right by n columns and fill the gap with fresh
    /// cells.
    ///
    /// We assume the parameters are valid.
    fn scroll_right_line(&mut self, at_row: isize, at_col: isize, scroll_cols: isize) {
        let row_index = self.matrix.cell_index(0, at_row);
        let mut n_to_move = self.width() - at_col - scroll_cols;
        let mut dst_index = row_index + self.width();
        while n_to_move > 0 {
            dst_index -= 1;
            self.matrix.cells[dst_index as usize] =
                self.matrix.cells[(dst_index - scroll_cols) as usize];
            n_to_move -= 1;
        }
        let c0 = at_col;
        let c1 = at_col + scroll_cols;
        for col in c0..c1 {
            self.matrix.cells[(row_index + col) as usize] = Cell::new(self.colors);
        }
    }

    /// Scroll the character matrix right by n columns and fill the gap with fresh cells.
    /// All columns, including at_col are moved to the right
    fn scroll_right(&mut self, at_col: isize, scroll_cols: isize) {
        let scroll_cols = cmp::min(scroll_cols, self.width());
        let at_col = cmp::max(at_col, 0);
        if scroll_cols >= 1 && at_col < self.width() {
            let (start_row, end_row, _) = self.determine_scroll_region();
            for row in start_row..(end_row + 1) {
                self.scroll_right_line(row, at_col, scroll_cols);
            }
        }
    }

    pub fn place_str(&mut self, s: &str) {
        for c in s.chars() {
            self.place_char(c);
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_character();
            self.place_char(c);
        }
    }

    /// Ensure that there is room to insert a character at the current position.
    pub fn make_room(&mut self) {
        if self.fixed_size {
            self.cursor.x = cmp::max(0, cmp::min(self.width() - 1, self.cursor.x));
            self.cursor.y = cmp::max(0, cmp::min(self.height() - 1, self.cursor.y));
        } else {
            // TODO: Also update the saved cursor position
            let (x, y) = (self.cursor.x, self.cursor.y);
            let (nx, ny) = self.make_room_for(x, y);
            self.cursor.x = nx;
            self.cursor.y = ny;
        }
    }

    /// Ensure that the height of the matrix is sufficient to include the cursor.
    ///
    /// Do not change the width.
    pub fn make_vertical_room(&mut self) {
        if !self.fixed_size {
            if self.width() == 0 {
                // Width is zero, no memory has been allocated. Ensure correct setting of height
                // and cursor.y.
                let y = self.cursor.y;
                if y < 0 || y >= self.height() {
                    let add_top = -cmp::min(y, 0);
                    let add_bottom = cmp::max(y, self.height() - 1) - self.height() + 1;
                    let new_h = self.height() + add_top + add_bottom;
                    self.matrix.height = new_h;
                }
            } else {
                // Matrix is at least one character wide. Ensure that a character could be placed
                // in the last column of the cursor row.
                self.make_room_for(self.matrix.width - 1, self.cursor.y);
            }
        }
    }

    /// Ensure that there is room to place a character at (x,y)
    ///
    /// Return the corrected position
    pub fn make_room_for(&mut self, x: isize, y: isize) -> (isize, isize) {
        if x < 0 || x >= self.width() || y < 0 || y >= self.height() {
            // Compute the new size and allocate
            let add_left = -cmp::min(x, 0);
            let add_right = cmp::max(x, self.width() - 1) - self.width() + 1;
            let add_top = -cmp::min(y, 0);
            let add_bottom = cmp::max(y, self.height() - 1) - self.height() + 1;

            let new_w = self.width() + add_left + add_right;
            let new_h = self.height() + add_top + add_bottom;

            let mut new_matrix = Vec::new();
            new_matrix.resize((new_w * new_h) as usize, Cell::new(self.colors));

            // Move the old content into the new matrix
            for y in 0..self.height() {
                let new_start = (new_w * (y + add_top) + add_left) as usize;
                let new_end = new_start + self.width() as usize;
                let old_start = (self.width() * y) as usize;
                let old_end = old_start + self.width() as usize;
                new_matrix[new_start..new_end]
                    .copy_from_slice(&self.matrix.cells[old_start..old_end]);
            }
            self.matrix.cells = new_matrix;

            // Fix cursor position and size
            self.matrix.width = new_w;
            self.matrix.height = new_h;
            (x + add_left, y + add_top)
        } else {
            (x, y)
        }
    }

    /// Compute the index of the cursor position into the cell array
    fn cursor_index(&self) -> usize {
        self.matrix.cell_index(self.cursor.x, self.cursor.y) as usize
    }

    /// Move the cursor to the left edge
    pub fn move_left_edge(&mut self) {
        self.cursor.x = 0;
    }

    /// Move cursor to the right edge. Moves it past the last possible character.
    pub fn move_right_edge(&mut self) {
        self.cursor.x = self.width();
    }

    /// Move cursor to the top edge
    pub fn move_top_edge(&mut self) {
        self.cursor.y = 0;
    }

    /// Move cursor to bottom edge. Moves it past the last possible character.
    pub fn move_bottom_edge(&mut self) {
        self.cursor.y = self.height();
    }

    /// Move one cell to the right
    pub fn move_right(&mut self, n: isize) {
        let c = self.cursor;
        self.move_cursor_to(c.x + n, c.y);
    }

    /// Move one cell to the left
    pub fn move_left(&mut self, n: isize) {
        let c = self.cursor;
        self.move_cursor_to(c.x - n, c.y);
    }

    /// Move n lines down. Stop at the border for fixed-sized screens.
    pub fn move_down(&mut self, n: isize) {
        let c = self.cursor;
        self.move_cursor_to(c.x, c.y + n);
    }

    /// Move n lines up. Stop at the border for fixed-sized screens.
    pub fn move_up(&mut self, n: isize) {
        let c = self.cursor;
        self.move_cursor_to(c.x, c.y - n);
    }

    /// Move to end of current line
    pub fn move_end_of_line(&mut self) {
        if 0 <= self.cursor.y && self.cursor.y < self.height() {
            let line = self.matrix.compacted_row_slice(self.cursor.y);
            self.cursor.x = line.len() as isize;
        }
    }

    /// Move the cursor down n lines and scroll up if necessary
    pub fn move_down_and_scroll(&mut self, n: isize) {
        // If we are outside an existing scroll region, do not scroll
        if let Some((start_row, end_row)) = self.scroll_region {
            let start_row = start_row as isize;
            let end_row = end_row as isize;
            if self.cursor.y < start_row || self.cursor.y > end_row {
                self.cursor.y = cmp::min(self.cursor.y + n, self.height());
                return;
            }
        }
        // Either we're inside a scroll region or there is none
        self.cursor.y += n;
        let (start_row, end_row, limited) = self.determine_scroll_region();
        if limited && self.cursor.y > end_row {
            let n = self.cursor.y - end_row;
            self.scroll_up(start_row, n);
            self.cursor.y -= n;
        }
    }

    /// Move the cursor up n lines and scroll down if necessary
    pub fn move_up_and_scroll(&mut self, n: isize) {
        self.cursor.y -= n;
        let (start_row, _, limited) = self.determine_scroll_region();
        if limited && self.cursor.y < start_row {
            let n = start_row - self.cursor.y;
            self.scroll_down(start_row, n);
            self.cursor.y += n;
        }
    }

    pub fn new_line(&mut self) {
        // Ensures a physical line to be there before we start a new one. make_room() is not
        // appropriate here as that would allocate memory to place a character at the end of the
        // line. As the current line ends, only height needs to be updated. width can remain the
        // same as no character needs to be inserted here.
        self.make_vertical_room();
        // Place the cursor in a virtual position, but do not allocate any memory.
        self.move_left_edge();
        self.move_down_and_scroll(1);
    }

    /// Insert a character at the current cursor position.
    ///
    /// Leaves an uninitialized character (space + CHARDRAWN = false) at the cursor and move the
    /// rest of the line to the right.
    pub fn insert_character(&mut self) {
        self.make_room();
        let mut row_end = self.matrix.cell_index(self.width() - 1, self.cursor.y) as usize;

        if self.matrix.cells[row_end].drawn() {
            // Last cell in row is drawn, need to resize
            let (x, y) = (self.width(), self.cursor.y);
            self.make_room_for(x, y);
            row_end = self.matrix.cell_index(self.width() - 1, self.cursor.y) as usize;
        }

        let current = self.matrix.cell_index(self.cursor.x, self.cursor.y) as usize;

        while row_end > current {
            row_end -= 1;
            self.matrix.cells[row_end + 1] = self.matrix.cells[row_end];
        }
        self.matrix.cells[row_end] = Cell::new(self.colors);
    }

    /// Delete the character under the cursor.
    ///
    /// Move the rest of the line to the left.
    pub fn delete_character(&mut self) {
        self.make_room();

        let mut current = self.matrix.cell_index(self.cursor.x, self.cursor.y) as usize;
        let row_end = self.matrix.cell_index(self.width() - 1, self.cursor.y) as usize;

        while current + 1 <= row_end {
            self.matrix.cells[current] = self.matrix.cells[current + 1];
            current += 1;
        }
        self.matrix.cells[current] = Cell::new(self.colors);
    }

    /// Delete the character left of the cursor
    pub fn delete_left(&mut self) {
        if self.cursor.x > 0 {
            self.move_left(1);
            self.delete_character();
        }
    }

    /// Insert a row between the current one and the next.
    pub fn insert_row(&mut self) {
        self.make_room();
        let w = self.width() as usize;

        // Make room in the array
        let old_len = self.matrix.cells.len();
        self.matrix
            .cells
            .resize(old_len + w, Cell::new(self.colors));

        self.matrix.height += 1;

        // Move down the cells
        let mut current = old_len;
        let next_row = self.matrix.cell_index(0, self.cursor.y + 1) as usize;
        while current >= next_row {
            let (top, bottom) = self.matrix.cells.split_at_mut(current);
            bottom[0..w].copy_from_slice(&top[(current - w)..current]);
            current -= w;
        }

        // Fill the next line
        for c in &mut self.matrix.cells[next_row..(next_row + w)] {
            *c = Cell::new(self.colors);
        }
    }

    /// Delete the given row
    pub fn delete_row_at(&mut self, y: isize) {
        self.make_room();

        let w = self.width() as usize;

        // Move the cells
        let mut delete_y = y;
        while delete_y + 1 < self.height() {
            let current_row = self.matrix.cell_index(0, delete_y) as usize;
            let next_row = self.matrix.cell_index(0, delete_y + 1) as usize;

            let (top, bottom) = self.matrix.cells.split_at_mut(next_row);
            top[current_row..(current_row + w)].copy_from_slice(&bottom[0..w]);

            delete_y += 1;
        }

        self.matrix.height -= 1;
        unsafe {
            self.matrix.cells.set_len(self.matrix.height as usize * w);
        }
    }

    /// Delete the current row
    pub fn delete_row(&mut self) {
        let y = self.cursor.y;
        self.delete_row_at(y);
    }

    /// Move the remainder of the current row to the next line
    pub fn break_line(&mut self) {
        self.insert_row();

        let w = self.width() as usize;
        let here = self.matrix.cell_index(self.cursor.x, self.cursor.y) as usize;
        let n = w - self.cursor.x as usize;
        let next_row = self.matrix.cell_index(0, self.cursor.y + 1) as usize;

        for i in 0..n {
            let old_cell = self.matrix.cells[here + i];
            self.matrix.cells[next_row + i] = old_cell;
            self.matrix.cells[here + i] = Cell::new(self.colors);
        }

        self.cursor.x = 0;
        self.cursor.y += 1;
    }

    /// Join the current line with next line
    pub fn join_next_line(&mut self) {
        if self.cursor.y + 1 < self.height() {
            // This line and the next are inside the screen. Resize so that both fit into the
            // matrix.
            let current_line_len = self.matrix.compacted_row_slice(self.cursor.y).len();
            let next_line_len = self.matrix.compacted_row_slice(self.cursor.y + 1).len();
            let y = self.cursor.y;
            self.make_room_for((current_line_len + next_line_len) as isize, y);

            // Copy the data
            let to_index = self.matrix.cell_index(current_line_len as isize, y) as usize;
            let from_index = self.matrix.cell_index(0, y + 1) as usize;
            for i in 0..next_line_len {
                self.matrix.cells[i + to_index] = self.matrix.cells[i + from_index];
            }

            self.delete_row_at(y + 1);
        }
    }

    /// Fill a rectangle with a the same cells.
    ///
    /// rect is assumed to be valid
    fn fill_rect(&mut self, rect: Rectangle, cell: Cell) {
        for y in rect.start.y..(rect.end.y + 1) {
            let from_index = self.matrix.cell_index(rect.start.x, y);
            let to_index = self.matrix.cell_index(rect.end.x + 1, y);
            for index in from_index..to_index {
                // TODO: Preserve color
                self.matrix.cells[index as usize] = cell;
            }
        }
    }

    /// Convert the screen to a Matrix that cannot be changed anymore
    pub fn freeze(self) -> Matrix {
        self.matrix
    }

    /// Interpret the parameter as a string of command codes and characters
    ///
    /// If any other event than Ignore, Cr, or NewLine are created, return them as error.
    /// This function is supposed to be used for known correct text only. DO NOT USE for text read
    /// from an uncontrolled source, i.e. bash.
    pub fn add_bytes(&mut self, bytes: &[u8]) -> Result<(), Event> {
        for c in bytes {
            let e = self.add_byte(*c);
            match e {
                Event::Ignore | Event::Cr | Event::NewLine => {}
                _ => return Err(e),
            }
        }
        Ok(())
    }

    /// Process a single byte in the state machine.
    ///
    /// Indicate certain events in the return code.
    pub fn add_byte(&mut self, byte: u8) -> Event {
        let action = self.parser.add_byte(byte);
        if action != Action::More {
            trace!("Action: {:?}", action);
        }
        match action {
            Action::More => Event::Ignore,
            Action::Error => Event::Ignore,
            Action::Cr => {
                self.move_left_edge();
                Event::Cr
            }
            Action::FormFeed | Action::VerticalTab | Action::NewLine => {
                // TODO: This should differ from NextLine by the handling of linefeed
                self.new_line();
                Event::NewLine
            }
            Action::Char(c) => {
                // Character set handling only happens here. At the moment, the UTF-8 handling is
                // slightly different from xterm, possibly incorrect in subtle ways, but otherwise
                // functional.
                let display_c = if (c as u32) < 128 {
                    charset::map_byte( self.gsets[self.curgl.clone() as usize].clone(), c as u8)
                } else if (c as u32) < 256 {
                    charset::map_byte( self.gsets[self.curgr.clone() as usize].clone(), ((c as u32) - 128) as u8)
                } else { c };
                self.last_char = display_c;
                self.place_char(display_c);
                Event::Ignore
            }
            Action::CharacterAttributes(attrs) => {
                for attr in attrs {
                    match attr {
                        CharacterAttribute::Normal => self.attributes = Attributes::empty(),
                        CharacterAttribute::Bold => self.attributes.insert(Attributes::BOLD),
                        CharacterAttribute::Faint => self.attributes.insert(Attributes::ATR_FAINT),
                        CharacterAttribute::Italicized => {
                            self.attributes.insert(Attributes::ATR_ITALIC)
                        }
                        CharacterAttribute::Underlined => {
                            self.attributes.insert(Attributes::UNDERLINE)
                        }
                        CharacterAttribute::Blink => self.attributes.insert(Attributes::BLINK),
                        CharacterAttribute::Inverse => self.attributes.insert(Attributes::INVERSE),
                        CharacterAttribute::Invisible => {
                            self.attributes.insert(Attributes::INVISIBLE)
                        }
                        CharacterAttribute::CrossedOut => {
                            self.attributes.insert(Attributes::ATR_STRIKEOUT)
                        }
                        CharacterAttribute::DoublyUnderlined => {
                            self.attributes.insert(Attributes::ATR_DBL_UNDER)
                        }
                        CharacterAttribute::NotBoldFaint => {
                            self.attributes.remove(Attributes::BOLD);
                            self.attributes.remove(Attributes::ATR_FAINT);
                        }
                        CharacterAttribute::NotItalicized => {
                            self.attributes.remove(Attributes::ATR_ITALIC)
                        }
                        CharacterAttribute::NotUnderlined => {
                            self.attributes.remove(Attributes::UNDERLINE)
                        }
                        CharacterAttribute::Steady => self.attributes.remove(Attributes::BLINK),
                        CharacterAttribute::Positive => self.attributes.remove(Attributes::INVERSE),
                        CharacterAttribute::Visible => {
                            self.attributes.remove(Attributes::INVISIBLE)
                        }
                        CharacterAttribute::NotCrossedOut => {
                            self.attributes.remove(Attributes::ATR_STRIKEOUT)
                        }
                        CharacterAttribute::Foreground(Color::Default) => {
                            self.attributes.remove(Attributes::FG_COLOR)
                        }
                        CharacterAttribute::Foreground(c) => {
                            self.attributes.insert(Attributes::FG_COLOR);
                            self.colors.foreground = Colors::fromColor(c);
                        }
                        CharacterAttribute::Background(Color::Default) => {
                            self.attributes.remove(Attributes::BG_COLOR)
                        }
                        CharacterAttribute::Background(c) => {
                            self.attributes.insert(Attributes::BG_COLOR);
                            self.colors.background = Colors::fromColor(c);
                        }
                    };
                }
                Event::Ignore
            }
            Action::HorizontalMove(n) => {
                self.cursor.x += n as isize;
                Event::Ignore
            }
            Action::VerticalPositionAbsolute(n) => {
                self.cursor.y = n as isize;
                Event::Ignore
            }
            Action::SaveCursor => {
                self.saved_cursor = self.cursor;
                Event::Ignore
            }
            Action::RestoreCursor => {
                self.cursor = self.saved_cursor;
                Event::Ignore
            }
            Action::CursorLowerLeft => {
                self.cursor.x = 0;
                self.cursor.y = cmp::max(self.height() - 1, 0);
                Event::Ignore
            }
            Action::CursorAbsoluteColumn(c) => {
                let y = self.cursor.y;
                self.move_cursor_to(c as isize, y);
                Event::Ignore
            }
            Action::CursorAbsolutePosition(r, c) => {
                self.move_cursor_to(c as isize, r as isize);
                Event::Ignore
            }
            Action::CursorUp(n) => {
                self.move_up(n as isize);
                Event::Ignore
            }
            Action::CursorDown(n) => {
                self.move_down(n as isize);
                Event::Ignore
            }
            Action::CursorForward(n) => {
                self.move_right(n as isize);
                Event::Ignore
            }
            Action::CursorBackward(n) => {
                self.move_left(n as isize);
                Event::Ignore
            }
            Action::Bell => Event::Bell,
            Action::VerticalPositionRelative(n) => {
                let x = self.cursor.x;
                let y = self.cursor.y;
                self.move_cursor_to(x, y + (n as isize));
                Event::Ignore
            }
            Action::CursorNextLine(n) => {
                let x = self.cursor.x;
                let y = self.cursor.y;
                self.move_cursor_to(x, y + (n as isize));
                self.move_left_edge();
                Event::Ignore
            }
            Action::CursorPrevLine(n) => {
                let x = self.cursor.x;
                let y = self.cursor.y;
                self.move_cursor_to(x, y - (n as isize));
                self.move_left_edge();
                Event::Ignore
            }
            Action::NextLine => {
                self.new_line();
                Event::Ignore
            }
            Action::Index => {
                self.make_room();
                self.move_down_and_scroll(1);
                Event::Ignore
            }
            Action::ReverseIndex => {
                self.make_room();
                self.move_up_and_scroll(1);
                Event::Ignore
            }
            Action::ScrollDown(n) => {
                self.make_room();
                let (start_row, _, limited) = self.determine_scroll_region();
                if limited {
                    self.scroll_down(start_row,n as isize);
                }
                Event::Ignore
            }
            Action::ScrollUp(n) => {
                self.make_room();
                let (start_row, _, limited) = self.determine_scroll_region();
                if limited {
                    self.scroll_up(start_row, n as isize);
                }
                Event::Ignore
            }
            Action::ScrollLeft(n) => {
                self.make_room();
                let (start_row, end_row, _) = self.determine_scroll_region();
                if start_row <= self.cursor.y && self.cursor.y <= end_row {
                    self.scroll_left(0, n as isize);
                }
                Event::Ignore
            }
            Action::ScrollRight(n) => {
                self.make_room();
                let (start_row, end_row, _) = self.determine_scroll_region();
                if start_row <= self.cursor.y && self.cursor.y <= end_row {
                self.scroll_right(0, n as isize);
                }
                Event::Ignore
            }
            Action::Backspace => {
                self.move_left(1);
                Event::Ignore
            }
            Action::DecBackIndex => {
                if self.fixed_size {
                    if self.cursor.x == 0 {
                        self.scroll_right(0,1);
                    } else {
                        self.move_left(1);
                    }
                } else {
                    self.move_left(1);
                }
                Event::Ignore
            }
            Action::DecForwardIndex => {
                if self.fixed_size {
                    if self.cursor.x + 1 == self.width() {
                        self.scroll_left(0, 1);
                    } else {
                        self.move_right(1);
                    }
                } else {
                    self.move_right(1);
                }
                Event::Ignore
            }
            Action::FillArea(c, rect) => {
                if (c >= 0x20 && c < 0x80) || (c >= 0xa1 && c < 0xff) {
                    self.make_room();
                    let rect = rect.clipped(&self.matrix.rectangle());
                    let mut cell = Cell::new(self.colors);
                    cell.code_point = unsafe { std::char::from_u32_unchecked(c as u32) };
                    cell.attributes = self.attributes;
                    cell.attributes.insert(Attributes::CHARDRAWN);
                    self.fill_rect(rect,cell);
                } else {
                    warn!("FillArea called for non-ascii character {}", c as u32);
                }
                Event::Ignore
            }
            Action::CopyArea(rect, _from_page, p, _to_page) => {
                // This mirrors the implementation in xterm in that it ignore pages and handles
                // overlap by copying the screen.
                self.make_room();
                let src_rect = rect.clipped(&self.matrix.rectangle());
                let displace = p - src_rect.start;
                let dst_rect = (src_rect.clone() + displace).clipped(&self.matrix.rectangle());

                let old_matrix = self.matrix.clone();
                let dst_width = dst_rect.end.x - dst_rect.start.x + 1;

                // Iterate over the dst_rect as this might be smaller.
                for dst_y in dst_rect.start.y..(dst_rect.end.y + 1) {
                    let src_y = dst_y - displace.y;
                    let src_from = self.matrix.cell_index(src_rect.start.x, src_y);
                    let dst_from = self.matrix.cell_index(dst_rect.start.x, dst_y);
                    for i in 0..dst_width {
                        self.matrix.cells[(dst_from + i) as usize] =
                            old_matrix.cells[(src_from + i) as usize];
                    }
                }
                Event::Ignore
            }
            Action::InsertColumns(n) => {
                let c =self.cursor;
               self.scroll_right( c.x, n as isize);
               Event::Ignore
            }
            Action::DeleteColumns(n) => {
                let c =self.cursor;
               self.scroll_left( c.x, n as isize);
               Event::Ignore
            }
            Action::EraseArea(rect, _) => {
                // TODO: handle protection
                let rect = rect.clipped(&self.matrix.rectangle());
                let c = self.clone_cell(' ');
                self.fill_rect(rect,c);
                Event::Ignore
            }
            Action::RepeatCharacter(n) => {
                let c = self.last_char;
                for _i in 0..n {
                    self.place_char( c);
                }
                Event::Ignore
            }
            Action::EraseCharacters(n) => {
                // Overwrite the next n characters with fresh cells
                let c = self.cursor;
                self.cursor.x += n as isize;
                self.make_room();
                self.cursor = c;
                let n = if self.fixed_size {
                    cmp::max(0,cmp::min(n as isize,self.width()-self.cursor.x))
                } else {
                    n as isize
                };
                let row_index = self.matrix.cell_index(c.x, c.y);
                let cell = self.clone_cell(' ');
                for offset in 0 .. n {
                    self.matrix.cells[(row_index+offset) as usize] = cell;
                }
                Event::Ignore
            }
            Action::EraseDisplay(what, _) => {
                // TODO: Handle selective
                self.make_room();
                let c = self.cursor;
                let (start_row, end_row) = match what {
                    EraseDisplay::Above => ( 0, c.y+1 ),
                    EraseDisplay::Below => (c.y, self.height()),
                    EraseDisplay::All | EraseDisplay::Saved => (0, self.height()),
                };
                let width = self.width() as usize;
                let cell = self.clone_cell(' ');
                trace!("EraseDisplay with {:?}", cell);
                for row in start_row .. end_row {
                    let row_index = self.matrix.cell_index(0,row) as usize;
                    for col in 0 .. width {
                        self.matrix.cells[row_index+col] = cell;
                    }
                }
                Event::Ignore
            }
            Action::EraseLine(what, _) => {
                // TODO: Handle selective
                self.make_room();
                let c=self.cursor;
                let width = self.width() as usize;
                let cell = self.clone_cell(' ');
                let (start_col, end_col) = match what {
                    EraseLine::Left => (0,(c.x+1) as usize),
                    EraseLine::Right => (c.x as usize, width),
                    EraseLine::All => (0,width),
                };
                let row_index = self.matrix.cell_index(0,c.y) as usize;
                for col in start_col .. end_col {
                    self.matrix.cells[row_index+col] = cell;
                }
                Event::Ignore
            }
            Action::InsertCharacters(n) => {
                self.make_room();
                let c=self.cursor;
                self.scroll_right_line( c.y, c.x, n as isize);
                Event::Ignore
            }
            Action::DeleteCharacters(n) => {
                self.make_room();
                let c=self.cursor;
                self.scroll_left_line( c.y, c.x, n as isize);
                Event::Ignore
            }
            Action::InsertLines(n) => {
                self.make_room();
                let c=self.cursor;
                self.scroll_down(c.y, n as isize);
                Event::Ignore
            }
            Action::DeleteLines(n) => {
                self.make_room();
                let c=self.cursor;
                let (start_row, end_row, limited) = self.determine_scroll_region();
                if start_row <= c.y && c.y<= end_row || !limited {
                    self.scroll_up(c.y, n as isize);
                }
                Event::Ignore
            }
            Action::Tabulator => {
                // TODO: Full tab support

                // Compute to the next multiple of 8.
                let cx=((self.cursor.x + 8) / 8 ) * 8;
                if self.fixed_size {
                    if cx < self.width() {
                        self.cursor.x = cx;
                    }
                } else {
                    self.cursor.x = cx;
                }
                Event::Ignore
            }
            Action::ScrollRegion(region) => {
                // If cursor is now outside the scroll region, move it to the
                // scroll region
                self.make_room();
                let c=self.cursor;
                self.scroll_region = region;
                if let Some((start_row,end_row)) = region {
                    let start_row = start_row as isize;
                    let end_row = end_row as isize;
                    if c.y < start_row || c.y > end_row {
                        self.cursor.y = start_row;
                    }
                }
                Event::Ignore
            }
            Action::DesignateCharacterSet(level, charset) => {
                self.gsets[level as usize]=charset;
                Event::Ignore
            }

            // Silently ignore these sequences until functionality is required.
            // Enter TUI mode.
            Action::DecApplicationKeypad(_) |
            Action::SetMode(_) |
            Action::ResetMode(_) |
            Action::SetPrivateMode(_) |
            Action::ResetPrivateMode(_) |
            Action::WindowOp(_) => {
                warn!("StartTui Actions not fully implemented");
                    Event::StartTui
                }

            // Category: Common change screen operations, Prio 1
            Action::LinesPerScreen(_) |
            Action::ColumnsPerPage(_) |
            // Category: Less common operations, Prio 2
            Action::PopVideoAttributes |
            Action::PushVideoAttributes(_) |
            Action::ChangeAttributesArea(_, _) |
            Action::ReverseAttributesArea(_, _) |
            Action::SoftReset |
            Action::FullReset |
            // Category: Tabulators and Margins, Prio 3
            Action::TabSet |
            Action::TabClear(_) |
            Action::CursorForwardTab(_) |
            Action::CursorBackwardTab(_) |
            Action::SetMargins(_, _) |
            Action::StartGuardedArea |
            Action::EndGuardedArea |
            Action::EnableFilterArea(_) |
            Action::AttributeChangeExtent(_) |
            // Category: Reports, Prio 4
            Action::TerminalUnitId |
            Action::TerminalEnquire |
            Action::RequestLocatorPosition |
            Action::ReportRendition(_) |
            Action::RequestTerminalParameters |
            Action::CursorInformationReport |
            Action::TabstopReport |
            Action::DecDeviceStatusReport |
            Action::PrinterStatusReport |
            Action::UdkStatusReport |
            Action::KeyboardStatusReport |
            Action::LocatorStatusReport |
            Action::LocatorTypeReport |
            Action::MacroStatusReport |
            Action::MemoryStatusReport(_) |
            Action::DataIntegrityReport |
            Action::MultiSessionReport |
            Action::StatusReport |
            Action::ReportCursorPosition |
            Action::LocatorReport(_, _) |
            Action::ChecksumArea(_, _, _) |
            Action::DA1(_) |
            Action::DA2(_) |
            // Category: Bells and whistles, Prio 5
            Action::SetMarginBellVolume(_) |
            Action::SetWarningBellVolume(_) |
            Action::CursorStyle(_) |
            Action::LoadLeds(_, _) |
            Action::ForegroundColorRgb(_, _, _) |
            Action::ForegroundColorIndex(_) |
            Action::BackgroundColorRgb(_, _, _) |
            Action::BackgroundColorIndex(_) |
            Action::SetTitleModes(_) |
            Action::ResetTitleModes(_) |
            Action::LockMemory(_) |
            Action::GraphicRegister(_, _) |
            Action::MediaCopy(_) |
            // Category: Mode switches, Prio 6
            Action::DecUserDefinedKeys(_) |
            Action::SetTextParameter(_, _) |
            Action::SetModFKeys(_, _) |
            Action::DisableModFKeys(_) |
            Action::CharacterProtection(_) |
            Action::ConformanceLevel(_, _) |
            Action::Show8BitControl(_) |
            Action::AnsiConformanceLevel(_) |
            Action::DecAlignmentTest |
            Action::DecDoubleHeight(_) |
            Action::DecDoubleWidth(_) |
            Action::RequestPrivateMode(_) |
            Action::RestorePrivateMode(_) |
            Action::SavePrivateMode(_) |
            Action::RequestAnsiMode(_) |
            // Category: Mouse handling, Prio 8
            Action::SelectLocatorEvents(_, _) |
            Action::PointerMode(_) |
            Action::MouseTracking(_, _, _, _, _) |
            // Category: Non-UTF-8 language support: Prio 9
            Action::SingleShift(_) |
            Action::InvokeCharSet(_, _) |
            // Category: String message, Prio 10
            Action::ApplicationProgramCommand(_) |
            Action::PrivacyMessage(_) |
            Action::StartOfString(_) => {
                warn!( "Action »{:?}« not implemented.", action);
                // TODO: Convert to event
                Event::Ignore
            }
        }
    }
}

impl PartialEq for Screen {
    fn eq(&self, other: &Screen) -> bool {
        self.matrix == other.matrix
    }
}
