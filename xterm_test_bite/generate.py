#
#  BiTE - Bash-integrated Terminal Emulator
#  Copyright (C) 2018  Lars Krüger
#
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#
#  This program is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
#
#  You should have received a copy of the GNU General Public License
#  along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# Code generator for xterm-unit-test, specialized to test bite's screen module.

import sys
import os

def rust_escape(s):
    return s

def rust_escape_char(c):
    if c == '\\':
        return "\\\\"
    if c == '\'':
        return "\\'"

    return c

code2attr = {
        'i': "Attributes::INVERSE",
        'u': "Attributes::UNDERLINE",
        'b': "Attributes::BOLD",
        'l': "Attributes::BLINK",
        'c': "Attributes::BG_COLOR",
        'f': "Attributes::FG_COLOR",
        'p': "Attributes::PROTECTED",
        'd': "Attributes::CHARDRAWN",
        'a': "Attributes::ATR_FAINT",
        't': "Attributes::ATR_ITALIC",
        's': "Attributes::ATR_STRIKEOUT",
        'w': "Attributes::ATR_DBL_UNDER",
        'v': "Attributes::INVISIBLE",
        }

def check_size(pat,check, filehandle):
    print(("  " + pat) % ("s.width()", "%d" % check.w), file=filehandle)
    print(("  " + pat) % ("s.height()", "%d" % check.h), file=filehandle)

def check_cpos(pat,check,filehandle):
    print(("  " + pat) % ("s.cursor_x()", "%d" % check.x), file=filehandle)
    print(("  " + pat) % ("s.cursor_y()", "%d" % check.y), file=filehandle)

def check_char(pat,check,filehandle):
    print("  assert!(0 <= %d && %d<s.width());" % (check.x, check.x), file=filehandle);
    print("  assert!(0 <= %d && %d<s.height());" % (check.y, check.y), file=filehandle);
    print(("  " + pat) % ("s.matrix.cell_at(%d,%d).code_point" % (check.x, check.y),
        "char::from(b'%s')" % rust_escape_char(check.c)),
        file=filehandle)

def check_attr(pat,check,filehandle):
    attr = check.a
    if attr == '':
        attr_str = "Attributes::empty()"
    else:
        attr_str = ""
        sep = ""
        for c in attr:
            attr_str = attr_str + sep + code2attr[c]
            sep = "|"

    print(("  " + pat) % ("s.matrix.cell_at(%d,%d).attributes" % (check.x, check.y),
        "%s" % attr_str),
        file=filehandle)

check_functions = {
    'CheckSize': check_size,
    'CheckCPos': check_cpos,
    'CheckChar': check_char,
    'CheckAttr': check_attr,
    }

class Generator:
    def __init__(self, out_dir):
        self.out_dir = out_dir
        pass

    def begin_file(self, filename):
        bn = os.path.basename(filename)
        (fn,_) = os.path.splitext(bn)
        out_file_name = os.path.join(self.out_dir,fn + ".rs")
        self.file = open(out_file_name, 'w')

    def begin_test(self, testname):
        print("#[test]\nfn %s() {" % testname, file=self.file)

    def create_screen(self, w, h):
        print("  let mut s = new_test_screen(%d,%d);" % (w,h), file=self.file)

    def place_cursor(self, x, y):
        print("  s.cursor.x = %d;\n  s.cursor.y = %d;" % (x,y), file=self.file)

    def export_sequence(self, seq):
        print("  s.add_bytes(b\"%s\");" % rust_escape(seq), file=self.file)

    def begin_checks(self):
        print("  let mut unexpected = false;", file=self.file)

    def add_check(self,check):
        if check.error:
            pat = "assert_eq!(%s,%s);"
        else:
            pat = "expect_eq!(unexpected,%s,%s);"

        fun = check_functions[check.__class__.__name__]
        fun(pat, check, self.file)

    def end_checks(self):
        print("  assert_eq!(unexpected,false);", file=self.file)

    def end_test(self, testname):
        print("}", file=self.file)

    def end_file(self, filename):
        self.file.close()


# Check the arguments
if len(sys.argv) < 3:
    print( "Usage: %s <path to xterm-unit-test> \
            <output directory>" % sys.argv[0], file=sys.stderr)
    sys.exit( 1)

xut_path = sys.argv[1]
out_path = sys.argv[2]

os.makedirs(out_path,exist_ok=True)

# Fix the python path so we can find the test framework.
sys.path.append( xut_path)

import xut

# Generate the tests
xut.generate( Generator(out_path), os.path.join(xut_path,"tests"))
