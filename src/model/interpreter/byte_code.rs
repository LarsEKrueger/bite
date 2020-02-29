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

use super::super::session::{InteractionHandle, OutputVisibility, SharedSession};
use super::jobs;
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
///     Begin Lit("ab") Word Lit("cd") Word Exec Wait
///
/// ## Pipe
///
/// Source:
///     ab cd | de ef
/// Byte Code:
///      Begin Lit("ab") Word Lit("cd") Word Exec Lit("de") Word Lit("ef") Word Exec Wait
///
/// Source:
///     ab cd |& de ef
/// Byte Code:
///      Begin Lit("ab") Word Lit("cd") Word Redirect(Stderr2Stderr) Exec Lit("de") Word Lit("ef") Word Exec Wait
///
/// ## Logical Processing
///
/// Source:
///     ab cd && de ef
/// Byte Code:
///      Begin Lit("ab") Word Lit("cd") Word Exec Wait Success JumpIfNot(6) Lit("de") Word Lit("ef") Word Exec Wait
///
/// Source:
///     ab cd || de ef
/// Byte Code:
///      Begin Lit("ab") Word Lit("cd") Word Exec Wait Success Not JumpIfNot(6) Lit("de") Word Lit("ef") Word Exec Wait
///
/// ## Backgrounding
///
/// Source:
///     ab cd && de ef &
/// Byte Code:
///      Begin Background([ Lit("ab") Word Lit("cd") Word Exec Wait Success JumpIfNot(6) Lit("de") Word Lit("ef") Word Exec Wait])
///
#[derive(Debug, PartialEq)]
pub enum Instruction {
    /// Begin a new pipeline
    Begin,

    /// Put a literal string on the stack of the last word in the launchpad
    Lit(String),

    /// Combine all stacks and store as words in the launchpad
    Word,

    /// Set the program name from the first word on the launch pad
    SetProgram,

    /// Run the program on the launch pad.
    ///
    /// Connect the last stdout to stdin. Remember stderr / stdin for the next program.
    ///
    /// Parameter: true if this is the last command of the pipeline
    Exec(bool),

    /// Wait for program to complete. Read from all remaining pipes until all programs close.
    ForegroundJob,

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

    /// Job list
    pub jobs: jobs::SharedJobs,

    /// Argument stacks for constructing arguments
    launchpad: Launchpad,

    /// Job being started
    current_pipeline: Option<jobs::PipelineBuilder>,
}

/// The array of stacks to construct command line arguments
#[derive(Debug)]
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

    fn clear(&mut self) {
        self.args = Vec::new();
        self.marker = 0;
    }
}

impl Runner {
    pub fn new(session: SharedSession, jobs: jobs::SharedJobs) -> Self {
        Self {
            session,
            jobs,
            launchpad: Launchpad::new(),
            current_pipeline: None,
        }
    }

    fn check_error<T, F>(
        &mut self,
        interaction: InteractionHandle,
        res: Result<T, String>,
        mut f: F,
    ) where
        F: FnMut(&mut Self, T),
    {
        match res {
            Ok(value) => f(self, value),
            Err(msg) => self.session.add_bytes(
                OutputVisibility::Error,
                interaction,
                format!("BiTE: {}", msg).as_bytes(),
            ),
        }
    }

    /// Run the instructions
    pub fn run(&mut self, instructions: &Instructions, interaction: InteractionHandle) {
        for i in instructions {
            match i {
                Instruction::Begin => {
                    if self.current_pipeline.is_some() {
                        error!(
                            "Overwriting existing pipeline builder »{:?}«",
                            self.current_pipeline
                        );
                    }
                    self.check_error(
                        interaction,
                        jobs::PipelineBuilder::new(interaction),
                        |runner, pb| runner.current_pipeline = Some(pb),
                    );
                }

                Instruction::Lit(s) => self.launchpad.lit(s),
                Instruction::Word => self.launchpad.finalize_words(),
                Instruction::SetProgram => {
                    // finalize the words to have single strings
                    self.launchpad.finalize_words();
                    if self.launchpad.args.len() != 1 {
                        warn!(
                            "Launchpad isn't 1 word long for program name: »{:?}«",
                            self.launchpad
                        );
                    }

                    if let Some(ref mut pb) = self.current_pipeline {
                        let mut name_stack = self.launchpad.args.remove(0);
                        let name = name_stack.remove(0);
                        pb.set_program(name);
                    } else {
                        error!("No pipeline builder in SetProgram");
                    }
                }
                Instruction::Exec(is_last) => {
                    // finalize the words to have single strings
                    self.launchpad.finalize_words();
                    if let Some(ref mut pb) = self.current_pipeline {
                        let args = self.launchpad.args.drain(0..).map(|mut w| w.remove(0));
                        let res = pb.start(*is_last, args);
                        self.check_error(interaction, res, |_, _| {});
                    } else {
                        error!("No pipeline builder in Exec");
                    }
                    self.launchpad.clear();
                }

                Instruction::ForegroundJob => {
                    let maybe_pb = std::mem::replace(&mut self.current_pipeline, None);
                    if let Some(pb) = maybe_pb {
                        self.jobs.foreground_job(self.session.clone(), pb);
                    } else {
                        error!("No pipeline builder in ForegroundJob");
                    }
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
    is_last: bool,
) -> Result<(), String> {
    match pipeline_command.command {
        Command::Program(args) => {
            let mut is_first = true;
            for a in args {
                instructions.push(Instruction::Lit(a.to_string()));
                instructions.push(Instruction::Word);
                if is_first {
                    instructions.push(Instruction::SetProgram);
                    is_first = false;
                }
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
    instructions.push(Instruction::Exec(is_last));

    Ok(())
}

fn compile_pipeline<'a>(
    instructions: &mut Instructions,
    jump_stack: &mut Vec<i32>,
    pipeline: Pipeline<'a>,
) -> Result<(), String> {
    instructions.push(Instruction::Begin);
    let num_commands = pipeline.commands.len();
    for (ind, cmd) in pipeline.commands.into_iter().enumerate() {
        let is_last = (ind + 1) == num_commands;
        compile_command(instructions, cmd, is_last)?;
    }
    instructions.push(Instruction::ForegroundJob);
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
                    Instruction::Begin,
                    Instruction::Lit("ab".to_string()),
                    Instruction::Word,
                    Instruction::SetProgram,
                    Instruction::Lit("cd".to_string()),
                    Instruction::Word,
                    Instruction::Exec(false),
                    Instruction::Lit("ef".to_string()),
                    Instruction::Word,
                    Instruction::SetProgram,
                    Instruction::Lit("gh".to_string()),
                    Instruction::Word,
                    Instruction::Lit("ij".to_string()),
                    Instruction::Word,
                    Instruction::Exec(true),
                    Instruction::ForegroundJob,
                    Instruction::Success,
                    Instruction::Not,
                    Instruction::JumpIfNot(7),
                    Instruction::Begin,
                    Instruction::Lit("stuff".to_string()),
                    Instruction::Word,
                    Instruction::SetProgram,
                    Instruction::Exec(true),
                    Instruction::ForegroundJob,
                ]
            );
        }
    }
}
