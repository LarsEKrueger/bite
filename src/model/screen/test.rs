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

//! Test for screen module

#![cfg(test)]

use super::*;

fn check_compacted_row(s: &Screen, row: isize, gt: &str) {
    let cr = s.matrix.compacted_row(row);
    let gti = gt.chars();
    //assert_eq!(cr.len(), gti.clone().count());
    let crc = cr.into_iter().map(|c| c.code_point);
    assert!(
        crc.clone().eq(gti.clone()),
        "found: '{}'. expected: '{}'",
        crc.collect::<String>(),
        gti.collect::<String>()
    );
}

/// Test object as thin wrapper around screen.
struct Test(Screen);

impl Test {
    /// Create a test object with an empty screen, then add bytes to the screen.
    fn e(bytes: &[u8]) -> Test {
        let mut s = Screen::new();
        s.add_bytes(bytes);
        Test(s)
    }

    /// Create a test object with a fixed-sized screen, then add bytes.
    fn s(w: isize, h: isize, bytes: &[u8]) -> Test {
        let mut s = Screen::new();
        s.make_room_for(w - 1, h - 1);
        s.fixed_size();
        s.add_bytes(bytes);
        Test(s)
    }

    fn check<T: PartialEq + std::fmt::Debug>(self, gt: T, map: fn(&Screen) -> T) -> Test {
        assert_eq!(map(&self.0), gt);
        self
    }

    /// Check that a compacted row matches the ground truth
    fn cr(self, row: isize, gt: &str) -> Test {
        check_compacted_row(&self.0, row, gt);
        self
    }

    /// Check the width of the screen
    fn width(self, gt: isize) -> Test {
        assert_eq!(self.0.width(), gt);
        self
    }

    /// Check the height of the screen
    fn height(self, gt: isize) -> Test {
        assert_eq!(self.0.height(), gt);
        self
    }

    /// Check if the cursor positions
    fn cp(self, gt_x: isize, gt_y: isize) -> Test {
        assert_eq!(self.0.cursor.x, gt_x);
        assert_eq!(self.0.cursor.y, gt_y);
        self
    }
}


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
    s.cursor.x = -3;
    s.make_room();
    assert_eq!(s.width(), 4);
    assert_eq!(s.height(), 1);
    assert_eq!(s.matrix.cells.len(), 4);
    assert_eq!(s.cursor.x, 0);
    assert_eq!(s.cursor.y, 0);
}

#[test]
fn grow_right() {
    let mut s = Screen::new();
    s.make_room();
    s.cursor.x = 3;
    s.make_room();
    assert_eq!(s.width(), 4);
    assert_eq!(s.height(), 1);
    assert_eq!(s.matrix.cells.len(), 4);
    assert_eq!(s.cursor.x, 3);
    assert_eq!(s.cursor.y, 0);
}

#[test]
fn grow_up() {
    let mut s = Screen::new();
    s.make_room();
    s.cursor.y = -3;
    s.make_room();
    assert_eq!(s.width(), 1);
    assert_eq!(s.height(), 4);
    assert_eq!(s.matrix.cells.len(), 4);
    assert_eq!(s.cursor.x, 0);
    assert_eq!(s.cursor.y, 0);
}

#[test]
fn grow_down() {
    let mut s = Screen::new();
    s.make_room();
    s.cursor.y = 3;
    s.make_room();
    assert_eq!(s.width(), 1);
    assert_eq!(s.height(), 4);
    assert_eq!(s.matrix.cells.len(), 4);
    assert_eq!(s.cursor.x, 0);
    assert_eq!(s.cursor.y, 3);
}

#[test]
fn compacted_row() {
    // Matrix contains:
    // hello
    //       world
    //
    let mut s = Screen::new();
    s.place_str("hello");
    s.move_down(1);
    s.place_str("world");
    s.move_down(1);
    s.make_room();

    assert_eq!(s.height(), 3);

    let l0 = s.matrix.compacted_row(0);
    assert_eq!(l0.len(), 5);
    let c0: Vec<char> = l0.iter().map(|c| c.code_point).collect();
    assert_eq!(c0, ['h', 'e', 'l', 'l', 'o']);

    check_compacted_row(&s, 0, "hello");
    check_compacted_row(&s, 1, "     world");

    let l2 = s.matrix.compacted_row(2);
    assert_eq!(l2.len(), 0);
}

