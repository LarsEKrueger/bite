#! /bin/bash

# Usage: scroll_region.sh [options]
# Options:
#   in     : Cursor inside region [default]
#   out    : Cursor outside region
#   nl     : New line in last row [default]
#   index  : Index in last row
#   right  : Scroll right
#   left   : Scroll left
#   up     : Scroll up
#   down   : Scroll down
#   convert: Print the test string for rust

INOUT=in
MODE=nl
UPDOWN=
LEFTRIGHT=
CONVERT=false
while test "$#" != 0;  do
  cmd=$1
  shift
  case $cmd in
    in|out|outforce)
      INOUT=$cmd
      ;;
    index)
      MODE=index
      ;;
    right|left|leftall|rightall)
      MODE=
      LEFTRIGHT=$cmd
      ;;
    up|down|upall|downall)
      MODE=
      UPDOWN=$cmd
      ;;
    convert)
      CONVERT=true
      ;;
    *)
      echo "Illegal command. Exiting."
      exit 1
      ;;
  esac
done

if test $CONVERT != true; then
  # Clear screen, go to top-left
  echo -e -n "\e[2J\e[1d"
fi

(
  # Print line numbers
  for ((i=0;$i<10;i++)); do
    printf '%02d\n' $i
  done
  echo -e -n "\e[2;8r"
  case $INOUT in
    in)
      echo -e -n "\e[8d"
      ;;
    out)
      ;;
    outforce)
      echo -e -n "\e[11d"
      ;;
    *)
      ;;
  esac
  case $MODE in
    nl)
      echo
      ;;
    index)
      echo -e -n "\eD"
      ;;
    *)
      ;;
  esac
  case $UPDOWN in
    up)
      echo -e -n "\e[1S"
      ;;
    down)
      echo -e -n "\e[1T"
      ;;
    upall)
      echo -e -n "\e[10S"
      ;;
    downall)
      echo -e -n "\e[10T"
      ;;
    *)
      ;;
  esac
  case $LEFTRIGHT in
    right)
      echo -e -n "\e[1 A"
      ;;
    left)
      echo -e -n "\e[1 @"
      ;;
    rightall)
      echo -e -n "\e[80 A"
      ;;
    leftall)
      echo -e -n "\e[80 @"
      ;;
    *)
      ;;
  esac
  echo -n "<-- Cursor Here"
  ) | if test $CONVERT == true ; then ( od -An -tc -w1000 | sed 's/033/\\x1b/g' | sed 's/\s\+//g' ) else cat; fi

if test $CONVERT != true; then
  # Clear scroll region
  echo -e -n "\e[r"

  # Go to bottom
  echo -e -n "\e[25d"
fi
