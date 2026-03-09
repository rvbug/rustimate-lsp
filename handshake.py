import subprocess
import json

# This is the message the Editor sends to your Rust LSP
message = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "capabilities": {},
        "processId": None,
        "rootUri": None
    }
}

# Convert to JSON string
content = json.dumps(message)
# LSP Requirement: Content-Length: <bytes>\r\n\r\n<json>
rpc_frame = f"Content-Length: {len(content)}\r\n\r\n{content}"

# Start your Rust binary and shove the message into it
process = subprocess.Popen(
    ['cargo', 'run', '-q'], 
    stdin=subprocess.PIPE, 
    stdout=subprocess.PIPE, 
    stderr=subprocess.PIPE,
    text=True
)

stdout, stderr = process.communicate(input=rpc_frame)

print("--- LSP RESPONSE ---")
print(stdout)
