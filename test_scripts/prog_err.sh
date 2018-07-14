#! /bin/bash

for ((i=0;$i<10;i++)); do
  echo "Iteration $i"
  >&2 echo -n "Error: $i "
  sleep 0.25
  >&2 echo -n ':'
  sleep 0.25
  >&2 echo -n '-'
  sleep 0.25
  >&2 echo -n ')'
  sleep 0.25
  >&2 echo -ne "\r"
  sleep 0.25
done
