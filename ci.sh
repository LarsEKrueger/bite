#! /bin/bash

LINES=$(tput lines)
((LINES=$LINES/2))

inotifywait -m -e close_write $(find src -type d) c_src | while read line; do
  if echo $line | grep '\.rs$'; then
     clear
     cargo build --color always
     if [ $? == 0 ] ; then
       cargo test --color always | grep -v '^test .*ok$'
     fi
  fi
done
