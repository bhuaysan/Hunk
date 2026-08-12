# Hunk roadmap

Hunk is a portable CHD workbench with Linux as its first release platform. This roadmap communicates direction, not a release-date promise. Detailed implementation decisions live in `docs/IMPLEMENTATION_PLAN.md`.

## 0.1 — Optical workbench

- Import files and folders through dialogs and drag and drop.
- Discover CUE/BIN, GDI, ISO, and CHD source sets.
- Create and extract CD/DVD CHDs.
- Inspect and verify CHDs.
- Run a safe serial batch queue with automatic verification and local history.
- Provide German and English interfaces.
- Ship Linux x86_64 Flatpak and AppImage builds.

## 0.2 — Advanced CHD media

- Hard-disk and raw-media images.
- Blank hard-disk templates.
- TOC and additional preservation-oriented optical workflows.
- Expanded batch reports and diagnostics.

## 0.3 — Relationships and metadata

- Parent and delta CHDs with explicit dependency validation.
- Carefully guarded metadata editing with backups.
- Compatibility profiles based on a maintained, tested matrix.

## Later — Collection tools

- Optional collection folders and multi-disc grouping.
- Search, storage analysis, and duplicate detection.
- Optional cover and online metadata integrations that remain opt-in.
- Windows, macOS, and Linux ARM64 releases when the core workflows are stable.

Hunk will continue to avoid telemetry and will never delete source images automatically.
