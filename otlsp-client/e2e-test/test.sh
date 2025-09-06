#!/bin/bash

export RUST_LOG=trace

echo "Starting server"
miniserve data/ -p 8080 --tls-cert localhost.crt --tls-key localhost.key &
SERVE_PID=$!


echo "Starting proxy"
( cd ../../ ; cargo build )
../../target/debug/otlsp-server &
PROXY_PID=$!

#( cd ../ ; wasm-pack test --headless --chrome)
#( cd ../ ; wasm-pack test --headless --firefox)

read -p "Press enter to continue"

echo "Stopping proxy"
kill $PROXY_PID

echo "Stopping server"
kill $SERVE_PID
