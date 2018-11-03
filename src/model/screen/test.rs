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
use ::tools;

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

/// Check if the two parts evaluate to identical values. If not, print a message and set the
/// variable `unexpected`.
///
/// Slightly adapted version of assert_eq macro.
macro_rules! expect_eq {
    ($unexpected:ident, $left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        if !((left_val) == (right_val)) {
            eprintln!(r#"expectation failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, left_val, right_val);
            $unexpected = true;
        }
    }
}

/// Allocate a new screen of the given size and fill it with pseudo-random, but valid data.
///
/// This function is the reference implementation for xterm-test's initialisation.
fn new_test_screen(w:usize,h:usize) -> Screen {
    // Allocate large-enough matrix
    let mut s = Screen::new();
    s.make_room_for((w as isize)-1, (h as isize) - 1);
    s.fixed_size();

    // Fill the character matrix with pseudo-random ascii characters.
    let mut state8 = 42;
    let mut state13 = 4242;
    for cell in s.matrix.cells.iter_mut() {
        cell.code_point = tools::prng::rng8_char(&mut state8);
        cell.attributes = Attributes::from_bits_truncate(state13);
        tools::prng::rng13(&mut state13);
    }
    s
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
    s.move_down(false);
    s.place_str("world");
    s.move_down(false);
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
fn newline() {
    let mut s = Screen::new();
    s.add_bytes(b"hello\nworld\n");

    assert_eq!(s.height(), 2);
    check_compacted_row(&s, 0, "hello");
    check_compacted_row(&s, 1, "world");

    assert_eq!(s.line_iter().count(), 2);
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
fn init_screen() {
  let s = new_test_screen(8,6);
  assert_eq!(s.matrix.columns(), 8);
  assert_eq!(s.matrix.rows(), 6);
  assert_eq!(s.cursor.x,0);
  assert_eq!(s.cursor.y,0);
  assert_eq!(s.matrix.cells[0].code_point, 74 as char);
  assert_eq!(s.matrix.cells[0].attributes, Attributes::from_bits_truncate(4242));
}

include!(concat!(env!("OUT_DIR"), "/xterm_tests/test_initializer.rs"));
include!(concat!(env!("OUT_DIR"), "/xterm_tests/test_simple_text.rs"));

// TODO: Test for protected
