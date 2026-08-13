# Hunk

Hunk is a modern, local desktop workbench for creating and managing CHD images used by emulators. Linux is the first release platform. The application is in active development and does not yet perform conversions.

[Deutsche Version](README.de.md)

## Current status

The repository foundation, source-discovery core, pinned `chdman` integration, and responsive workbench interface are complete. Hunk recursively recognizes CUE/BIN, GDI, ISO, and CHD inputs, groups referenced track files into source sets, and reports unsafe or incomplete descriptors before any conversion can start. The durable verified job engine is the next implementation milestone.

The accepted scope and safety model live in [the implementation plan](docs/IMPLEMENTATION_PLAN.md). The concise public direction is in [the roadmap](ROADMAP.md).

## Development

Install the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for Linux, Rust, Node.js 22, and pnpm 11. Then run:

```sh
pnpm install
pnpm tauri dev
```

Useful checks:

```sh
pnpm format:check
pnpm check
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Never add disc images, files from the local `Test/` directory, generated `chdman` binaries, packages, application state, or credentials to the repository. See [AGENTS.md](AGENTS.md) for the complete working agreements.

## License

Hunk is free software licensed under the [GNU General Public License v3.0 or later](LICENSE).
