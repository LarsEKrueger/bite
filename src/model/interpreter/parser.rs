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

//! Bash script parser.

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{line_ending, none_of, not_line_ending, space0, space1};
use nom::combinator::{map, recognize};
use nom::multi::{many1, separated_list, separated_nonempty_list};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a str>;

/// The result of the parse
#[derive(Debug, PartialEq)]
pub enum ParsingResult<'a> {
    /// Nothing to do, e.g. an empty string.
    Nothing,
    /// A comment. It ends in this line. The returned span does not include the comment leader.
    Comment(Span<'a>),
    /// One or more external programs that depend on each other's exit code.
    Logical(Vec<Pipeline<'a>>, BackgroundMode),
}

#[derive(Debug, PartialEq)]
struct Pipeline<'a> {
    commands: Vec<PipelineCommand<'a>>,
    operator: LogicalOperator,
}

/// How to react on the failure of a command
#[derive(Debug, PartialEq)]
pub enum LogicalOperator {
    /// Default value, only to be set for the last element of ParsingResult::Logical
    Nothing,
    /// Short-cut AND
    And,
    /// Short-cut OR
    Or,
}

/// What to do with a logical expression
#[derive(Debug, PartialEq)]
pub enum BackgroundMode {
    /// Execute asynchronously
    Background,
    /// Wait for completion
    Foreground,
}

/// How to connect commands in a pipeline
#[derive(Debug, PartialEq)]
pub enum PipelineOperator {
    Nothing,
    StdoutOnly,
    StderrAndStdout,
}

/// A command in a pipeline
#[derive(Debug, PartialEq)]
pub struct PipelineCommand<'a> {
    command: Command<'a>,
    operator: PipelineOperator,
}

/// Command the shell can handle
#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    /// Call a program
    Program(Vec<Span<'a>>),
}

/// Parse a (partial) bash script.
///
/// It parses a single command, which can be as simple as one word or as complicated as a long for
/// loop. The function consumes the terminator and returns the next position.
///
/// The parser expects a line terminator as the last character.
pub fn script(input: Span) -> IResult<Span, ParsingResult> {
    alt((empty_line, comment, logical))(input)
}

