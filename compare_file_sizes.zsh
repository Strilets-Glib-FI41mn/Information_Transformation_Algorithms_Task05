#!/bin/bash
folder1=$1
folder2=$2

compression_ratios=()
total_ratio=0
count=0

for file in "$folder1"/*; do
    filename=$(basename "$file")

    # Check for multiple appended extensions
    for ext in "$folder2"/"$filename".*; do
        if [ -e "$ext" ]; then
            size1=$(wc -c < "$file")
            size2=$(wc -c < "$ext")

            if [ "$size2" -ne 0 ]; then
                compression=$(echo "scale=4; $size1 / $size2" | bc)

                # Validate if the calculated compression is a number
                if [[ $compression =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
                    total_ratio=$(echo "$total_ratio + $compression" | bc)
                    compression_ratios+=("$compression")
                    ((count++))
                else
                    echo "Invalid compression: $compression"
                fi

            else
                compression="undefined (size is zero)"
            fi
            
            echo "Comparing: $filename"
            echo "Size in $folder1: $size1 bytes"
            echo "Size in $folder2: $size2 bytes"
            echo "Compression Ratio: $compression"
            echo ""
        fi
    done
done

# Check if count has positive values before proceeding
if [ "$count" -gt 0 ]; then
    min=${compression_ratios[0]}
    max=${compression_ratios[0]}

    for ratio in "${compression_ratios[@]}"; do
        if (( $(echo "$ratio < $min" | bc -l) )); then
            min=$ratio
        fi
        if (( $(echo "$ratio > $max" | bc -l) )); then
            max=$ratio
        fi
    done

    average=$(echo "scale=4; $total_ratio / $count" | bc)

    # Print results
    echo "Compression Ratios Collected:"
    for ratio in "${compression_ratios[@]}"; do
        echo "$ratio"
    done
    echo "Minimum Compression Ratio: $min"
    echo "Maximum Compression Ratio: $max"
    echo "Average Compression Ratio: $average"
else
    echo "No valid compression ratios were collected."
fi
