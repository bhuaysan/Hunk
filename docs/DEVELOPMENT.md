# Development

## Prerequisites

Hunk development targets Linux x86_64 and requires:

- Rust 1.88 or newer with `rustfmt` and `clippy`;
- Node.js 22 and pnpm 11.9.0;
- the [Tauri 2 Linux prerequisites](https://v2.tauri.app/start/prerequisites/);
- WebKitGTK 4.1 and Ayatana AppIndicator development packages; and
- Bash plus normal C/C++ build tools when building `chdman` or packages.

On Debian or Ubuntu, the core native packages are typically:

```sh
sudo apt-get install build-essential libayatana-appindicator3-dev \
  librsvg2-dev libsdl2-dev libwebkit2gtk-4.1-dev python3
```

Packaging additionally needs `appstreamcli`, `desktop-file-validate`, `dpkg-deb`, `file`, Flatpak,
and either `flatpak-builder` or `org.flatpak.Builder`.

## Setup and launch

Install the locked frontend dependencies and start the Tauri application:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

The UI can be previewed with `pnpm dev`, but filesystem and queue operations require the Tauri
backend. Real conversions require the approved sidecar:

```sh
./scripts/build-chdman.sh
```

The build script downloads one exact MAME commit, verifies its SHA-256, builds only `chdman`, and
refuses to replace an existing output. Generated files remain under ignored directories.
Normal development, lint, and test builds do not require a placeholder sidecar; the package-only
Tauri configuration adds the external binary after the packaging script has verified it exists.

## Repository map

- `src/`: Svelte workbench, presentation logic, typed backend adapter, localization, and UI tests.
- `src-tauri/src/`: Rust domain, discovery, `chdman`, queue, persistence, and Tauri commands.
- `src-tauri/tests/`: generated fixtures and process/integration tests.
- `packaging/`: desktop, AppStream, Flatpak, and Tauri package metadata.
- `scripts/`: pinned sidecar build, compliance preparation, package validation/build, and smoke tests.
- `docs/`: canonical implementation, architecture, dependency, packaging, and release documentation.

## Checks

Run all normal checks from the repository root:

```sh
pnpm format:check
pnpm check
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
./scripts/check-linux-packaging.sh
```

Use `pnpm format` only for intentional mechanical formatting. Rust tests create fixtures in temporary
directories and include queue success, cancellation, collision, low-space, permission, crash,
malformed-output, and source-preservation paths.

## Sidecar and representative-data tests

After building the approved sidecar, run its capability and generated round-trip tests exactly as
documented in [CHDMAN.md](CHDMAN.md). The ignored local `Test/` directory is never part of normal CI.
It may be exercised only through:

```sh
./scripts/test-local-media.sh
```

That harness writes output to temporary storage and verifies every input again after processing. Do
not copy, rename, derive fixtures from, or commit any local representative media.

## Packaging

Validate metadata without building packages:

```sh
./scripts/check-linux-packaging.sh
```

Build the complete AppImage/Flatpak artifact set with:

```sh
./scripts/build-linux-packages.sh
```

The build refuses to reuse ambiguous or existing output. Package requirements, portal behavior, and
manual smoke tests are in [LINUX_PACKAGING.md](LINUX_PACKAGING.md).

## Change discipline

Keep Rust and TypeScript types aligned at the Tauri boundary. New filesystem or process behavior
belongs in Rust and should expose the narrowest useful command. Update both locales together, add
user-visible changes under `Unreleased`, and update the implementation plan when changing an
accepted decision. Use focused Conventional Commits and follow [CONTRIBUTING.md](../CONTRIBUTING.md).
