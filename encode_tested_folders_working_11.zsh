#!/bin/zsh
cargo build --release

./encode_files_in_folder_paralel_11.zsh "epub" &
./encode_files_in_folder_paralel_11.zsh "executable" &
wait
./encode_files_in_folder_paralel_11.zsh "pages" &
./encode_files_in_folder_paralel_11.zsh "txt" &
wait
./encode_files_in_folder_paralel_11.zsh "pdf"
