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

//! Color table handling

pub fn setupColors(col: &mut [u32; 256]) {
    col[0] = 0x000000; // black
    col[1] = 0xcd0000; // red3
    col[2] = 0x00cd00; // green3
    col[3] = 0xcdcd00; // yellow3
    col[4] = 0x0000ee; // blue2
    col[5] = 0xcd00cd; // magenta3
    col[6] = 0x00cdcd; // cyan3
    col[7] = 0xe5e5e5; // gray90
    col[8] = 0x7f7f7f; // gray50
    col[9] = 0xff0000; // red
    col[10] = 0x00ff00; // green
    col[11] = 0xffff00; // yellow
    col[12] = 0x5c5cff; // rgb:5c/5c/ff
    col[13] = 0xff00ff; // magenta
    col[14] = 0x00ffff; // cyan
    col[15] = 0xffffff; // white
}
