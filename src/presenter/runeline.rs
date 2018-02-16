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

struct Cursor {
    char_index: usize,
    byte_index: usize,
}

pub struct Runeline {
    line: String,
    cursor: Cursor,
}

impl Cursor {
    fn new() -> Self {
        Self {
            char_index: 0,
            byte_index: 0,
        }
    }
}

impl Runeline {
    pub fn new() -> Self {
        Self {
            line: String::new(),
            cursor: Cursor::new(),
        }
    }

    pub fn clear(&mut self) -> String {
        use std::mem;
        self.cursor.char_index = 0;
        self.cursor.byte_index = 0;
        mem::replace(&mut self.line, String::new())
    }

    pub fn text(&self) -> &str {
        &self.line
    }

    pub fn text_before_cursor(&self) -> &str {
        &self.line[0..self.cursor.byte_index]
    }

    #[allow(dead_code)]
    pub fn char_index(&self) -> usize {
        self.cursor.char_index
    }

    pub fn move_left(&mut self) {
        if self.cursor.char_index > 0 {
            loop {
                assert!(self.cursor.byte_index > 0);
                self.cursor.byte_index -= 1;
                if self.is_char_boundary() {
                    break;
                }
            }
            self.cursor.char_index -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor.byte_index < self.line.len() {
            loop {
                self.cursor.byte_index += 1;
                if self.cursor.byte_index >= self.line.len() {
                    break;
                }
                if self.is_char_boundary() {
                    break;
                }
            }
            self.cursor.char_index += 1;
        }
    }

    pub fn insert_str(&mut self, text: &str) {
        self.line.insert_str(self.cursor.byte_index, text);
        self.cursor.byte_index += text.len();
        self.cursor.char_index += text.chars().count();
    }

    pub fn delete_right(&mut self) {
        // Find the length of the character under the cursor
        let mut rest = self.line.split_off(self.cursor.byte_index);
        let bytes_in_char = match rest.chars().next() {
            Some(c) => c.len_utf8(),
            None => 0,
        };
        self.line.push_str(&rest.split_off(bytes_in_char));
    }

    pub fn delete_left(&mut self) {
        if self.cursor.char_index > 0 {
            self.move_left();
            self.delete_right();
        }
    }

    fn is_char_boundary(&self) -> bool {
        self.line.is_char_boundary(self.cursor.byte_index)
    }

    pub fn replace(&mut self, s: String, stay_there: bool) {
        self.line = s;

        let ci = self.cursor.char_index;
        self.cursor.byte_index = 0;
        self.cursor.char_index = 0;
        if stay_there {
            for _i in 0..ci {
                self.move_right();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn append_empty() {
        {
            let mut rl = Runeline::new();
            rl.insert_str("h");
            assert_eq!(rl.line.len(), 1);
            assert_eq!(rl.cursor.byte_index, 1);
            assert_eq!(rl.cursor.char_index, 1);
        }
        {
            let mut rl = Runeline::new();
            rl.insert_str("ä");
            assert_eq!(rl.line.len(), 2);
            assert_eq!(rl.cursor.byte_index, 2);
            assert_eq!(rl.cursor.char_index, 1);
        }
    }

    #[test]
    fn move_around() {
        let mut rl = Runeline::new();
        rl.insert_str("Hällö");
        assert_eq!(rl.line.len(), 7);
        // Past the string
        assert_eq!(rl.cursor.byte_index, 7);
        assert_eq!(rl.cursor.char_index, 5);
        rl.move_left();
        // before ö
        assert_eq!(rl.cursor.byte_index, 5);
        assert_eq!(rl.cursor.char_index, 4);
        rl.move_left();
        // before l
        assert_eq!(rl.cursor.byte_index, 4);
        assert_eq!(rl.cursor.char_index, 3);
        rl.move_left();
        // before l
        assert_eq!(rl.cursor.byte_index, 3);
        assert_eq!(rl.cursor.char_index, 2);
        rl.move_left();
        // before ä
        assert_eq!(rl.cursor.byte_index, 1);
        assert_eq!(rl.cursor.char_index, 1);
        rl.move_left();
        // before H
        assert_eq!(rl.cursor.byte_index, 0);
        assert_eq!(rl.cursor.char_index, 0);
        rl.move_left();
        // before H
        assert_eq!(rl.cursor.byte_index, 0);
        assert_eq!(rl.cursor.char_index, 0);
        rl.move_right();
        // before ä
        assert_eq!(rl.cursor.byte_index, 1);
        assert_eq!(rl.cursor.char_index, 1);
        rl.move_right();
        // before l
        assert_eq!(rl.cursor.byte_index, 3);
        assert_eq!(rl.cursor.char_index, 2);
        rl.move_right();
        // before l
        assert_eq!(rl.cursor.byte_index, 4);
        assert_eq!(rl.cursor.char_index, 3);
        rl.move_right();
        // before ö
        assert_eq!(rl.cursor.byte_index, 5);
        assert_eq!(rl.cursor.char_index, 4);
        rl.move_right();
        // Past the string
        assert_eq!(rl.cursor.byte_index, 7);
        assert_eq!(rl.cursor.char_index, 5);
        rl.move_right();
        // Past the string
        assert_eq!(rl.cursor.byte_index, 7);
        assert_eq!(rl.cursor.char_index, 5);
    }

    #[test]
    fn delete_chars() {
        let mut rl = Runeline::new();
        rl.insert_str("Hällö Wörld!");

        assert_eq!(rl.line.len(), 15);

        for _i in 0..11 {
            rl.move_left();
        }
        rl.delete_right();

        assert_eq!(rl.line, "Hllö Wörld!");
        assert_eq!(rl.cursor.byte_index, 1);
        assert_eq!(rl.cursor.char_index, 1);

        rl.delete_right();

        assert_eq!(rl.line, "Hlö Wörld!");
        assert_eq!(rl.cursor.byte_index, 1);
        assert_eq!(rl.cursor.char_index, 1);

        rl.delete_left();

        assert_eq!(rl.line, "lö Wörld!");
        assert_eq!(rl.cursor.byte_index, 0);
        assert_eq!(rl.cursor.char_index, 0);
    }

}
