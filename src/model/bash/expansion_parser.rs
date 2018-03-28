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

use super::script_parser;
use super::script_parser::identifier;
use super::super::types::*;

named!(pub expansion<Expansion>,
       terminated!(
           alt!(
               exp_tilde_first |
               many1!(exp_span)
               ),
           eof!())
      );

named!(exp_span<ExpSpan>,
       alt!(
           exp_variable |
           exp_variable_br |
           exp_bracket |
           exp_glob |
           exp_other
           )
);

named!(exp_other<ExpSpan>,
       map!(many1!(none_of!("${*?")),
       |s| ExpSpan::Verbatim(s.iter().collect()))
);

named!(exp_variable<ExpSpan>,
       do_parse!(
           tag!("$") >>
           id : identifier >>
           (ExpSpan::Variable(String::from_utf8_lossy(id).into_owned()))
                )
);

named!(exp_variable_br<ExpSpan>,
       do_parse!(
           tag!("${") >>
           id : identifier >>
           tag!("}") >>
           (ExpSpan::Variable(String::from_utf8_lossy(id).into_owned()))
                )
);

named!(exp_tilde_first<Expansion>,
       do_parse!(
           tag!("~") >>
           spans : fold_many0!(exp_span,vec![ExpSpan::Tilde], |mut v:Vec<_>,i| { v.push(i); v}) >>
           ( spans )
           )
);

named!(exp_bracket<ExpSpan>,
       map!(
           delimited!(
               tag!("{"),
               expbr_inner,
               tag!("}")
               ),
               ExpSpan::Bracket
           )
      );

named!(expbr_inner<Vec<String>>,
       separated_list!(tag!(","),expbr_span)
      );

named!(expbr_span<String>,
       map!(many1!(none_of!(",}")), |s| s.into_iter().collect())
      );

named!(exp_glob<ExpSpan>,
       map!(recognize!(many1!(one_of!("*?"))),|s| ExpSpan::Glob(String::from_utf8_lossy(s).into_owned()))
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
}
