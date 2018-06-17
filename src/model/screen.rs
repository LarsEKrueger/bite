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

use super::emulator::{Emulator, Action};

/// Colors are pairs of foreground/background indices into the same palette.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
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

/// A cell is a character and its colors and attributes.
///
/// TODO: Pack data more tightly
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
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
        const INVERSE       = 0b000000000001;
        const UNDERLINE     = 0b000000000010;
        const BOLD          = 0b000000000100;
        const BLINK         = 0b000000001000;
        /// true if background set
        const BG_COLOR      = 0b00010000;
        /// true if foreground set
        const FG_COLOR      = 0b000000100000;
        /// a character that cannot be erased
        const PROTECTED     = 0b000001000000;
        /// a character has been drawn here on the screen.  Used to distinguish blanks from empty
        /// parts of the screen when selecting
        const CHARDRAWN     = 0b000010000000;

        const ATR_FAINT     = 0b000100000000;
        const ATR_ITALIC    = 0b001000000000;
        const ATR_STRIKEOUT = 0b010000000000;
        const ATR_DBL_UNDER = 0b100000000000;

        const SGR_MASK2     = Self::ATR_FAINT.bits | Self::ATR_ITALIC.bits |
                              Self::ATR_STRIKEOUT.bits | Self::ATR_DBL_UNDER.bits;

        /// mask for video-attributes only
        const SGR_MASK      = Self::BOLD.bits | Self::BLINK.bits | Self::UNDERLINE.bits |
                              Self::INVERSE.bits;

        /// mask: user-visible attributes
        const ATTRIBUTES    = Self::SGR_MASK.bits | Self::SGR_MASK2.bits | Self::BG_COLOR.bits |
                              Self::FG_COLOR.bits | Self::PROTECTED.bits;

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

    fn cell_index(&self, x: isize, y: isize) -> usize {
        debug_assert!(0 <= x);
        debug_assert!(x < self.width);
        debug_assert!(0 <= y);
        debug_assert!(y < self.height);

        (x + y * self.width) as usize
    }

    pub fn compacted_row(&self, row: isize) -> Vec<Cell> {
        let row_start = self.cell_index(0, row);
        let mut row_end = self.cell_index(self.width - 1, row);
        while row_end >= row_start {
            if self.cells[row_end].attributes.contains(
                Attributes::CHARDRAWN,
            )
            {
                break;
            }
            row_end -= 1;
        }

        self.cells[row_start..(row_end + 1)].to_vec()
    }
}

impl PartialEq for Matrix {
    /// Visual equality. If it looks the same, it's the same.
    fn eq(&self, other: &Matrix) -> bool {
        self.width == other.width && self.height == other.height && self.cells == other.cells
    }
}

/// A screen is rectangular area of cells and the position of the cursor.
///
/// The cursor can be outside the allocated screen. If a visible character is inserted there, the
/// screen is reallocated. Coordinate system origin is top-left with x increasing to the right and
/// y down.
#[allow(dead_code)]
pub struct Screen {
    /// A matrix of cells
    matrix: Matrix,

    /// Horizontal cursor position. Might be negative.
    x: isize,

    /// Vertical cursor position. Might be negative.
    y: isize,

    /// Attributes for next character
    attributes: Attributes,

    /// Colors for next character
    colors: Colors,

    /// State for the state machine to interpret the byte stream as a terminal.
    emulator: Emulator,
}

#[allow(dead_code)]
impl Screen {
    /// Create a new, empty screen
    pub fn new() -> Self {
        Self {
            matrix: Matrix::new(),
            x: 0,
            y: 0,
            attributes: Attributes::empty(),
            colors: Colors {
                foreground: 1,
                background: 0,
            },
            emulator: Emulator::new(),
        }
    }

    /// Direct conversion to one-line vector of cells
    pub fn one_line_cell_vec(line: &[u8]) -> Vec<Cell> {
        Self::one_line_matrix(line).compacted_row(0)
    }

    /// Direct conversion to one-line matrix
    pub fn one_line_matrix(bytes: &[u8]) -> Matrix {
        let mut s = Screen::new();
        s.add_bytes(bytes);
        s.freeze()
    }

    /// Get width of matrix
    pub fn width(&self) -> isize {
        self.matrix.width
    }

    /// Get height of matrix
    pub fn height(&self) -> isize {
        self.matrix.height
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
        self.x += 1;
    }

    pub fn place_str(&mut self, s: &str) {
        for c in s.chars() {
            self.place_char(c);
        }
    }

    /// Ensure that there is room for the character at the current position.
    fn make_room(&mut self) {
        if self.x < 0 || self.x >= self.width() || self.y < 0 || self.y >= self.height() {
            // Compute the new size and allocate
            let add_left = -cmp::min(self.x, 0);
            let add_right = cmp::max(self.x, self.width() - 1) - self.width() + 1;
            let add_top = -cmp::min(self.y, 0);
            let add_bottom = cmp::max(self.y, self.height() - 1) - self.height() + 1;

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
            self.x += add_left;
            self.y += add_top;
        }
    }

