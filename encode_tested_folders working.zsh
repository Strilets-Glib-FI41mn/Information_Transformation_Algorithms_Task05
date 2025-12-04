#!/bin/zsh
cargo build --release

./encode_files_in_folder_working.zsh "epub" &
./encode_files_in_folder_working.zsh "executable" &
wait
./encode_files_in_folder_working.zsh "pages" &
./encode_files_in_folder_working.zsh "txt" &
wait
./encode_files_in_folder_working.zsh "pdf"
