#! /bin/bash

set -e

cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir app/wasm --no-typescript ./target/wasm32-unknown-unknown/release/asteroids.wasm
