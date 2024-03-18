#!/bin/bash

set -e

cd ..
cd sdk
npx tsc
npm run node:start &
sleep 5
cd ..
cd sdk-test
npm run start &
test_pid=$!

wait $test_pid
test_status=$?

exit $test_status