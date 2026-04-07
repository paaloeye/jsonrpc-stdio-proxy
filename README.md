# JSONRPC stdio proxy

## Features

- [x] single statically link binary
- [x] logging via [OSLog framework](https://developer.apple.com/documentation/oslog?language=objc)
- [x] CLI configuration
- [ ] Performance counters

## .mcp.json example

{
    "mcpServers": {
        "lldb": {
            "command": "jsonrpc-stdio-proxy"
            "args": [
            	"--subsystem",
            	"com.paaloeye.flight.engineer",
            	"--",
            	"/usr/bin/nc",
            	"localhost",
            	"59998"
            ]
        },
    }
}


## Apple OS log

```nu
log stream --predicate 'subsystem == "com.paaloeye.flight.engineer"' --debug --info

# or

log show --predicate 'subsystem == "com.paaloeye.flight.engineer"' --debug --info
```
