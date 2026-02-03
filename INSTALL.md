# Installation Guide

This document provides instructions for setting up the Nihilism game manually for development or production without using Docker.

## Manual Installation

### 1. Configure LLM API

Set environment variables for your OpenAI-compatible API:

```bash
export LLM_BASE_URL="http://localhost:8080/v1"  # Your LLM endpoint
export LLM_API_KEY="sk-your-api-key"            # API key
export LLM_MODEL="gpt-4"                        # Model name
```

### 2. Run the Server

You can run the server directly using Cargo:

```bash
cargo run --release
```

The server starts on port 3001 by default. If you want to use the pre-compiled binary:

```bash
cargo build --release
./target/release/nihilism
```

### 3. Run the Client (Development)

```bash
cd client
bun install
bun run dev
```

Open http://localhost:3000 to play.

### 4. Build for Production

```bash
# Build server
cargo build --release

# Build client
cd client && bun run build
```

---

## 🐳 Development

### Local Development (Build from source)

If you have made changes to the code and want to rebuild the container:

```bash
docker-compose up --build
```

To just start the existing build:

```bash
docker-compose up -d
```

### Docker Management

```bash
docker-compose pull       # Update image
docker-compose logs -f    # View logs
docker-compose down       # Stop
```

---