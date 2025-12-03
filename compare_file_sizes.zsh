#!/bin/bash

# Define the directories
folder1=$1
folder2=$2

# Iterate over files in the first folder
for file in "$folder1"/*; do
    filename=$(basename "$file")
    #echo $filename
    # Check for multiple appended extensions
    for ext in "$folder2"/"$filename".*; do
        if [ -e "$ext" ]; then
            size1=$(wc -c < "$file")
            size2=$(wc -c < "$ext")
            echo "Comparing: $filename"
            echo "Size in $folder1: $size1 bytes"
            echo "Size in $folder2: $size2 bytes"
            if [ $size2 -ne 0 ]; then
                compression=$(echo "scale=4; $size1 / $size2" | bc)
            else
                compression="undefined (size is zero)"
            fi
            echo "compression = $compression"
            echo ""
        fi
    done
done
