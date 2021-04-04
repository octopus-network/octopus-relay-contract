#!/bin/bash
cargo build --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/octopus_relay.wasm ./out/main.wasm