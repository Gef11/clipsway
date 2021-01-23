#!/bin/bash
if ! [ -d ~/.clipsway ]; then
  mkdir ~/.clipsway
fi
touch ~/.clipsway/history
cargo build --release
strip target/release/clipsway
cp -f target/release/clipsway ~/.clipsway/
cp -f ./daemon.sh ~/.clipsway/
