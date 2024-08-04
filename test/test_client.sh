#!/bin/zsh

# Setting default values
if [[ -z "$url" ]]; then
  url="ws://127.0.0.1:8080"
fi

if [[ -z "$username" ]]; then 
  username="test"
fi

# Inject the test data
cat ./CLIENT_TEST_CODE_SNIPPET.rs > ./test_injection.rs

cargo run --features "integration_testing_client" client $url $username

echo "// this file should be injected by the when testing is done by client
// you sould write a function named run_test" > ./test_injection.rs
