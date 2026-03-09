#!/bin/bash

# Define the JSON message
RPC='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"processId":null,"rootUri":null}}'

# Calculate the length automatically
LENGTH=$(echo -n "$RPC" | wc -c)

# Send it to the compiled binary
echo -e "Content-Length: $LENGTH\r\n\r\n$RPC" | ./target/debug/rustimate-lsp

echo -e "\n\n--- Handshake Complete ---"
