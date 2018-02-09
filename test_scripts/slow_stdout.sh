#! /bin/bash

for ((i=0;$i<10;i++)); do
  echo "Iteration: $i"
  >&2 echo "error $i"
  sleep 1
done
