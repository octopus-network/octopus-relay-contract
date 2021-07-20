#!/bin/bash
cargo build --all --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/*.wasm ./res/
cp target/wasm32-unknown-unknown/release/octopus_relay.wasm ./out/main.wasm
