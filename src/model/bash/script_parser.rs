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

use nom::newline;

/*
%token IF THEN ELSE ELIF FI CASE ESAC FOR SELECT WHILE UNTIL DO DONE FUNCTION COPROC
%token COND_START COND_END COND_ERROR
%token IN BANG TIME TIMEOPT TIMEIGN
%token <word> WORD ASSIGNMENT_WORD REDIR_WORD
%token <number> NUMBER
%token <word_list> ARITH_CMD ARITH_FOR_EXPRS
%token <command> COND_CMD
%token AND_AND OR_OR GREATER_GREATER LESS_LESS LESS_AND LESS_LESS_LESS
%token GREATER_AND SEMI_SEMI SEMI_AND SEMI_SEMI_AND
%token LESS_LESS_MINUS AND_GREATER AND_GREATER_GREATER LESS_GREATER
%token GREATER_BAR BAR_AND
%start inputunit

%left '&' ';' '\n' yacc_EOF
%left AND_AND OR_OR
%right '|' BAR_AND
*/

named!(
    pub parse_script<super::Command>,
    terminated!(simple_command
                , newline)
);
/*
inputunit:	simple_list simple_list_terminator
	|	'\n'
	|	error '\n'
	|	yacc_EOF
	;
*/

/*
word_list:	WORD
	|	word_list WORD
	;

redirection:	'>' WORD
	|	'<' WORD
	|	NUMBER '>' WORD
	|	NUMBER '<' WORD
	|	REDIR_WORD '>' WORD
	|	REDIR_WORD '<' WORD
	|	GREATER_GREATER WORD
	|	NUMBER GREATER_GREATER WORD
	|	REDIR_WORD GREATER_GREATER WORD
	|	GREATER_BAR WORD
	|	NUMBER GREATER_BAR WORD
	|	REDIR_WORD GREATER_BAR WORD
	|	LESS_GREATER WORD
	|	NUMBER LESS_GREATER WORD
	|	REDIR_WORD LESS_GREATER WORD
	|	LESS_LESS WORD
	|	NUMBER LESS_LESS WORD
	|	REDIR_WORD LESS_LESS WORD
	|	LESS_LESS_MINUS WORD
	|	NUMBER LESS_LESS_MINUS WORD
	|	REDIR_WORD  LESS_LESS_MINUS WORD
	|	LESS_LESS_LESS WORD
	|	NUMBER LESS_LESS_LESS WORD
	|	REDIR_WORD LESS_LESS_LESS WORD
	|	LESS_AND NUMBER
	|	NUMBER LESS_AND NUMBER
	|	REDIR_WORD LESS_AND NUMBER
	|	GREATER_AND NUMBER
	|	NUMBER GREATER_AND NUMBER
	|	REDIR_WORD GREATER_AND NUMBER
	|	LESS_AND WORD
	|	NUMBER LESS_AND WORD
	|	REDIR_WORD LESS_AND WORD
	|	GREATER_AND WORD
	|	NUMBER GREATER_AND WORD
	|	REDIR_WORD GREATER_AND WORD
	|	GREATER_AND '-'
	|	NUMBER GREATER_AND '-'
	|	REDIR_WORD GREATER_AND '-'
	|	LESS_AND '-'
	|	NUMBER LESS_AND '-'
	|	REDIR_WORD LESS_AND '-'
	|	AND_GREATER WORD
	|	AND_GREATER_GREATER WORD
	;
*/

//named!(simple_command_element<String>, do_parse!(word));
/*
simple_command_element: WORD
	|	ASSIGNMENT_WORD
	|	redirection
	;

redirection_list: redirection
	|	redirection_list redirection
	;
*/

named!(myspace, eat_separator!(&b" \t"[..]));

macro_rules! spaced (
  ($i:expr, $($args:tt)*) => (
    {
      sep!($i, myspace, $($args)*)
    }
  )
);

named!(
    simple_command<super::Command>,
    map!(spaced!(many1!(word)), |cs| {
        super::Command::SimpleCommand(cs)
    })
);

/*
simple_command:	simple_command_element
	|	simple_command simple_command_element
	;
*/

//named!(command<Command>, do_parse!(simple_command));

