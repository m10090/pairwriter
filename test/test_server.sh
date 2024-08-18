#!/bin/bash

# Setting default values for the port
if [[ -z "$port" ]]; then
  port=8080
fi


cat ./SERVER_TEST_CODE_SNIPPET.rs > ./test_injection.rs

# Injecting the test code into the server code
timeout -s SIGINT 50s cargo run --features "integration_testing_server"  server $port

if grep -q "Test Passed!" test_result; then
  echo "Test passed"
  result=0
else
  echo "Test failed"
  result=1
fi
./clean_test.sh
exit $result

