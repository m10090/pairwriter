#!/bin/bash

# Setting default values
if [ -z "$url" ]; then
  url="ws://127.0.0.1:8080"
fi

if [ -z "$username" ]; then 
  username="test"
fi
if [ -z "$file" ]; then 
  file="./CLIENT/CLIENT_EDIT_FILE.rs"
fi
if [ $(command -v brew && $?) ]; then
  brew install coreutils
fi 

# Inject the test data
cat $file > ./test_injection.rs


timeout -s SIGINT 50s cargo run -q --features "integration_testing_client" client $url $username > test_result

result=0
if grep -q "Test Passed!" test_result; then
  echo "Test passed"
  ./clean_test.sh
  result=0
else
  echo "Test failed"
  printf "\n\n\n"
  cat test_result
  exit 1
fi
