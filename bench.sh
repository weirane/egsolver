#!/bin/sh

for i in 1 147
do
  /usr/bin/time -v cargo run --release ../2018-tracks/PBE_BV_Track/PRE_"$i"_10.sl egg 10 3 2>&1 | tee benchmarks/"bv-pre-$i-10-egg.txt"
done
