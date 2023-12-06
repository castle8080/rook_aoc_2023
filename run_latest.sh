#!/bin/bash

mkdir -p results

if !cargo build --release; then
    exit 1
fi

cargo run --release | tee results/latest.txt

./results_to_csv.sh < results/latest.txt > results/latest.csv

grep -v 'Elapsed Time' results/latest.txt > results/latest_net.txt
grep -v 'Elapsed Time' results/last.txt > results/last_net.txt

diff_txt=`diff -U 1 results/last_net.txt results/latest_net.txt`
diff_rc="$?"

if [ $diff_rc -gt 0 ]; then
    echo "========================================="
    echo "Differences:"
    echo "========================================="
    echo "$diff_txt"
    echo "========================================="
fi
