#!/bin/bash

awk '
    BEGIN {
        printf("Problem,Time,Answer\n");
    }
    /Running:/      { problem=$2; }
    /Answer:/       { answer=$2; }
    /Elapsed Time:/ {
        time=$3;
        printf("%s,%s,%s\n", problem, time, answer);
    }
'
