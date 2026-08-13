# Security policy

## Supported versions

Security fixes are developed on `main` and applied to the latest released 0.1.x version when a
release exists. Older development snapshots and locally modified packages are not supported.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use
[GitHub's private vulnerability reporting](https://github.com/bhuaysan/Hunk/security/advisories/new)
and include:

- the affected version or commit and package format;
- a clear reproduction using generated or freely redistributable data;
- the expected impact, especially any source modification, overwrite, path escape, or command
  execution risk; and
- suggested mitigations, if known.

Do not attach copyrighted disc images, credentials, private paths, or other sensitive user data.
Reports will be acknowledged as capacity permits. A fix and coordinated disclosure timeline depend
on severity, reproducibility, and upstream components; no fixed response deadline is promised.

## Security model

Hunk treats all imported descriptors, paths, CHDs, and `chdman` output as untrusted. The Rust backend
validates source-set containment and destination policy, launches the pinned sidecar without a
shell, and owns all filesystem and database access. The webview has only the Tauri permissions
listed in `src-tauri/capabilities/default.json`.

Hunk is designed to preserve source files and existing destinations even on cancellation, process
failure, malformed output, low space, and restart. A report that violates either invariant is
security-sensitive. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the complete boundary.

Official packages are expected to match the published checksums and contain MAME license/source
compliance material. Reports about a repackaged or modified build should first be directed to that
distributor unless the issue also affects Hunk's source.
