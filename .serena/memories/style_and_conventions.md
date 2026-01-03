# Code Style & Conventions

- **Language:** Rust
- **Formatting:** Standard Rust formatting (`cargo fmt`)
- **Linting:** Standard Clippy lints (`cargo clippy`)
- **Error Handling:** Uses `anyhow` for app-level errors.
- **Async:** Heavily relies on `tokio` and `async/await`.
- **API Design:** 
    - `src/anthropic` handles external API interface.
    - `src/kiro` handles internal backend communication.
    - Types are separated into `types.rs` or `model/` directories.
