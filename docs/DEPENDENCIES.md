# Dependency and license audit

This document records the release audit performed for Hunk 0.1.0 on 2026-08-13. Lockfiles are the
canonical exact inventory: `pnpm-lock.yaml` for the webview/build toolchain and
`src-tauri/Cargo.lock` for Rust.

## Inventory and license result

The Linux Rust graph resolved 303 packages, including Hunk, and every package declared a license or
license file. The installed pnpm graph contained 88 unique package/version pairs and every package
declared a license. License expressions were limited to GPL-3.0-or-later and compatible permissive or
weak-copyleft families including Apache-2.0, MIT, BSD, ISC, MPL-2.0, Unicode-3.0, Zlib, 0BSD, CC0,
MIT-0, and Unlicense alternatives.

Hunk does not redistribute web fonts. `Barlow Semi Condensed`, `Inter`, and `JetBrains Mono` appear
only as preferred CSS family names; the application uses an installed system font or the documented
generic fallback. No font files or font licenses are present in the source tree or packages.

The frontend bundle contains Svelte-generated application code and the Tauri JavaScript API needed
at runtime. Rust dependencies are linked into the native application where applicable. WebKitGTK and
other platform libraries remain supplied by the host/AppImage runtime or Flatpak runtime rather than
being Hunk source dependencies.

## Security audit

`pnpm audit --audit-level moderate` reported no known vulnerabilities. RustSec initially found
RUSTSEC-2026-0194 and RUSTSEC-2026-0195 in `quick-xml 0.38.4` plus RUSTSEC-2026-0009 in
`time 0.3.45`. The release lockfile updates `plist`, `quick-xml`, and `time` to fixed versions and
raises the Rust minimum to 1.88. A second `cargo audit` reported no vulnerabilities.

RustSec continues to report 17 non-vulnerability warnings in the Linux graph: the unmaintained
gtk-rs GTK3 bindings required transitively by Tauri/WebKitGTK, unmaintained build/parser helpers, and
RUSTSEC-2024-0429 for a `glib::VariantStrIter` API Hunk does not call. There is no patched
Tauri/WebKitGTK-compatible GTK3 line. These warnings are accepted for 0.1.0, remain subject to every
release audit, and must not be promoted to blanket ignores in CI.

Reproduce the audits with current advisory data:

```sh
pnpm audit --audit-level moderate
cargo audit --file src-tauri/Cargo.lock
```

Review every warning and its reverse dependency path with `cargo tree -i PACKAGE@VERSION`. A release
is blocked by any vulnerability; warnings require a documented reachability and migration review.

## MAME sidecar and source offer

Hunk bundles a separate `chdman` built from MAME tag `mame0289`, commit
`f34f02505e32c1993c6a782b6814232cbfc74e36`. The verified source archive, build recipe, pin record,
MAME `COPYING`, and complete `docs/legal` tree accompany official artifact sets. Both installed
package formats include the legal/compliance directory and Hunk's third-party notice.

The source archive checksum and redistribution details are in
[THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) and [CHDMAN.md](CHDMAN.md). The Hunk source for a
release is the corresponding public tag, including both lockfiles and reproducible build scripts.
Packages, checksums, the Hunk source tag, and the exact MAME source archive must remain available from
the same release location.

## Release audit procedure

For every release:

1. install both lockfiles without updates and enumerate all license expressions;
2. run npm and RustSec vulnerability audits with current databases;
3. inspect new license files, notices, native libraries, assets, and fonts;
4. verify the MAME pin, archive checksum, legal directory, and package contents;
5. confirm public source tags and package source offers identify the exact shipped revisions; and
6. record changed conclusions here or in the release notes.
