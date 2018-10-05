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

def check_size(pat,check, filehandle):
    print(("  " + pat) % ("s.width()", "%d" % check.w), file=filehandle)
    print(("  " + pat) % ("s.height()", "%d" % check.h), file=filehandle)

def check_cpos(pat,check,filehandle):
    print(("  " + pat) % ("s.cursor_x()", "%d" % check.x), file=filehandle)
    print(("  " + pat) % ("s.cursor_y()", "%d" % check.y), file=filehandle)

check_functions = {
    'CheckSize': check_size,
    'CheckCPos': check_cpos,
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