#[test]
fn empty_cell_vec() {
    let v = Screen::one_line_cell_vec(b"");
    assert_eq!(v.len(), 0);
}

#[test]
fn delete_char() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Delete the e
    s.cursor.x = 1;
    s.cursor.y = 0;
    s.delete_character();

    assert_eq!(s.height(), 2);
    check_compacted_row(&s, 0, "hllo");
    check_compacted_row(&s, 1, "world");
}

#[test]
fn insert_char() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Insert before the e
    s.cursor.x = 1;
    s.cursor.y = 0;
    s.insert_character();

    assert_eq!(s.height(), 2);
    check_compacted_row(&s, 0, "h ello");
    check_compacted_row(&s, 1, "world");
}

#[test]
fn delete_row_0() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    assert_eq!(s.height(), 2);
    s.delete_row();
    assert_eq!(s.height(), 2);
    check_compacted_row(&s, 0, "hello");
    check_compacted_row(&s, 1, "world");

    // Delete the first row
    s.cursor.x = 1;
    s.cursor.y = 0;
    s.delete_row();

    assert_eq!(s.height(), 1);
    check_compacted_row(&s, 0, "world");
}

#[test]
fn delete_row_1() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Delete the first row
    s.cursor.x = 1;
    s.cursor.y = 1;
    s.delete_row();

    assert_eq!(s.height(), 1);
    check_compacted_row(&s, 0, "hello");
}

#[test]
fn insert_row() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Insert a row between the two
    s.cursor.x = 1;
    s.cursor.y = 0;
    s.insert_row();

    assert_eq!(s.height(), 3);
    check_compacted_row(&s, 0, "hello");
    check_compacted_row(&s, 1, "");
    check_compacted_row(&s, 2, "world");
}

#[test]
fn insert_row_one_line() {
    let mut s = Screen::new();
    s.add_bytes(b"hello");
    s.insert_row();

    assert_eq!(s.height(), 2);
    check_compacted_row(&s, 0, "hello");
    check_compacted_row(&s, 1, "");
}

#[test]
fn break_line() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Break the first line between the l
    s.cursor.x = 3;
    s.cursor.y = 0;
    s.break_line();

    assert_eq!(s.cursor.x, 0);
    assert_eq!(s.cursor.y, 1);

    assert_eq!(s.height(), 3);
    check_compacted_row(&s, 0, "hel");
    check_compacted_row(&s, 1, "lo");
    check_compacted_row(&s, 2, "world");
}

#[test]
fn break_line_at_end() {
    let mut s = Screen::new();
    s.add_bytes(b"hello");
    s.break_line();

    assert_eq!(s.cursor.x, 0);
    assert_eq!(s.cursor.y, 1);

    assert_eq!(s.height(), 2);
    check_compacted_row(&s, 0, "hello");
    check_compacted_row(&s, 1, "");
}

#[test]
fn text_before_cursor() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Get hell
    s.cursor.x = 4;
    s.cursor.y = 0;
    let tbc = s.text_before_cursor();
    assert_eq!(tbc.as_str(), "hell");
}

#[test]
fn join_next_line() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    // Get hell
    s.cursor.x = 5;
    s.cursor.y = 0;
    s.join_next_line();
    assert_eq!(s.height(), 1);
    check_compacted_row(&s, 0, "helloworld");
}

#[test]
fn empty_screen_iter() {
    let mut s = Screen::new();
    assert_eq!(s.line_iter().count(), 0);

    s.add_bytes(b"stuff\n");
    assert_eq!(s.line_iter().count(), 1);
    s.add_bytes(b"stuff\nstuff\n");
    assert_eq!(s.line_iter().count(), 3);
}

#[test]
fn bell() {
    let mut s = Screen::new();
    assert_eq!(s.add_byte(b'\x07'), Event::Bell);
}

