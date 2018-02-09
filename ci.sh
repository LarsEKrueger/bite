#! /bin/sh

inotifywait -m -e close_write $(find src -type d) | while read line; do
  if echo $line | grep '\.rs$'; then
     clear
     cargo test
  fi
done
