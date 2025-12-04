#!/bin/zsh
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/00 -e zwlu12 -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/bwt -e zwlu12 --bwt -o

target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu12/mtf -e zwlu12 --mtf -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu12/11 -e zwlu12 --bwt --mtf -o
echo "zwlu12 done"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/00 -e huffman  -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/bwt -e huffman --bwt -o

target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/huffman/mtf -e huffman --mtf -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/huffman/11 -e huffman --bwt --mtf -o
echo "huffman done"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/00 -e zwlu16 -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/bwt -e zwlu16 --bwt -o

target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu16/mtf -e zwlu16 --mtf -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu16/11 -e zwlu16 --bwt --mtf -o
echo "zwl16 done"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/00 -e zwlu32 -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/bwt -e zwlu32 --bwt -o

target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu32/mtf -e zwlu32 --mtf -o
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu32/11 -e zwlu32 --bwt --mtf -o

echo "zwl32 done"