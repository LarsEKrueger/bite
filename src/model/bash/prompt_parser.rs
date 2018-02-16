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

use nom::oct_digit;
use super::Bash;

/* Parse a prompt string and build a string.
 */
named_args!(pub parse_prompt<'a>(bash:&Bash)<String>,
         map!(many0!(
             alt!(call!(backslashy,bash) | history | something)
             )
             , |v| v.iter().flat_map( |s| s.chars()).collect()
             )
       );

fn array_to_string(array: &[u8]) -> Result<String, ::std::string::FromUtf8Error> {
    String::from_utf8(array.to_vec())
}

fn now_as(format: &str) -> Result<String, ::time::ParseError> {
    ::time::strftime(format, &::time::now())
}

/* Parse a history sequence and convert them to:

   !!   an exclamation mark !
   !    history number (same as \!), TODO
 */
named!(
    history<String>,
    alt_complete!(map!(tag!("!!"), |_| String::from("!")) | map!(tag!("!"), |_| String::from("1")))
);

/* Parse the backslash sequences and convert them to:

 * \a       bell (ascii 07)
 * \d       the date in Day Mon Date format
 * \e       escape (ascii 033)
 * \h       the hostname up to the first `.'
 * \H       the hostname
   \j       the number of active jobs, TODO
 * \l       the basename of the shell's tty device name -> always "tty" as we don't have a tty
 * \n       CRLF
 * \r       CR
 * \s       the name of the shell -> BiTE
 * \t       the time in 24-hour hh:mm:ss format
 * \T       the time in 12-hour hh:mm:ss format
 * \@       the time in 12-hour hh:mm am/pm format
 * \A       the time in 24-hour hh:mm format
 * \D{fmt}  the result of passing FMT to strftime(3)
 * \u       your username
 * \v       the version of bash (e.g., 2.00)
 * \V       the release of bash, version + patchlevel (e.g., 2.00.0)
   \w       the current working directory, TODO
   \W       the last element of $PWD, TODO
   \!       the history number of this command, TODO
   \#       the command number of this command, TODO
 * \$       a $ or a # if you are root
 * \nnn     character code nnn in octal
 * \\       a backslash

   The following two are only required for readline. We replace them with nothing as we don't need
   the markers.
 * \[       begin a sequence of non-printing chars
 * \]       end a sequence of non-printing chars
*/
named_args!(
    backslashy<'a>(bash:&Bash)<String>,
    do_parse!(tag!("\\") >> 
       res: alt!(
           map!(tag!("a"),|_| String::from("\x07")) |
           map!(tag!("e"),|_| String::from("\x1B")) |
           map!(tag!("\\"),|_| String::from("\\")) |
           map!(tag!("n"),|_| String::from("\n")) |
           map!(tag!("r"),|_| String::from("\r")) |
           map!(tag!("s"),|_| String::from("BiTE")) |
           map!(tag!("["),|_| String::from("")) |
           map!(tag!("]"),|_| String::from("")) |
           map!(tag!("v"),|_| String::from(Bash::version())) |
           map!(tag!("V"),|_| String::from(Bash::version_and_patchlevel())) |
           map!(tag!("H"),|_| String::from(bash.get_current_host_name())) |
           map!(tag!("h"),|_| { let hn = bash.get_current_host_name();
                match hn.find('.') {
                    None => String::from(hn),
                    Some(dot) => String::from(&hn[..dot])
                }
           }) |
           map!(tag!("l"),|_| String::from("tty")) |
           map_res!(tag!("d"),|_| now_as("%a %b %d") )|
           map_res!(tag!("t"),|_| now_as( "%H:%M:%S"))|
           map_res!(tag!("T"),|_| now_as( "%I:%M:%S"))|
           map_res!(tag!("@"),|_| now_as( "%I:%M %p"))|
           map_res!(tag!("A"),|_| now_as( "%H:%M"))|
           map!(tag!("u"),|_| String::from(bash.get_current_user_name()))|
           map!(tag!("$"),|_| String::from(if bash.current_user_is_root() { "#" } else { "$" }))|
           user_time|
           octal
           ) >>
         (res)
        )
);

