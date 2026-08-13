# Hunk

Hunk is a local desktop workbench for creating and managing optical CHD images used by emulators.
It discovers complete disc source sets, exposes only valid operations, and runs conversions through a
durable, collision-safe queue. Linux x86_64 is the first supported platform.

[Deutsche Version](README.de.md)

## Release status

The source tree is prepared for Hunk 0.1.0. AppImage and Flatpak packages will appear on the
[GitHub Releases page](https://github.com/bhuaysan/Hunk/releases) when a binary release is
published. If no package is available yet, use the development instructions below to run Hunk from
source.

## Features

- Import files and folders with native dialogs or drag and drop.
- Recursively discover CUE/BIN, GDI, ISO, and CHD source sets without duplicating referenced tracks.
- Create CD and DVD CHDs and verify every newly created CHD before publication.
- Extract CD CHDs to CUE/BIN and DVD CHDs to ISO.
- Inspect metadata and verify existing CHDs without modifying them.
- Pause, cancel, and retry work in a persistent serial queue with the latest 100 history records.
- Use the complete interface in English or German with light, dark, and reduced-motion support.

## Safety and privacy

Hunk never deletes or modifies source images and never overwrites an existing output. Mutating jobs
write a uniquely named temporary file on the destination filesystem, verify newly created CHDs, and
publish without replacement only after success. Hunk has no telemetry and needs no runtime network
access. Job history and settings stay in a backend-owned local SQLite database.

The Flatpak needs host filesystem access so descriptors can resolve sibling track files and output
can be written beside arbitrary sources. The webview still receives no shell or general filesystem
access. See the [architecture](docs/ARCHITECTURE.md) and
[Linux packaging guide](docs/LINUX_PACKAGING.md) for the complete boundary.

## Supported workflows

| Input          | Available operations                                | Default output                     |
| -------------- | --------------------------------------------------- | ---------------------------------- |
| CUE/BIN or GDI | Create CD CHD                                       | CHD beside the descriptor          |
| ISO            | Create CD or DVD CHD after an explicit media choice | CHD beside the ISO                 |
| CD CHD         | Extract CUE/BIN, inspect, or verify                 | One BIN and one CUE beside the CHD |
| DVD CHD        | Extract ISO, inspect, or verify                     | ISO beside the CHD                 |

CD extraction can optionally create one BIN per track. Hunk deliberately does not offer parent/delta
images, writable CHDs, metadata mutation, `verify --fix`, or automatic source cleanup in 0.1.

## Installing packages

Download the AppImage or Flatpak, `SHA256SUMS`, and `mame-mame0289-source.tar.gz` from the same
release. Keep the MAME source archive with redistributed packages and verify the downloads from the
directory containing them:

```sh
sha256sum --check SHA256SUMS
```

Run the AppImage:

```sh
chmod +x Hunk_0.1.0_amd64.AppImage
./Hunk_0.1.0_amd64.AppImage
```

Or install the Flatpak bundle for the current user:

```sh
flatpak --user install Hunk_x86_64.flatpak
flatpak run app.hunk.Hunk
```

The AppImage baseline is Ubuntu 22.04. The Flatpak uses GNOME runtime 50 and may download that
runtime during installation.

## Development quick start

Install the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for Linux, Rust 1.88
or newer, Node.js 22, and pnpm 11. Then run:

```sh
pnpm install --frozen-lockfile
pnpm tauri dev
```

The checked-in source does not contain `chdman`. Build the pinned MAME 0.289 sidecar before testing
real conversions:

```sh
./scripts/build-chdman.sh
```

See the [development guide](docs/DEVELOPMENT.md) for system packages, test tiers, generated fixtures,
and packaging. Never add disc images, local `Test/` data, binaries, packages, application state, or
credentials to the repository.

## Project documentation

- [Architecture and trust boundaries](docs/ARCHITECTURE.md)
- [Development and testing](docs/DEVELOPMENT.md)
- [Approved `chdman` sidecar](docs/CHDMAN.md)
- [Linux packaging](docs/LINUX_PACKAGING.md)
- [Dependency and license audit](docs/DEPENDENCIES.md)
- [Release process](docs/RELEASING.md)
- [Implementation specification](docs/IMPLEMENTATION_PLAN.md)
- [Roadmap](ROADMAP.md)

Contributions are welcome; read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.
Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

Hunk is free software licensed under the [GNU General Public License v3.0 or later](LICENSE). The
bundled MAME `chdman` sidecar and other dependencies retain their own licenses and notices; see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
