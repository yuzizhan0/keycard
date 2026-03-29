# Contributing

## Reproducible builds

- Pin Rust via `rust-toolchain.toml` if you introduce one; document MSRV in `keycard-core` when you raise it.
- Commit `Cargo.lock` for application crates (workspace root lockfile covers members).

## Running tests

```bash
cargo test -p keycard-core
cargo test -p keycard-cli
```

CLI integration tests require the `keycard` binary to be built (`cargo test -p keycard-cli` builds it for `assert_cmd`).

## Tauri

Develop from `apps/keycard-app`. Capabilities live in `src-tauri/capabilities/`; keep permissions minimal when adding features.
