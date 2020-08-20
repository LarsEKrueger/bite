/*
    BiTE - Bash-integrated Terminal Emulator
    Copyright (C) 2020  Lars Krüger

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

//! Bash script grammar for sesd
//!
//! Ported from bash's parse.y

use sesd::{char::CharMatcher, DynamicGrammar, TextGrammar, TextRule};

pub type Parser = sesd::Parser<char,CharMatcher,DynamicGrammar<char,CharMatcher>>;

grammar!{pub script2,
   {
       use sesd::char::CharMatcher::*;
   },
   // Type of the tokens
   char,
   // Type of the token matcher
   sesd::char::CharMatcher,
   // Name of start symbol.
   INPUTUNIT,
   // Empty non-terminals
   [
       WS_STAR,
       COMMENT_TEXT,
       COND_CMD ,
       IDENTIFIER1_STAR
   ],
   // Other non-terminals
   [
       AND_AND ,
       AND_GREATER ,
       AND_GREATER_GREATER ,
       ARITH_COMMAND ,
       ARITH_FOR_COMMAND ,
       ARITH_FOR_EXPRS ,
       ASSIGNMENT_WORD ,
       BANG ,
       BAR_AND ,
       CASE ,
       CASE_CLAUSE ,
       CASE_CLAUSE_SEQUENCE ,
       CASE_COMMAND ,
       COMMAND ,
       COMMENT ,
       COMPOUND_LIST ,
       COND_COMMAND ,
       COND_END ,
       COND_START ,
       COPROC ,
       DO ,
       DONE ,
       ELIF ,
       ELIF_CLAUSE ,
       ELSE ,
       ESAC ,
       EVAL ,
       FI ,
       FOR ,
       FOR_COMMAND ,
       FUNCTION ,
       FUNCTION_BODY ,
       FUNCTION_DEF ,
       GAP ,
       GREATER_AND ,
       GREATER_BAR ,
       GREATER_GREATER ,
       GROUP_COMMAND ,
       IDENTIFIER ,
       IDENTIFIER0 ,
       IDENTIFIER1 ,
       IF ,
       IF_COMMAND ,
       IN ,
       INPUTUNIT ,
       LESS_AND ,
       LESS_GREATER ,
       LESS_LESS ,
       LESS_LESS_LESS ,
       LESS_LESS_MINUS ,
       LET ,
       LIST ,
       LIST0 ,
       LIST1 ,
       LIST_TERMINATOR ,
       LOGICAL ,
       LOGICAL_SEP_BG,
       LOGICAL_SEP_FG ,
       NEWLINE ,
       NUMBER ,
       OR_OR ,
       PATTERN ,
       PATTERN_LIST ,
       PIPELINE ,
       PIPELINE_COMMAND ,
       REDIRECTION ,
       REDIRECTION_LIST ,
       REDIR_WORD ,
       SELECT ,
       SELECT_COMMAND ,
       SEMI_AND ,
       SEMI_SEMI ,
       SEMI_SEMI_AND ,
       SHELL_COMMAND ,
       SIMPLE_COMMAND ,
       SIMPLE_COMMAND_ELEMENT ,
       SUBSHELL ,
       THEN ,
       TIME ,
       TIMEIGN ,
       TIMEOPT ,
       TIMESPEC ,
       UNTIL ,
       WHILE ,
       WORD ,
       WORD_LETTER ,
       WORD_LIST ,
       WS ,
       WSCHAR
   ],
   // Matchers
   [
       T_HASH = Exact('#'),
       T_AMPERSAND = Exact('&'),
       T_SEMICOLON = Exact(';'),
       T_SPACE = Exact(' '),
       T_TAB = Exact('\t'),
       T_NEWLINE = Exact('\n'),
       T_NO_NEWLINE = OtherThan( '\n'),
       T_GREATER = Exact('>'),
       T_LESS = Exact('<'),
       T_MINUS = Exact('-'),
       T_BANG = Exact('!'),
       T_PIPE = Exact('|'),
       T_A = Exact('a'),
       T_C = Exact('c'),
       T_D = Exact('d'),
       T_E = Exact('e'),
       T_F = Exact('f'),
       T_H = Exact('h'),
       T_I = Exact('i'),
       T_L = Exact('l'),
       T_M = Exact('m'),
       T_N = Exact('n'),
       T_O = Exact('o'),
       T_P = Exact('p'),
       T_R = Exact('r'),
       T_S = Exact('s'),
       T_T = Exact('t'),
       T_U = Exact('u'),
       T_V = Exact('v'),
       T_W = Exact('w'),
       T_SQ_OPEN = Exact('['),
       T_SQ_CLOSE = Exact(']'),
       T_BR_OPEN = Exact('{'),
       T_BR_CLOSE = Exact('}'),
       T_PAREN_OPEN = Exact('('),
       T_PAREN_CLOSE = Exact(')'),
       T_EQUAL = Exact('='),
       T_UNDERSCORE = Exact('_'),
       T_DIGIT = Range('0', '9'),
       T_LC_LETTER = Range('a', 'z'),
       T_UC_LETTER = Range('A', 'Z'),
       T_PERCENT = Exact('\x25'),
       T_STAR_COLON = Range('\x2a','\x3a'),
       T_QUESTION_Z = Range('\x3f','\x5a'),
       T_CARET_UNDERSCORE = Range('\x5e','\x5f'),
       T_LC_A_Z = Range('\x61','\x7a'),
       T_TILDE = Exact('\x7e'),
       T_OTHER = Range('\u{80}','\u{10ffff}')
   ],
   // Rules.
   [
       // A comment at the last position, no newline
       INPUTUNIT = COMMENT,
       // A logical at the last position, no newline
       INPUTUNIT= LOGICAL,
       // A logical at the last position, exlicit foreground separator, i.e. semicolon or newline
       INPUTUNIT = LOGICAL WS_STAR LOGICAL_SEP_FG GAP,
       // A logical at the last position, exlicit background separator. i.e. ampersand
       INPUTUNIT = LOGICAL WS_STAR LOGICAL_SEP_BG GAP,
       // A comment followed by more input units
       INPUTUNIT = COMMENT NEWLINE INPUTUNIT,
       // A logical with exlicit foreground separator, followed by more input units
       INPUTUNIT = LOGICAL WS_STAR LOGICAL_SEP_FG INPUTUNIT,
       // A logical with exlicit background separator, followed by more input units
       INPUTUNIT = LOGICAL WS_STAR LOGICAL_SEP_BG WS_STAR INPUTUNIT,

       // The separators
       LOGICAL_SEP_FG = T_SEMICOLON,
       LOGICAL_SEP_FG = NEWLINE,
       LOGICAL_SEP_BG = T_AMPERSAND,
       NEWLINE = T_NEWLINE,

       // A comment skips the whitespace before its marker, then eats the hash and the rest of the
       // line, but not the newline
       COMMENT = WS_STAR T_HASH COMMENT_TEXT,
       COMMENT_TEXT = T_NO_NEWLINE COMMENT_TEXT,

       // Whitespace characters. Does not include newline.
       WSCHAR = T_SPACE,
       WSCHAR = T_TAB,

       // At least one whitespace character
       WS = WSCHAR ,
       WS = WSCHAR WS ,

       // Zero or more whitespace characters
       WS_STAR = WSCHAR WS_STAR ,

       // gap = white space + newlines
       GAP = WS_STAR ,
       GAP = WS_STAR T_NEWLINE,
       GAP = WS_STAR T_NEWLINE GAP ,

       // Logical expressions are pipelines separated by && or ||
       LOGICAL = WS_STAR PIPELINE_COMMAND ,
       LOGICAL = WS_STAR PIPELINE_COMMAND WS_STAR AND_AND GAP LOGICAL ,
       LOGICAL = WS_STAR PIPELINE_COMMAND WS_STAR OR_OR GAP LOGICAL ,

       WORD_LIST = WORD WORD_LIST ,
       WORD_LIST = WORD ,

       REDIRECTION = T_GREATER WORD ,
       REDIRECTION = T_LESS WORD ,
       REDIRECTION = NUMBER T_GREATER WORD ,
       REDIRECTION = NUMBER T_LESS WORD ,
       REDIRECTION = REDIR_WORD T_GREATER WORD ,
       REDIRECTION = REDIR_WORD T_LESS WORD ,
       REDIRECTION = GREATER_GREATER WORD ,
       REDIRECTION = NUMBER GREATER_GREATER WORD ,
       REDIRECTION = REDIR_WORD GREATER_GREATER WORD ,
       REDIRECTION = GREATER_BAR WORD ,
       REDIRECTION = NUMBER GREATER_BAR WORD ,
       REDIRECTION = REDIR_WORD GREATER_BAR WORD ,
       REDIRECTION = LESS_GREATER WORD ,
       REDIRECTION = NUMBER LESS_GREATER WORD ,
       REDIRECTION = REDIR_WORD LESS_GREATER WORD ,
       REDIRECTION = LESS_LESS WORD ,
       REDIRECTION = NUMBER LESS_LESS WORD ,
       REDIRECTION = REDIR_WORD LESS_LESS WORD ,
       REDIRECTION = LESS_LESS_MINUS WORD ,
       REDIRECTION = NUMBER LESS_LESS_MINUS WORD ,
       REDIRECTION = REDIR_WORD LESS_LESS_MINUS WORD ,
       REDIRECTION = LESS_LESS_LESS WORD ,
       REDIRECTION = NUMBER LESS_LESS_LESS WORD ,
       REDIRECTION = REDIR_WORD LESS_LESS_LESS WORD ,
       REDIRECTION = LESS_AND NUMBER ,
       REDIRECTION = NUMBER LESS_AND NUMBER ,
       REDIRECTION = REDIR_WORD LESS_AND NUMBER ,
       REDIRECTION = GREATER_AND NUMBER ,
       REDIRECTION = NUMBER GREATER_AND NUMBER ,
       REDIRECTION = REDIR_WORD GREATER_AND NUMBER ,
       REDIRECTION = LESS_AND WORD ,
       REDIRECTION = NUMBER LESS_AND WORD ,
       REDIRECTION = REDIR_WORD LESS_AND WORD ,
       REDIRECTION = GREATER_AND WORD ,
       REDIRECTION = NUMBER GREATER_AND WORD ,
       REDIRECTION = REDIR_WORD GREATER_AND WORD ,
       REDIRECTION = GREATER_AND T_MINUS,
       REDIRECTION = NUMBER GREATER_AND T_MINUS,
       REDIRECTION = REDIR_WORD GREATER_AND T_MINUS,
       REDIRECTION = LESS_AND T_MINUS,
       REDIRECTION = NUMBER LESS_AND T_MINUS,
       REDIRECTION = REDIR_WORD LESS_AND T_MINUS,
       REDIRECTION = AND_GREATER WORD ,
       REDIRECTION = AND_GREATER_GREATER WORD ,

       BANG = T_BANG,
       TIMEIGN = T_MINUS T_MINUS,
       TIMEOPT = T_MINUS T_P,
       AND_AND = T_AMPERSAND T_AMPERSAND,
       OR_OR = T_PIPE T_PIPE,
       GREATER_GREATER = T_GREATER T_GREATER,
       LESS_LESS = T_LESS T_LESS,
       LESS_AND = T_LESS T_AMPERSAND,
       GREATER_AND = T_GREATER T_AMPERSAND,
       SEMI_SEMI = T_SEMICOLON T_SEMICOLON,
       SEMI_AND = T_SEMICOLON T_AMPERSAND,
       SEMI_SEMI_AND = T_SEMICOLON T_SEMICOLON T_AMPERSAND,
       LESS_LESS_MINUS = T_LESS T_LESS T_MINUS,
       LESS_LESS_LESS = T_LESS T_LESS T_LESS,
       AND_GREATER = T_AMPERSAND T_GREATER,
       AND_GREATER_GREATER = T_AMPERSAND T_GREATER T_GREATER,
       LESS_GREATER = T_LESS T_GREATER,
       GREATER_BAR = T_GREATER T_PIPE,
       BAR_AND = T_PIPE T_AMPERSAND,
       COND_START = T_SQ_OPEN T_SQ_OPEN,
       COND_END = T_SQ_CLOSE T_SQ_CLOSE,

       SIMPLE_COMMAND_ELEMENT = WORD ,
       SIMPLE_COMMAND_ELEMENT = ASSIGNMENT_WORD ,
       SIMPLE_COMMAND_ELEMENT = REDIRECTION ,

       ASSIGNMENT_WORD = WORD T_EQUAL WORD ,
       ASSIGNMENT_WORD = LET WORD T_EQUAL WORD ,
       ASSIGNMENT_WORD = EVAL WORD T_EQUAL WORD ,

       REDIRECTION_LIST = REDIRECTION ,
       REDIRECTION_LIST = REDIRECTION REDIRECTION_LIST ,

       SIMPLE_COMMAND = SIMPLE_COMMAND_ELEMENT WS_STAR ,
       SIMPLE_COMMAND = SIMPLE_COMMAND_ELEMENT WS SIMPLE_COMMAND ,

       COMMAND = SIMPLE_COMMAND ,
       COMMAND = SHELL_COMMAND ,
       COMMAND = SHELL_COMMAND REDIRECTION_LIST ,
       COMMAND = FUNCTION_DEF ,
       COMMAND = COPROC ,

       SHELL_COMMAND = FOR_COMMAND ,
       SHELL_COMMAND = CASE_COMMAND ,
       SHELL_COMMAND = WHILE COMPOUND_LIST DO COMPOUND_LIST DONE ,
       SHELL_COMMAND = UNTIL COMPOUND_LIST DO COMPOUND_LIST DONE ,
       SHELL_COMMAND = SELECT_COMMAND ,
       SHELL_COMMAND = IF_COMMAND ,
       SHELL_COMMAND = SUBSHELL ,
       SHELL_COMMAND = GROUP_COMMAND ,
       SHELL_COMMAND = ARITH_COMMAND ,
       SHELL_COMMAND = COND_COMMAND ,
       SHELL_COMMAND = ARITH_FOR_COMMAND ,

       FOR_COMMAND = FOR WS WORD GAP DO COMPOUND_LIST DONE ,
       FOR_COMMAND = FOR WS WORD GAP T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,
       FOR_COMMAND = FOR WS WORD T_SEMICOLON GAP DO COMPOUND_LIST DONE ,
       FOR_COMMAND = FOR WS WORD T_SEMICOLON GAP T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,
       FOR_COMMAND = FOR WS WORD GAP IN WORD_LIST LIST_TERMINATOR GAP DO COMPOUND_LIST DONE ,
       FOR_COMMAND = FOR WS WORD GAP IN WORD_LIST LIST_TERMINATOR GAP T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,
       FOR_COMMAND = FOR WS WORD GAP IN LIST_TERMINATOR GAP DO COMPOUND_LIST DONE ,
       FOR_COMMAND = FOR WS WORD GAP IN LIST_TERMINATOR GAP T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,

       ARITH_FOR_COMMAND = FOR WS ARITH_FOR_EXPRS LIST_TERMINATOR GAP DO COMPOUND_LIST DONE ,
       ARITH_FOR_COMMAND = FOR WS ARITH_FOR_EXPRS LIST_TERMINATOR GAP T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,
       ARITH_FOR_COMMAND = FOR WS ARITH_FOR_EXPRS DO COMPOUND_LIST DONE ,
       ARITH_FOR_COMMAND = FOR WS ARITH_FOR_EXPRS T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,

       SELECT_COMMAND = SELECT WS WORD GAP DO LIST DONE ,
       SELECT_COMMAND = SELECT WS WORD GAP T_BR_OPEN LIST T_BR_CLOSE,
       SELECT_COMMAND = SELECT WS WORD T_SEMICOLON GAP DO LIST DONE ,
       SELECT_COMMAND = SELECT WS WORD T_SEMICOLON GAP T_BR_OPEN LIST T_BR_CLOSE,
       SELECT_COMMAND = SELECT WS WORD GAP IN WORD_LIST LIST_TERMINATOR GAP DO LIST DONE ,
       SELECT_COMMAND = SELECT WS WORD GAP IN WORD_LIST LIST_TERMINATOR GAP T_BR_OPEN LIST T_BR_CLOSE,

       CASE_COMMAND = CASE WS WORD GAP IN GAP ESAC ,
       CASE_COMMAND = CASE WS WORD GAP IN CASE_CLAUSE_SEQUENCE GAP ESAC ,
       CASE_COMMAND = CASE WS WORD GAP IN CASE_CLAUSE ESAC ,

       FUNCTION_DEF = WORD T_PAREN_OPEN T_PAREN_CLOSE GAP FUNCTION_BODY ,
       FUNCTION_DEF = FUNCTION WORD T_PAREN_OPEN T_PAREN_CLOSE GAP FUNCTION_BODY ,
       FUNCTION_DEF = FUNCTION WORD GAP FUNCTION_BODY ,
       FUNCTION_BODY = SHELL_COMMAND ,
       FUNCTION_BODY = SHELL_COMMAND REDIRECTION_LIST ,

       SUBSHELL = T_PAREN_OPEN COMPOUND_LIST T_PAREN_CLOSE,

       COPROC = COPROC SHELL_COMMAND ,
       COPROC = COPROC SHELL_COMMAND REDIRECTION_LIST ,
       COPROC = COPROC WORD SHELL_COMMAND ,
       COPROC = COPROC WORD SHELL_COMMAND REDIRECTION_LIST ,
       COPROC = COPROC SIMPLE_COMMAND ,

       IF_COMMAND = IF COMPOUND_LIST THEN COMPOUND_LIST FI ,
       IF_COMMAND = IF COMPOUND_LIST THEN COMPOUND_LIST ELSE COMPOUND_LIST FI ,
       IF_COMMAND = IF COMPOUND_LIST THEN COMPOUND_LIST ELIF_CLAUSE FI ,

       GROUP_COMMAND = T_BR_OPEN COMPOUND_LIST T_BR_CLOSE,

       COND_COMMAND = COND_START COND_CMD COND_END ,

       ELIF_CLAUSE = ELIF COMPOUND_LIST THEN COMPOUND_LIST ,
       ELIF_CLAUSE = ELIF COMPOUND_LIST THEN COMPOUND_LIST ELSE COMPOUND_LIST ,
       ELIF_CLAUSE = ELIF COMPOUND_LIST THEN COMPOUND_LIST ELIF_CLAUSE ,

       IF = T_I T_F,
       THEN = T_T T_H T_E T_N,
       FI = T_F T_I,
       ELIF = T_E T_L T_I T_F,
       ELSE = T_E T_L T_S T_E,
       SELECT = T_S T_E T_L T_E T_C T_T,
       FOR = T_F T_O T_R,
       IN = T_I T_N,
       DO = T_D T_O,
       DONE = T_D T_O T_N T_E,
       WHILE = T_W T_H T_I T_L T_E,
       UNTIL = T_U T_N T_T T_I T_L,
       COPROC = T_C T_O T_P T_R T_O T_C,
       LET = T_L T_E T_T,
       EVAL = T_E T_V T_A T_L,
       TIME = T_T T_I T_M T_E,
       FUNCTION = T_F T_U T_N T_C T_T T_I T_O T_N,
       CASE = T_C T_A T_S T_E,
       ESAC = T_E T_S T_A T_C,

       CASE_CLAUSE = PATTERN_LIST ,
       CASE_CLAUSE = CASE_CLAUSE_SEQUENCE PATTERN_LIST ,

       PATTERN_LIST = GAP PATTERN T_PAREN_CLOSE COMPOUND_LIST ,
       PATTERN_LIST = GAP PATTERN T_PAREN_CLOSE GAP ,
       PATTERN_LIST = GAP T_PAREN_OPEN PATTERN T_PAREN_CLOSE COMPOUND_LIST ,
       PATTERN_LIST = GAP T_PAREN_OPEN PATTERN T_PAREN_CLOSE GAP ,

       CASE_CLAUSE_SEQUENCE = PATTERN_LIST SEMI_SEMI ,
       CASE_CLAUSE_SEQUENCE = CASE_CLAUSE_SEQUENCE PATTERN_LIST SEMI_SEMI ,
       CASE_CLAUSE_SEQUENCE = PATTERN_LIST SEMI_AND ,
       CASE_CLAUSE_SEQUENCE = CASE_CLAUSE_SEQUENCE PATTERN_LIST SEMI_AND ,
       CASE_CLAUSE_SEQUENCE = PATTERN_LIST SEMI_SEMI_AND ,
       CASE_CLAUSE_SEQUENCE = CASE_CLAUSE_SEQUENCE PATTERN_LIST SEMI_SEMI_AND ,

       PATTERN = WORD ,
       PATTERN = WORD T_PIPE PATTERN ,

       LIST = GAP LIST0 ,

       COMPOUND_LIST = LIST ,
       COMPOUND_LIST = GAP LIST1 ,

       LIST0 = LIST1 T_NEWLINE GAP ,
       LIST0 = LIST1 T_AMPERSAND GAP ,
       LIST0 = LIST1 T_SEMICOLON GAP ,

       LIST1 = LIST1 AND_AND GAP LIST1 ,
       LIST1 = LIST1 OR_OR GAP LIST1 ,
       LIST1 = LIST1 T_AMPERSAND GAP LIST1 ,
       LIST1 = LIST1 T_SEMICOLON GAP LIST1 ,
       LIST1 = LIST1 T_NEWLINE GAP LIST1 ,
       LIST1 = PIPELINE_COMMAND ,

       LIST_TERMINATOR = T_NEWLINE,
       LIST_TERMINATOR = T_SEMICOLON,

       PIPELINE_COMMAND = PIPELINE ,
       PIPELINE_COMMAND = BANG PIPELINE_COMMAND ,
       PIPELINE_COMMAND = TIMESPEC PIPELINE_COMMAND ,
       PIPELINE_COMMAND = TIMESPEC LIST_TERMINATOR ,
       PIPELINE_COMMAND = BANG LIST_TERMINATOR ,

       PIPELINE = PIPELINE WS_STAR T_PIPE GAP PIPELINE ,
       PIPELINE = PIPELINE WS_STAR BAR_AND GAP PIPELINE ,
       PIPELINE = COMMAND ,

       TIMESPEC = TIME ,
       TIMESPEC = TIME TIMEOPT ,
       TIMESPEC = TIME TIMEOPT TIMEIGN ,

       NUMBER = T_DIGIT NUMBER ,
       NUMBER = T_DIGIT ,

       IDENTIFIER = IDENTIFIER0 IDENTIFIER1_STAR ,

       IDENTIFIER0 = T_LC_LETTER,
       IDENTIFIER0 = T_UC_LETTER,
       IDENTIFIER0 = T_UNDERSCORE,

       IDENTIFIER1_STAR = IDENTIFIER1 IDENTIFIER1_STAR ,

       IDENTIFIER1 = T_LC_LETTER,
       IDENTIFIER1 = T_UC_LETTER,
       IDENTIFIER1 = T_DIGIT,
       IDENTIFIER1 = T_UNDERSCORE,

       REDIR_WORD = T_BR_OPEN IDENTIFIER T_BR_CLOSE,

       // TODO: Add condition parser
       COND_CMD = WORD ,

       // TODO: Add arithmetic for
       ARITH_FOR_EXPRS = T_PAREN_OPEN T_PAREN_OPEN T_PAREN_CLOSE T_PAREN_CLOSE,

       // TODO: Add arithmetic command
       ARITH_COMMAND =  T_PAREN_OPEN T_PAREN_OPEN T_PAREN_CLOSE T_PAREN_CLOSE,

       // TODO: Add complete expansion parser
       WORD = WORD_LETTER WORD ,
       WORD = WORD_LETTER ,
       WORD_LETTER =  T_PERCENT,
       WORD_LETTER =  T_STAR_COLON,
       WORD_LETTER =  T_QUESTION_Z,
       WORD_LETTER =  T_CARET_UNDERSCORE,
       WORD_LETTER =  T_LC_A_Z,
       WORD_LETTER =  T_TILDE,
       WORD_LETTER =  T_OTHER
   ]
}

/// Build and compile a grammar.
pub fn script() -> DynamicGrammar<char, CharMatcher> {
    let mut grammar = TextGrammar::new();
    grammar.set_start("inputunit".to_string());

    use sesd::char::CharMatcher::*;

    // One section of input to be executed at a time. In contrast to bash, which processes input
    // in chunks of logical expressions (|| and &&), this parser needs to handle:
    // * comments (bash filters them out in the lexer)
    // * last line without newline (bash handles this with EOF)
    // * multiple logicals in one input unit
    // * logicals must not be empty ( i.e. a & & is not allowed)
    //
    // As comments are separated by newlines only, but logicals can be separated by ampersands,
    // semicolons or newlines, and ampersands need to be detectable for the compiler, the inputunit
    // TextRules catch these cases individually.

    // A comment at the last position, no newline
    grammar.add(TextRule::new("inputunit").nt("comment"));
    // A logical at the last position, no newline
    grammar.add(TextRule::new("inputunit").nt("logical"));
    // A logical at the last position, exlicit foreground separator, i.e. semicolon or newline
    grammar.add(
        TextRule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_fg")
            .nt("gap"),
    );
    // A logical at the last position, exlicit background separator. i.e. ampersand
    grammar.add(
        TextRule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_bg")
            .nt("gap"),
    );
    // A comment followed by more input units
    grammar.add(
        TextRule::new("inputunit")
            .nt("comment")
            .nt("newline")
            .nt("inputunit"),
    );
    // A logical with exlicit foreground separator, followed by more input units
    grammar.add(
        TextRule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_fg")
            .nt("inputunit"),
    );
    // A logical with exlicit background separator, followed by more input units
    grammar.add(
        TextRule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_bg")
            .nt("ws*")
            .nt("inputunit"),
    );

    // The separators
    grammar.add(TextRule::new("logical_sep_fg").t(Exact(';')));
    grammar.add(TextRule::new("logical_sep_fg").nt("newline"));
    grammar.add(TextRule::new("logical_sep_bg").t(Exact('&')));
    grammar.add(TextRule::new("newline").t(Exact('\n')));

    // A comment skips the whitespace before its marker, then eats the hash and the rest of the
    // line, but not the newline
    grammar.add(
        TextRule::new("comment")
            .nt("ws*")
            .t(Exact('#'))
            .nt("comment-text"),
    );
    grammar.add(TextRule::new("comment-text"));
    grammar.add(
        TextRule::new("comment-text")
            .t(OtherThan('\n'))
            .nt("comment-text"),
    );

    // Whitespace characters. Does not include newline.
    grammar.add(TextRule::new("wschar").t(Exact(' ')));
    grammar.add(TextRule::new("wschar").t(Exact('\t')));

    // At least one whitespace character
    grammar.add(TextRule::new("ws").nt("wschar"));
    grammar.add(TextRule::new("ws").nt("wschar").nt("ws"));

    // Zero or more whitespace characters
    grammar.add(TextRule::new("ws*"));
    grammar.add(TextRule::new("ws*").nt("wschar").nt("ws*"));

    // gap = white space + newlines
    grammar.add(TextRule::new("gap").nt("ws*"));
    grammar.add(TextRule::new("gap").nt("ws*").t(Exact('\n')));
    grammar.add(TextRule::new("gap").nt("ws*").t(Exact('\n')).nt("gap"));

    // Logical expressions are pipelines separated by && or ||
    grammar.add(TextRule::new("logical").nt("ws*").nt("pipeline_command"));
    grammar.add(
        TextRule::new("logical")
            .nt("ws*")
            .nt("pipeline_command")
            .nt("ws*")
            .nt("AND_AND")
            .nt("gap")
            .nt("logical"),
    );
    grammar.add(
        TextRule::new("logical")
            .nt("ws*")
            .nt("pipeline_command")
            .nt("ws*")
            .nt("OR_OR")
            .nt("gap")
            .nt("logical"),
    );

    grammar.add(TextRule::new("word_list").nt("WORD").nt("word_list"));
    grammar.add(TextRule::new("word_list").nt("WORD"));

    grammar.add(TextRule::new("redirection").t(Exact('>')).nt("WORD"));
    grammar.add(TextRule::new("redirection").t(Exact('<')).nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .t(Exact('>'))
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .t(Exact('<'))
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .t(Exact('>'))
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .t(Exact('<'))
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("GREATER_GREATER").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_GREATER")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_GREATER")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("GREATER_BAR").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_BAR")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_BAR")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_GREATER").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_GREATER")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_GREATER")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_LESS").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_LESS_MINUS").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_LESS_MINUS")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_LESS_MINUS")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_LESS_LESS").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_AND").nt("NUMBER"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_AND")
            .nt("NUMBER"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_AND")
            .nt("NUMBER"),
    );
    grammar.add(TextRule::new("redirection").nt("GREATER_AND").nt("NUMBER"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_AND")
            .nt("NUMBER"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_AND")
            .nt("NUMBER"),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_AND").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_AND")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_AND")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("GREATER_AND").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_AND")
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_AND")
            .nt("WORD"),
    );
    grammar.add(TextRule::new("redirection").nt("GREATER_AND").t(Exact('-')));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_AND")
            .t(Exact('-')),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_AND")
            .t(Exact('-')),
    );
    grammar.add(TextRule::new("redirection").nt("LESS_AND").t(Exact('-')));
    grammar.add(
        TextRule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_AND")
            .t(Exact('-')),
    );
    grammar.add(
        TextRule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_AND")
            .t(Exact('-')),
    );
    grammar.add(TextRule::new("redirection").nt("AND_GREATER").nt("WORD"));
    grammar.add(
        TextRule::new("redirection")
            .nt("AND_GREATER_GREATER")
            .nt("WORD"),
    );

    grammar.add(TextRule::new("BANG").t(Exact('!')));
    grammar.add(TextRule::new("TIMEIGN").ts("--".chars().map(Exact)));
    grammar.add(TextRule::new("TIMEOPT").ts("-p".chars().map(Exact)));
    grammar.add(TextRule::new("AND_AND").ts("&&".chars().map(Exact)));
    grammar.add(TextRule::new("OR_OR").ts("||".chars().map(Exact)));
    grammar.add(TextRule::new("GREATER_GREATER").ts(">>".chars().map(Exact)));
    grammar.add(TextRule::new("LESS_LESS").ts("<<".chars().map(Exact)));
    grammar.add(TextRule::new("LESS_AND").ts("<&".chars().map(Exact)));
    grammar.add(TextRule::new("GREATER_AND").ts(">&".chars().map(Exact)));
    grammar.add(TextRule::new("SEMI_SEMI").ts(";;".chars().map(Exact)));
    grammar.add(TextRule::new("SEMI_AND").ts(";&".chars().map(Exact)));
    grammar.add(TextRule::new("SEMI_SEMI_AND").ts(";;&".chars().map(Exact)));
    grammar.add(TextRule::new("LESS_LESS_MINUS").ts("<<-".chars().map(Exact)));
    grammar.add(TextRule::new("LESS_LESS_LESS").ts("<<<".chars().map(Exact)));
    grammar.add(TextRule::new("AND_GREATER").ts("&>".chars().map(Exact)));
    grammar.add(TextRule::new("AND_GREATER_GREATER").ts("&>>".chars().map(Exact)));
    grammar.add(TextRule::new("LESS_GREATER").ts("<>".chars().map(Exact)));
    grammar.add(TextRule::new("GREATER_BAR").ts(">|".chars().map(Exact)));
    grammar.add(TextRule::new("BAR_AND").ts("|&".chars().map(Exact)));
    grammar.add(TextRule::new("COND_START").ts("[[".chars().map(Exact)));
    grammar.add(TextRule::new("COND_END").ts("]]".chars().map(Exact)));

    grammar.add(TextRule::new("simple_command_element").nt("WORD"));
    grammar.add(TextRule::new("simple_command_element").nt("ASSIGNMENT_WORD"));
    grammar.add(TextRule::new("simple_command_element").nt("redirection"));

    grammar.add(
        TextRule::new("ASSIGNMENT_WORD")
            .nt("WORD")
            .t(Exact('='))
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("ASSIGNMENT_WORD")
            .nt("LET")
            .nt("WORD")
            .t(Exact('='))
            .nt("WORD"),
    );
    grammar.add(
        TextRule::new("ASSIGNMENT_WORD")
            .nt("EVAL")
            .nt("WORD")
            .t(Exact('='))
            .nt("WORD"),
    );

    grammar.add(TextRule::new("redirection_list").nt("redirection"));
    grammar.add(
        TextRule::new("redirection_list")
            .nt("redirection")
            .nt("redirection_list"),
    );

    grammar.add(
        TextRule::new("simple_command")
            .nt("simple_command_element")
            .nt("ws*"),
    );
    grammar.add(
        TextRule::new("simple_command")
            .nt("simple_command_element")
            .nt("ws")
            .nt("simple_command"),
    );

    grammar.add(TextRule::new("command").nt("simple_command"));
    grammar.add(TextRule::new("command").nt("shell_command"));
    grammar.add(
        TextRule::new("command")
            .nt("shell_command")
            .nt("redirection_list"),
    );
    grammar.add(TextRule::new("command").nt("function_def"));
    grammar.add(TextRule::new("command").nt("coproc"));

    grammar.add(TextRule::new("shell_command").nt("for_command"));
    grammar.add(TextRule::new("shell_command").nt("case_command"));
    grammar.add(
        TextRule::new("shell_command")
            .nt("WHILE")
            .nt("compound_list")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("shell_command")
            .nt("UNTIL")
            .nt("compound_list")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(TextRule::new("shell_command").nt("select_command"));
    grammar.add(TextRule::new("shell_command").nt("if_command"));
    grammar.add(TextRule::new("shell_command").nt("subshell"));
    grammar.add(TextRule::new("shell_command").nt("group_command"));
    grammar.add(TextRule::new("shell_command").nt("arith_command"));
    grammar.add(TextRule::new("shell_command").nt("cond_command"));
    grammar.add(TextRule::new("shell_command").nt("arith_for_command"));

    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .t(Exact(';'))
            .nt("gap")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .t(Exact(';'))
            .nt("gap")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("word_list")
            .nt("list_terminator")
            .nt("gap")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("word_list")
            .nt("list_terminator")
            .nt("gap")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("list_terminator")
            .nt("gap")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("list_terminator")
            .nt("gap")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );

    grammar.add(
        TextRule::new("arith_for_command")
            .nt("FOR")
            .nt("ws")
            .nt("ARITH_FOR_EXPRS")
            .nt("list_terminator")
            .nt("gap")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("arith_for_command")
            .nt("FOR")
            .nt("ws")
            .nt("ARITH_FOR_EXPRS")
            .nt("list_terminator")
            .nt("gap")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );
    grammar.add(
        TextRule::new("arith_for_command")
            .nt("FOR")
            .nt("ws")
            .nt("ARITH_FOR_EXPRS")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("arith_for_command")
            .nt("FOR")
            .nt("ws")
            .nt("ARITH_FOR_EXPRS")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );

    grammar.add(
        TextRule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("DO")
            .nt("list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .t(Exact('{'))
            .nt("list")
            .t(Exact('}')),
    );
    grammar.add(
        TextRule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .t(Exact(';'))
            .nt("gap")
            .nt("DO")
            .nt("list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .t(Exact(';'))
            .nt("gap")
            .t(Exact('{'))
            .nt("list")
            .t(Exact('}')),
    );
    grammar.add(
        TextRule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("word_list")
            .nt("list_terminator")
            .nt("gap")
            .nt("DO")
            .nt("list")
            .nt("DONE"),
    );
    grammar.add(
        TextRule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("word_list")
            .nt("list_terminator")
            .nt("gap")
            .t(Exact('{'))
            .nt("list")
            .t(Exact('}')),
    );

    grammar.add(
        TextRule::new("case_command")
            .nt("CASE")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("gap")
            .nt("ESAC"),
    );
    grammar.add(
        TextRule::new("case_command")
            .nt("CASE")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("case_clause_sequence")
            .nt("gap")
            .nt("ESAC"),
    );
    grammar.add(
        TextRule::new("case_command")
            .nt("CASE")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("case_clause")
            .nt("ESAC"),
    );

    grammar.add(
        TextRule::new("function_def")
            .nt("WORD")
            .t(Exact('('))
            .t(Exact(')'))
            .nt("gap")
            .nt("function_body"),
    );
    grammar.add(
        TextRule::new("function_def")
            .nt("FUNCTION")
            .nt("WORD")
            .t(Exact('('))
            .t(Exact(')'))
            .nt("gap")
            .nt("function_body"),
    );
    grammar.add(
        TextRule::new("function_def")
            .nt("FUNCTION")
            .nt("WORD")
            .nt("gap")
            .nt("function_body"),
    );

    grammar.add(TextRule::new("function_body").nt("shell_command"));
    grammar.add(
        TextRule::new("function_body")
            .nt("shell_command")
            .nt("redirection_list"),
    );

    grammar.add(
        TextRule::new("subshell")
            .t(Exact('('))
            .nt("compound_list")
            .t(Exact(')')),
    );

    grammar.add(TextRule::new("coproc").nt("COPROC").nt("shell_command"));
    grammar.add(
        TextRule::new("coproc")
            .nt("COPROC")
            .nt("shell_command")
            .nt("redirection_list"),
    );
    grammar.add(
        TextRule::new("coproc")
            .nt("COPROC")
            .nt("WORD")
            .nt("shell_command"),
    );
    grammar.add(
        TextRule::new("coproc")
            .nt("COPROC")
            .nt("WORD")
            .nt("shell_command")
            .nt("redirection_list"),
    );
    grammar.add(TextRule::new("coproc").nt("COPROC").nt("simple_command"));

    grammar.add(
        TextRule::new("if_command")
            .nt("IF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("FI"),
    );
    grammar.add(
        TextRule::new("if_command")
            .nt("IF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("ELSE")
            .nt("compound_list")
            .nt("FI"),
    );
    grammar.add(
        TextRule::new("if_command")
            .nt("IF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("elif_clause")
            .nt("FI"),
    );

    grammar.add(
        TextRule::new("group_command")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );

    grammar.add(
        TextRule::new("cond_command")
            .nt("COND_START")
            .nt("COND_CMD")
            .nt("COND_END"),
    );

    grammar.add(
        TextRule::new("elif_clause")
            .nt("ELIF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list"),
    );
    grammar.add(
        TextRule::new("elif_clause")
            .nt("ELIF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("ELSE")
            .nt("compound_list"),
    );
    grammar.add(
        TextRule::new("elif_clause")
            .nt("ELIF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("elif_clause"),
    );

    grammar.add(TextRule::new("IF").ts("if".chars().map(Exact)));
    grammar.add(TextRule::new("THEN").ts("then".chars().map(Exact)));
    grammar.add(TextRule::new("FI").ts("fi".chars().map(Exact)));
    grammar.add(TextRule::new("ELIF").ts("elif".chars().map(Exact)));
    grammar.add(TextRule::new("ELSE").ts("else".chars().map(Exact)));
    grammar.add(TextRule::new("SELECT").ts("select".chars().map(Exact)));
    grammar.add(TextRule::new("FOR").ts("for".chars().map(Exact)));
    grammar.add(TextRule::new("IN").ts("in".chars().map(Exact)));
    grammar.add(TextRule::new("DO").ts("do".chars().map(Exact)));
    grammar.add(TextRule::new("DONE").ts("done".chars().map(Exact)));
    grammar.add(TextRule::new("WHILE").ts("while".chars().map(Exact)));
    grammar.add(TextRule::new("UNTIL").ts("until".chars().map(Exact)));
    grammar.add(TextRule::new("COPROC").ts("coproc".chars().map(Exact)));
    grammar.add(TextRule::new("LET").ts("let".chars().map(Exact)));
    grammar.add(TextRule::new("EVAL").ts("eval".chars().map(Exact)));
    grammar.add(TextRule::new("TIME").ts("time".chars().map(Exact)));
    grammar.add(TextRule::new("FUNCTION").ts("function".chars().map(Exact)));
    grammar.add(TextRule::new("CASE").ts("case".chars().map(Exact)));
    grammar.add(TextRule::new("ESAC").ts("esac".chars().map(Exact)));

    grammar.add(TextRule::new("case_clause").nt("pattern_list"));
    grammar.add(
        TextRule::new("case_clause")
            .nt("case_clause_sequence")
            .nt("pattern_list"),
    );

    grammar.add(
        TextRule::new("pattern_list")
            .nt("gap")
            .nt("pattern")
            .t(Exact(')'))
            .nt("compound_list"),
    );
    grammar.add(
        TextRule::new("pattern_list")
            .nt("gap")
            .nt("pattern")
            .t(Exact(')'))
            .nt("gap"),
    );
    grammar.add(
        TextRule::new("pattern_list")
            .nt("gap")
            .t(Exact('('))
            .nt("pattern")
            .t(Exact(')'))
            .nt("compound_list"),
    );
    grammar.add(
        TextRule::new("pattern_list")
            .nt("gap")
            .t(Exact('('))
            .nt("pattern")
            .t(Exact(')'))
            .nt("gap"),
    );

    grammar.add(
        TextRule::new("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI"),
    );
    grammar.add(
        TextRule::new("case_clause_sequence")
            .nt("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI"),
    );
    grammar.add(
        TextRule::new("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_AND"),
    );
    grammar.add(
        TextRule::new("case_clause_sequence")
            .nt("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_AND"),
    );
    grammar.add(
        TextRule::new("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI_AND"),
    );
    grammar.add(
        TextRule::new("case_clause_sequence")
            .nt("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI_AND"),
    );

    grammar.add(TextRule::new("pattern").nt("WORD"));
    grammar.add(TextRule::new("pattern").nt("WORD").t(Exact('|')).nt("pattern"));

    grammar.add(TextRule::new("list").nt("gap").nt("list0"));

    grammar.add(TextRule::new("compound_list").nt("list"));
    grammar.add(TextRule::new("compound_list").nt("gap").nt("list1"));

    grammar.add(TextRule::new("list0").nt("list1").t(Exact('\n')).nt("gap"));
    grammar.add(TextRule::new("list0").nt("list1").t(Exact('&')).nt("gap"));
    grammar.add(TextRule::new("list0").nt("list1").t(Exact(';')).nt("gap"));

    grammar.add(
        TextRule::new("list1")
            .nt("list1")
            .nt("AND_AND")
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        TextRule::new("list1")
            .nt("list1")
            .nt("OR_OR")
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        TextRule::new("list1")
            .nt("list1")
            .t(Exact('&'))
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        TextRule::new("list1")
            .nt("list1")
            .t(Exact(';'))
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        TextRule::new("list1")
            .nt("list1")
            .t(Exact('\n'))
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(TextRule::new("list1").nt("pipeline_command"));

    grammar.add(TextRule::new("list_terminator").t(Exact('\n')));
    grammar.add(TextRule::new("list_terminator").t(Exact(';')));

    grammar.add(TextRule::new("pipeline_command").nt("pipeline"));
    grammar.add(
        TextRule::new("pipeline_command")
            .nt("BANG")
            .nt("pipeline_command"),
    );
    grammar.add(
        TextRule::new("pipeline_command")
            .nt("timespec")
            .nt("pipeline_command"),
    );
    grammar.add(
        TextRule::new("pipeline_command")
            .nt("timespec")
            .nt("list_terminator"),
    );
    grammar.add(
        TextRule::new("pipeline_command")
            .nt("BANG")
            .nt("list_terminator"),
    );

    grammar.add(
        TextRule::new("pipeline")
            .nt("pipeline")
            .nt("ws*")
            .t(Exact('|'))
            .nt("gap")
            .nt("pipeline"),
    );
    grammar.add(
        TextRule::new("pipeline")
            .nt("pipeline")
            .nt("ws*")
            .nt("BAR_AND")
            .nt("gap")
            .nt("pipeline"),
    );
    grammar.add(TextRule::new("pipeline").nt("command"));

    grammar.add(TextRule::new("timespec").nt("TIME"));
    grammar.add(TextRule::new("timespec").nt("TIME").nt("TIMEOPT"));
    grammar.add(TextRule::new("timespec").nt("TIME").nt("TIMEOPT").nt("TIMEIGN"));

    grammar.add(TextRule::new("NUMBER").nt("DIGIT").nt("NUMBER"));
    grammar.add(TextRule::new("NUMBER").nt("DIGIT"));
    grammar.add(TextRule::new("DIGIT").t(Range('0', '9')));

    grammar.add(TextRule::new("IDENTIFIER").nt("IDENTIFIER0").nt("IDENTIFIER1*"));

    grammar.add(TextRule::new("IDENTIFIER0").t(Range('a', 'z')));
    grammar.add(TextRule::new("IDENTIFIER0").t(Range('A', 'Z')));
    grammar.add(TextRule::new("IDENTIFIER0").t(Exact('_')));

    grammar.add(TextRule::new("IDENTIFIER1*"));
    grammar.add(
        TextRule::new("IDENTIFIER1*")
            .nt("IDENTIFIER1")
            .nt("IDENTIFIER1*"),
    );

    grammar.add(TextRule::new("IDENTIFIER1").t(Range('a', 'z')));
    grammar.add(TextRule::new("IDENTIFIER1").t(Range('A', 'Z')));
    grammar.add(TextRule::new("IDENTIFIER1").t(Range('0', '9')));
    grammar.add(TextRule::new("IDENTIFIER1").t(Exact('_')));

    grammar.add(
        TextRule::new("REDIR_WORD")
            .t(Exact('{'))
            .nt("IDENTIFIER")
            .t(Exact('}')),
    );

    // TODO: Add condition parser
    grammar.add(TextRule::new("COND_CMD"));

    // TODO: Add arithmetic for
    grammar.add(
        TextRule::new("ARITH_FOR_EXPRS")
            .ts("((".chars().map(Exact))
            .ts("))".chars().map(Exact)),
    );

    // TODO: Add arithmetic command
    grammar.add(
        TextRule::new("arith_command")
            .ts("((".chars().map(Exact))
            .ts("))".chars().map(Exact)),
    );

    // TODO: Add complete expansion parser
    grammar.add(TextRule::new("WORD").nt("WORD_LETTER").nt("WORD"));
    grammar.add(TextRule::new("WORD").nt("WORD_LETTER"));

    grammar.add(TextRule::new("WORD_LETTER").t(Exact('\x25')));
    grammar.add(TextRule::new("WORD_LETTER").t(Range('\x2a','\x3a')));
    grammar.add(TextRule::new("WORD_LETTER").t(Range('\x3f','\x5a')));
    grammar.add(TextRule::new("WORD_LETTER").t(Range('\x5e','\x5f')));
    grammar.add(TextRule::new("WORD_LETTER").t(Range('\x61','\x7a')));
    grammar.add(TextRule::new("WORD_LETTER").t(Exact('\x7e')));
    grammar.add(TextRule::new("WORD_LETTER").t(Range('\u{80}','\u{10ffff}')));

    let res = grammar.compile();
    if let Err(ref e) = res {
        debug!("Compile SESD grammar for bash script: {:?}", e);
    }
    let res = res.expect("compiling bash script grammar should not fail");

    res.debug_tables();
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use sesd::{Verdict};

    /// Test helper to parse a string that should not fail and be accepted at the last character
    fn ok(parser: &mut Parser, input: &str) {
        let mut chars_iter = input.chars().enumerate();
        let mut last = chars_iter.next();

        loop {
            let this = chars_iter.next();
            if this.is_none() {
                break;
            }
            assert!(last.is_some());
            let last_tuple = last.unwrap();
            let res = parser.update(last_tuple.0, last_tuple.1);
            assert!(
                res == Verdict::Accept || res == Verdict::More,
                parser.print_chart()
            );
            last = this;
        }
        let last_tuple = last.unwrap();
        let res = parser.update(last_tuple.0, last_tuple.1);
        assert!(res == Verdict::Accept, parser.print_chart());
    }

    /// Test sections of comments
    #[test]
    fn comment() {
        let mut parser = Parser::new(script());

        // Comment without newline
        ok(&mut parser, "# comment");
        ok(&mut parser, " # comment");
        ok(&mut parser, "\t# comment");
        ok(&mut parser, "  # comment");

        // Comment with newline
        ok(&mut parser, "# comment\n");
        ok(&mut parser, "# comment\n # Another");
        ok(&mut parser, "# comment\n # Another\n");
    }

    /// Test logicals
    #[test]
    fn logical() {
        let mut parser = Parser::new(script());

        // Various stages of input, single command
        ok(&mut parser, "ls");
        ok(&mut parser, " ls");
        ok(&mut parser, "ls ");
        ok(&mut parser, "ls -al");

        // Multiple lines
        ok(&mut parser, "ls -al\nxxx yyy");
        ok(&mut parser, "ls -al\nxxx yyy\n");

        // Logical expression
        ok(&mut parser, "a&&b");
        ok(&mut parser, "a||b");
        ok(&mut parser, "a &&b");
        ok(&mut parser, "a&& b");
        ok(&mut parser, "a && b && c");
        ok(&mut parser, "a && b &&\nc");
        ok(&mut parser, "a && b && c\n");
        ok(&mut parser, "a && b || c\n");
        ok(&mut parser, "a || b && c\n");
    }
}
