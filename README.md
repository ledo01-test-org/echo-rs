# echo-rs

A simple HTTP server that captures and displays incoming requests. Useful for testing webhooks.

## Usage

```
cargo run
```

- **Server**: http://127.0.0.1:8080
- **Dashboard**: http://127.0.0.1:8080/_dashboard

All requests (except `/_*` paths) are captured and displayed in the web dashboard with full headers and body.

## Features

- Real-time updates via SSE
- JSON body pretty-printing
- Request history (persists while server is running)
- Simple terminal access log
