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

//! Byte Code for Shell Scripts

use super::super::session::{InteractionHandle, OutputVisibility, RunningStatus, SharedSession};
use super::parser::{
    AbstractSyntaxTree, Command, LogicalOperator, Pipeline, PipelineCommand, PipelineOperator,
};

/// Instructions to execute
pub type Instructions = Vec<Instruction>;

/// One instruction for the shell interpreter
///
/// # Example instructions
/// ## Simple command
///
/// Source:
///     ab cd
/// Byte Code:
///     Lit("ab") Word Lit("cd") Word Exec Wait
///
/// ## Pipe
///
/// Source:
///     ab cd | de ef
/// Byte Code:
///      Lit("ab") Word Lit("cd") Word Exec Lit("de") Word Lit("ef") Word Exec Wait
///
/// Source:
///     ab cd |& de ef
/// Byte Code:
///      Lit("ab") Word Lit("cd") Word Redirect(Stderr2Stderr) Exec Lit("de") Word Lit("ef") Word Exec Wait
///
/// ## Logical Processing
///
/// Source:
///     ab cd && de ef
/// Byte Code:
///      Lit("ab") Word Lit("cd") Word Exec Wait Success JumpIfNot(6) Lit("de") Word Lit("ef") Word Exec Wait
///
/// Source:
///     ab cd || de ef
/// Byte Code:
///      Lit("ab") Word Lit("cd") Word Exec Wait Success Not JumpIfNot(6) Lit("de") Word Lit("ef") Word Exec Wait
///
/// ## Backgrounding
///
/// Source:
///     ab cd && de ef &
/// Byte Code:
///      Background([ Lit("ab") Word Lit("cd") Word Exec Wait Success JumpIfNot(6) Lit("de") Word Lit("ef") Word Exec Wait])
///
#[derive(Debug, PartialEq)]
pub enum Instruction {
    /// Put a literal string on the stack of the last word in the launchpad
    Lit(String),

    /// Combine all stacks and store as words in the launchpad
    Word,

    /// Run the program on the launch pad.
    ///
    /// Connect the last stdout to stdin. Remember stderr / stdin for the next program.
    Exec,

    /// Wait for program to complete. Read from all remaining pipes until all programs close.
    Wait,

    /// Create a thread and a subshell, execute instructions.
    Background(Instructions),

    /// Create a subshell, execute instructions, then drop subshell.
    Subshell(Instructions),

    /// If the last command was a success, put true on the stack
    Success,

    /// If true was on the stack, continue with the next instruction. If anything else was there,
    /// move the instruction pointer according to the parameter.
    JumpIfNot(i32),

    /// Invert logical value of top-of-stack value
    Not,

    /// Placeholder for redirection
    Redirect,
}

/// The byte code interpreter.
///
/// Each instance is a separate shell.
pub struct Runner {
    /// Session to write output to
    session: SharedSession,

    /// Argument stacks for constructing arguments
    launchpad: Launchpad,
}

/// The array of stacks to construct command line arguments
struct Launchpad {
    /// One stack (inner Vec) for each future argument (outer Vec)
    args: Vec<Vec<String>>,

    /// Index of first argument that hasn't been finalized
    marker: usize,
}

impl Launchpad {
    fn new() -> Self {
        Self {
            args: Vec::new(),
            marker: 0,
        }
    }

    /// Ensure that there is at least one unmarked argument
    fn prepare_arg(&mut self) {
        if self.marker >= self.args.len() {
            self.marker = self.args.len();
            self.args.push(Vec::new());
        }
    }

    /// Return the incomplete argumens
    fn incomplete_args(&mut self) -> &mut [Vec<String>] {
        &mut self.args[self.marker..]
    }

    /// Add a literal string to all incomplete words
    fn lit(&mut self, l: &str) {
        self.prepare_arg();
        for arg in self.incomplete_args() {
            arg.push(l.to_string());
        }
    }

    /// Complete ann incomplete words
    fn finalize_words(&mut self) {
        for arg in self.incomplete_args() {
            if arg.len() != 1 {
                let mut res = String::new();
                for s in &*arg {
                    res.push_str(&s);
                }
                *arg = vec![res];
            }
        }
        self.marker = self.args.len();
    }
}

impl Runner {
    pub fn new(session: SharedSession) -> Self {
        Self {
            session,
            launchpad: Launchpad::new(),
        }
    }