named!(
    octal<String>,
    map_res!(
        tuple!(oct_digit, oct_digit, oct_digit),
        |(s2, s1, s0): (&[u8], &[u8], &[u8])| {
            let d2 = s2[0] - 48;
            let d1 = s1[0] - 48;
            let d0 = s0[0] - 48;

            let c = vec![d2 * 8 * 8 + d1 * 8 + d0];
            String::from_utf8(c)
        }
    )
);

named!(
    user_time<String>,
    map!(
        do_parse!(
        tag!("D{") >>
        res: take_until!("}")>>
        tag!("}") >>
        (res)
        ),
        |f| {
            let tm = ::time::now();
            let f = ::std::str::from_utf8(f).unwrap_or("%X");
            match ::time::strftime(f, &tm).or_else(|_| ::time::strftime("%X", &tm)) {
                Ok(s) => s,
                Err(_) => String::from("??time??"),
            }
        }
    )
);

// Parse a character.
named!(something<String>, map_res!(take!(1), array_to_string));

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult;

    impl super::Bash {
        fn test_set_host_name(&mut self, hn: &str) {
            self.current_host_name = String::from(hn);
        }

        fn test_set_user_name(&mut self, un: &str) {
            self.current_user.name = String::from(un);
        }

        fn test_set_user_uid(&mut self, uid: ::libc::uid_t) {
            self.current_user.uid = uid;
        }
    }

    #[test]
    fn basic() {
        let mut bash = Bash::new();
        bash.test_set_host_name("my.host.name");
        bash.test_set_user_name("myname");
        assert_eq!(
            parse_prompt(&b"abcd"[..], &bash),
            IResult::Done(&b""[..], String::from("abcd"))
        );

        assert_eq!(
            parse_prompt(&b"ab!!cd"[..], &bash),
            IResult::Done(&b""[..], String::from("ab!cd"))
        );

        assert_eq!(
            parse_prompt(&b"ab!cd"[..], &bash),
            IResult::Done(&b""[..], String::from("ab1cd"))
        );

        assert_eq!(
            parse_prompt(&b"abcd!"[..], &bash),
            IResult::Done(&b""[..], String::from("abcd1"))
        );

        assert_eq!(
            parse_prompt(&b"ab\\acd"[..], &bash),
            IResult::Done(&b""[..], String::from("ab\x07cd"))
        );

        assert_eq!(
            parse_prompt(&b"ab\\ecd"[..], &bash),
            IResult::Done(&b""[..], String::from("ab\x1Bcd"))
        );

        assert_eq!(
            parse_prompt(&b"ab\\r\\ncd"[..], &bash),
            IResult::Done(&b""[..], String::from("ab\r\ncd"))
        );

        assert_eq!(
            parse_prompt(&b"ab\\\\cd"[..], &bash),
            IResult::Done(&b""[..], String::from("ab\\cd"))
        );

        assert_eq!(
            parse_prompt(&b"I \\s you."[..], &bash),
            IResult::Done(&b""[..], String::from("I BiTE you."))
        );

        assert_eq!(
            parse_prompt(&b"\\v"[..], &bash),
            IResult::Done(&b""[..], String::from("0.0"))
        );
        assert_eq!(
            parse_prompt(&b"\\V"[..], &bash),
            IResult::Done(&b""[..], String::from("0.0.0"))
        );

        assert_eq!(
            parse_prompt(&b"\\h"[..], &bash),
            IResult::Done(&b""[..], String::from("my"))
        );
        assert_eq!(
            parse_prompt(&b"\\H"[..], &bash),
            IResult::Done(&b""[..], String::from("my.host.name"))
        );

        assert_eq!(
            parse_prompt(&b"\\l"[..], &bash),
            IResult::Done(&b""[..], String::from("tty"))
        );
        assert_eq!(
            parse_prompt(&b"\\u"[..], &bash),
            IResult::Done(&b""[..], String::from("myname"))
        );

        bash.test_set_user_uid(0);
        assert_eq!(
            parse_prompt(&b"\\$"[..], &bash),
            IResult::Done(&b""[..], String::from("#"))
        );
        bash.test_set_user_uid(1);
        assert_eq!(
            parse_prompt(&b"\\$"[..], &bash),
            IResult::Done(&b""[..], String::from("$"))
        );
    }

}