/*
command:	simple_command
	|	shell_command
	|	shell_command redirection_list
	|	function_def
	|	coproc
	;

shell_command:	for_command
	|	case_command
 	|	WHILE compound_list DO compound_list DONE
	|	UNTIL compound_list DO compound_list DONE
	|	select_command
	|	if_command
	|	subshell
	|	group_command
	|	arith_command
	|	cond_command
	|	arith_for_command
	;

for_command:	FOR WORD newline_list DO compound_list DONE
	|	FOR WORD newline_list '{' compound_list '}'
	|	FOR WORD ';' newline_list DO compound_list DONE
	|	FOR WORD ';' newline_list '{' compound_list '}'
	|	FOR WORD newline_list IN word_list list_terminator newline_list DO compound_list DONE
	|	FOR WORD newline_list IN word_list list_terminator newline_list '{' compound_list '}'
	|	FOR WORD newline_list IN list_terminator newline_list DO compound_list DONE
	|	FOR WORD newline_list IN list_terminator newline_list '{' compound_list '}'
	;

arith_for_command:	FOR ARITH_FOR_EXPRS list_terminator newline_list DO compound_list DONE
	|		FOR ARITH_FOR_EXPRS list_terminator newline_list '{' compound_list '}'
	|		FOR ARITH_FOR_EXPRS DO compound_list DONE
	|		FOR ARITH_FOR_EXPRS '{' compound_list '}'
	;

select_command:	SELECT WORD newline_list DO list DONE
	|	SELECT WORD newline_list '{' list '}'
	|	SELECT WORD ';' newline_list DO list DONE
	|	SELECT WORD ';' newline_list '{' list '}'
	|	SELECT WORD newline_list IN word_list list_terminator newline_list DO list DONE
	|	SELECT WORD newline_list IN word_list list_terminator newline_list '{' list '}'
	;

case_command:	CASE WORD newline_list IN newline_list ESAC
	|	CASE WORD newline_list IN case_clause_sequence newline_list ESAC
	|	CASE WORD newline_list IN case_clause ESAC
	;

function_def:	WORD '(' ')' newline_list function_body
	|	FUNCTION WORD '(' ')' newline_list function_body
	|	FUNCTION WORD newline_list function_body
	;

function_body:	shell_command
	|	shell_command redirection_list
	;

subshell:	'(' compound_list ')'
	;

coproc:		COPROC shell_command
	|	COPROC shell_command redirection_list
	|	COPROC WORD shell_command
	|	COPROC WORD shell_command redirection_list
	|	COPROC simple_command
	;

if_command:	IF compound_list THEN compound_list FI
	|	IF compound_list THEN compound_list ELSE compound_list FI
	|	IF compound_list THEN compound_list elif_clause FI
	;


group_command:	'{' compound_list '}'
	;

arith_command:	ARITH_CMD
	;

cond_command:	COND_START COND_CMD COND_END
	; 

elif_clause:	ELIF compound_list THEN compound_list
	|	ELIF compound_list THEN compound_list ELSE compound_list
	|	ELIF compound_list THEN compound_list elif_clause
	;

case_clause:	pattern_list
	|	case_clause_sequence pattern_list
	;

pattern_list:	newline_list pattern ')' compound_list
	|	newline_list pattern ')' newline_list
	|	newline_list '(' pattern ')' compound_list
	|	newline_list '(' pattern ')' newline_list
	;

case_clause_sequence:  pattern_list SEMI_SEMI
	|	case_clause_sequence pattern_list SEMI_SEMI
	|	pattern_list SEMI_AND
	|	case_clause_sequence pattern_list SEMI_AND
	|	pattern_list SEMI_SEMI_AND
	|	case_clause_sequence pattern_list SEMI_SEMI_AND
	;

pattern:	WORD
	|	pattern '|' WORD
	;

/* A list allows leading or trailing newlines and
   newlines as operators (equivalent to semicolons).
   It must end with a newline or semicolon.
   Lists are used within commands such as if, for, while.  */

list:		newline_list list0
	;

compound_list:	list
	|	newline_list list1
	;

list0:  	list1 '\n' newline_list
	|	list1 '&' newline_list
	|	list1 ';' newline_list
	;

list1:		list1 AND_AND newline_list list1
	|	list1 OR_OR newline_list list1
	|	list1 '&' newline_list list1
	|	list1 ';' newline_list list1
	|	list1 '\n' newline_list list1
	|	pipeline_command
	;
*/

/*
list_terminator:'\n'
	|	';'
	|	yacc_EOF
	;

newline_list:
	|	newline_list '\n'
	;
*/

// named!(simple_list, apply!(simple_list1));

/*
simple_list:	simple_list1
	|	simple_list1 '&'
	|	simple_list1 ';'
	;
*/

