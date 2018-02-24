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
    temporary: Option<Context>,
    frames: Vec<Context>,
    global: Context,
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
            temporary: None,
            frames: vec![],
            global: Context::new(ContextType::Global, "", 0),
            remake_export_env: false,
        }
    }

    pub fn import_from_environment(&mut self) -> Result<()> {
        let mut remake_export_env = false;
        for (key, value) in ::std::env::vars() {
            let mut temp_var = self.bind_variable(&key, &value)?;
            temp_var.set_exported();
            if !script_parser::legal_identifier(&key) {
                temp_var.set_invisible();
            }
            remake_export_env = true;
        }
        self.remake_export_env = remake_export_env;
        Ok(())
    }

    pub fn bind_variable(&mut self, name: &str, value: &str) -> Result<&mut Variable> {
        if let Some(ref mut temp) = self.temporary {
            temp.bind_variable(name, value)
        } else {
            if let Some(frm) = self.frames.iter_mut().rev().find(
                |frm| frm.has_variable(name),
            )
            {
                frm.bind_variable(name, value)
            } else {
                self.global.bind_variable(name, value)
            }
        }
    }

    pub fn iter_exported<'a>(&'a self) -> Box<Iterator<Item=(OsString,OsString)>+'a>{
       let frame_vars = self.frames.iter().rev().flat_map( |frm| frm.iter_exported());

       let iter : Box<Iterator<Item=(OsString,OsString)>> = match &self.temporary {
           &Some(ref t) => Box::new(frame_vars.chain(t.iter_exported())),
           &None => Box::new(frame_vars)
       };
       Box::new(self.global.iter_exported().chain(iter))
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

    fn bind_variable<'a>(&'a mut self, name: &str, value: &str) -> Result<&'a mut Variable> {
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

    fn iter_exported<'a>(&'a self) -> Box<Iterator<Item=(OsString,OsString)>+'a>{
       Box::new(
           self.variables
           .iter()
           .filter(|&(_,v)| v.is_exported())
           .map(|(k,v)| (OsString::from(k),OsString::from(v.as_string()))))

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
            VariableValue::Scalar(_, ref s) => s
        }

    }

    pub fn is_writeable(&self) -> bool {
        !self.read_only
    }

    pub fn set_exported(&mut self) {
        self.exported = true;
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
}
