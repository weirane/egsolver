#!/bin/sh
echo "    base               egg"
echo "id, baseTime, baseRss, eggSearch, eggExtract, eggRss"
mkdir -p plots
cd benchmarks
for sygusid in *
do
  baselineTime=$(grep "time =" $sygusid/baseline* | cut -d" " -f9 | cut -d"m" -f1 | datamash -R2 mean 1)
  baselineRss=$(grep "Maximum resident" $sygusid/baseline* | cut -d" " -f6 | datamash -R2 mean 1 | awk '{printf "%.2f", $1/1000}')
  eggSearchTime=$(grep "egg. search time" $sygusid/egg* | cut -d" " -f5 | cut -d"m" -f1 | datamash -R2 mean 1)
  eggExtractTime=$(grep "egg. extract_time time" $sygusid/egg* | cut -d" " -f5 | cut -d"m" -f1 | datamash -R2 mean 1)
  eggRss=$(grep "Maximum resident" $sygusid/egg* | cut -d" " -f6 | datamash -R2 mean 1 | awk '{printf "%.2f", $1/1000}')
  echo $sygusid, $baselineTime, $baselineRss, $eggSearchTime, $eggExtractTime, $eggRss
done | sort -n | tee ../plots/raw.csv
