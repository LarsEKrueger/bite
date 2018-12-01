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

use super::control_sequence::action::{Action, CharacterAttribute, Color};
use super::control_sequence::parser::Parser;

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

    pub fn foreground_color(&self) -> Option<u8> {
        if self.attributes.contains(Attributes::FG_COLOR) {
            Some(self.colors.foreground)
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

/// Attributes as bitflags
bitflags! {
    struct Attributes: u16 {
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

    fn cell_at_mut(&mut self, x: isize, y: isize) -> &mut Cell {
        let index = self.cell_index(x, y) as usize;
        &mut self.cells[index]
    }

    pub fn compacted_row_slice(&self, row: isize) -> &[Cell] {
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

    pub fn compacted_row(&self, row: isize) -> Vec<Cell> {
        self.compacted_row_slice(row).to_vec()
    }

    pub fn line_iter(&self) -> impl Iterator<Item = &[Cell]> {
        (0..self.height).map(move |r| self.compacted_row_slice(r))
    }

    pub fn reset(&mut self) {
        self.cells.clear();
        self.width = 0;
        self.height = 0;
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
}

#[derive(Copy, Clone)]
struct Cursor {
    /// Horizontal cursor position. Might be negative.
    x: isize,

    /// Vertical cursor position. Might be negative.
    y: isize,
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
        }
    }

    /// Direct conversion to one-line vector of cells
    pub fn one_line_cell_vec(line: &[u8]) -> Vec<Cell> {
        if line.is_empty() {
            Vec::new()
        } else {
            Self::one_line_matrix(line).compacted_row(0)
        }
    }

    /// Direct conversion to one-line matrix
    pub fn one_line_matrix(bytes: &[u8]) -> Matrix {
        let mut s = Screen::new();
        s.add_bytes(bytes);
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
    pub fn fixed_size(&mut self) {
        self.fixed_size = true;
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

    /// Check if the cursor is at the end of the line
    pub fn cursor_at_end_of_line(&self) -> bool {
        if 0 <= self.cursor.y && self.cursor.y < self.height() {
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

    pub fn text_before_cursor(&mut self) -> String {
        self.make_room();

        let mut text = String::new();
        let mut current_index = self.matrix.cell_index(0, self.cursor.y) as usize;
        let cursor_index = self.cursor_index() as usize;
        while current_index < cursor_index {
            text.push(self.matrix.cells[current_index].code_point);
            current_index += 1;
        }
        text
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

    /// Scroll the character matrix up by n rows and fill the last rows with fresh cells.
    fn scroll_up(&mut self, scroll_rows: isize) {
        let scroll_rows = cmp::min(scroll_rows, self.height());
        if scroll_rows >= 1 {
            // Scroll up
            let mut offset = 0;
            let src_index = (scroll_rows * self.width()) as usize;
            let n = (self.width() * (self.height() - scroll_rows)) as usize;
            while offset < n {
                self.matrix.cells[offset] = self.matrix.cells[offset + src_index];
                offset += 1;
            }
            // Clear last rows
            let n = (self.width() * self.height()) as usize;
            while offset < n {
                self.matrix.cells[offset] = Cell::new(self.colors);
                offset += 1;
            }
        }
    }

    /// Scroll the character matrix left by n columns and fill the last rows with fresh cells.
    fn scroll_left(&mut self, scroll_cols: isize) {
        let scroll_cols = cmp::min(scroll_cols, self.width());
        if scroll_cols >= 1 {
            // Scroll left
            let mut offset = 0;
            let src_index = scroll_cols as usize;
            let n = (self.width() * self.height() - scroll_cols) as usize;
            while offset < n {
                self.matrix.cells[offset] = self.matrix.cells[offset + src_index];
                offset += 1;
            }
            // Clear last cols
            let c0 = self.width() - scroll_cols;
            let c1 = self.width();
            for row in 0..self.height() {
                for col in c0..c1 {
                    *self.matrix.cell_at_mut(col, row) = Cell::new(self.colors);
                }
            }
        }
    }

    /// Scroll the character matrix down by n rows and fill the first rows with fresh cells.
    fn scroll_down(&mut self, scroll_rows: isize) {
        let scroll_rows = cmp::min(scroll_rows, self.height());
        if scroll_rows >= 1 {
            // Scroll down
            let dst_index = (self.width() * scroll_rows) as usize;
            let mut offset = (self.width() * (self.height() - scroll_rows)) as usize;
            while offset > 0 {
                offset -= 1;
                self.matrix.cells[dst_index + offset] = self.matrix.cells[offset];
            }
            // Clear first rows
            let mut offset = 0;
            let n = (self.width() * scroll_rows) as usize;
            while offset < n {
                self.matrix.cells[offset] = Cell::new(self.colors);
                offset += 1;
            }
        }
    }

    /// Scroll the character matrix right by n columns and fill the first columns with fresh cells.
    fn scroll_right(&mut self, scroll_cols: isize) {
        let scroll_cols = cmp::min(scroll_cols, self.height());
        if scroll_cols >= 1 {
            // Scroll down
            let dst_index = scroll_cols as usize;
            let mut offset = (self.width() * self.height() - scroll_cols) as usize;
            while offset > 0 {
                offset -= 1;
                self.matrix.cells[dst_index + offset] = self.matrix.cells[offset];
            }
            // Clear first columns
            let c0 = 0;
            let c1 = scroll_cols;
            for row in 0..self.height() {
                for col in c0..c1 {
                    *self.matrix.cell_at_mut(col, row) = Cell::new(self.colors);
                }
            }
        }
    }

    pub fn place_str(&mut self, s: &str) {
        for c in s.chars() {
            self.place_char(c);
        }
    }

    /// Ensure that there is room for the character at the current position.
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

    fn make_room_for(&mut self, x: isize, y: isize) -> (isize, isize) {
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
                new_matrix[new_start..new_end].copy_from_slice(
                    &self.matrix.cells
                        [old_start..old_end],
                );
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
        let c = self.cursor;
        if self.fixed_size {
            if c.y + n < self.height() {
                self.cursor.y += n;
            } else {
                // We need to scroll
                let scroll_lines = c.y + n - self.height() + 1;
                self.scroll_up(scroll_lines);
                self.cursor.y = self.height() - 1;
            }
        } else {
            self.cursor.y += n;
        }
    }

    /// Move the cursor up n lines and scroll down if necessary
    pub fn move_up_and_scroll(&mut self, n: isize) {
        let c = self.cursor;
        if self.fixed_size {
            if n <= c.y {
                self.cursor.y -= n;
            } else {
                // We need to scroll
                let scroll_lines = n - c.y;
                self.scroll_down(scroll_lines);
                self.cursor.y = 0;
            }
        } else {
            self.cursor.y -= n;
        }
    }
    pub fn new_line(&mut self) {
        self.move_left_edge();
        self.cursor.y += 1;
        if self.fixed_size {
            if self.cursor.y == self.height() {
                self.scroll_up(1);
                self.cursor.y -= 1;
            }
        }
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
        self.matrix.cells.resize(
            old_len + w,
            Cell::new(self.colors),
        );

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

    /// Convert the screen to a Matrix that cannot be changed anymore
    pub fn freeze(self) -> Matrix {
        self.matrix
    }

    /// Interpret the parameter as a string of command codes and characters
    pub fn add_bytes(&mut self, bytes: &[u8]) {
        for c in bytes {
            self.add_byte(*c);
        }
    }

    /// Process a single byte in the state machine.
    ///
    /// Indicate certain events in the return code.
    pub fn add_byte(&mut self, byte: u8) -> Event {
        match self.parser.add_byte(byte) {
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
                self.place_char(c);
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
                self.move_down_and_scroll(1);
                Event::Ignore
            }
            Action::ReverseIndex => {
                self.move_up_and_scroll(1);
                Event::Ignore
            }
            Action::ScrollDown(n) => {
                self.scroll_down(n as isize);
                Event::Ignore
            }
            Action::ScrollUp(n) => {
                self.scroll_up(n as isize);
                Event::Ignore
            }
            Action::ScrollLeft(n) => {
                self.scroll_left(n as isize);
                Event::Ignore
            }
            Action::ScrollRight(n) => {
                self.scroll_right(n as isize);
                Event::Ignore
            }
            Action::Backspace => {
                self.move_left(1);
                Event::Ignore
            }

            Action::DecBackIndex |
            Action::DecForwardIndex |
            Action::FillArea(_, _) |
            Action::CopyArea(_, _, _, _) |
            Action::InsertColumns(_) |
            Action::DeleteColumns(_) |
            Action::EraseArea(_, _) |
            Action::MediaCopy(_) |
            Action::RepeatCharacter(_) |
            Action::EraseCharacters(_) |
            Action::EraseDisplay(_, _) |
            Action::EraseLine(_, _) |
            Action::InsertCharacters(_) |
            Action::InsertLines(_) |
            Action::DeleteLines(_) |
            Action::DeleteCharacters(_) |
            Action::TabSet |
            Action::Tabulator |
            Action::TabClear(_) |
            Action::CursorForwardTab(_) |
            Action::CursorBackwardTab(_) |
            Action::SetMargins(_, _) |
            Action::StartGuardedArea |
            Action::EndGuardedArea |
            Action::EnableFilterArea(_) |
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
            Action::SingleShift(_) |
            Action::StartOfString(_) |
            Action::PrivacyMessage(_) |
            Action::SetTextParameter(_, _) |
            Action::LinesPerScreen(_) |
            Action::ColumnsPerPage(_) |
            Action::PopVideoAttributes |
            Action::PushVideoAttributes(_) |
            Action::SelectLocatorEvents(_, _) |
            Action::LocatorReport(_, _) |
            Action::AttributeChangeExtent(_) |
            Action::ChecksumArea(_, _, _) |
            Action::SetMarginBellVolume(_) |
            Action::SetWarningBellVolume(_) |
            Action::ChangeAttributesArea(_, _) |
            Action::ReverseAttributesArea(_, _) |
            Action::ScrollRegion(_, _) |
            Action::CharacterProtection(_) |
            Action::CursorStyle(_) |
            Action::LoadLeds(_, _) |
            Action::PointerMode(_) |
            Action::SoftReset |
            Action::ConformanceLevel(_, _) |
            Action::SetModFKeys(_, _) |
            Action::DisableModFKeys(_) |
            Action::ForegroundColorRgb(_, _, _) |
            Action::ForegroundColorIndex(_) |
            Action::BackgroundColorRgb(_, _, _) |
            Action::BackgroundColorIndex(_) |
            Action::SetMode(_) |
            Action::ResetMode(_) |
            Action::RequestAnsiMode(_) |
            Action::SetPrivateMode(_) |
            Action::ResetPrivateMode(_) |
            Action::RequestPrivateMode(_) |
            Action::RestorePrivateMode(_) |
            Action::SavePrivateMode(_) |
            Action::DA1(_) |
            Action::DA2(_) |
            Action::SetTitleModes(_) |
            Action::ResetTitleModes(_) |
            Action::MouseTracking(_, _, _, _, _) |
            Action::DecUserDefinedKeys(_) |
            Action::ApplicationProgramCommand(_) |
            Action::LockMemory(_) |
            Action::FullReset |
            Action::DesignateCharacterSet(_, _) |
            Action::InvokeCharSet(_, _) |
            Action::WindowOp(_) |
            Action::Show8BitControl(_) |
            Action::AnsiConformanceLevel(_) |
            Action::GraphicRegister(_, _) |
            Action::DecApplicationKeypad(_) |
            Action::DecAlignmentTest |
            Action::DecDoubleHeight(_) |
            Action::DecDoubleWidth(_) => {
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
