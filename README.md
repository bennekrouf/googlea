
# Google Calendar CLI Tool

A command-line interface tool for managing Google Calendar events, built with Rust.

## Features

- OAuth2 authentication with Google Calendar API
- Create calendar events from the command line
- Secure token storage using Sled database
- Automatic token refresh handling

## Prerequisites

- Rust and Cargo installed
- Google Cloud Project with Calendar API enabled
- OAuth 2.0 Client credentials (Client ID and Client Secret)

## Setup

1. Create a `.env` file in the project root:
```env
GOOGLE_CLIENT_ID=your_client_id_here
GOOGLE_CLIENT_SECRET=your_client_secret_here
OAUTH_CALLBACK_PORT=8080
```
or

Copy `.env.example` to `.env`:
```bash
cp .env.example .env


2. Build the project:
```bash
cargo build
```

## Usage

### First-time Authentication
```bash
cargo run -- auth
```
Follow the browser prompts to authenticate with your Google account.

### Create an Event
```bash
cargo run -- create-event "Your event description"
```

### Debug Mode
For detailed logging, run with:
```bash
RUST_LOG=debug cargo run -- create-event "Your event description"
```

## Implementation Details

- Uses `google-calendar3` for Calendar API integration
- Token management with Sled embedded database
- Hyper for HTTP client functionality
- Tokio for async runtime
- OAuth2 authentication with automatic refresh token handling

## Development

To run tests:
```bash
cargo test
```

## License

MIT License

Copyright (c) 2024 Mayorana

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
