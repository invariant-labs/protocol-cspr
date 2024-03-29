#!/bin/bash
set -e

cd src/decimal
cargo test

cd decimal_core
cargo test
cd ../../..

cargo fmt --all -- --check
cargo clippy --all-targets -- --no-deps -D warnings

cargo test
cargo odra test
cargo odra test -b casper
cargo odra test -b casper -- --features base-e2e
cargo odra test -b casper -- --features time-consuming-e2e
cargo odra build -b casper