#[test]
fn simple_text() {
    Test::e(b"he\rwo").cr(0, "wo").height(1);
    Test::e(b"he\nwo\n")
        .cr(0, "he")
        .cr(1, "wo")
        .height(2)
        .check(2, |ref s| s.line_iter().count());
}

#[test]
fn cursor_motion() {
    // CursorLowerLeft
    Test::s(80, 25, b"Hello\x1bFWorld!")
        .cr(0, "Hello")
        .cr(1, "")
        .cr(24, "World!")
        .cp(6, 24);
    // Scroll up due to newline in last row
    Test::s(80, 25, b"Hello\x1bFWorld!\n").cr(23, "World!").cp(
        0,
        24,
    );

    // CursorAbsoluteColumn
    Test::s(80, 25, b"Hello World!\x1b[5Gxxxx")
        .cr(0, "Hellxxxxrld!")
        .cp(8, 0);
    Test::e(b"Hello World!\x1b[5Gxxxx\x1b[32G")
        .cr(0, "Hellxxxxrld!")
        .cp(31, 0);

    // CursorAbsolutePosition
    Test::s(80, 25, b"Hello\nWorld!\x1b[1;5Hxxxx")
        .cr(0, "Hellxxxx")
        .cr(1, "World!");
    // Wrap around at right edge
    Test::s(10, 25, b"Hello\nWorld!\x1b[1;5Hxxxx\x1b[2;80HStuff")
        .width(10)
        .cr(0, "Hellxxxx")
        .cr(1, "World!   S")
        .cr(2, "tuff");
    // Scroll up last row
    Test::s(10, 25, b"Hello\nWorld!\x1b[1;5Hxxxx\x1b[25;80HStuff")
        .width(10)
        .cr(0, "World!")
        .cr(23, "         S")
        .cr(24, "tuff");
    // Adaptive size will create new rows/columns as required
    Test::e(b"Hello\nWorld!\x1b[3;7Hxxxx")
        .width(10)
        .height(3)
        .cr(0, "Hello")
        .cr(1, "World!")
        .cr(2, "      xxxx");

    // CursorDown
    Test::s(80, 25, b"Hello\x1b[5BWorld").cr(0, "Hello").cr(
        5,
        "     World",
    );
    Test::e(b"Hello\x1b[5BWorld").height(6).cr(0, "Hello").cr(
        5,
        "     World",
    );

    // CursorUp
    Test::s(80, 25, b"\x1b[5BHello\x1b[2AWorld")
        .cr(5, "Hello")
        .cr(3, "     World");
    Test::e(b"World!\x1b[1AHello")
        .height(2)
        .cr(0, "      Hello")
        .cr(1, "World!");

    // CursorForward
    Test::s(10, 25, b"\x1b[12CHello\nWorld")
        .cr(0, "         H")
        .cr(1, "ello")
        .cr(2, "World");
    Test::e(b"\x1b[12CHello\nWorld")
        .height(2)
        .width(17)
        .cr(0, "            Hello")
        .cr(1, "World");

    // CursorBackward
    Test::s(10, 25, b"Hello\x1b[12DWorld").cr(0, "World");
    Test::e(b"Hello\x1b[12DWorld").cr(0, "World  Hello");

    // VerticalPositionRelative
    Test::s(80, 25, b"Hello\x1b[2eWorld")
        .cp(10, 2)
        .cr(0, "Hello")
        .cr(2, "     World");

    // VerticalPositionAbsolute
    Test::s(80, 25, b"\x1b[5dHello\x1b[2dWorld")
        .cp(10, 1)
        .cr(4, "Hello")
        .cr(1, "     World");

    // Cursor Next Line
    Test::s(80, 25, b"Hello\x1b[3EWorld")
        .cp(5, 3)
        .cr(0, "Hello")
        .cr(3, "World");

    // Cursor Previous Line
    Test::s(80, 25, b"\x1b[5eHello\x1b[3FWorld")
        .cp(5, 2)
        .cr(5, "Hello")
        .cr(2, "World");

    // Next Line
    Test::s(80, 25, b"Hello\x1bEWorld")
        .cp(5, 1)
        .cr(0, "Hello")
        .cr(1, "World");

    // Index
    Test::s(80, 25, b"Hello\x1bDWorld")
        .cp(10, 1)
        .cr(0, "Hello")
        .cr(1, "     World");
    Test::s(80, 25, b"\x1b[25dHello\x1bDWorld")
        .cp(10, 24)
        .cr(23, "Hello")
        .cr(24, "     World");

    // Reverse Index
    Test::s(80, 25, b"Hello\x1bMWorld")
        .cp(10, 0)
        .cr(0, "     World")
        .cr(1, "Hello");

    // Scroll up
    Test::s(80, 25, b"\n\n\n\nHello\nWorld\x1b[4STest")
        .cp(9, 5)
        .cr(0, "Hello")
        .cr(1, "World")
        .cr(5, "     Test");

    // Scroll down
    Test::s(80, 25, b"Hello World\n\x1b[1TTest ").cp(5, 1).cr(
        1,
        "Test  World",
    );

    // Scroll Left
    Test::s(10, 25, b"Hello\n0123456789World\x1b[4 @")
        .cp(5, 2)
        .cr(0, "o")
        .cr(1, "456789")
        .cr(2, "d");

    // Scroll Right
    Test::s(10, 25, b"Hello\n0123456789World\x1b[4 A")
        .cp(5, 2)
        .cr(0, "    Hello")
        .cr(1, "    012345")
        .cr(2, "    World");

    // Backspace
    Test::s(80, 25, b"Hello\x08\x08\x08art").cp(5, 0).cr(
        0,
        "Heart",
    );

    // DEC Back Index
    Test::s(80, 25, b"Hello\x1b6\x1b6\x1b6\x1b6a\x1b6\x1b6\x1b6_")
        .cp(1, 0)
        .cr(0, "_Hallo");

    // DEC Forward Index
    Test::s(10, 25, b"Hello\x1b9\x1b9\x1b9\x1b9\x1b9")
        .cp(9, 0)
        .cr(0, "ello");

    // FillArea
    Test::s(80, 25, b"Hello\x1b[33;2;4;10;12$x")
        .cp(5, 0)
        .cr(0, "Hello")
        .cr(1, "   !!!!!!!!!")
        .cr(2, "   !!!!!!!!!")
        .cr(3, "   !!!!!!!!!")
        .cr(4, "   !!!!!!!!!")
        .cr(5, "   !!!!!!!!!")
        .cr(6, "   !!!!!!!!!")
        .cr(7, "   !!!!!!!!!")
        .cr(8, "   !!!!!!!!!")
        .cr(9, "   !!!!!!!!!");

    // Copy Area
    Test::s(80, 25, b"Hello World\n0123456789\x1b[1;3;2;6;0;3;12;0$v")
        .cp(10, 1)
        .cr(0, "Hello World")
        .cr(1, "0123456789")
        .cr(2, "           llo ")
        .cr(3, "           2345");
    Test::s(20, 25, b"Hello World\n0123456789\x1b[1;3;2;10;0;3;18;0$v")
        .cp(10, 1)
        .cr(0, "Hello World")
        .cr(1, "0123456789")
        .cr(2, "                 llo")
        .cr(3, "                 234");

    // Insert Column
    Test::s(80, 25, b"Hello World\n0123456789\n     \x1b[4'}Stuff")
        .cp(10, 2)
        .cr(0, "Hello     World")
        .cr(1, "01234    56789")
        .cr(2, "     Stuff");

    // Delete Column
    Test::s(80, 25, b"Hello World\n0123456789\n     \x1b[4'~Stuff")
        .cp(10, 2)
        .cr(0, "Hellold")
        .cr(1, "012349")
        .cr(2, "     Stuff");

    // EraseArea
    Test::s(80, 25, b"Hello World\n0123456789\n01234\n\x1b[2;3;4;9$z")
        .cp(0, 3)
        .cr(0, "Hello World")
        .cr(1, "01       9")
        .cr(2, "01");

}

// TODO: Test for protected
