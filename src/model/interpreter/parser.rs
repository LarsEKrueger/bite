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

use nom::character::complete::{none_of, space0, space1};
use nom::combinator::{map, recognize};
use nom::multi::{many1, separated_list};
use nom::sequence::preceded;
use nom::IResult;

use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a str>;

/// A command and its parameters
#[derive(Debug, PartialEq)]
pub struct Command<'a> {
    /// The words of this command
    pub words: Vec<Span<'a>>,
    // pub mode: PipelineMode,
}

/// Parse a (partial) bash script.
pub fn script(input: Span) -> IResult<Span, Command> {
    simple_command(input)
}

fn simple_command(input: Span) -> IResult<Span, Command> {
    map(preceded(space0, separated_list(space1, word)), |words| {
        Command { words }
    })(input)
}

fn word(input: Span) -> IResult<Span, Span> {
    recognize(many1(word_letter))(input)
}

fn word_letter(input: Span) -> IResult<Span, char> {
    none_of(" \n\t\"\'|&;()<>")(input)
}

//named!(word_letter<Span,Span>, none_of!();

//   /*
//   named!(pub script<ParsedCommand>,
//       alt_complete!(
//           map!(preceded!(myspace,newline), |_| ParsedCommand::None) |
//           terminated!( command_sequence, preceded!(myspace,newline))
//           )
//   );*/
//
//   /*
//   redirection:	'>' WORD
//       |	'<' WORD
//       |	NUMBER '>' WORD
//       |	NUMBER '<' WORD
//       |	REDIR_WORD '>' WORD
//       |	REDIR_WORD '<' WORD
//       |	GREATER_GREATER WORD
//       |	NUMBER GREATER_GREATER WORD
//       |	REDIR_WORD GREATER_GREATER WORD
//       |	GREATER_BAR WORD
//       |	NUMBER GREATER_BAR WORD
//       |	REDIR_WORD GREATER_BAR WORD
//       |	LESS_GREATER WORD
//       |	NUMBER LESS_GREATER WORD
//       |	REDIR_WORD LESS_GREATER WORD
//       |	LESS_LESS WORD
//       |	NUMBER LESS_LESS WORD
//       |	REDIR_WORD LESS_LESS WORD
//       |	LESS_LESS_MINUS WORD
//       |	NUMBER LESS_LESS_MINUS WORD
//       |	REDIR_WORD  LESS_LESS_MINUS WORD
//       |	LESS_LESS_LESS WORD
//       |	NUMBER LESS_LESS_LESS WORD
//       |	REDIR_WORD LESS_LESS_LESS WORD
//       |	LESS_AND NUMBER
//       |	NUMBER LESS_AND NUMBER
//       |	REDIR_WORD LESS_AND NUMBER
//       |	GREATER_AND NUMBER
//       |	NUMBER GREATER_AND NUMBER
//       |	REDIR_WORD GREATER_AND NUMBER
//       |	LESS_AND WORD
//       |	NUMBER LESS_AND WORD
//       |	REDIR_WORD LESS_AND WORD
//       |	GREATER_AND WORD
//       |	NUMBER GREATER_AND WORD
//       |	REDIR_WORD GREATER_AND WORD
//       |	GREATER_AND '-'
//       |	NUMBER GREATER_AND '-'
//       |	REDIR_WORD GREATER_AND '-'
//       |	LESS_AND '-'
//       |	NUMBER LESS_AND '-'
//       |	REDIR_WORD LESS_AND '-'
//       |	AND_GREATER WORD
//       |	AND_GREATER_GREATER WORD
//       ;
//   */
//
//   //named!(simple_command_element<String>, do_parse!(word));
//   /*
//   simple_command_element: WORD
//       |	ASSIGNMENT_WORD
//       |	redirection
//       ;
//
//   redirection_list: redirection
//       |	redirection_list redirection
//       ;
//   */
//
//   /*
//   named!(pub assignment<super::Assignment>,
//       do_parse!(
//           var_name : identifier >>
//           tag!("=") >>
//           var_value : word >>
//           ({
//               super::Assignment::new(String::from_utf8_lossy(var_name).into_owned(),var_value)
//           } )
//       )
//   );
//   */
//
//   /*
//   command:	simple_command
//       |	shell_command
//       |	shell_command redirection_list
//       ;
//
//   shell_command:	for_command
//       |	WHILE compound_list DO compound_list DONE
//       |	if_command
//       |	subshell
//       |	group_command
//       |	cond_command
//       ;
//
//   for_command:	FOR WORD newline_list DO compound_list DONE
//       |	FOR WORD newline_list '{' compound_list '}'
//       |	FOR WORD ';' newline_list DO compound_list DONE
//       |	FOR WORD ';' newline_list '{' compound_list '}'
//       |	FOR WORD newline_list IN word_list list_terminator newline_list DO compound_list DONE
//       |	FOR WORD newline_list IN word_list list_terminator newline_list '{' compound_list '}'
//       |	FOR WORD newline_list IN list_terminator newline_list DO compound_list DONE
//       |	FOR WORD newline_list IN list_terminator newline_list '{' compound_list '}'
//       ;
//
//   subshell:	'(' compound_list ')'
//       ;
//
//   if_command:	IF compound_list THEN compound_list FI
//       |	IF compound_list THEN compound_list ELSE compound_list FI
//       |	IF compound_list THEN compound_list elif_clause FI
//       ;
//
//
//   group_command:	'{' compound_list '}'
//       ;
//
//   cond_command:	COND_START COND_CMD COND_END
//       ;
//
//   elif_clause:	ELIF compound_list THEN compound_list
//       |	ELIF compound_list THEN compound_list ELSE compound_list
//       |	ELIF compound_list THEN compound_list elif_clause
//       ;
//
//   /* A list allows leading or trailing newlines and
//   newlines as operators (equivalent to semicolons).
//   It must end with a newline or semicolon.
//   Lists are used within commands such as if, for, while.  */
//
//   list:		newline_list list0
//       ;
//
//   compound_list:	list
//       |	newline_list list1
//       ;
//
//   list0:  	list1 '\n' newline_list
//       |	list1 '&' newline_list
//       |	list1 ';' newline_list
//       ;
//
//   list1:		list1 AND_AND newline_list list1
//       |	list1 OR_OR newline_list list1
//       |	list1 '&' newline_list list1
//       |	list1 ';' newline_list list1
//       |	list1 '\n' newline_list list1
//       |	pipeline_command
//       ;
//   */
//
//   // separated_list_map that handles the return value of the separator.
//   //
//   // `separated_list_map!(
//   //   I -> IResult<I,T>,
//   //   (T,&mut Vec<O>) -> (),
//   //   I -> IResult<I,O>) =>
//   //   I -> IResult<I, Vec<O>>`
//   // separated_list_map(sep, updateFun, X) returns Vec<X> will return Incomplete if there may be
//   // more elements
//   #[macro_export]
//   macro_rules! separated_list_map(
//   ($i:expr, $sep:ident!( $($args:tt)* ), $u:expr, $submac:ident!( $($args2:tt)* )) => (
//       {
//       //FIXME: use crate vec
//       let mut res   = ::std::vec::Vec::new();
//       let mut input = $i.clone();
//
//       // get the first element
//       let input_ = input.clone();
//       match $submac!(input_, $($args2)*) {
//           IResult::Error(_)      => IResult::Done(input, ::std::vec::Vec::new()),
//           IResult::Incomplete(i) => IResult::Incomplete(i),
//           IResult::Done(i,o)     => {
//           if i.input_len() == input.input_len() {
//               IResult::Error(error_position!(ErrorKind::SeparatedList,input))
//           } else {
//               res.push(o);
//               input = i;
//
//               let ret;
//
//               loop {
//               // get the separator first
//               let input_ = input.clone();
//               match $sep!(input_, $($args)*) {
//                   IResult::Error(_) => {
//                   ret = IResult::Done(input, res);
//                   break;
//                   }
//                   IResult::Incomplete(Needed::Unknown) => {
//                   ret = IResult::Incomplete(Needed::Unknown);
//                   break;
//                   },
//                   IResult::Incomplete(Needed::Size(needed)) => {
//                   let (size,overflowed) =
//                       needed.overflowing_add(($i).input_len() - input.input_len());
//                   ret = match overflowed {
//                       true  => IResult::Incomplete(Needed::Unknown),
//                       false => IResult::Incomplete(Needed::Size(size)),
//                   };
//                   break;
//                   },
//                   IResult::Done(i2,sep_val)     => {
//                   let i2_len = i2.input_len();
//                   if i2_len == input.input_len() {
//                       ret = IResult::Done(input, res);
//                       break;
//                   }
//
//                   $u(&mut res, sep_val);
//
//                   // get the element next
//                   match $submac!(i2, $($args2)*) {
//                       IResult::Error(_) => {
//                       ret = IResult::Done(input, res);
//                       break;
//                       },
//                       IResult::Incomplete(Needed::Unknown) => {
//                       ret = IResult::Incomplete(Needed::Unknown);
//                       break;
//                       },
//                       IResult::Incomplete(Needed::Size(needed)) => {
//                       let (size,overflowed) = needed.overflowing_add(($i).input_len() - i2_len);
//                       ret = match overflowed {
//                           true  => IResult::Incomplete(Needed::Unknown),
//                           false => IResult::Incomplete(Needed::Size(size)),
//                       };
//                       break;
//                       },
//                       IResult::Done(i3,o3)    => {
//                       if i3.input_len() == i2_len {
//                           ret = IResult::Done(input, res);
//                           break;
//                       }
//                       res.push(o3);
//                       input = i3;
//                       }
//                   }
//                   }
//               }
//               }
//
//               ret
//           }
//           },
//       }
//       }
//   );
//   ($i:expr, $submac:ident!( $($args:tt)* ), $u:expr, $g:expr) => (
//       separated_list_map!($i, $submac!($($args)*), $u, call!($g));
//   );
//   ($i:expr, $f:expr, $u:expr, $submac:ident!( $($args:tt)* )) => (
//       separated_list_map!($i, call!($f), $u, $submac!($($args)*));
//   );
//   ($i:expr, $f:expr, $u:expr, $g:expr) => (
//       separated_list_map!($i, call!($f), $u, call!($g));
//   );
//   );
//
//   // Parse a list of commands to be executed in one run.
//   //
//   // This parser is equivalent to bash's simple_list and simple_list1 rules.
//   //
//   // We also need to handle the precedence of && and || over ; and &. We do this by breaking
//   // simple_list1 with the four alternatives into two nested rules that are used to return an
//   // expression tree.
//   //
//   // The last CommandInfo can have an additional ; or &, but not an && or ||
//   named!(
//       command_sequence<ParsedCommand>,
//       // This expressions of lower precedence: ; and &
//       do_parse!(
//           seq: separated_list_map!(command_sequence_sep, updateReaction, command_logic)
//               >> cr: opt!(command_sequence_sep)
//               >> (ParsedCommand::new_sequence(seq, cr))
//       )
//   );
//
//   /// Helper to set the CommandReaction of the last entry
//   fn updateReaction(cis: &mut Vec<CommandLogic>, cr: CommandReaction) {
//       cis.last_mut().map(|ci| ci.set_reaction(cr));
//   }
//
//   // Separator for simple_list, i.e. & and ;
//   named!(
//       command_sequence_sep<CommandReaction>,
//       preceded!(
//           myspace,
//           alt_complete!(
//               map!(tag!(";"), |_| CommandReaction::Normal)
//                   | map!(tag!("&"), |_| CommandReaction::Background)
//           )
//       )
//   );
//
//   // Helper parser to ensure precedence of && and || over & and ;
//   named!(
//       command_logic<CommandLogic>,
//       // This expression has higher precedence: && and ||
//       map!(
//           separated_list_map!(command_logic_sep, updateReaction_cl, pipeline_command),
//           CommandLogic::new
//       )
//   );
//
//   /// Helper to set the CommandReaction of the last entry
//   fn updateReaction_cl(cis: &mut Vec<Pipeline>, cr: CommandReaction) {
//       cis.last_mut().map(|ci| ci.set_reaction(cr));
//   }
//
//   // Separator for command_logic, i.e. && and ||
//   named!(
//       command_logic_sep<CommandReaction>,
//       preceded!(
//           myspace,
//           alt_complete!(
//               map!(tag!("&&"), |_| CommandReaction::And) | map!(tag!("||"), |_| CommandReaction::Or)
//           )
//       )
//   );
//
//   // Pipeline with optional inversion
//   named!(
//       pipeline_command<Pipeline>,
//       alt!(
//           do_parse!(
//               myspace
//                   >> tag!("!")
//                   >> many1!(one_of!(" \t"))
//                   >> ci: pipeline_command
//                   >> (Pipeline {
//                       commands: ci.commands,
//                       reaction: ci.reaction,
//                       invert: true ^ ci.invert
//                   })
//           ) | pipeline
//       )
//   );
//
//   /*
//   pipeline_command: pipeline
//       |	BANG pipeline_command
//       |	timespec pipeline_command
//       |	timespec list_terminator
//       |	BANG list_terminator
//       ;
//   */
//
//   // Pipeline command
//   named!(
//       pipeline<Pipeline>,
//       map!(
//           separated_list_map!(pipeline_sep, update_pipeline, simple_command),
//           Pipeline::new
//       )
//   );
//
//   // Separator for pipelines
//   named!(
//       pipeline_sep<PipelineMode>,
//       preceded!(
//           myspace,
//           alt_complete!(
//               map!(tag!("|&"), |_| PipelineMode::StdOutStdErr)
//                   | do_parse!(not!(tag!("||")) >> tag!("|") >> (PipelineMode::StdOut))
//           )
//       )
//   );
//
//   /// Updater for pipeline commands
//   fn update_pipeline(commands: &mut Vec<Command>, mode: PipelineMode) {
//       commands.last_mut().map(|cmd| cmd.set_pipeline_mode(mode));
//   }
//
//   /*
//   timespec:	TIME
//       |	TIME TIMEOPT
//       |	TIME TIMEOPT TIMEIGN
//       ;
//   */
//
//   fn is_alphanum_or_underscore(c: u8) -> bool {
//       is_alphanumeric(c) || c == b'_'
//   }
//
//   named!(alpha_or_underscore, alt!(alpha | tag!("_")));
//   named!(pub identifier,
//       recognize!(preceded!(
//           alpha_or_underscore,
//           take_while!(is_alphanum_or_underscore)))
//   );
//
//   pub fn legal_identifier(s: &str) -> bool {
//       if let IResult::Done(b"", _) = identifier(s.as_bytes()) {
//           true
//       } else {
//           false
//       }
//   }
//
//   named!(pub assignment_or_name<(&[u8],Option<String>)>,
//   do_parse!(
//       name : identifier >>
//       value : alt_complete!(
//           do_parse!(
//               tag!("=") >>
//               w:word >> (Some(w))
//               ) |
//           map!(eof!(),|_| None)
//           )
//       >> (name,value)
//       )
//   );

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

    #[test]
    fn parse_word() {
        assert_eq!(
            word(Span::new("bc")),
            Ok((span(2, 1, ""), span(0, 1, "bc")))
        );
        assert_eq!(
            word(Span::new("bc<")),
            Ok((span(2, 1, "<"), span(0, 1, "bc")))
        );
    }

    #[test]
    fn parse_simple_command() {
        assert_eq!(
            simple_command(Span::new("ab bc   cd \t\tde")),
            Ok((
                span(15, 1, ""),
                Command {
                    words: vec![
                        span(0, 1, "ab"),
                        span(3, 1, "bc"),
                        span(8, 1, "cd"),
                        span(13, 1, "de"),
                    ]
                }
            ))
        );

        assert_eq!(
            simple_command(Span::new(" \tab bc   cd \t\tde")),
            Ok((
                span(17, 1, ""),
                Command {
                    words: vec![
                        span(2, 1, "ab"),
                        span(5, 1, "bc"),
                        span(10, 1, "cd"),
                        span(15, 1, "de"),
                    ]
                }
            ))
        );

        // A simple command ends at the end of the line
        assert_eq!(
            simple_command(Span::new("ab cd\nef")),
            Ok((
                span(5, 1, "\nef"),
                Command {
                    words: vec![span(0, 1, "ab"), span(3, 1, "cd"),]
                }
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
