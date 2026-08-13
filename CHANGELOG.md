# Changelog

All notable user-visible changes to Hunk will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases will follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Reproducible Linux x86_64 AppImage and Flatpak builds with desktop integration, AppStream metadata,
  portal guidance, Fedora KDE/Wayland and Ubuntu baseline smoke checks, and complete bundled
  `chdman` license/source compliance artifacts.
- A durable serial job engine with preflight checks, pause/resume, cancellation, retry, bounded logs, progress events, SQLite settings, and the latest 100 history records.
- Verified temporary CHD creation, collision-safe publication, safe cancellation cleanup, multi-file CD extraction ordering, and interrupted-job recovery after restart.
- A responsive optical workbench with native file/folder import, drag and drop, contextual source inspection, destination and advanced conversion controls, a patterned track band, prepared queue, and history views.
- Explicit CD/DVD selection for ISO sources, output-conflict feedback, validation guidance, and safe create, extract, verify, and information workflows connected to the durable job engine.
- Complete English and German interfaces with a persisted language choice and locale-aware dates, numbers, sizes, native dialogs, validation guidance, queue states, and history.
- Accessible keyboard and screen-reader behavior across navigation, the responsive source drawer, progress bands, and close confirmation, with verified light/dark contrast, visible focus, and reduced motion.
- Safe integration for the pinned MAME 0.289 `chdman`, including capability checks and structured information, progress, verification, and error parsing.
- Recursive CUE/BIN, GDI, ISO, and CHD source discovery with source-set de-duplication.
- Structured validation for missing, duplicate, escaping, unreadable, and malformed track references.
- Launchable Tauri 2 desktop foundation with a Svelte 5 and TypeScript interface.
- Automated formatting, type, frontend build, Rust lint, and Rust test checks.
- Equivalent English and German project entry points.
- Initial product, implementation, documentation, and release specification.
- End-to-end conversion validation with generated CD/DVD fixtures, deterministic safety fault coverage, approved-sidecar round trips, and an opt-in local representative-media harness.
