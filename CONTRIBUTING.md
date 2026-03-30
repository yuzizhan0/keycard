# Contributing

Thanks for helping improve Keycard. For license and open-source workflow context, see [`docs/OPEN_SOURCE.md`](docs/OPEN_SOURCE.md).  
To report security issues privately, see [`SECURITY.md`](SECURITY.md).

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

## CLI (cross-platform)

End-user setup references: [docs/cli-setup-macos.md](docs/cli-setup-macos.md), [docs/cli-setup-windows.md](docs/cli-setup-windows.md). Update these when changing default paths, `env` output format, or install steps.

## Community

- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- Issue / PR templates: [.github/ISSUE_TEMPLATE/](.github/ISSUE_TEMPLATE/), [.github/pull_request_template.md](.github/pull_request_template.md)
