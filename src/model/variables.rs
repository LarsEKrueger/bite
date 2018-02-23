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

enum ContextType {
    Function,
    Builtin,
    Temp,
}

struct VariableContext {
    name: String,
    scope: i32,
    ctxType: ContextType,
    has_locals: bool,
    has_tempvars: bool,
    variables: HashMap<String, Variable>,
}

struct VariableContextStack(Vec<VariableContext>);

enum VariableType {
    Integer,
    String,
    LowerCase,
    UpperCase,
}

trait DynamicVariable {
    fn get(&self) -> String;
    fn set(&mut self, &str);
}

enum VariableKind {
    NameRef(String),
    Scalar(VariableType, String),
    Indexed(VariableType, Vec<String>),
    Associated(VariableType, HashMap<String, String>),
    Dynamic(Box<DynamicVariable>),
}

struct Variable {
    kind: VariableKind,
    ro: bool,
    exported: bool,
}
