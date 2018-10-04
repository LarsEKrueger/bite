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

class Generator:
    def __init__(self, out_dir):
        self.out_dir = out_dir
        pass

    def begin_file(self, filename):
        print("Generating for file '%s'" % filename)

    def begin_test(self, testname):
        print("  Generating test '%s'" % testname)

    def end_test(self, testname):
        print("  Done generating test '%s'" % testname)

    def end_file(self, filename):
        print("Done generating for file '%s'" % filename)



# Check the arguments
if len(sys.argv) < 3:
    print( "Usage: %s <path to xterm-unit-test> \
            <output directory>" % sys.argv[0], file=sys.stderr)
    sys.exit( 1)

xut_path = sys.argv[1]
out_path = sys.argv[2]

# Fix the python path so we can find the test framework.
sys.path.append( xut_path)

import xut

# Generate the tests
xut.generate( Generator(out_path), os.path.join(xut_path,"tests"))