    /// Compute the index of the cursor position into the cell array
    #[allow(dead_code)]
    fn cursor_index(&self) -> usize {
        self.matrix.cell_index(self.x, self.y)
    }

    /// Move the cursor to the left edge
    #[allow(dead_code)]
    pub fn move_left_edge(&mut self) {
        self.x = 0;
    }

    /// Move cursor to the right edge. Moves it past the last possible character.
    #[allow(dead_code)]
    pub fn move_right_edge(&mut self) {
        self.x = self.width();
    }

    /// Move cursor to the top edge
    #[allow(dead_code)]
    pub fn move_top_edge(&mut self) {
        self.y = 0;
    }

    /// Move cursor to bottom edge. Moves it past the last possible character.
    #[allow(dead_code)]
    pub fn move_bottom_edge(&mut self) {
        self.y = self.height();
    }

    /// Move one cell to the right
    #[allow(dead_code)]
    pub fn move_right(&mut self) {
        self.x += 1;
    }

    /// Move one cell to the left
    #[allow(dead_code)]
    pub fn move_left(&mut self) {
        self.x -= 1;
    }

    /// Move one line down
    #[allow(dead_code)]
    pub fn move_down(&mut self) {
        self.y += 1;
    }

    /// Move one line up
    #[allow(dead_code)]
    pub fn move_up(&mut self) {
        self.y -= 1;
    }

    pub fn new_line(&mut self) {
        self.move_left_edge();
        self.move_down();
    }

    /// Check if the frozen representation of the screen looks different that the given matrix
    pub fn looks_different(&self, other: &Matrix) -> bool {
        self.matrix != *other
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
    /// TODO: Indicate certain events in the return code.
    pub fn add_byte(&mut self, byte: u8) {

        match self.emulator.add_byte(byte) {
            Action::More => {}
            Action::Error => {}
            Action::CrLf => {
                // TODO: Handle line-feed
            }
            Action::Char(c) => self.place_char(c),
        }
    }
}

impl PartialEq for Screen {
    fn eq(&self, other: &Screen) -> bool {
        self.matrix == other.matrix
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn start_screen() {

        let mut s = Screen::new();
        s.make_room();
        assert_eq!(s.width(), 1);
        assert_eq!(s.height(), 1);
        assert_eq!(s.matrix.cells.len(), 1);
    }

    #[test]
    fn place_letter() {
        let mut s = Screen::new();
        s.place_char('H');
        assert_eq!(s.width(), 1);
        assert_eq!(s.height(), 1);
        assert_eq!(s.matrix.cells.len(), 1);
        assert_eq!(s.matrix.cells[0].code_point, 'H');
    }

    #[test]
    fn grow_left() {
        let mut s = Screen::new();
        s.make_room();
        s.x = -3;
        s.make_room();
        assert_eq!(s.width(), 4);
        assert_eq!(s.height(), 1);
        assert_eq!(s.matrix.cells.len(), 4);
        assert_eq!(s.x, 0);
        assert_eq!(s.y, 0);
    }

    #[test]
    fn grow_right() {
        let mut s = Screen::new();
        s.make_room();
        s.x = 3;
        s.make_room();
        assert_eq!(s.width(), 4);
        assert_eq!(s.height(), 1);
        assert_eq!(s.matrix.cells.len(), 4);
        assert_eq!(s.x, 3);
        assert_eq!(s.y, 0);
    }

    #[test]
    fn grow_up() {
        let mut s = Screen::new();
        s.make_room();
        s.y = -3;
        s.make_room();
        assert_eq!(s.width(), 1);
        assert_eq!(s.height(), 4);
        assert_eq!(s.matrix.cells.len(), 4);
        assert_eq!(s.x, 0);
        assert_eq!(s.y, 0);
    }

    #[test]
    fn grow_down() {
        let mut s = Screen::new();
        s.make_room();
        s.y = 3;
        s.make_room();
        assert_eq!(s.width(), 1);
        assert_eq!(s.height(), 4);
        assert_eq!(s.matrix.cells.len(), 4);
        assert_eq!(s.x, 0);
        assert_eq!(s.y, 3);
    }


    #[test]
    fn compacted_row() {

        // Matrix contains:
        // hello
        //       world
        //

        let mut s = Screen::new();
        s.place_str("hello");
        s.move_down();
        s.place_str("world");
        s.move_down();
        s.make_room();

        assert_eq!(s.height(), 3);

        let l0 = s.matrix.compacted_row(0);
        assert_eq!(l0.len(), 5);
        let c0: Vec<char> = l0.iter().map(|c| c.code_point).collect();
        assert_eq!(c0, ['h', 'e', 'l', 'l', 'o']);

        let l1 = s.matrix.compacted_row(1);
        assert_eq!(l1.len(), 10);
        let c1: Vec<char> = l1.iter().map(|c| c.code_point).collect();
        assert_eq!(c1, [' ', ' ', ' ', ' ', ' ', 'w', 'o', 'r', 'l', 'd']);

        let l2 = s.matrix.compacted_row(2);
        assert_eq!(l2.len(), 0);
    }

    // TODO: Test for protected
}
