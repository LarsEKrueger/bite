#! /bin/bash

for ((i=0;$i<10;i++)); do
  echo -n "Iteration: $i "
  sleep 0.25
  echo -n ':'
  sleep 0.25
  echo -n '-'
  sleep 0.25
  echo -n ')'
  sleep 0.25
  echo -ne "\r"
  sleep 0.25
done
