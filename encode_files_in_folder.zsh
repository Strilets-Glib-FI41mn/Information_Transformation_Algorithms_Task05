#!/bin/zsh

echo "encoding $1"

echo "huffman 0/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/00 -e huffman -o &
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/bwt -e huffman --bwt -o &
wait
echo "huffman 2/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/mtf -e huffman --mtf -o &
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/huffman/11 -e huffman --bwt --mtf -o &
wait
echo "huffman done"

echo "zwlu12 0/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/00 -e zwlu12 -o &
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/bwt -e zwlu12 --bwt -o &
wait
echo "zwlu12 2/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/mtf -e zwlu12 --mtf -o &
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu12/11 -e zwlu12 --bwt --mtf -o &
wait
echo "zwlu12 done"

echo "zwl16 0/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/00 -e zwlu16 -o &
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/bwt -e zwlu16 --bwt -o &
wait
echo "zwl16 2/4"
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/mtf -e zwlu16 --mtf -o &
target/release/Information_Transformation_Algorithms_Task05 tested/$1 tested/$1/zwlu16/11 -e zwlu16 --bwt --mtf -o &
wait
echo "zwl16 done"