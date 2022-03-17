#!/bin/sh

PBETRACKS=../2018-tracks/PBE_BV_Track
REPEAT=20
for i in 1 11 15 17 23 41 45 46 51 53 55 69 85 95 101 111 115 123 139 147 
do
  name=$i
  folder=benchmarks/$name
  mkdir -p $folder
  for run in $(seq 1 $REPEAT)
  do
    /usr/bin/time -v cargo run --release $PBETRACKS/PRE_"$i"_10.sl baseline 10 1 0 2>&1 | tee $folder/baseline$run.txt
    /usr/bin/time -v cargo run --release $PBETRACKS/PRE_"$i"_10.sl egg 10 1 0 2>&1 | tee $folder/egg$run.txt
  done
done
