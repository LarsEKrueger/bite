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

//! Environment variables for bash interpreter.

use std::collections::HashMap;
use boolinator::Boolinator;

use model::bash::script_parser;
use model::error::{Error, Result};

use std::ffi::OsString;

/// Stack of variables.
///
///
pub struct Stack {
    frames: Vec<Context>,
    remake_export_env: bool,
}

/// A stack frame, named context as in bash.
///
/// TODO: Does scope have to be signed?
pub struct Context {
    name: String,
    scope: i32,
    ctxType: ContextType,
    has_locals: bool,
    has_tempvars: bool,
    variables: HashMap<String, Variable>,
}

/// The type of the context.
///
/// There should be only one global frame at the bottom of the stack.
#[derive(PartialEq)]
pub enum ContextType {
    Global,
    Function,
    Builtin,
    Temp,
}

/// A variable and its flags.
pub struct Variable {
    value: VariableValue,
    read_only: bool,
    exported: bool,
    visible: bool,
}

/// The value of a variable.
///
/// As arrays have the same type for all values, we need to move that outside the value type. We
/// store the even the integer values as strings like bash does, even if it hurts to read.
pub enum VariableValue {
    //NameRef(String),
    Scalar(VariableType, String),
    //Indexed(VariableType, Vec<String>),
    //Associated(VariableType, HashMap<String, String>),
    //Dynamic(Box<DynamicVariable>),
}

/// The type of the variable.
///
/// This mostly influences the setters.
#[derive(Clone, Copy)]
pub enum VariableType {
    // Integer,
    String,
    // LowerCase,
    // UpperCase,
}

/// A dynamic variable will be read and might be set.
pub trait DynamicVariable {
    fn get(&self) -> String;
    fn set(&mut self, &str);
}

impl Stack {
    pub fn new() -> Self {
        Self {
            frames: vec![Context::new(ContextType::Global, "", 0)],
            remake_export_env: false,
        }
    }

    pub fn import_from_environment(&mut self) -> Result<()> {
        let mut remake_export_env = false;
        for (key, value) in ::std::env::vars() {
            let mut temp_var = self.bind_variable(&key, &value)?;
            temp_var.set_exported(true);
            if !script_parser::legal_identifier(&key) {
                temp_var.set_invisible();
            }
            remake_export_env = true;
        }
        self.remake_export_env = remake_export_env;
        Ok(())
    }

    pub fn bind_variable(&mut self, name: &str, value: &str) -> Result<&mut Variable> {
        if let Some(pos) = self.frames.iter_mut().rev().position(
            |frm| frm.has_variable(name),
        )
        {
            let pos = self.frames.len() - 1 - pos;
            self.frames[pos].bind_variable(name, value)
        } else {
            // Set it in global context.
            if let Some(global) = self.frames.first_mut() {
                global.bind_variable(name, value)
            } else {
                Err(Error::InternalError(
                    file!(),
                    line!(),
                    String::from("no global context found"),
                ))
            }
        }
    }

    pub fn find_variable(&mut self, name: &str) -> Option<&mut Variable> {
        if let Some(pos) = self.frames.iter_mut().rev().position(|ctx| {
            ctx.find_variable(name).is_some()
        })
        {
            let pos = self.frames.len() - 1 - pos;
            let ref mut frm = self.frames[pos];
            frm.find_variable(name)
        } else {
            None
        }
    }

