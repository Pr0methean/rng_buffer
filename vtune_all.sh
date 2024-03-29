#!/bin/bash
pushd /opt/intel/oneapi/vtune/latest/sepdk/src || exit 1
./insmod-sep -g vtune -pu
./boot-script --install

# FIXME: Find a way to do the above in the Dockerfile

cd /opt/intel/oneapi/vtune/latest/bin64/ || exit 1
sh ./vtune-self-checker.sh
popd

BENCHES=( os buffer_size_2 buffer_size_4 buffer_size_8 buffer_size_16 )
for BENCH in "${BENCHES[@]}"; do
  rm -rf "$BENCH"-results
  mkdir "$BENCH"-results
  /opt/intel/oneapi/vtune/latest/bin64/vtune -collect uarch-exploration -result-dir="$BENCH"-results -- $(pwd)/"$BENCH"
  for FILE in "$BENCH"-results/*; do
    aws s3 cp ./"$BENCH"-results/"$FILE" s3:///rng-buffer-reports/"$BENCH"-results/"$FILE"
  done
done