// named!(simple_list1, apply!(pipeline_command));
/*
simple_list1:	simple_list1 AND_AND newline_list simple_list1
	|	simple_list1 OR_OR newline_list simple_list1
	|	simple_list1 '&' simple_list1
	|	simple_list1 ';' simple_list1
	|	pipeline_command
	;
*/

//named!(pipeline_command, apply!(pipeline));
/*
pipeline_command: pipeline
	|	BANG pipeline_command
	|	timespec pipeline_command
	|	timespec list_terminator
	|	BANG list_terminator
	;
*/

//named!(pipeline, apply!(command));
/*
pipeline:	pipeline '|' newline_list pipeline
	|	pipeline BAR_AND newline_list pipeline
	|	command
	;

timespec:	TIME
	|	TIME TIMEOPT
	|	TIME TIMEOPT TIMEIGN
	;

STRING_INT_ALIST word_token_alist[] = {
  { "if", IF },
  { "then", THEN },
  { "else", ELSE },
  { "elif", ELIF },
  { "fi", FI },
  { "case", CASE },
  { "esac", ESAC },
  { "for", FOR },
#if defined (SELECT_COMMAND)
  { "select", SELECT },
#endif
  { "while", WHILE },
  { "until", UNTIL },
  { "do", DO },
  { "done", DONE },
  { "in", IN },
  { "function", FUNCTION },
#if defined (COMMAND_TIMING)
  { "time", TIME },
#endif
  { "{", '{' },
  { "}", '}' },
  { "!", BANG },
#if defined (COND_COMMAND)
  { "[[", COND_START },
  { "]]", COND_END },
#endif
#if defined (COPROCESS_SUPPORT)
  { "coproc", COPROC },
#endif
  { (char *)NULL, 0}
};

STRING_INT_ALIST other_token_alist[] = {
  { "--", TIMEIGN },
  { "-p", TIMEOPT },
  { "&&", AND_AND },
  { "||", OR_OR },
  { ">>", GREATER_GREATER },
  { "<<", LESS_LESS },
  { "<&", LESS_AND },
  { ">&", GREATER_AND },
  { ";;", SEMI_SEMI },
  { ";&", SEMI_AND },
  { ";;&", SEMI_SEMI_AND },
  { "<<-", LESS_LESS_MINUS },
  { "<<<", LESS_LESS_LESS },
  { "&>", AND_GREATER },
  { "&>>", AND_GREATER_GREATER },
  { "<>", LESS_GREATER },
  { ">|", GREATER_BAR },
  { "|&", BAR_AND },
  { "EOF", yacc_EOF },
  { ">", '>' },
  { "<", '<' },
  { "-", '-' },
  { "{", '{' },
  { "}", '}' },
  { ";", ';' },
  { "(", '(' },
  { ")", ')' },
  { "|", '|' },
  { "&", '&' },
  { "newline", '\n' },
  { (char *)NULL, 0}
};
*/

/* others not listed here:
	WORD			look at yylval.word
	ASSIGNMENT_WORD		look at yylval.word
	NUMBER			look at yylval.number
	ARITH_CMD		look at yylval.word_list
	ARITH_FOR_EXPRS		look at yylval.word_list
	COND_CMD		look at yylval.command
*/

named!(
    word<String>,
    map!(many1!(word_letter), |c| c.into_iter().collect())
);

named!(word_letter<char>, none_of!(" \n\t\"\'|&;()<>"));

#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::*;
    use super::super::*;

    #[test]
    fn test_word() {
        assert_eq!(word(b"bc"), IResult::Done(&b""[..], "bc".to_string()));
        assert_eq!(word(b"bc<"), IResult::Done(&b"<"[..], "bc".to_string()));
    }

    #[test]
    fn test_simple_command() {
        assert_eq!(
            simple_command(b"ab bc   cd \t\tde"),
            IResult::Done(
                &b""[..],
                Command::SimpleCommand(vec![
                    String::from("ab"),
                    String::from("bc"),
                    String::from("cd"),
                    String::from("de"),
                ])
            )
        );
        assert_eq!(
            simple_command(b" \tab bc   cd \t\tde"),
            IResult::Done(
                &b""[..],
                Command::SimpleCommand(vec![
                    String::from("ab"),
                    String::from("bc"),
                    String::from("cd"),
                    String::from("de"),
                ])
            )
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse_script(b" ab bc   cd \t\tde\n"),
            IResult::Done(
                &b""[..],
                Command::SimpleCommand(vec![
                    String::from("ab"),
                    String::from("bc"),
                    String::from("cd"),
                    String::from("de"),
                ])
            )
        );
    }
}
