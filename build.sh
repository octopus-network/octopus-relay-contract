#!/bin/bash
cargo build --all --target wasm32-unknown-unknown --release
if [ ! -d "out" ]; then
    mkdir -p "out"
fi
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
cp target/wasm32-unknown-unknown/release/octopus_relay.wasm ./out/main.wasm