    /// Run the instructions
    pub fn run(&mut self, instructions: &Instructions, interaction: InteractionHandle) {
        for i in instructions {
            match i {
                Instruction::Lit(s) => self.launchpad.lit(s),
                Instruction::Word => self.launchpad.finalize_words(),
                Instruction::Exec => {
                    // finalize the words to have single strings
                    self.launchpad.finalize_words();
                }

                _ => {
                    error!("Unhandled instruction {:?}", i);
                }
            }
        }
    }
}

fn compile_command<'a>(
    instructions: &mut Instructions,
    pipeline_command: PipelineCommand<'a>,
) -> Result<(), String> {
    match pipeline_command.command {
        Command::Program(args) => {
            for a in args {
                instructions.push(Instruction::Lit(a.to_string()));
                instructions.push(Instruction::Word);
            }
        }
    }
    match pipeline_command.operator {
        PipelineOperator::StderrAndStdout => {
            // TODO: Proper redirection
            instructions.push(Instruction::Redirect);
        }
        _ => {
            // No redirection required
        }
    }
    instructions.push(Instruction::Exec);

    Ok(())
}

fn compile_pipeline<'a>(
    instructions: &mut Instructions,
    jump_stack: &mut Vec<i32>,
    pipeline: Pipeline<'a>,
) -> Result<(), String> {
    for cmd in pipeline.commands {
        compile_command(instructions, cmd)?;
    }
    instructions.push(Instruction::Wait);
    match pipeline.operator {
        LogicalOperator::Nothing => {
            // Do nothing
        }
        LogicalOperator::Or | LogicalOperator::And => {
            instructions.push(Instruction::Success);
            if pipeline.operator == LogicalOperator::Or {
                instructions.push(Instruction::Not);
            }
            jump_stack.push(instructions.len() as i32);
            instructions.push(Instruction::JumpIfNot(0));
        }
    }

    Ok(())
}

pub fn compile<'a>(
    instructions: &mut Instructions,
    ast: AbstractSyntaxTree<'a>,
) -> Result<(), String> {
    match ast {
        AbstractSyntaxTree::Comment(_) | AbstractSyntaxTree::Nothing => {
            // Do nothing
        }
        AbstractSyntaxTree::Logical(terms, background_mode) => {
            // Compile the terms one by one
            // Remember where forward jumps were, so their targets can be fixed
            let mut jump_stack: Vec<i32> = Vec::new();
            for p in terms {
                compile_pipeline(instructions, &mut jump_stack, p)?;
            }
            let jump_tgt = instructions.len() as i32;
            for jump_source in jump_stack {
                if let Instruction::JumpIfNot(_) = instructions[jump_source as usize] {
                    instructions[jump_source as usize] =
                        Instruction::JumpIfNot(jump_tgt - jump_source);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::parser;
    use super::*;

    #[test]
    fn lit_and_finalize() {
        let mut lp = Launchpad::new();
        lp.lit("ten");
        lp.lit("nine");
        lp.lit("eight");

        assert_eq!(lp.marker, 0);
        assert_eq!(lp.args, vec![vec!["ten", "nine", "eight"]]);

        lp.finalize_words();
        assert_eq!(lp.marker, 1);
        assert_eq!(lp.args, vec![vec!["tennineeight"]]);
    }

    #[test]
    fn compile_logical() {
        let input = parser::Span::new("ab cd | ef gh ij || stuff\n");
        let ast = parser::script(input);
        assert_eq!(ast.is_ok(), true);
        if let Ok((rest, ast)) = ast {
            assert_eq!(
                rest,
                parser::Span {
                    offset: 26,
                    line: 2,
                    fragment: "",
                    extra: {}
                }
            );
            let mut instructions = Vec::new();
            let compile_result = super::compile(&mut instructions, ast);
            assert_eq!(compile_result.is_ok(), true);

            assert_eq!(
                instructions,
                vec![
                    Instruction::Lit("ab".to_string()),
                    Instruction::Word,
                    Instruction::Lit("cd".to_string()),
                    Instruction::Word,
                    Instruction::Exec,
                    Instruction::Lit("ef".to_string()),
                    Instruction::Word,
                    Instruction::Lit("gh".to_string()),
                    Instruction::Word,
                    Instruction::Lit("ij".to_string()),
                    Instruction::Word,
                    Instruction::Exec,
                    Instruction::Wait,
                    Instruction::Success,
                    Instruction::Not,
                    Instruction::JumpIfNot(5),
                    Instruction::Lit("stuff".to_string()),
                    Instruction::Word,
                    Instruction::Exec,
                    Instruction::Wait,
                ]
            );
        }
    }
}
