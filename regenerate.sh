#!/bin/bash

echo 'input,output' > golden.csv

while read w; do
    echo "$w" > tmp.txt
    printf '"%s","' "$w" >> golden.csv
    ./convert tmp.txt | tr -d '\n' >> golden.csv
    printf '"\n' >> golden.csv
done < words.txt
