#!/bin/zsh

echo "encoding $1"

# target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/00 -e huffman  -o &
# target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/bwt -e huffman --bwt -o &
# sleep
# echo "huffman 2/4"

# target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/mtf -e huffman --mtf -o &
# target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/11 -e huffman --bwt --mtf -o &
# sleep
# echo "huffman done"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/00 -e huffman -o
sleep 1
echo "huffman 1/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/bwt -e huffman --bwt -o
sleep 1
echo "huffman 2/4"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/mtf -e huffman --mtf -o
sleep 1
echo "huffman 3/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/11 -e huffman --bwt --mtf -o
sleep 1
echo "huffman done"


target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/00 -e zwlu12 -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/bwt -e zwlu12 --bwt -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/mtf -e zwlu12 --mtf -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/11 -e zwlu12 --bwt --mtf -o
sleep 1
echo "zwlu12 done"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/00 -e zwlu16 -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/bwt -e zwlu16 --bwt -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/mtf -e zwlu16 --mtf -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/11 -e zwlu16 --bwt --mtf -o
sleep 1
echo "zwl16 done"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/00 -e zwlu32 -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/bwt -e zwlu32 --bwt -o
sleep 1

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/mtf -e zwlu32 --mtf -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/11 -e zwlu32 --bwt --mtf -o
sleep 1

echo "zwl32 done"

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/00 -e zwlu64 -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/bwt -e zwlu64 --bwt -o
sleep 1

target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/mtf -e zwlu64 --mtf -o
sleep 1
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu32/11 -e zwlu64 --bwt --mtf -o

echo "zwl64 done"