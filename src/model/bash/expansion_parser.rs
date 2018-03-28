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

//! Convert words to expandable components.

use super::script_parser::identifier;
use super::super::types::*;
use nom::{Err, IResult, ErrorKind};

named!(pub expansion<Expansion>,
       terminated!(
           alt_complete!(
               exp_tilde_first |
               exp_assignment |
               fold_many1!(exp_span,Expansion::new(),
               |mut v:Expansion,mut is:Expansion| {
                   for i in is.drain(..) { v.push(i)} v })
               ),
           eof!())
      );

named!(exp_span<Expansion>,
       alt!(
           exp_single_quote |
           exp_double_quote |
           exp_variable |
           exp_variable_br |
           exp_bracket |
           exp_glob |
           exp_other
           )
);

named!(exp_other<Expansion>,
       map!(many1!(none_of!("${*?'\"")),
       |s| vec![ExpSpan::Verbatim(s.iter().collect())])
);

named!(exp_other_with_sq<Expansion>,
       map!(many1!(none_of!("${*?\"")),
       |s| vec![ExpSpan::Verbatim(s.iter().collect())])
);

named!(exp_single_quote<Expansion>,
       map!(delimited!(
           tag!("'"),
           expsq_inner,
           tag!("'")
           ),
           |s| vec![ExpSpan::Verbatim(s)])
       );

named!(expsq_inner<String>,
       map!(
           escaped!(none_of!("\\'"),'\\',one_of!("'\\")),
           |s| String::from_utf8_lossy(s).into_owned().chars().filter(|c| *c != '\\').collect()
           )
       );

named!(exp_double_quote<Expansion>,
       delimited!(
           tag!("\""),
           fold_many0!(expdq_inner,Expansion::new(),
           |mut v:Expansion,mut is:Expansion| {
               for i in is.drain(..) { v.push(i)} v }),
               tag!("\"")
               )
      );

named!(expdq_inner<Expansion>,
       alt!(
           exp_variable |
           exp_variable_br |
           exp_bracket |
           exp_glob |
           exp_other
           )
      );

named!(exp_variable<Expansion>,
       do_parse!(
           tag!("$") >>
           id : identifier >>
           (vec![ExpSpan::Variable(String::from_utf8_lossy(id).into_owned())])
                )
);

named!(exp_variable_br<Expansion>,
       do_parse!(
           tag!("${") >>
           id : identifier >>
           tag!("}") >>
           (vec![ExpSpan::Variable(String::from_utf8_lossy(id).into_owned())])
                )
);

named!(exp_tilde_first<Expansion>,
       do_parse!(
           tag!("~") >>
           spans : fold_many0!(exptil_span,vec![ExpSpan::Tilde],
                               |mut v:Expansion,mut is:Expansion| {
                                   for i in is.drain(..) { v.push(i) } v})
           >>
           ( spans )
           )
);

named!(exptil_span<Expansion>,
       alt_complete!(
           exp_single_quote |
           exp_double_quote |
           exp_variable |
           exp_variable_br |
           exp_bracket |
           exp_glob |
           exp_other
           )
       );

named!(exp_assignment<Expansion>,
       do_parse!(
           id : identifier >>
           tag!("=") >>
           spans : map!(expass_rhs,
                        |mut is:Expansion| {
                            let mut v = vec![
                                ExpSpan::Verbatim(String::from_utf8_lossy(id).into_owned()),
                                ExpSpan::Verbatim(String::from("=")),
                            ];
                            for i in is.drain(..) { v.push(i) }
                            v})
           >> eof!()
           >> ( spans )
           )
       );

named!(expass_rhs<Expansion>,
  alt_complete!(expass_tilde |
       expass_notilde)
       );

named!(expass_tilde<Expansion>,
       do_parse!(
           tag!("~") >>
           spans : fold_many0!(expat_span,vec![ExpSpan::Tilde],
                               |mut v:Expansion,mut is:Expansion| {
                                   for i in is.drain(..) { v.push(i) } v})
           >>
           ( spans )
           )
      );

fn expat_span(input: &[u8]) -> IResult<&[u8], Expansion> {
    match exp_single_quote(input) {
        IResult::Done(_, _) => IResult::Error(Err::Code(ErrorKind::IsNot)),
        _ => expat_span_ok(input),
    }
}

named!(expat_span_ok<Expansion>,
       alt_complete!(
           exp_double_quote |
           exp_variable |
           exp_variable_br |
           exp_bracket |
           exp_glob |
           exp_other_with_sq
           )
      );

named!(expass_notilde<Expansion>,
       fold_many0!(exp_span,vec![],
                   |mut v:Expansion,mut is:Expansion| {
                       for i in is.drain(..) { v.push(i) } v})
      );

named!(exp_bracket<Expansion>,
       map!(
           delimited!(
               tag!("{"),
               expbr_inner,
               tag!("}")
               ),
               |s| vec![ExpSpan::Bracket(s)]
           )
      );

named!(expbr_inner<Vec<String>>,
       separated_list!(tag!(","),expbr_span)
      );

named!(expbr_span<String>,
       map!(many1!(none_of!(",}")), |s| s.into_iter().collect())
      );

named!(exp_glob<Expansion>,
       map!(recognize!(many1!(one_of!("*?"))),
       |s| vec![ExpSpan::Glob(String::from_utf8_lossy(s).into_owned())])
       );

