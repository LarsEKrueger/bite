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

//! Data Stack for Byte Code Interpreter

enum Value {
    Bool(bool),
    Int(i32),
    String(String),
}

pub struct Stack(Vec<Value>);

impl Stack {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push_bool(&mut self, b: bool) {
        self.0.push(Value::Bool(b));
    }

    pub fn push_int(&mut self, i: i32) {
        self.0.push(Value::Int(i));
    }

    pub fn push_str(&mut self, s: String) {
        self.0.push(Value::String(s));
    }

    fn pop<F>(&mut self, f: F) -> Value
    where
        F: Fn() -> Value,
    {
        if let Some(v) = self.0.pop() {
            v
        } else {
            f()
        }
    }

    pub fn pop_bool(&mut self, d: bool) -> bool {
        match self.pop(|| Value::Bool(d)) {
            Value::Bool(b) => b,
            Value::Int(i) => {
                error!("Top of stack was Int({}), expected Bool.", i);
                i != 0
            }
            Value::String(s) => {
                error!("Top of stack was String({}), expected Bool.", s);
                if (s == "yes") || (s == "true") {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn pop_int(&mut self, d: i32) -> i32 {
        match self.pop(|| Value::Int(d)) {
            Value::Bool(b) => {
                error!("Top of stack was Bool({}), expected Int.", b);
                if b {
                    1
                } else {
                    0
                }
            }
            Value::Int(i) => i,
            Value::String(s) => {
                error!("Top of stack was String({}), expected Int.", s);
                match s.parse::<i32>() {
                    Ok(i) => i,
                    _ => d,
                }
            }
        }
    }

    pub fn pop_str(&mut self, d: &str) -> String {
        match self.pop(|| Value::String(d.to_string())) {
            Value::Bool(b) => {
                error!("Top of stack was Bool({}), expected String.", b);
                if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Int(i) => {
                error!("Top of stack was Int({}), expected String.", i);
                format!("{}", i)
            }
            Value::String(s) => s,
        }
    }
}
