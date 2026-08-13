# Approved `chdman` sidecar

Hunk uses the `chdman` tool from the official MAME `mame0289` source tag. The tag resolves to commit `f34f02505e32c1993c6a782b6814232cbfc74e36`, and Hunk accepts sidecar version `0.289` only.

The source archive is downloaded from GitHub by commit ID and checked against SHA-256 `17d50a6effe503e5cd23818daf42ee2a60f471d1cda41c13e0e7cc4ae78c5e11` before extraction. This prevents the build from following a moving release or unverified source archive.

## Linux build

The build needs Bash, curl, GNU Make, GCC 11 or newer, and the MAME Linux build dependencies. On Debian or Ubuntu, the minimal tools-only build currently needs at least `build-essential`, `python3`, and `libsdl2-dev`.

Run:

```sh
./scripts/build-chdman.sh
```

The script builds only the approved `chdman` target and writes the untracked Tauri sidecar to
`src-tauri/binaries/chdman-x86_64-unknown-linux-gnu`. It also retains the verified source archive and
copies MAME's `COPYING` and `docs/legal` files into the same ignored staging directory for Linux
package compliance. It refuses to replace any existing output. Set `HUNK_BUILD_JOBS` to limit
parallel compilation, or pass a destination directory as the first argument.

To run the opt-in integration check against the resulting executable:

```sh
HUNK_CHDMAN="$PWD/src-tauri/binaries/chdman-x86_64-unknown-linux-gnu" \
  cargo test --manifest-path src-tauri/Cargo.toml \
  --test chdman_process approved_real_binary_reports_required_capabilities -- --ignored
```

To generate tiny redistributable CD/DVD fixtures at test time and exercise create, verify, info,
extract, and recreate round trips through the full job engine:

```sh
HUNK_CHDMAN="$PWD/src-tauri/binaries/chdman-x86_64-unknown-linux-gnu" \
  cargo test --manifest-path src-tauri/Cargo.toml \
  --test end_to_end generated_fixture_round_trip_with_real_chdman -- --ignored --exact
```

The manually dispatched `Approved chdman` workflow runs both checks. No generated disc or CHD data
is retained as an artifact or committed to the repository.

## Runtime contract

Before its first operation, the backend checks both the exact version and the presence of `createcd`, `createdvd`, `extractcd`, `extractdvd`, `verify`, and `info`. Paths are passed directly as process arguments. Hunk never exposes `--force`, never offers `verify --fix`, and never treats unknown progress text as proof of completion.

## License and distribution

MAME is a separate work made available under the GNU General Public License version 2, with
individual source files and bundled components carrying the notices recorded in MAME's `COPYING` and
`docs/legal` directory. The build script does not copy a generated executable or MAME source into
Git. Each Linux artifact set ships the verified source archive alongside the packages, and both
installed packages include the license directory. Details are recorded in
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) and the
[Linux packaging guide](LINUX_PACKAGING.md).
