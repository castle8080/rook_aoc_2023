#!/bin/bash

mkdir -p results

if !cargo build --release; then
    exit 1
fi

cargo run --release | tee results/latest.txt

grep -v 'Elapsed Time' results/latest.txt > results/latest_net.txt
grep -v 'Elapsed Time' results/last.txt > results/last_net.txt

echo "===================================="
echo "Showing result diffs"
echo "===================================="
diff -U 1 results/last_net.txt results/latest_net.txt

