#!/bin/zsh

# Setting default values for the port
if [[ -z "$port" ]]; then
  port=8080
fi


cat ./SERVER_TEST_CODE_SNIPPET.rs > ./test_injection.rs

# Injecting the test code into the server code
cargo run --features "integration_testing_server"  server $port

echo "// this file should be injected by the when testing is done by client
// you sould write a function named run_test" > ./test_injection.rs

