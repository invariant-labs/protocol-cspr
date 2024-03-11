#!/bin/bash

set -e

npm run node:start &
sleep 10
npm run clean:target &
sleep 5
npm run test &
test_pid=$!

wait $test_pid
test_status=$?

exit $test_status