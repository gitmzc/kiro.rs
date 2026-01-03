# Kiro-rs Project Overview

## Purpose
A Rust-based proxy service that provides an Anthropic Claude API compatible interface, translating requests to the internal Kiro API. It supports streaming (SSE), token management, and multi-credential failover.

## Tech Stack
- **Language:** Rust (2024 edition)
- **Web Framework:** Axum 0.8
- **Async Runtime:** Tokio
- **HTTP Client:** Reqwest (with stream, json, socks support)
- **Serialization:** Serde & Serde JSON
- **Logging:** Tracing
- **CLI Args:** Clap

## Key Features
- **Anthropic API Compatibility:** `/v1/messages`, `/v1/models`
- **Streaming:** SSE support
- **Token Management:** Automatic refresh of OAuth tokens
- **Failover:** Multi-credential support with priority
- **Tool Use:** Supports Anthropic function calling
- **Thinking Mode:** Supports Claude extended thinking

## Project Structure
- `src/main.rs`: Entry point
- `src/anthropic/`: Anthropic API layer (handlers, conversion, types)
- `src/kiro/`: Kiro API client, token manager, machine ID, AWS Event Stream parser
- `src/model/`: Config and Argument models