    pub fn find_variable_or_create_global(&mut self, name: &str) -> Result<&mut Variable> {
        if let Some(pos) = self.frames.iter_mut().rev().position(
            |frm| frm.has_variable(name),
        )
        {
            let pos = self.frames.len() - 1 - pos;
            let l = self.frames.len();
            if let Some(variable) = self.frames[pos].find_variable(name) {
                Ok(variable)
            } else {
                Err(Error::InternalError(
                    file!(),
                    line!(),
                    format!(
                        "variable could not be found again (pos={},frames={})",
                        pos,
                        l
                    ),
                ))
            }
        } else {
            // Set it in global context.
            if let Some(global) = self.frames.first_mut() {
                global.bind_variable(name, "")
            } else {
                Err(Error::InternalError(
                    file!(),
                    line!(),
                    String::from("no global context found"),
                ))
            }
        }
    }

    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = (&String, &Variable)> + 'a> {
        Box::new(self.frames.iter().rev().flat_map(|frm| frm.iter()))
    }

    pub fn iter_exported<'a>(&'a self) -> Box<Iterator<Item = (OsString, OsString)> + 'a> {
        Box::new(self.frames.iter().rev().flat_map(|frm| frm.iter_exported()))
    }

    pub fn get_global_context(&mut self) -> Result<&mut Context> {
        if let Some(ctx) = self.frames.first_mut() {
            Ok(ctx)
        } else {
            Err(Error::InternalError(
                file!(),
                line!(),
                String::from("no global context found"),
            ))
        }
    }

    pub fn create_temp_context(&mut self) -> &mut Context {
        if let Some(pos) = self.frames.iter().rev().position(|ctx| ctx.is_temp()) {
            let pos = self.frames.len() - 1 - pos;
            &mut self.frames[pos]
        } else {
            let l = self.frames.len();
            self.frames.push(
                Context::new(ContextType::Temp, "", (l) as i32),
            );
            self.frames.last_mut().unwrap()
        }
    }

    pub fn drop_temp_context(&mut self) {
        loop {
            let drop = if let Some(true) = self.frames.last().map(|t| t.is_temp()) {
                true
            } else {
                false
            };
            if drop {
                self.frames.pop();
            } else {
                break;
            }
        }
    }
}

impl Context {
    fn new(ctxType: ContextType, name: &str, scope: i32) -> Self {
        Self {
            name: String::from(name),
            scope,
            ctxType,
            has_locals: false,
            has_tempvars: false,
            variables: HashMap::new(),
        }
    }

    fn is_temp(&self) -> bool {
        self.ctxType == ContextType::Temp
    }

    fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn bind_variable<'a>(&'a mut self, name: &str, value: &str) -> Result<&'a mut Variable> {
        use std::collections::hash_map::Entry;
        match self.variables.entry(String::from(name)) {
            Entry::Occupied(o) => {
                let var = o.into_mut();
                var.is_writeable().ok_or_else(|| {
                    Error::VariableIsReadOnly(String::from(name))
                })?;
                var.set_value(value);
                var.set_visible();
                Ok(var)
            }
            Entry::Vacant(v) => Ok(v.insert(Variable::new_scalar_string(value))),
        }
    }

    fn find_variable<'a>(&'a mut self, name: &str) -> Option<&'a mut Variable> {
        self.variables.get_mut(name)
    }

    fn iter<'a>(&'a self) -> Box<Iterator<Item = (&String, &Variable)> + 'a> {
        Box::new(self.variables.iter())
    }

    fn iter_exported<'a>(&'a self) -> Box<Iterator<Item = (OsString, OsString)> + 'a> {
        Box::new(
            self.variables
                .iter()
                .filter(|&(_, v)| v.is_exported())
                .map(|(k, v)| (OsString::from(k), OsString::from(v.as_string()))),
        )

    }
}


impl Variable {
    fn new_scalar_string(value: &str) -> Self {
        Self {
            value: VariableValue::Scalar(VariableType::String, String::from(value)),
            read_only: false,
            exported: false,
            visible: true,
        }
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = match self.value {
            VariableValue::Scalar(ref var_type, _) => {
                VariableValue::Scalar(*var_type, String::from(value))
            }
        };
    }

    pub fn as_string(&self) -> &String {
        match self.value {
            VariableValue::Scalar(_, ref s) => s,
            // TODO: Add cases for other types
        }
    }

    pub fn is_writeable(&self) -> bool {
        !self.read_only
    }

    pub fn is_readonly(&self) -> bool {
        self.read_only
    }

    pub fn set_readonly(&mut self, ro: bool) {
        self.read_only = ro;
    }

    pub fn set_exported(&mut self, exp: bool) {
        self.exported = exp;
    }

    pub fn is_exported(&self) -> bool {
        self.exported
    }

    pub fn set_visible(&mut self) {
        self.visible = true;
    }

    pub fn set_invisible(&mut self) {
        self.visible = false;
    }

    pub fn print_for_builtins(&self, name: &str, w: &mut ::std::io::Write) {
        let mut flags = String::new();
        if self.read_only {
            flags += "r"
        }
        if self.exported {
            flags += "x"
        }
        let (fs, assignment) = match self.value {
            VariableValue::Scalar(VariableType::String, ref s) => ("", format!("=\"{}\"", s)),
            // TODO: Add cases for other types
        };
        flags += fs;
        if !flags.is_empty() {
            flags.insert(0, '-');
        }
        write!(w, "declare {} {}{}\n", flags, name, assignment).expect("internal error");
    }
}
