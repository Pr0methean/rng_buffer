#!/bin/bash
cd /opt/intel/oneapi/vtune/latest/bin64/ || exit 1
sh ./vtune-self-checker.sh
cd /

BENCHES=( os buffer_size_2 buffer_size_4 buffer_size_8 buffer_size_16 )
for BENCH in "${BENCHES[@]}"; do
  mkdir "$BENCH"-results
  vtune -collect uarch-exploration -result-dir="$BENCH"/results -- /"$BENCH"
  for FILE in $BENCH-results/*; do
    aws s3api put-object --bucket rng-buffer-reports --body "$FILE" --key "$FILE"
  done
done