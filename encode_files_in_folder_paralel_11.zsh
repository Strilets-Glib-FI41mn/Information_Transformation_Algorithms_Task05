#!/bin/zsh
cargo build --release
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu12/11 -e zwlu12 --bwt --mtf &
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/huffman/11 -e huffman --bwt --mtf &
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu16/11 -e zwlu16 --bwt --mtf &
target/release/Information_Transformation_Algorithms_Task05 tested/$1  tested/$1/zwlu32/11 -e zwlu32 --bwt --mtf &
wait