npm run node:start &
sleep 1
npm run test &
test_pid=$!

wait $test_pid
test_status=$?

exit $test_status