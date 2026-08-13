# Approved `chdman` sidecar

Hunk uses the `chdman` tool from the official MAME `mame0289` source tag. The tag resolves to commit `f34f02505e32c1993c6a782b6814232cbfc74e36`, and Hunk accepts sidecar version `0.289` only.

The source archive is downloaded from GitHub by commit ID and checked against SHA-256 `17d50a6effe503e5cd23818daf42ee2a60f471d1cda41c13e0e7cc4ae78c5e11` before extraction. This prevents the build from following a moving release or unverified source archive.

## Linux build

The build needs Bash, curl, GNU Make, GCC 11 or newer, and the MAME Linux build dependencies. On Debian or Ubuntu, the minimal tools-only build currently needs at least `build-essential`, `python3`, and `libsdl2-dev`.

Run:

```sh
./scripts/build-chdman.sh
```

The script builds only the approved `chdman` target and writes the untracked Tauri sidecar to `src-tauri/binaries/chdman-x86_64-unknown-linux-gnu`. It refuses to replace an existing sidecar. Set `HUNK_BUILD_JOBS` to limit parallel compilation, or pass a destination directory as the first argument.

To run the opt-in integration check against the resulting executable:

```sh
HUNK_CHDMAN=src-tauri/binaries/chdman-x86_64-unknown-linux-gnu \
  cargo test --manifest-path src-tauri/Cargo.toml \
  --test chdman_process approved_real_binary_reports_required_capabilities -- --ignored
```

## Runtime contract

Before its first operation, the backend checks both the exact version and the presence of `createcd`, `createdvd`, `extractcd`, `extractdvd`, `verify`, and `info`. Paths are passed directly as process arguments. Hunk never exposes `--force`, never offers `verify --fix`, and never treats unknown progress text as proof of completion.

## License and distribution

MAME is a separate work made available under the GNU General Public License version 2, with individual source files and bundled components carrying the notices recorded in MAME's `COPYING` and `docs/legal` directory. The build script does not copy a generated executable or MAME source into Git. Linux packaging must ship the applicable MAME copyright and license texts and meet the corresponding source-code obligations; that compliance bundle remains part of milestone M7.
