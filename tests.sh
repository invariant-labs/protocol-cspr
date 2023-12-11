#!/bin/bash
set -e

cd src/decimal
cargo test

cd decimal_core
cargo test
cd ../../..

cd src/token
cargo fmt --all -- --check
cargo clippy --all-targets -- --no-deps -D warnings
cargo odra test
cargo odra build -b casper
cd ../..

cargo fmt --all -- --check
# cargo clippy --all-targets -- --no-deps -D warnings

cargo test
cargo odra test
cargo odra build -b casper