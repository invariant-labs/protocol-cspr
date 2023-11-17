#!/bin/bash
set -e

cargo test
cargo odra test -b casper