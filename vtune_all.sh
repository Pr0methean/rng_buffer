#!/bin/bash
vtune -collect uarch-exploration -result-dir=/results -- /os
vtune -collect uarch-exploration -result-dir=/results -- /buffer_size_2.rs
vtune -collect uarch-exploration -result-dir=/results -- /buffer_size_4.rs
vtune -collect uarch-exploration -result-dir=/results -- /buffer_size_8.rs
vtune -collect uarch-exploration -result-dir=/results -- /buffer_size_16.rs
cd results || exit 1
for FILE in *; do
  aws s3api put-object --bucket rng-buffer-reports --body "$FILE" --key "$FILE"
done