#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(expansion(b"STUFF"), IResult::Done(&b""[..], vec![ExpSpan::Verbatim(String::from("STUFF"))]));
        assert_eq!(expansion(b"$STUFF"), IResult::Done(&b""[..], vec![ExpSpan::Variable(String::from("STUFF"))]));
        assert_eq!(expansion(b"${STUFF}"), IResult::Done(&b""[..], vec![ExpSpan::Variable(String::from("STUFF"))]));

        assert_eq!(
            expansion(b"{STU,FF}"), 
            IResult::Done(
                &b""[..], 
                vec![ ExpSpan::Bracket(vec![String::from("STU"),String::from("FF")])
                ]));

        assert_eq!( expansion(b"*"), IResult::Done( &b""[..], vec![ ExpSpan::Glob(String::from("*"))]) );
        assert_eq!( expansion(b"???"), IResult::Done( &b""[..], vec![ ExpSpan::Glob(String::from("???"))]) );
        assert_eq!( expansion(b"*?"), IResult::Done( &b""[..], vec![ ExpSpan::Glob(String::from("*?"))]) );

        assert_eq!( expsq_inner(b"l"), IResult::Done( &b""[..], String::from("l")) );
        assert_eq!( expsq_inner(b"l\\'x"), IResult::Done( &b""[..], String::from("l'x")) );
        assert_eq!( exp_single_quote(b"'x'"), IResult::Done( &b""[..], vec![ExpSpan::Verbatim(String::from("x"))]));

        assert_eq!(
            expansion(b"a'l'"),
            IResult::Done(
                &b""[..],
                vec![
                    ExpSpan::Verbatim(String::from("a")),
                    ExpSpan::Verbatim(String::from("l"))
                ]));

        assert_eq!(
            expansion(b"a'l\"'"),
            IResult::Done(
                &b""[..],
                vec![
                    ExpSpan::Verbatim(String::from("a")),
                    ExpSpan::Verbatim(String::from("l\""))
                ]));
        assert_eq!(
            expansion(b"a'l\\''"),
            IResult::Done(
                &b""[..],
                vec![
                    ExpSpan::Verbatim(String::from("a")),
                    ExpSpan::Verbatim(String::from("l'"))
                ]));
        assert_eq!(
            expansion(b"~/'$STUFF'"),
            IResult::Done(
                &b""[..],
                vec![
                ExpSpan::Tilde,
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Verbatim(String::from("$STUFF"))
                ]));
    }

    #[test]
    fn double_quotes() {
        assert_eq!(expansion(b"\"STUFF\""), IResult::Done(&b""[..], vec![ExpSpan::Verbatim(String::from("STUFF"))]));
        assert_eq!(expansion(b"\"$STUFF\""), IResult::Done(&b""[..], vec![ExpSpan::Variable(String::from("STUFF"))]));
        assert_eq!(expansion(b"\"${STUFF}\""), IResult::Done(&b""[..], vec![ExpSpan::Variable(String::from("STUFF"))]));

        assert_eq!(
            expansion(b"\"{STU,FF}\""), 
            IResult::Done(
                &b""[..], 
                vec![ ExpSpan::Bracket(vec![String::from("STU"),String::from("FF")])
                ]));
    }

    #[test]
    fn full() {
        assert_eq!(
            expansion(b"~/STUFF"),
            IResult::Done(
                &b""[..],
                vec![ExpSpan::Tilde, ExpSpan::Verbatim(String::from("/STUFF"))]));
        assert_eq!(
            expansion(b"~/$STUFF"),
            IResult::Done(
                &b""[..],
                vec![ExpSpan::Tilde, 
                     ExpSpan::Verbatim(String::from("/")),
                     ExpSpan::Variable(String::from("STUFF"))]));
        assert_eq!(
            expansion(b"~/\"$STUFF\""),
            IResult::Done(
                &b""[..],
                vec![ExpSpan::Tilde,
                     ExpSpan::Verbatim(String::from("/")),
                     ExpSpan::Variable(String::from("STUFF"))]));

        assert_eq!(
            expansion(b"~/${STUFF}/{st,u,ff}/*.???"),
            IResult::Done(
                &b""[..],
                vec![
                ExpSpan::Tilde,
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Variable(String::from("STUFF")),
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Bracket(vec![String::from("st"),String::from("u"),String::from("ff")]),
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Glob(String::from("*")),
                ExpSpan::Verbatim(String::from(".")),
                ExpSpan::Glob(String::from("???"))
                ]));
        assert_eq!(
            expansion(b"~/${STUFF}/{st,u,ff}/*.???"),
            IResult::Done(
                &b""[..],
                vec![
                ExpSpan::Tilde,
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Variable(String::from("STUFF")),
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Bracket(vec![String::from("st"),String::from("u"),String::from("ff")]),
                ExpSpan::Verbatim(String::from("/")),
                ExpSpan::Glob(String::from("*")),
                ExpSpan::Verbatim(String::from(".")),
                ExpSpan::Glob(String::from("???"))
                ]));
    }

    #[test]
    fn assignment1() {
        assert_eq!(
            expansion(b"A=~"),
            IResult::Done(
                &b""[..],
                vec![
                ExpSpan::Verbatim(String::from("A")),
                ExpSpan::Verbatim(String::from("=")),
                ExpSpan::Tilde,
                ]));
        assert_eq!(
            expansion(b"A=~/STUFF"),
            IResult::Done(
                &b""[..],
                vec![
                ExpSpan::Verbatim(String::from("A")),
                ExpSpan::Verbatim(String::from("=")),
                ExpSpan::Tilde,
                ExpSpan::Verbatim(String::from("/STUFF"))
                ]));
    }

    #[test]
    fn assignment2() {
        assert_eq!(
            expansion(b"A=~'/STUFF'"),
            IResult::Done(
                &b""[..],
                vec![
                ExpSpan::Verbatim(String::from("A=~")),
                ExpSpan::Verbatim(String::from("/STUFF"))
                ]));
    }
}
