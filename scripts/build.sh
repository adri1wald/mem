#!/bin/bash
cargo build --release
cp target/release/mem ~/.local/bin/
chmod +x ~/.local/bin/mem
