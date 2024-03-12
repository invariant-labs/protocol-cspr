#!/bin/bash

set -e

npm i &&
npm run wasm:strip &&
npm run lint &&
npm run docs:copy &&
npm run build