/// Version of nom's separated_list that can fix the last parsed output by the value of the
/// separator
fn separated_list_fix<I, O, O2, E, F, G, Fix>(
    sep: G,
    f: F,
    fix: Fix,
) -> impl Fn(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    G: Fn(I) -> IResult<I, O2, E>,
    Fix: Fn(&mut O, O2),
    E: nom::error::ParseError<I>,
{
    move |i: I| {
        let mut res = Vec::new();
        let mut i = i.clone();

        match f(i.clone()) {
            Err(nom::Err::Error(_)) => return Ok((i, res)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                if i1 == i {
                    return Err(nom::Err::Error(E::from_error_kind(
                        i1,
                        nom::error::ErrorKind::SeparatedList,
                    )));
                }

                res.push(o);
                i = i1;
            }
        }

        loop {
            match sep(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, o2)) => {
                    if i1 == i {
                        return Err(nom::Err::Error(E::from_error_kind(
                            i1,
                            nom::error::ErrorKind::SeparatedList,
                        )));
                    }

                    match f(i1.clone()) {
                        Err(nom::Err::Error(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            if i2 == i {
                                return Err(nom::Err::Error(E::from_error_kind(
                                    i2,
                                    nom::error::ErrorKind::SeparatedList,
                                )));
                            }
                            fix(res.last_mut().unwrap(), o2);

                            res.push(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    }
}

pub fn separated_nonempty_list_fix<I, O, O2, E, F, G, Fix>(
    sep: G,
    f: F,
    fix: Fix,
) -> impl Fn(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    G: Fn(I) -> IResult<I, O2, E>,
    Fix: Fn(&mut O, O2),
    E: nom::error::ParseError<I>,
{
    move |i: I| {
        let mut res = Vec::new();
        let mut i = i.clone();

        // Parse the first element
        match f(i.clone()) {
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                if i1 == i {
                    return Err(nom::Err::Error(E::from_error_kind(
                        i1,
                        nom::error::ErrorKind::SeparatedList,
                    )));
                }

                res.push(o);
                i = i1;
            }
        }

        loop {
            match sep(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, o2)) => {
                    if i1 == i {
                        return Err(nom::Err::Error(E::from_error_kind(
                            i1,
                            nom::error::ErrorKind::SeparatedList,
                        )));
                    }

                    match f(i1.clone()) {
                        Err(nom::Err::Error(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            if i2 == i {
                                return Err(nom::Err::Error(E::from_error_kind(
                                    i2,
                                    nom::error::ErrorKind::SeparatedList,
                                )));
                            }

                            fix(res.last_mut().unwrap(), o2);

                            res.push(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    }
}

/// Parse an empty line
fn empty_line(input: Span) -> IResult<Span, ParsingResult> {
    map(terminated(space0, line_ending), |_| ParsingResult::Nothing)(input)
}

/// Parse a comment. Skip any spaces
fn comment(input: Span) -> IResult<Span, ParsingResult> {
    map(
        tuple((space0, tag("#"), space0, not_line_ending, line_ending)),
        |(_, _, _, cmt, _)| ParsingResult::Comment(cmt),
    )(input)
}

/// Parse a logical conjunction of pipelines. In contrast to bash, the parsing stops at the first
/// separator (e.g. &).
fn logical(input: Span) -> IResult<Span, ParsingResult> {
    let unterminated_expression =
        separated_nonempty_list_fix(logical_operator, pipeline, |pipe, op| pipe.operator = op);
    let expression_terminators = alt((
        map(tag("&"), |_| BackgroundMode::Background),
        map(tag(";"), |_| BackgroundMode::Foreground),
        map(line_ending, |_| BackgroundMode::Foreground),
    ));
    map(
        tuple((
            space0,
            unterminated_expression,
            space0,
            expression_terminators,
        )),
        |(_, pipelines, _, bg_mode)| ParsingResult::Logical(pipelines, bg_mode),
    )(input)
}

/// Parse a logical operator
fn logical_operator(input: Span) -> IResult<Span, LogicalOperator> {
    preceded(
        space0,
        alt((
            map(tag("&&"), |_| LogicalOperator::And),
            map(tag("||"), |_| LogicalOperator::Or),
        )),
    )(input)
}

/// Parse a pipeline
///
/// TODO: Handle ! and time.
fn pipeline(input: Span) -> IResult<Span, Pipeline> {
    let pipe = map(
        separated_nonempty_list_fix(pipeline_operator, command, |cmd, op| cmd.operator = op),
        |cmds| Pipeline {
            commands: cmds,
            operator: LogicalOperator::Nothing,
        },
    );
    preceded(space0, pipe)(input)
}

/// Pipeline operator, i.e. | or |&
///
/// TODO: Newlines are allowed after the operators.
fn pipeline_operator(input: Span) -> IResult<Span, PipelineOperator> {
    let operators = alt((
        map(tag("|&"), |_| PipelineOperator::StderrAndStdout),
        map(tag("|"), |_| PipelineOperator::StdoutOnly),
    ));
    preceded(space0, operators)(input)
}

/// Down to basic commands
fn command(input: Span) -> IResult<Span, PipelineCommand> {
    map(simple_command, |c| PipelineCommand {
        command: c,
        operator: PipelineOperator::Nothing,
    })(input)
}

fn simple_command(input: Span) -> IResult<Span, Command> {
    map(
        preceded(space0, separated_nonempty_list(space1, word)),
        |words| Command::Program(words),
    )(input)
}

fn word(input: Span) -> IResult<Span, Span> {
    recognize(many1(word_letter))(input)
}

fn word_letter(input: Span) -> IResult<Span, char> {
    none_of(" \n\t\"\'|&;()<>")(input)
}

//   /* Reserved words.  Members of the first group are only recognized
//   in the case that they are preceded by a list_terminator.  Members
//   of the second group are for [[...]] commands.  Members of the
//   third group are recognized only under special circumstances. */
//   %token IF THEN ELSE ELIF FI CASE ESAC FOR SELECT WHILE UNTIL DO DONE FUNCTION COPROC
//   %token COND_START COND_END COND_ERROR
//   %token IN BANG TIME TIMEOPT TIMEIGN
//
//   /* More general tokens. yylex () knows how to make these. */
//   %token <word> WORD ASSIGNMENT_WORD REDIR_WORD
//   %token <number> NUMBER
//   %token <word_list> ARITH_CMD ARITH_FOR_EXPRS
//   %token <command> COND_CMD
//   %token AND_AND OR_OR GREATER_GREATER LESS_LESS LESS_AND LESS_LESS_LESS
//   %token GREATER_AND SEMI_SEMI SEMI_AND SEMI_SEMI_AND
//   %token LESS_LESS_MINUS AND_GREATER AND_GREATER_GREATER LESS_GREATER
//   %token GREATER_BAR BAR_AND
//
//   /* The types that the various syntactical units return. */
//
//   %type <command> script command pipeline pipeline_command
//   %type <command> list list0 list1 compound_list simple_list simple_list1
//   %type <command> simple_command shell_command
//   %type <command> for_command select_command case_command group_command
//   %type <command> arith_command
//   %type <command> cond_command
//   %type <command> arith_for_command
//   %type <command> coproc
//   %type <command> function_def function_body if_command elif_clause subshell
//   %type <redirect> redirection redirection_list
//   %type <element> simple_command_element
//   %type <word_list> word_list pattern
//   %type <pattern> pattern_list case_clause_sequence case_clause
//   %type <number> timespec
//   %type <number> list_terminator
//
//   %left '&' ';' '\n' yacc_EOF
//   %left AND_AND OR_OR
//   %right '|' BAR_AND
//   %%
//
//   script:	simple_list simple_list_terminator
//           |	'\n'
//           |	error '\n'
//           |	yacc_EOF
//           ;
//
//   word_list:	WORD
//           |	word_list WORD
//           ;
//
//   redirection:	'>' WORD
//                           {
//                           source.dest = 1;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_output_direction, redir, 0);
//                           }
//           |	'<' WORD
//                           {
//                           source.dest = 0;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_input_direction, redir, 0);
//                           }
//           |	NUMBER '>' WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_output_direction, redir, 0);
//                           }
//           |	NUMBER '<' WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_input_direction, redir, 0);
//                           }
//           |	REDIR_WORD '>' WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_output_direction, redir, REDIR_VARASSIGN);
//                           }
//           |	REDIR_WORD '<' WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_input_direction, redir, REDIR_VARASSIGN);
//                           }
//           |	GREATER_GREATER WORD
//                           {
//                           source.dest = 1;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_appending_to, redir, 0);
//                           }
//           |	NUMBER GREATER_GREATER WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_appending_to, redir, 0);
//                           }
//           |	REDIR_WORD GREATER_GREATER WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_appending_to, redir, REDIR_VARASSIGN);
//                           }
//           |	GREATER_BAR WORD
//                           {
//                           source.dest = 1;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_output_force, redir, 0);
//                           }
//           |	NUMBER GREATER_BAR WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_output_force, redir, 0);
//                           }
//           |	REDIR_WORD GREATER_BAR WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_output_force, redir, REDIR_VARASSIGN);
//                           }
//           |	LESS_GREATER WORD
//                           {
//                           source.dest = 0;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_input_output, redir, 0);
//                           }
//           |	NUMBER LESS_GREATER WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_input_output, redir, 0);
//                           }
//           |	REDIR_WORD LESS_GREATER WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_input_output, redir, REDIR_VARASSIGN);
//                           }
//           |	LESS_LESS WORD
//                           {
//                           source.dest = 0;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_reading_until, redir, 0);
//                           push_heredoc ($$);
//                           }
//           |	NUMBER LESS_LESS WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_reading_until, redir, 0);
//                           push_heredoc ($$);
//                           }
//           |	REDIR_WORD LESS_LESS WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_reading_until, redir, REDIR_VARASSIGN);
//                           push_heredoc ($$);
//                           }
//           |	LESS_LESS_MINUS WORD
//                           {
//                           source.dest = 0;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_deblank_reading_until, redir, 0);
//                           push_heredoc ($$);
//                           }
//           |	NUMBER LESS_LESS_MINUS WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_deblank_reading_until, redir, 0);
//                           push_heredoc ($$);
//                           }
//           |	REDIR_WORD  LESS_LESS_MINUS WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_deblank_reading_until, redir, REDIR_VARASSIGN);
//                           push_heredoc ($$);
//                           }
//           |	LESS_LESS_LESS WORD
//                           {
//                           source.dest = 0;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_reading_string, redir, 0);
//                           }
//           |	NUMBER LESS_LESS_LESS WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_reading_string, redir, 0);
//                           }
//           |	REDIR_WORD LESS_LESS_LESS WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_reading_string, redir, REDIR_VARASSIGN);
//                           }
//           |	LESS_AND NUMBER
//                           {
//                           source.dest = 0;
//                           redir.dest = $2;
//                           $$ = make_redirection (source, r_duplicating_input, redir, 0);
//                           }
//           |	NUMBER LESS_AND NUMBER
//                           {
//                           source.dest = $1;
//                           redir.dest = $3;
//                           $$ = make_redirection (source, r_duplicating_input, redir, 0);
//                           }
//           |	REDIR_WORD LESS_AND NUMBER
//                           {
//                           source.filename = $1;
//                           redir.dest = $3;
//                           $$ = make_redirection (source, r_duplicating_input, redir, REDIR_VARASSIGN);
//                           }
//           |	GREATER_AND NUMBER
//                           {
//                           source.dest = 1;
//                           redir.dest = $2;
//                           $$ = make_redirection (source, r_duplicating_output, redir, 0);
//                           }
//           |	NUMBER GREATER_AND NUMBER
//                           {
//                           source.dest = $1;
//                           redir.dest = $3;
//                           $$ = make_redirection (source, r_duplicating_output, redir, 0);
//                           }
//           |	REDIR_WORD GREATER_AND NUMBER
//                           {
//                           source.filename = $1;
//                           redir.dest = $3;
//                           $$ = make_redirection (source, r_duplicating_output, redir, REDIR_VARASSIGN);
//                           }
//           |	LESS_AND WORD
//                           {
//                           source.dest = 0;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_duplicating_input_word, redir, 0);
//                           }
//           |	NUMBER LESS_AND WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_duplicating_input_word, redir, 0);
//                           }
//           |	REDIR_WORD LESS_AND WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_duplicating_input_word, redir, REDIR_VARASSIGN);
//                           }
//           |	GREATER_AND WORD
//                           {
//                           source.dest = 1;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_duplicating_output_word, redir, 0);
//                           }
//           |	NUMBER GREATER_AND WORD
//                           {
//                           source.dest = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_duplicating_output_word, redir, 0);
//                           }
//           |	REDIR_WORD GREATER_AND WORD
//                           {
//                           source.filename = $1;
//                           redir.filename = $3;
//                           $$ = make_redirection (source, r_duplicating_output_word, redir, REDIR_VARASSIGN);
//                           }
//           |	GREATER_AND '-'
//                           {
//                           source.dest = 1;
//                           redir.dest = 0;
//                           $$ = make_redirection (source, r_close_this, redir, 0);
//                           }
//           |	NUMBER GREATER_AND '-'
//                           {
//                           source.dest = $1;
//                           redir.dest = 0;
//                           $$ = make_redirection (source, r_close_this, redir, 0);
//                           }
//           |	REDIR_WORD GREATER_AND '-'
//                           {
//                           source.filename = $1;
//                           redir.dest = 0;
//                           $$ = make_redirection (source, r_close_this, redir, REDIR_VARASSIGN);
//                           }
//           |	LESS_AND '-'
//                           {
//                           source.dest = 0;
//                           redir.dest = 0;
//                           $$ = make_redirection (source, r_close_this, redir, 0);
//                           }
//           |	NUMBER LESS_AND '-'
//                           {
//                           source.dest = $1;
//                           redir.dest = 0;
//                           $$ = make_redirection (source, r_close_this, redir, 0);
//                           }
//           |	REDIR_WORD LESS_AND '-'
//                           {
//                           source.filename = $1;
//                           redir.dest = 0;
//                           $$ = make_redirection (source, r_close_this, redir, REDIR_VARASSIGN);
//                           }
//           |	AND_GREATER WORD
//                           {
//                           source.dest = 1;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_err_and_out, redir, 0);
//                           }
//           |	AND_GREATER_GREATER WORD
//                           {
//                           source.dest = 1;
//                           redir.filename = $2;
//                           $$ = make_redirection (source, r_append_err_and_out, redir, 0);
//                           }
//           ;
//
//   simple_command_element: WORD
//                           { $$.word = $1; $$.redirect = 0; }
//           |	ASSIGNMENT_WORD
//                           { $$.word = $1; $$.redirect = 0; }
//           |	redirection
//                           { $$.redirect = $1; $$.word = 0; }
//           ;
//
//   redirection_list: redirection
//                           {
//                           $$ = $1;
//                           }
//           |	redirection_list redirection
//                           {
//                           register REDIRECT *t;
//
//                           for (t = $1; t->next; t = t->next)
//                               ;
//                           t->next = $2;
//                           $$ = $1;
//                           }
//           ;
//
//   simple_command:	simple_command_element
//                           { $$ = make_simple_command ($1, (COMMAND *)NULL); }
//           |	simple_command simple_command_element
//                           { $$ = make_simple_command ($2, $1); }
//           ;
//
//   command:	simple_command
//                           { $$ = clean_simple_command ($1); }
//           |	shell_command
//                           { $$ = $1; }
//           |	shell_command redirection_list
//                           {
//                           COMMAND *tc;
//
//                           tc = $1;
//                           if (tc->redirects)
//                               {
//                               register REDIRECT *t;
//                               for (t = tc->redirects; t->next; t = t->next)
//                                   ;
//                               t->next = $2;
//                               }
//                           else
//                               tc->redirects = $2;
//                           $$ = $1;
//                           }
//           |	function_def
//                           { $$ = $1; }
//           |	coproc
//                           { $$ = $1; }
//           ;
//
//   shell_command:	for_command
//                           { $$ = $1; }
//           |	case_command
//                           { $$ = $1; }
//           |	WHILE compound_list DO compound_list DONE
//                           { $$ = make_while_command ($2, $4); }
//           |	UNTIL compound_list DO compound_list DONE
//                           { $$ = make_until_command ($2, $4); }
//           |	select_command
//                           { $$ = $1; }
//           |	if_command
//                           { $$ = $1; }
//           |	subshell
//                           { $$ = $1; }
//           |	group_command
//                           { $$ = $1; }
//           |	arith_command
//                           { $$ = $1; }
//           |	cond_command
//                           { $$ = $1; }
//           |	arith_for_command
//                           { $$ = $1; }
//           ;
//
//   for_command:	FOR WORD newline_list DO compound_list DONE
//                           {
//                           $$ = make_for_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $5, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD newline_list '{' compound_list '}'
//                           {
//                           $$ = make_for_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $5, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD ';' newline_list DO compound_list DONE
//                           {
//                           $$ = make_for_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $6, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD ';' newline_list '{' compound_list '}'
//                           {
//                           $$ = make_for_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $6, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD newline_list IN word_list list_terminator newline_list DO compound_list DONE
//                           {
//                           $$ = make_for_command ($2, REVERSE_LIST ($5, WORD_LIST *), $9, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD newline_list IN word_list list_terminator newline_list '{' compound_list '}'
//                           {
//                           $$ = make_for_command ($2, REVERSE_LIST ($5, WORD_LIST *), $9, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD newline_list IN list_terminator newline_list DO compound_list DONE
//                           {
//                           $$ = make_for_command ($2, (WORD_LIST *)NULL, $8, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	FOR WORD newline_list IN list_terminator newline_list '{' compound_list '}'
//                           {
//                           $$ = make_for_command ($2, (WORD_LIST *)NULL, $8, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           ;
//
//   arith_for_command:	FOR ARITH_FOR_EXPRS list_terminator newline_list DO compound_list DONE
//                                   {
//                                   $$ = make_arith_for_command ($2, $6, arith_for_lineno);
//                                   if (word_top > 0) word_top--;
//                                   }
//           |		FOR ARITH_FOR_EXPRS list_terminator newline_list '{' compound_list '}'
//                                   {
//                                   $$ = make_arith_for_command ($2, $6, arith_for_lineno);
//                                   if (word_top > 0) word_top--;
//                                   }
//           |		FOR ARITH_FOR_EXPRS DO compound_list DONE
//                                   {
//                                   $$ = make_arith_for_command ($2, $4, arith_for_lineno);
//                                   if (word_top > 0) word_top--;
//                                   }
//           |		FOR ARITH_FOR_EXPRS '{' compound_list '}'
//                                   {
//                                   $$ = make_arith_for_command ($2, $4, arith_for_lineno);
//                                   if (word_top > 0) word_top--;
//                                   }
//           ;
//
//   select_command:	SELECT WORD newline_list DO list DONE
//                           {
//                           $$ = make_select_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $5, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	SELECT WORD newline_list '{' list '}'
//                           {
//                           $$ = make_select_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $5, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	SELECT WORD ';' newline_list DO list DONE
//                           {
//                           $$ = make_select_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $6, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	SELECT WORD ';' newline_list '{' list '}'
//                           {
//                           $$ = make_select_command ($2, add_string_to_list ("\"$@\"", (WORD_LIST *)NULL), $6, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	SELECT WORD newline_list IN word_list list_terminator newline_list DO list DONE
//                           {
//                           $$ = make_select_command ($2, REVERSE_LIST ($5, WORD_LIST *), $9, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	SELECT WORD newline_list IN word_list list_terminator newline_list '{' list '}'
//                           {
//                           $$ = make_select_command ($2, REVERSE_LIST ($5, WORD_LIST *), $9, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           ;
//
//   case_command:	CASE WORD newline_list IN newline_list ESAC
//                           {
//                           $$ = make_case_command ($2, (PATTERN_LIST *)NULL, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	CASE WORD newline_list IN case_clause_sequence newline_list ESAC
//                           {
//                           $$ = make_case_command ($2, $5, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           |	CASE WORD newline_list IN case_clause ESAC
//                           {
//                           $$ = make_case_command ($2, $5, word_lineno[word_top]);
//                           if (word_top > 0) word_top--;
//                           }
//           ;
//
//   function_def:	WORD '(' ')' newline_list function_body
//                           { $$ = make_function_def ($1, $5, function_dstart, function_bstart); }
//
//           |	FUNCTION WORD '(' ')' newline_list function_body
//                           { $$ = make_function_def ($2, $6, function_dstart, function_bstart); }
//
//           |	FUNCTION WORD newline_list function_body
//                           { $$ = make_function_def ($2, $4, function_dstart, function_bstart); }
//           ;
//
//   function_body:	shell_command
//                           { $$ = $1; }
//           |	shell_command redirection_list
//                           {
//                           COMMAND *tc;
//
//                           tc = $1;
//                           /* According to Posix.2 3.9.5, redirections
//                               specified after the body of a function should
//                               be attached to the function and performed when
//                               the function is executed, not as part of the
//                               function definition command. */
//                           /* XXX - I don't think it matters, but we might
//                               want to change this in the future to avoid
//                               problems differentiating between a function
//                               definition with a redirection and a function
//                               definition containing a single command with a
//                               redirection.  The two are semantically equivalent,
//                               though -- the only difference is in how the
//                               command printing code displays the redirections. */
//                           if (tc->redirects)
//                               {
//                               register REDIRECT *t;
//                               for (t = tc->redirects; t->next; t = t->next)
//                                   ;
//                               t->next = $2;
//                               }
//                           else
//                               tc->redirects = $2;
//                           $$ = $1;
//                           }
//           ;
//
//   subshell:	'(' compound_list ')'
//                           {
//                           $$ = make_subshell_command ($2);
//                           $$->flags |= CMD_WANT_SUBSHELL;
//                           }
//           ;
//
//   coproc:		COPROC shell_command
//                           {
//                           $$ = make_coproc_command ("COPROC", $2);
//                           $$->flags |= CMD_WANT_SUBSHELL|CMD_COPROC_SUBSHELL;
//                           }
//           |	COPROC shell_command redirection_list
//                           {
//                           COMMAND *tc;
//
//                           tc = $2;
//                           if (tc->redirects)
//                               {
//                               register REDIRECT *t;
//                               for (t = tc->redirects; t->next; t = t->next)
//                                   ;
//                               t->next = $3;
//                               }
//                           else
//                               tc->redirects = $3;
//                           $$ = make_coproc_command ("COPROC", $2);
//                           $$->flags |= CMD_WANT_SUBSHELL|CMD_COPROC_SUBSHELL;
//                           }
//           |	COPROC WORD shell_command
//                           {
//                           $$ = make_coproc_command ($2->word, $3);
//                           $$->flags |= CMD_WANT_SUBSHELL|CMD_COPROC_SUBSHELL;
//                           }
//           |	COPROC WORD shell_command redirection_list
//                           {
//                           COMMAND *tc;
//
//                           tc = $3;
//                           if (tc->redirects)
//                               {
//                               register REDIRECT *t;
//                               for (t = tc->redirects; t->next; t = t->next)
//                                   ;
//                               t->next = $4;
//                               }
//                           else
//                               tc->redirects = $4;
//                           $$ = make_coproc_command ($2->word, $3);
//                           $$->flags |= CMD_WANT_SUBSHELL|CMD_COPROC_SUBSHELL;
//                           }
//           |	COPROC simple_command
//                           {
//                           $$ = make_coproc_command ("COPROC", clean_simple_command ($2));
//                           $$->flags |= CMD_WANT_SUBSHELL|CMD_COPROC_SUBSHELL;
//                           }
//           ;
//
//   if_command:	IF compound_list THEN compound_list FI
//                           { $$ = make_if_command ($2, $4, (COMMAND *)NULL); }
//           |	IF compound_list THEN compound_list ELSE compound_list FI
//                           { $$ = make_if_command ($2, $4, $6); }
//           |	IF compound_list THEN compound_list elif_clause FI
//                           { $$ = make_if_command ($2, $4, $5); }
//           ;
//
//
//   group_command:	'{' compound_list '}'
//                           { $$ = make_group_command ($2); }
//           ;
//
//   arith_command:	ARITH_CMD
//                           { $$ = make_arith_command ($1); }
//           ;
//
//   cond_command:	COND_START COND_CMD COND_END
//                           { $$ = $2; }
//           ;
//
//   elif_clause:	ELIF compound_list THEN compound_list
//                           { $$ = make_if_command ($2, $4, (COMMAND *)NULL); }
//           |	ELIF compound_list THEN compound_list ELSE compound_list
//                           { $$ = make_if_command ($2, $4, $6); }
//           |	ELIF compound_list THEN compound_list elif_clause
//                           { $$ = make_if_command ($2, $4, $5); }
//           ;
//
//   case_clause:	pattern_list
//           |	case_clause_sequence pattern_list
//                           { $2->next = $1; $$ = $2; }
//           ;
//
//   pattern_list:	newline_list pattern ')' compound_list
//                           { $$ = make_pattern_list ($2, $4); }
//           |	newline_list pattern ')' newline_list
//                           { $$ = make_pattern_list ($2, (COMMAND *)NULL); }
//           |	newline_list '(' pattern ')' compound_list
//                           { $$ = make_pattern_list ($3, $5); }
//           |	newline_list '(' pattern ')' newline_list
//                           { $$ = make_pattern_list ($3, (COMMAND *)NULL); }
//           ;
//
//   case_clause_sequence:  pattern_list SEMI_SEMI
//                           { $$ = $1; }
//           |	case_clause_sequence pattern_list SEMI_SEMI
//                           { $2->next = $1; $$ = $2; }
//           |	pattern_list SEMI_AND
//                           { $1->flags |= CASEPAT_FALLTHROUGH; $$ = $1; }
//           |	case_clause_sequence pattern_list SEMI_AND
//                           { $2->flags |= CASEPAT_FALLTHROUGH; $2->next = $1; $$ = $2; }
//           |	pattern_list SEMI_SEMI_AND
//                           { $1->flags |= CASEPAT_TESTNEXT; $$ = $1; }
//           |	case_clause_sequence pattern_list SEMI_SEMI_AND
//                           { $2->flags |= CASEPAT_TESTNEXT; $2->next = $1; $$ = $2; }
//           ;
//
//   pattern:	WORD
//                           { $$ = make_word_list ($1, (WORD_LIST *)NULL); }
//           |	pattern '|' WORD
//                           { $$ = make_word_list ($3, $1); }
//           ;
//
//   /* A list allows leading or trailing newlines and
//   newlines as operators (equivalent to semicolons).
//   It must end with a newline or semicolon.
//   Lists are used within commands such as if, for, while.  */
//
//   list:		newline_list list0
//                           {
//                           $$ = $2;
//                           if (need_here_doc)
//                               gather_here_documents ();
//                           }
//           ;
//
//   compound_list:	list
//           |	newline_list list1
//                           {
//                           $$ = $2;
//                           }
//           ;
//
//   list0:  	list1 '\n' newline_list
//           |	list1 '&' newline_list
//                           {
//                           if ($1->type == cm_connection)
//                               $$ = connect_async_list ($1, (COMMAND *)NULL, '&');
//                           else
//                               $$ = command_connect ($1, (COMMAND *)NULL, '&');
//                           }
//           |	list1 ';' newline_list
//
//           ;
//
//   list1:		list1 AND_AND newline_list list1
//                           { $$ = command_connect ($1, $4, AND_AND); }
//           |	list1 OR_OR newline_list list1
//                           { $$ = command_connect ($1, $4, OR_OR); }
//           |	list1 '&' newline_list list1
//                           {
//                           if ($1->type == cm_connection)
//                               $$ = connect_async_list ($1, $4, '&');
//                           else
//                               $$ = command_connect ($1, $4, '&');
//                           }
//           |	list1 ';' newline_list list1
//                           { $$ = command_connect ($1, $4, ';'); }
//           |	list1 '\n' newline_list list1
//                           { $$ = command_connect ($1, $4, ';'); }
//           |	pipeline_command
//                           { $$ = $1; }
//           ;
//
//   simple_list_terminator:	'\n'
//           |	yacc_EOF
//           ;
//
//   list_terminator:'\n'
//                   { $$ = '\n'; }
//           |	';'
//                   { $$ = ';'; }
//           |	yacc_EOF
//                   { $$ = yacc_EOF; }
//           ;
//
//   newline_list:
//           |	newline_list '\n'
//           ;
//
//   simple_list:	simple_list1
//           |	simple_list1 '&'
//           |	simple_list1 ';'
//           ;
//
//   simple_list1:	simple_list1 AND_AND newline_list simple_list1
//           |	simple_list1 OR_OR newline_list simple_list1
//           |	simple_list1 '&' simple_list1
//           |	simple_list1 ';' simple_list1
//           |	pipeline_command
//           ;
//
//   pipeline_command: pipeline
//           |	BANG pipeline_command
//                           {
//                           if ($2)
//                               $2->flags ^= CMD_INVERT_RETURN;	/* toggle */
//                           $$ = $2;
//                           }
//           |	timespec pipeline_command
//                           {
//                           if ($2)
//                               $2->flags |= $1;
//                           $$ = $2;
//                           }
//           |	timespec list_terminator
//                           {
//                           ELEMENT x;
//
//                           /* Boy, this is unclean.  `time' by itself can
//                               time a null command.  We cheat and push a
//                               newline back if the list_terminator was a newline
//                               to avoid the double-newline problem (one to
//                               terminate this, one to terminate the command) */
//                           x.word = 0;
//                           x.redirect = 0;
//                           $$ = make_simple_command (x, (COMMAND *)NULL);
//                           $$->flags |= $1;
//                           /* XXX - let's cheat and push a newline back */
//                           if ($2 == '\n')
//                               token_to_read = '\n';
//                           else if ($2 == ';')
//                               token_to_read = ';';
//                           }
//           |	BANG list_terminator
//                           {
//                           ELEMENT x;
//
//                           /* This is just as unclean.  Posix says that `!'
//                               by itself should be equivalent to `false'.
//                               We cheat and push a
//                               newline back if the list_terminator was a newline
//                               to avoid the double-newline problem (one to
//                               terminate this, one to terminate the command) */
//                           x.word = 0;
//                           x.redirect = 0;
//                           $$ = make_simple_command (x, (COMMAND *)NULL);
//                           $$->flags |= CMD_INVERT_RETURN;
//                           /* XXX - let's cheat and push a newline back */
//                           if ($2 == '\n')
//                               token_to_read = '\n';
//                           if ($2 == ';')
//                               token_to_read = ';';
//                           }
//           ;
//
//   pipeline:	pipeline '|' newline_list pipeline
//                           { $$ = command_connect ($1, $4, '|'); }
//           |	pipeline BAR_AND newline_list pipeline
//                           {
//                           /* Make cmd1 |& cmd2 equivalent to cmd1 2>&1 | cmd2 */
//                           COMMAND *tc;
//                           REDIRECTEE rd, sd;
//                           REDIRECT *r;
//
//                           tc = $1->type == cm_simple ? (COMMAND *)$1->value.Simple : $1;
//                           sd.dest = 2;
//                           rd.dest = 1;
//                           r = make_redirection (sd, r_duplicating_output, rd, 0);
//                           if (tc->redirects)
//                               {
//                               register REDIRECT *t;
//                               for (t = tc->redirects; t->next; t = t->next)
//                                   ;
//                               t->next = r;
//                               }
//                           else
//                               tc->redirects = r;
//
//                           $$ = command_connect ($1, $4, '|');
//                           }
//           |	command
//                           { $$ = $1; }
//           ;
//
//   timespec:	TIME
//                           { $$ = CMD_TIME_PIPELINE; }
//           |	TIME TIMEOPT
//                           { $$ = CMD_TIME_PIPELINE|CMD_TIME_POSIX; }
//           |	TIME TIMEOPT TIMEIGN
//                           { $$ = CMD_TIME_PIPELINE|CMD_TIME_POSIX; }
//           ;
//   %%

#[cfg(test)]
mod tests {
    use super::*;

    fn span(offset: usize, line: u32, fragment: &str) -> Span {
        Span {
            offset,
            line,
            fragment,
            extra: (),
        }
    }

    //   #[test]
    //   fn parse_word() {
    //       assert_eq!(
    //           word(Span::new("bc")),
    //           Ok((span(2, 1, ""), span(0, 1, "bc")))
    //       );
    //       assert_eq!(
    //           word(Span::new("bc<")),
    //           Ok((span(2, 1, "<"), span(0, 1, "bc")))
    //       );
    //   }

    #[test]
    fn parse_command_empty() {
        assert_eq!(
            script(Span::new("\n")),
            Ok((span(1, 2, ""), ParsingResult::Nothing))
        );
        assert_eq!(
            script(Span::new(" \n")),
            Ok((span(2, 2, ""), ParsingResult::Nothing))
        );
    }

    #[test]
    fn parse_command_comment() {
        assert_eq!(
            script(Span::new("# Stuff \n")),
            Ok((span(9, 2, ""), ParsingResult::Comment(span(2, 1, "Stuff "))))
        );
        assert_eq!(
            script(Span::new(" # Stuff \n")),
            Ok((
                span(10, 2, ""),
                ParsingResult::Comment(span(3, 1, "Stuff "))
            ))
        );
    }

    #[test]
    fn parse_simple_command() {
        assert_eq!(
            simple_command(Span::new("ab bc   cd \t\tde\n")),
            Ok((
                span(15, 1, "\n"),
                Command::Program(vec![
                    span(0, 1, "ab"),
                    span(3, 1, "bc"),
                    span(8, 1, "cd"),
                    span(13, 1, "de"),
                ])
            ))
        );

        assert_eq!(
            simple_command(Span::new(" \tab bc   cd \t\tde\n")),
            Ok((
                span(17, 1, "\n"),
                Command::Program(vec![
                    span(2, 1, "ab"),
                    span(5, 1, "bc"),
                    span(10, 1, "cd"),
                    span(15, 1, "de"),
                ])
            ))
        );

        // A simple command ends at the end of the line
        assert_eq!(
            simple_command(Span::new("ab cd\nef")),
            Ok((
                span(5, 1, "\nef"),
                Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd"),])
            ))
        );
        // A simple command with trailing spaces
        assert_eq!(
            simple_command(Span::new("ab \n")),
            Ok((span(2, 1, " \n"), Command::Program(vec![span(0, 1, "ab"),])))
        );
    }

    #[test]
    fn parse_pipeline() {
        // A simple command with trailing spaces
        assert_eq!(
            pipeline(Span::new("ab")),
            Ok((
                span(2, 1, ""),
                Pipeline {
                    commands: vec![PipelineCommand {
                        command: Command::Program(vec![span(0, 1, "ab"),]),
                        operator: PipelineOperator::Nothing
                    }],
                    operator: LogicalOperator::Nothing
                }
            ))
        );

        // Two commands, normal piping
        assert_eq!(
            pipeline(Span::new("ab cd | de fg")),
            Ok((
                span(13, 1, ""),
                Pipeline {
                    commands: vec![
                        PipelineCommand {
                            command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                            operator: PipelineOperator::StdoutOnly
                        },
                        PipelineCommand {
                            command: Command::Program(vec![span(8, 1, "de"), span(11, 1, "fg"),]),
                            operator: PipelineOperator::Nothing
                        },
                    ],
                    operator: LogicalOperator::Nothing
                }
            ))
        );

        // Two commands, stderr piping
        assert_eq!(
            pipeline(Span::new("ab cd |& de fg")),
            Ok((
                span(14, 1, ""),
                Pipeline {
                    commands: vec![
                        PipelineCommand {
                            command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                            operator: PipelineOperator::StderrAndStdout
                        },
                        PipelineCommand {
                            command: Command::Program(vec![span(9, 1, "de"), span(12, 1, "fg"),]),
                            operator: PipelineOperator::Nothing
                        },
                    ],
                    operator: LogicalOperator::Nothing
                }
            ))
        );
    }

    #[test]
    fn parse_logical() {
        // Two commands, and
        assert_eq!(
            logical(Span::new("ab cd && de fg\n")),
            Ok((
                span(15, 2, ""),
                ParsingResult::Logical(
                    vec![
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::And
                        },
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![
                                    span(9, 1, "de"),
                                    span(12, 1, "fg"),
                                ]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::Nothing
                        }
                    ],
                    BackgroundMode::Foreground
                )
            ))
        );

        // Two commands, or
        assert_eq!(
            logical(Span::new("ab cd || de fg\n")),
            Ok((
                span(15, 2, ""),
                ParsingResult::Logical(
                    vec![
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::Or
                        },
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![
                                    span(9, 1, "de"),
                                    span(12, 1, "fg"),
                                ]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::Nothing
                        }
                    ],
                    BackgroundMode::Foreground
                )
            ))
        );

        // Two pipelines, or
        assert_eq!(
            logical(Span::new("ab cd | de fg || gh ij | kl mn\n")),
            Ok((
                span(31, 2, ""),
                ParsingResult::Logical(
                    vec![
                        Pipeline {
                            commands: vec![
                                PipelineCommand {
                                    command: Command::Program(vec![
                                        span(0, 1, "ab"),
                                        span(3, 1, "cd")
                                    ]),
                                    operator: PipelineOperator::StdoutOnly
                                },
                                PipelineCommand {
                                    command: Command::Program(vec![
                                        span(8, 1, "de"),
                                        span(11, 1, "fg")
                                    ]),
                                    operator: PipelineOperator::Nothing
                                },
                            ],
                            operator: LogicalOperator::Or
                        },
                        Pipeline {
                            commands: vec![
                                PipelineCommand {
                                    command: Command::Program(vec![
                                        span(17, 1, "gh"),
                                        span(20, 1, "ij"),
                                    ]),
                                    operator: PipelineOperator::StdoutOnly
                                },
                                PipelineCommand {
                                    command: Command::Program(vec![
                                        span(25, 1, "kl"),
                                        span(28, 1, "mn"),
                                    ]),
                                    operator: PipelineOperator::Nothing
                                }
                            ],
                            operator: LogicalOperator::Nothing
                        }
                    ],
                    BackgroundMode::Foreground
                )
            ))
        );

        // Two commands, and, semicolon
        assert_eq!(
            logical(Span::new("ab cd && de fg;")),
            Ok((
                span(15, 1, ""),
                ParsingResult::Logical(
                    vec![
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::And
                        },
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![
                                    span(9, 1, "de"),
                                    span(12, 1, "fg"),
                                ]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::Nothing
                        }
                    ],
                    BackgroundMode::Foreground
                )
            ))
        );

        // Two commands, and, space, semicolon
        assert_eq!(
            logical(Span::new("ab cd && de fg ;")),
            Ok((
                span(16, 1, ""),
                ParsingResult::Logical(
                    vec![
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::And
                        },
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![
                                    span(9, 1, "de"),
                                    span(12, 1, "fg"),
                                ]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::Nothing
                        }
                    ],
                    BackgroundMode::Foreground
                )
            ))
        );

        // Two commands, and, space, ampersand
        assert_eq!(
            logical(Span::new("ab cd && de fg &")),
            Ok((
                span(16, 1, ""),
                ParsingResult::Logical(
                    vec![
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![span(0, 1, "ab"), span(3, 1, "cd")]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::And
                        },
                        Pipeline {
                            commands: vec![PipelineCommand {
                                command: Command::Program(vec![
                                    span(9, 1, "de"),
                                    span(12, 1, "fg"),
                                ]),
                                operator: PipelineOperator::Nothing
                            },],
                            operator: LogicalOperator::Nothing
                        }
                    ],
                    BackgroundMode::Background
                )
            ))
        );
    }

    //   #[test]
    //   fn parse_script_one() {
    //       assert_eq!(
    //           parse_script(b" ab bc   cd \t\tde\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![CommandLogic::new(vec![Pipeline::new(vec![
    //                   Command::new(vec![
    //                       String::from("ab"),
    //                       String::from("bc"),
    //                       String::from("cd"),
    //                       String::from("de"),
    //                   ])
    //               ])])])
    //           )
    //       );
    //
    //       assert_eq!(
    //           parse_script(b" \t \t\t\n"),
    //           IResult::Done(&b""[..], ParsedCommand::None)
    //       );
    //   }
    //
    //   #[test]
    //   fn parse_command_logic() {
    //       fn test_cmd_new(word: &str, cr: CommandReaction) -> Pipeline {
    //           Pipeline {
    //               commands: vec![Command {
    //                   words: vec![String::from(word)],
    //                   mode: PipelineMode::Nothing,
    //               }],
    //               reaction: cr,
    //               invert: false,
    //           }
    //       }
    //
    //       assert_eq!(
    //           parse_script(b"ab&&bc||cd\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![CommandLogic::new(vec![
    //                   test_cmd_new("ab", CommandReaction::And),
    //                   test_cmd_new("bc", CommandReaction::Or),
    //                   test_cmd_new("cd", CommandReaction::Normal)
    //               ])])
    //           )
    //       );
    //
    //       assert_eq!(
    //           parse_script(b"ab && bc || cd\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![CommandLogic::new(vec![
    //                   test_cmd_new("ab", CommandReaction::And),
    //                   test_cmd_new("bc", CommandReaction::Or),
    //                   test_cmd_new("cd", CommandReaction::Normal)
    //               ])])
    //           )
    //       );
    //   }
    //
    //   #[test]
    //   fn parse_simple_list_lo() {
    //       fn test_ct_new(word: &str, cr: CommandReaction) -> CommandLogic {
    //           CommandLogic {
    //               pipelines: vec![Pipeline {
    //                   commands: vec![Command {
    //                       words: vec![String::from(word)],
    //                       mode: PipelineMode::Nothing,
    //                   }],
    //                   reaction: cr,
    //                   invert: false,
    //               }],
    //           }
    //       }
    //       assert_eq!(
    //           parse_script(b"ab;bc&cd\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![
    //                   test_ct_new("ab", CommandReaction::Normal),
    //                   test_ct_new("bc", CommandReaction::Background),
    //                   test_ct_new("cd", CommandReaction::Normal)
    //               ])
    //           )
    //       );
    //       assert_eq!(
    //           parse_script(b"ab;\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![test_ct_new("ab", CommandReaction::Normal),])
    //           )
    //       );
    //       assert_eq!(
    //           parse_script(b"ab;bc&cd&\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![
    //                   test_ct_new("ab", CommandReaction::Normal),
    //                   test_ct_new("bc", CommandReaction::Background),
    //                   test_ct_new("cd", CommandReaction::Background)
    //               ])
    //           )
    //       );
    //
    //       // Weird corner cases
    //       assert_eq!(
    //           parse_script(b"ab && bc & cd&\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![
    //                   CommandLogic {
    //                       pipelines: vec![
    //                           Pipeline {
    //                               commands: vec![Command {
    //                                   words: vec![String::from("ab")],
    //                                   mode: PipelineMode::Nothing
    //                               }],
    //                               reaction: CommandReaction::And,
    //                               invert: false
    //                           },
    //                           Pipeline {
    //                               commands: vec![Command {
    //                                   words: vec![String::from("bc")],
    //                                   mode: PipelineMode::Nothing
    //                               }],
    //                               reaction: CommandReaction::Background,
    //                               invert: false
    //                           }
    //                       ],
    //                   },
    //                   test_ct_new("cd", CommandReaction::Background)
    //               ])
    //           )
    //       );
    //
    //       // Parsing errors
    //       assert_eq!(
    //           parse_script(b"ab ; && bc\n"),
    //           IResult::Error(::nom::Err::Position(
    //               ::nom::ErrorKind::Alt,
    //               &b"ab ; && bc\n"[..]
    //           ))
    //       );
    //   }
    //
    //   #[test]
    //   fn parse_pipeline_command() {
    //       assert_eq!(
    //           pipeline_command(b"! ab"),
    //           IResult::Done(
    //               &b""[..],
    //               Pipeline {
    //                   commands: vec![Command {
    //                       words: vec![String::from("ab")],
    //                       mode: PipelineMode::Nothing
    //                   }],
    //                   reaction: CommandReaction::Normal,
    //                   invert: true
    //               }
    //           )
    //       );
    //       assert_eq!(
    //           pipeline_command(b" ! ab"),
    //           IResult::Done(
    //               &b""[..],
    //               Pipeline {
    //                   commands: vec![Command {
    //                       words: vec![String::from("ab")],
    //                       mode: PipelineMode::Nothing
    //                   }],
    //                   reaction: CommandReaction::Normal,
    //                   invert: true
    //               }
    //           )
    //       );
    //       assert_eq!(
    //           pipeline_command(b"! ! ab"),
    //           IResult::Done(
    //               &b""[..],
    //               Pipeline {
    //                   commands: vec![Command {
    //                       words: vec![String::from("ab")],
    //                       mode: PipelineMode::Nothing
    //                   }],
    //                   reaction: CommandReaction::Normal,
    //                   invert: false
    //               }
    //           )
    //       );
    //       assert_eq!(
    //           pipeline_command(b"!ab"),
    //           IResult::Done(
    //               &b""[..],
    //               Pipeline {
    //                   commands: vec![Command {
    //                       words: vec![String::from("!ab")],
    //                       mode: PipelineMode::Nothing
    //                   }],
    //                   reaction: CommandReaction::Normal,
    //                   invert: false
    //               }
    //           )
    //       );
    //   }
    //
    //   #[test]
    //   fn parse_pipeline() {
    //       assert_eq!(
    //           pipeline_command(b"ab|cd de | de fg"),
    //           IResult::Done(
    //               &b""[..],
    //               Pipeline {
    //                   commands: vec![
    //                       Command {
    //                           words: vec![String::from("ab")],
    //                           mode: PipelineMode::StdOut
    //                       },
    //                       Command {
    //                           words: vec![String::from("cd"), String::from("de")],
    //                           mode: PipelineMode::StdOut
    //                       },
    //                       Command {
    //                           words: vec![String::from("de"), String::from("fg")],
    //                           mode: PipelineMode::Nothing
    //                       },
    //                   ],
    //                   reaction: CommandReaction::Normal,
    //                   invert: false
    //               }
    //           )
    //       );
    //
    //       assert_eq!(
    //           parse_script(b"ab|cd de || ! de fg | hi\n"),
    //           IResult::Done(
    //               &b""[..],
    //               ParsedCommand::CommandSequence(vec![CommandLogic {
    //                   pipelines: vec![
    //                       Pipeline {
    //                           commands: vec![
    //                               Command {
    //                                   words: vec![String::from("ab")],
    //                                   mode: PipelineMode::StdOut
    //                               },
    //                               Command {
    //                                   words: vec![String::from("cd"), String::from("de")],
    //                                   mode: PipelineMode::Nothing,
    //                               },
    //                           ],
    //                           reaction: CommandReaction::Or,
    //                           invert: false
    //                       },
    //                       Pipeline {
    //                           commands: vec![
    //                               Command {
    //                                   words: vec![String::from("de"), String::from("fg")],
    //                                   mode: PipelineMode::StdOut
    //                               },
    //                               Command {
    //                                   words: vec![String::from("hi")],
    //                                   mode: PipelineMode::Nothing
    //                               },
    //                           ],
    //                           reaction: CommandReaction::Normal,
    //                           invert: true
    //                       }
    //                   ]
    //               }])
    //           )
    //       );
    //   }
    //
    //   #[test]
    //   fn parse_identifier() {
    //       use nom::verbose_errors::Err;
    //       use nom::ErrorKind;
    //       assert_eq!(identifier(b"bc"), IResult::Done(&b""[..], &b"bc"[..]));
    //       assert_eq!(identifier(b"_bc"), IResult::Done(&b""[..], &b"_bc"[..]));
    //       assert_eq!(identifier(b"_bc0_"), IResult::Done(&b""[..], &b"_bc0_"[..]));
    //       assert_eq!(
    //           identifier(b"_bc0_."),
    //           IResult::Done(&b"."[..], &b"_bc0_"[..])
    //       );
    //       assert_eq!(
    //           identifier(b"0_bc0_"),
    //           IResult::Error(Err::Position(ErrorKind::Alt, &b"0_bc0_"[..]))
    //       );
    //
    //       assert_eq!(legal_identifier("id10t"), true);
    //       assert_eq!(legal_identifier("1d10t"), false);
    //   }
    //
    //   #[test]
    //   fn test_assignment_or_name() {
    //       assert_eq!(
    //           assignment_or_name(b"STUFF"),
    //           IResult::Done(&b""[..], (&b"STUFF"[..], None))
    //       );
    //       assert_eq!(
    //           assignment_or_name(b"STUFF=thing"),
    //           IResult::Done(&b""[..], (&b"STUFF"[..], Some(String::from("thing"))))
    //       );
    //   }

}
