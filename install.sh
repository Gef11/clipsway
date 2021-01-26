#!/bin/bash
if ! [ -d ~/.clipsway ]; then
  mkdir ~/.clipsway
fi
if ! [ -d ~/.clipsway/images ]; then
  mkdir ~/.clipsway/images
fi
touch ~/.clipsway/history.ron
cargo build --release
strip target/release/clipsway
cp -f target/release/clipsway ~/.clipsway/
