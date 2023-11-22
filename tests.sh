#!/bin/bash
set -e

cd src/decimal
cargo test

cd decimal_core
cargo test
cd ../../..

cargo test
cargo odra test -b casper