#!/bin/zsh
cargo run --release tested/$1 tested/$1/zwlu12/00 -e zwlu12
cargo run --release tested/$1 tested/$1/zwlu12/bwt -e zwlu12 --bwt
cargo run --release tested/$1  tested/$1/zwlu12/mtf -e zwlu12 --mtf
cargo run --release tested/$1  tested/$1/zwlu12/11 -e zwlu12 --bwt --mtf


cargo run --release tested/$1 tested/$1/huffman/00 -e huffman
cargo run --release tested/$1 tested/$1/huffman/bwt -e huffman --bwt
cargo run --release tested/$1  tested/$1/huffman/mtf -e huffman --mtf
cargo run --release tested/$1  tested/$1/huffman/11 -e huffman --bwt --mtf