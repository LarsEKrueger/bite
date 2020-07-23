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

use sesd::{char::CharMatcher, CompiledGrammar, Grammar, Rule};

/// Build and compile a grammar.
pub fn script() -> CompiledGrammar<char, CharMatcher> {
    let mut grammar = Grammar::new();
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
    // rules catch these cases individually.

    // A comment at the last position, no newline
    grammar.add(Rule::new("inputunit").nt("comment"));
    // A logical at the last position, no newline
    grammar.add(Rule::new("inputunit").nt("logical"));
    // A logical at the last position, exlicit foreground separator, i.e. semicolon or newline
    grammar.add(
        Rule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_fg")
            .nt("gap"),
    );
    // A logical at the last position, exlicit background separator. i.e. ampersand
    grammar.add(
        Rule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_bg")
            .nt("gap"),
    );
    // A comment followed by more input units
    grammar.add(
        Rule::new("inputunit")
            .nt("comment")
            .nt("newline")
            .nt("inputunit"),
    );
    // A logical with exlicit foreground separator, followed by more input units
    grammar.add(
        Rule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_fg")
            .nt("inputunit"),
    );
    // A logical with exlicit background separator, followed by more input units
    grammar.add(
        Rule::new("inputunit")
            .nt("logical")
            .nt("ws*")
            .nt("logical_sep_bg")
            .nt("ws*")
            .nt("inputunit"),
    );

    // The separators
    grammar.add(Rule::new("logical_sep_fg").t(Exact(';')));
    grammar.add(Rule::new("logical_sep_fg").nt("newline"));
    grammar.add(Rule::new("logical_sep_bg").t(Exact('&')));
    grammar.add(Rule::new("newline").t(Exact('\n')));

    // A comment skips the whitespace before its marker, then eats the hash and the rest of the
    // line, but not the newline
    grammar.add(
        Rule::new("comment")
            .nt("ws*")
            .t(Exact('#'))
            .nt("comment-text"),
    );
    grammar.add(Rule::new("comment-text"));
    grammar.add(
        Rule::new("comment-text")
            .t(NoneOf(vec!['\n']))
            .nt("comment-text"),
    );

    // Whitespace characters. Does not include newline.
    grammar.add(Rule::new("wschar").t(Exact(' ')));
    grammar.add(Rule::new("wschar").t(Exact('\t')));

    // At least one whitespace character
    grammar.add(Rule::new("ws").nt("wschar"));
    grammar.add(Rule::new("ws").nt("wschar").nt("ws"));

    // Zero or more whitespace characters
    grammar.add(Rule::new("ws*"));
    grammar.add(Rule::new("ws*").nt("wschar").nt("ws*"));

    // gap = white space + newlines
    grammar.add(Rule::new("gap").nt("ws*"));
    grammar.add(Rule::new("gap").nt("ws*").t(Exact('\n')));
    grammar.add(Rule::new("gap").nt("ws*").t(Exact('\n')).nt("gap"));

    // Logical expressions are pipelines separated by && or ||
    grammar.add(Rule::new("logical").nt("ws*").nt("pipeline_command"));
    grammar.add(
        Rule::new("logical")
            .nt("ws*")
            .nt("pipeline_command")
            .nt("ws*")
            .nt("AND_AND")
            .nt("gap")
            .nt("logical"),
    );
    grammar.add(
        Rule::new("logical")
            .nt("ws*")
            .nt("pipeline_command")
            .nt("ws*")
            .nt("OR_OR")
            .nt("gap")
            .nt("logical"),
    );

    grammar.add(Rule::new("word_list").nt("WORD").nt("word_list"));
    grammar.add(Rule::new("word_list").nt("WORD"));

    grammar.add(Rule::new("redirection").t(Exact('>')).nt("WORD"));
    grammar.add(Rule::new("redirection").t(Exact('<')).nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .t(Exact('>'))
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .t(Exact('<'))
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .t(Exact('>'))
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .t(Exact('<'))
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("GREATER_GREATER").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_GREATER")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_GREATER")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("GREATER_BAR").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_BAR")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_BAR")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("LESS_GREATER").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_GREATER")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_GREATER")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("LESS_LESS").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("LESS_LESS_MINUS").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_LESS_MINUS")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_LESS_MINUS")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("LESS_LESS_LESS").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_LESS_LESS")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("LESS_AND").nt("NUMBER"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_AND")
            .nt("NUMBER"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_AND")
            .nt("NUMBER"),
    );
    grammar.add(Rule::new("redirection").nt("GREATER_AND").nt("NUMBER"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_AND")
            .nt("NUMBER"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_AND")
            .nt("NUMBER"),
    );
    grammar.add(Rule::new("redirection").nt("LESS_AND").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_AND")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_AND")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("GREATER_AND").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_AND")
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_AND")
            .nt("WORD"),
    );
    grammar.add(Rule::new("redirection").nt("GREATER_AND").t(Exact('-')));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("GREATER_AND")
            .t(Exact('-')),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("GREATER_AND")
            .t(Exact('-')),
    );
    grammar.add(Rule::new("redirection").nt("LESS_AND").t(Exact('-')));
    grammar.add(
        Rule::new("redirection")
            .nt("NUMBER")
            .nt("LESS_AND")
            .t(Exact('-')),
    );
    grammar.add(
        Rule::new("redirection")
            .nt("REDIR_WORD")
            .nt("LESS_AND")
            .t(Exact('-')),
    );
    grammar.add(Rule::new("redirection").nt("AND_GREATER").nt("WORD"));
    grammar.add(
        Rule::new("redirection")
            .nt("AND_GREATER_GREATER")
            .nt("WORD"),
    );

    grammar.add(Rule::new("BANG").t(Exact('!')));
    grammar.add(Rule::new("TIMEIGN").ts("--".chars().map(Exact)));
    grammar.add(Rule::new("TIMEOPT").ts("-p".chars().map(Exact)));
    grammar.add(Rule::new("AND_AND").ts("&&".chars().map(Exact)));
    grammar.add(Rule::new("OR_OR").ts("||".chars().map(Exact)));
    grammar.add(Rule::new("GREATER_GREATER").ts(">>".chars().map(Exact)));
    grammar.add(Rule::new("LESS_LESS").ts("<<".chars().map(Exact)));
    grammar.add(Rule::new("LESS_AND").ts("<&".chars().map(Exact)));
    grammar.add(Rule::new("GREATER_AND").ts(">&".chars().map(Exact)));
    grammar.add(Rule::new("SEMI_SEMI").ts(";;".chars().map(Exact)));
    grammar.add(Rule::new("SEMI_AND").ts(";&".chars().map(Exact)));
    grammar.add(Rule::new("SEMI_SEMI_AND").ts(";;&".chars().map(Exact)));
    grammar.add(Rule::new("LESS_LESS_MINUS").ts("<<-".chars().map(Exact)));
    grammar.add(Rule::new("LESS_LESS_LESS").ts("<<<".chars().map(Exact)));
    grammar.add(Rule::new("AND_GREATER").ts("&>".chars().map(Exact)));
    grammar.add(Rule::new("AND_GREATER_GREATER").ts("&>>".chars().map(Exact)));
    grammar.add(Rule::new("LESS_GREATER").ts("<>".chars().map(Exact)));
    grammar.add(Rule::new("GREATER_BAR").ts(">|".chars().map(Exact)));
    grammar.add(Rule::new("BAR_AND").ts("|&".chars().map(Exact)));
    grammar.add(Rule::new("COND_START").ts("[[".chars().map(Exact)));
    grammar.add(Rule::new("COND_END").ts("]]".chars().map(Exact)));

    grammar.add(Rule::new("simple_command_element").nt("WORD"));
    grammar.add(Rule::new("simple_command_element").nt("ASSIGNMENT_WORD"));
    grammar.add(Rule::new("simple_command_element").nt("redirection"));

    grammar.add(
        Rule::new("ASSIGNMENT_WORD")
            .nt("WORD")
            .t(Exact('='))
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("ASSIGNMENT_WORD")
            .nt("LET")
            .nt("WORD")
            .t(Exact('='))
            .nt("WORD"),
    );
    grammar.add(
        Rule::new("ASSIGNMENT_WORD")
            .nt("EVAL")
            .nt("WORD")
            .t(Exact('='))
            .nt("WORD"),
    );

    grammar.add(Rule::new("redirection_list").nt("redirection"));
    grammar.add(
        Rule::new("redirection_list")
            .nt("redirection")
            .nt("redirection_list"),
    );

    grammar.add(
        Rule::new("simple_command")
            .nt("simple_command_element")
            .nt("ws*"),
    );
    grammar.add(
        Rule::new("simple_command")
            .nt("simple_command_element")
            .nt("ws")
            .nt("simple_command"),
    );

    grammar.add(Rule::new("command").nt("simple_command"));
    grammar.add(Rule::new("command").nt("shell_command"));
    grammar.add(
        Rule::new("command")
            .nt("shell_command")
            .nt("redirection_list"),
    );
    grammar.add(Rule::new("command").nt("function_def"));
    grammar.add(Rule::new("command").nt("coproc"));

    grammar.add(Rule::new("shell_command").nt("for_command"));
    grammar.add(Rule::new("shell_command").nt("case_command"));
    grammar.add(
        Rule::new("shell_command")
            .nt("WHILE")
            .nt("compound_list")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        Rule::new("shell_command")
            .nt("UNTIL")
            .nt("compound_list")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(Rule::new("shell_command").nt("select_command"));
    grammar.add(Rule::new("shell_command").nt("if_command"));
    grammar.add(Rule::new("shell_command").nt("subshell"));
    grammar.add(Rule::new("shell_command").nt("group_command"));
    grammar.add(Rule::new("shell_command").nt("arith_command"));
    grammar.add(Rule::new("shell_command").nt("cond_command"));
    grammar.add(Rule::new("shell_command").nt("arith_for_command"));

    grammar.add(
        Rule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        Rule::new("for_command")
            .nt("FOR")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );
    grammar.add(
        Rule::new("for_command")
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
        Rule::new("for_command")
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
        Rule::new("for_command")
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
        Rule::new("for_command")
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
        Rule::new("for_command")
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
        Rule::new("for_command")
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
        Rule::new("arith_for_command")
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
        Rule::new("arith_for_command")
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
        Rule::new("arith_for_command")
            .nt("FOR")
            .nt("ws")
            .nt("ARITH_FOR_EXPRS")
            .nt("DO")
            .nt("compound_list")
            .nt("DONE"),
    );
    grammar.add(
        Rule::new("arith_for_command")
            .nt("FOR")
            .nt("ws")
            .nt("ARITH_FOR_EXPRS")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );

    grammar.add(
        Rule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("DO")
            .nt("list")
            .nt("DONE"),
    );
    grammar.add(
        Rule::new("select_command")
            .nt("SELECT")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .t(Exact('{'))
            .nt("list")
            .t(Exact('}')),
    );
    grammar.add(
        Rule::new("select_command")
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
        Rule::new("select_command")
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
        Rule::new("select_command")
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
        Rule::new("select_command")
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
        Rule::new("case_command")
            .nt("CASE")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("gap")
            .nt("ESAC"),
    );
    grammar.add(
        Rule::new("case_command")
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
        Rule::new("case_command")
            .nt("CASE")
            .nt("ws")
            .nt("WORD")
            .nt("gap")
            .nt("IN")
            .nt("case_clause")
            .nt("ESAC"),
    );

    grammar.add(
        Rule::new("function_def")
            .nt("WORD")
            .t(Exact('('))
            .t(Exact(')'))
            .nt("gap")
            .nt("function_body"),
    );
    grammar.add(
        Rule::new("function_def")
            .nt("FUNCTION")
            .nt("WORD")
            .t(Exact('('))
            .t(Exact(')'))
            .nt("gap")
            .nt("function_body"),
    );
    grammar.add(
        Rule::new("function_def")
            .nt("FUNCTION")
            .nt("WORD")
            .nt("gap")
            .nt("function_body"),
    );

    grammar.add(Rule::new("function_body").nt("shell_command"));
    grammar.add(
        Rule::new("function_body")
            .nt("shell_command")
            .nt("redirection_list"),
    );

    grammar.add(
        Rule::new("subshell")
            .t(Exact('('))
            .nt("compound_list")
            .t(Exact(')')),
    );

    grammar.add(Rule::new("coproc").nt("COPROC").nt("shell_command"));
    grammar.add(
        Rule::new("coproc")
            .nt("COPROC")
            .nt("shell_command")
            .nt("redirection_list"),
    );
    grammar.add(
        Rule::new("coproc")
            .nt("COPROC")
            .nt("WORD")
            .nt("shell_command"),
    );
    grammar.add(
        Rule::new("coproc")
            .nt("COPROC")
            .nt("WORD")
            .nt("shell_command")
            .nt("redirection_list"),
    );
    grammar.add(Rule::new("coproc").nt("COPROC").nt("simple_command"));

    grammar.add(
        Rule::new("if_command")
            .nt("IF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("FI"),
    );
    grammar.add(
        Rule::new("if_command")
            .nt("IF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("ELSE")
            .nt("compound_list")
            .nt("FI"),
    );
    grammar.add(
        Rule::new("if_command")
            .nt("IF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("elif_clause")
            .nt("FI"),
    );

    grammar.add(
        Rule::new("group_command")
            .t(Exact('{'))
            .nt("compound_list")
            .t(Exact('}')),
    );

    grammar.add(
        Rule::new("cond_command")
            .nt("COND_START")
            .nt("COND_CMD")
            .nt("COND_END"),
    );

    grammar.add(
        Rule::new("elif_clause")
            .nt("ELIF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list"),
    );
    grammar.add(
        Rule::new("elif_clause")
            .nt("ELIF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("ELSE")
            .nt("compound_list"),
    );
    grammar.add(
        Rule::new("elif_clause")
            .nt("ELIF")
            .nt("compound_list")
            .nt("THEN")
            .nt("compound_list")
            .nt("elif_clause"),
    );

    grammar.add(Rule::new("IF").ts("if".chars().map(Exact)));
    grammar.add(Rule::new("THEN").ts("then".chars().map(Exact)));
    grammar.add(Rule::new("FI").ts("fi".chars().map(Exact)));
    grammar.add(Rule::new("ELIF").ts("elif".chars().map(Exact)));
    grammar.add(Rule::new("ELSE").ts("else".chars().map(Exact)));
    grammar.add(Rule::new("SELECT").ts("select".chars().map(Exact)));
    grammar.add(Rule::new("FOR").ts("for".chars().map(Exact)));
    grammar.add(Rule::new("IN").ts("in".chars().map(Exact)));
    grammar.add(Rule::new("DO").ts("do".chars().map(Exact)));
    grammar.add(Rule::new("DONE").ts("done".chars().map(Exact)));
    grammar.add(Rule::new("WHILE").ts("while".chars().map(Exact)));
    grammar.add(Rule::new("UNTIL").ts("until".chars().map(Exact)));
    grammar.add(Rule::new("COPROC").ts("coproc".chars().map(Exact)));
    grammar.add(Rule::new("LET").ts("let".chars().map(Exact)));
    grammar.add(Rule::new("EVAL").ts("eval".chars().map(Exact)));
    grammar.add(Rule::new("TIME").ts("time".chars().map(Exact)));
    grammar.add(Rule::new("FUNCTION").ts("function".chars().map(Exact)));
    grammar.add(Rule::new("CASE").ts("case".chars().map(Exact)));
    grammar.add(Rule::new("ESAC").ts("esac".chars().map(Exact)));

    grammar.add(Rule::new("case_clause").nt("pattern_list"));
    grammar.add(
        Rule::new("case_clause")
            .nt("case_clause_sequence")
            .nt("pattern_list"),
    );

    grammar.add(
        Rule::new("pattern_list")
            .nt("gap")
            .nt("pattern")
            .t(Exact(')'))
            .nt("compound_list"),
    );
    grammar.add(
        Rule::new("pattern_list")
            .nt("gap")
            .nt("pattern")
            .t(Exact(')'))
            .nt("gap"),
    );
    grammar.add(
        Rule::new("pattern_list")
            .nt("gap")
            .t(Exact('('))
            .nt("pattern")
            .t(Exact(')'))
            .nt("compound_list"),
    );
    grammar.add(
        Rule::new("pattern_list")
            .nt("gap")
            .t(Exact('('))
            .nt("pattern")
            .t(Exact(')'))
            .nt("gap"),
    );

    grammar.add(
        Rule::new("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI"),
    );
    grammar.add(
        Rule::new("case_clause_sequence")
            .nt("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI"),
    );
    grammar.add(
        Rule::new("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_AND"),
    );
    grammar.add(
        Rule::new("case_clause_sequence")
            .nt("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_AND"),
    );
    grammar.add(
        Rule::new("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI_AND"),
    );
    grammar.add(
        Rule::new("case_clause_sequence")
            .nt("case_clause_sequence")
            .nt("pattern_list")
            .nt("SEMI_SEMI_AND"),
    );

    grammar.add(Rule::new("pattern").nt("WORD"));
    grammar.add(Rule::new("pattern").nt("WORD").t(Exact('|')).nt("pattern"));

    grammar.add(Rule::new("list").nt("gap").nt("list0"));

    grammar.add(Rule::new("compound_list").nt("list"));
    grammar.add(Rule::new("compound_list").nt("gap").nt("list1"));

    grammar.add(Rule::new("list0").nt("list1").t(Exact('\n')).nt("gap"));
    grammar.add(Rule::new("list0").nt("list1").t(Exact('&')).nt("gap"));
    grammar.add(Rule::new("list0").nt("list1").t(Exact(';')).nt("gap"));

    grammar.add(
        Rule::new("list1")
            .nt("list1")
            .nt("AND_AND")
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        Rule::new("list1")
            .nt("list1")
            .nt("OR_OR")
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        Rule::new("list1")
            .nt("list1")
            .t(Exact('&'))
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        Rule::new("list1")
            .nt("list1")
            .t(Exact(';'))
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(
        Rule::new("list1")
            .nt("list1")
            .t(Exact('\n'))
            .nt("gap")
            .nt("list1"),
    );
    grammar.add(Rule::new("list1").nt("pipeline_command"));

    grammar.add(Rule::new("list_terminator").t(Exact('\n')));
    grammar.add(Rule::new("list_terminator").t(Exact(';')));

    grammar.add(Rule::new("pipeline_command").nt("pipeline"));
    grammar.add(
        Rule::new("pipeline_command")
            .nt("BANG")
            .nt("pipeline_command"),
    );
    grammar.add(
        Rule::new("pipeline_command")
            .nt("timespec")
            .nt("pipeline_command"),
    );
    grammar.add(
        Rule::new("pipeline_command")
            .nt("timespec")
            .nt("list_terminator"),
    );
    grammar.add(
        Rule::new("pipeline_command")
            .nt("BANG")
            .nt("list_terminator"),
    );

    grammar.add(
        Rule::new("pipeline")
            .nt("pipeline")
            .nt("ws*")
            .t(Exact('|'))
            .nt("gap")
            .nt("pipeline"),
    );
    grammar.add(
        Rule::new("pipeline")
            .nt("pipeline")
            .nt("ws*")
            .nt("BAR_AND")
            .nt("gap")
            .nt("pipeline"),
    );
    grammar.add(Rule::new("pipeline").nt("command"));

    grammar.add(Rule::new("timespec").nt("TIME"));
    grammar.add(Rule::new("timespec").nt("TIME").nt("TIMEOPT"));
    grammar.add(Rule::new("timespec").nt("TIME").nt("TIMEOPT").nt("TIMEIGN"));

    grammar.add(Rule::new("NUMBER").nt("DIGIT").nt("NUMBER"));
    grammar.add(Rule::new("NUMBER").nt("DIGIT"));
    grammar.add(Rule::new("DIGIT").t(Range('0', '9')));

    grammar.add(Rule::new("IDENTIFIER").nt("IDENTIFIER0").nt("IDENTIFIER1*"));

    grammar.add(Rule::new("IDENTIFIER0").t(Range('a', 'z')));
    grammar.add(Rule::new("IDENTIFIER0").t(Range('A', 'Z')));
    grammar.add(Rule::new("IDENTIFIER0").t(Exact('_')));

    grammar.add(Rule::new("IDENTIFIER1*"));
    grammar.add(
        Rule::new("IDENTIFIER1*")
            .nt("IDENTIFIER1")
            .nt("IDENTIFIER1*"),
    );

    grammar.add(Rule::new("IDENTIFIER1").t(Range('a', 'z')));
    grammar.add(Rule::new("IDENTIFIER1").t(Range('A', 'Z')));
    grammar.add(Rule::new("IDENTIFIER1").t(Range('0', '9')));
    grammar.add(Rule::new("IDENTIFIER1").t(Exact('_')));

    grammar.add(
        Rule::new("REDIR_WORD")
            .t(Exact('{'))
            .nt("IDENTIFIER")
            .t(Exact('}')),
    );

    // TODO: Add condition parser
    grammar.add(Rule::new("COND_CMD"));

    // TODO: Add arithmetic for
    grammar.add(
        Rule::new("ARITH_FOR_EXPRS")
            .ts("((".chars().map(Exact))
            .ts("))".chars().map(Exact)),
    );

    // TODO: Add arithmetic command
    grammar.add(
        Rule::new("arith_command")
            .ts("((".chars().map(Exact))
            .ts("))".chars().map(Exact)),
    );

    // TODO: Add complete expansion parser
    grammar.add(Rule::new("WORD").nt("WORD_LETTER").nt("WORD"));
    grammar.add(Rule::new("WORD").nt("WORD_LETTER"));
    grammar.add(Rule::new("WORD_LETTER").t(NoneOf(" \n\t\"\'|&;()<>=#".chars().collect())));

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
    use sesd::{char::CharMatcher, Parser, Verdict};

    /// Test helper to parse a string that should not fail and be accepted at the last character
    fn ok(parser: &mut Parser<char, CharMatcher>, input: &str) {
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
        let mut parser = Parser::<char, CharMatcher>::new(script());

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
        let mut parser = Parser::<char, CharMatcher>::new(script());

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
