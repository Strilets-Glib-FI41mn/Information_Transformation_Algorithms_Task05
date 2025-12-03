#!/bin/zsh
cargo run --release tested/$1 tested/$1/zwlu12/00 -e zwlu12 &
cargo run --release tested/$1 tested/$1/zwlu12/bwt -e zwlu12 --bwt &
wait
cargo run --release tested/$1  tested/$1/zwlu12/mtf -e zwlu12 --mtf &
cargo run --release tested/$1  tested/$1/zwlu12/11 -e zwlu12 --bwt --mtf &
wait

cargo run --release tested/$1 tested/$1/huffman/00 -e huffman &
cargo run --release tested/$1 tested/$1/huffman/bwt -e huffman --bwt &
wait
cargo run --release tested/$1  tested/$1/huffman/mtf -e huffman --mtf &
cargo run --release tested/$1  tested/$1/huffman/11 -e huffman --bwt --mtf &
wait

cargo run --release tested/$1 tested/$1/zwlu16/00 -e zwlu16 &
cargo run --release tested/$1 tested/$1/zwlu16/bwt -e zwlu16 --bwt &
wait
cargo run --release tested/$1  tested/$1/zwlu16/mtf -e zwlu16 --mtf &
cargo run --release tested/$1  tested/$1/zwlu16/11 -e zwlu16 --bwt --mtf &
wait

cargo run --release tested/$1 tested/$1/zwlu32/00 -e zwlu32 &
cargo run --release tested/$1 tested/$1/zwlu32/bwt -e zwlu32 --bwt &
wait
cargo run --release tested/$1  tested/$1/zwlu32/mtf -e zwlu32 --mtf &
cargo run --release tested/$1  tested/$1/zwlu32/11 -e zwlu32 --bwt --mtf &
wait