#! /bin/bash

set -e

cargo build -p app --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir app/www/wasm --no-typescript ./target/wasm32-unknown-unknown/release/app.wasm

cargo build -p examples --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir examples/www/wasm --no-typescript ./target/wasm32-unknown-unknown/release/examples.wasm
