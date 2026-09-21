# Legacy Python Reference

This directory is retained only for migration parity checks against the Rust
implementation.

- It is not started by `scripts/start.sh`.
- The React frontend does not call it.
- It is not an alternate production backend.
- It must never receive Longbridge or model-provider credentials.

All new API, analytics, Copilot, and safety work belongs in `rust-backend/`.
