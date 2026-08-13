# Hunk

Hunk is a modern, local desktop workbench for creating and managing CHD images used by emulators. Linux is the first release platform. The application is in active development.

[Deutsche Version](README.de.md)

## Current status

The repository foundation, source discovery, pinned `chdman` integration, durable verified job engine, responsive localized workbench, accessibility pass, and end-to-end validation are complete. Hunk recursively recognizes CUE/BIN, GDI, ISO, and CHD inputs, groups referenced track files into source sets, processes them through a collision-safe serial queue, and retains the latest 100 job records locally. Linux packaging is the next implementation milestone.

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

### End-to-end validation

The regular Rust suite creates small deterministic CD/DVD fixtures in temporary directories and
uses a controllable sidecar double to cover successful round trips and safety failures. The opt-in
real-sidecar check is documented in [docs/CHDMAN.md](docs/CHDMAN.md).

After building the approved sidecar, the ignored local `Test/` data can be validated explicitly:

```sh
./scripts/test-local-media.sh
```

The harness expects exactly the three representative source sets described in the implementation
plan, writes every output to temporary storage, and confirms that all source files remain unchanged.

## License

Hunk is free software licensed under the [GNU General Public License v3.0 or later](LICENSE).
