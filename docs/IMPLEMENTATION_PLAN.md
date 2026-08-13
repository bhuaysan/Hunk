# Hunk implementation plan

Status: accepted specification, M0 repository foundation and M1 source discovery completed.

This document is the canonical specification for Hunk. Update it when a milestone is completed or an accepted decision changes. `ROADMAP.md` is the shorter public overview; `CHANGELOG.md` records delivered behavior.

## 1. Product definition

Hunk is a modern, local desktop workbench for creating and managing CHD images used by emulators. Linux is the first release platform, while the application architecture remains portable.

### Release 0.1 goals

- Import individual files, multiple files, or folders by dialog and drag and drop.
- Recursively discover optical disc source sets without treating referenced track files as separate jobs.
- Support CUE/BIN, GDI, ISO, and CHD inputs.
- Create CD and DVD CHDs.
- Extract CD CHDs to CUE/BIN and DVD CHDs to ISO.
- Display CHD information and verify CHD integrity.
- Process a durable serial job queue with progress, cancellation, retry, and a local history of the latest 100 jobs.
- Provide complete German and English interfaces.
- Distribute Linux x86_64 builds as Flatpak and AppImage.

### Explicitly deferred

- Hard-disk, raw-media, LaserDisc, blank-image, and TOC workflows.
- Parent/delta CHDs, writable CHDs, and metadata mutation.
- Library cataloguing, covers, online metadata, duplicate detection, and emulator launching.
- Automatic source deletion or cleanup.
- Windows, macOS, and Linux ARM64 releases.

These capabilities should fit the core operation model later without being exposed as incomplete controls in 0.1.

## 2. Accepted product decisions

- The initial product is a tool workbench, not a game library.
- The default UI is simple; technical controls live in a collapsed advanced section.
- CUE and GDI determine CD mode automatically.
- ISO input always requires an explicit CD or DVD choice. File size must not be used as an authoritative heuristic.
- Output is created next to each source by default. A chosen destination directory mirrors relative source directories.
- Existing output is never overwritten. The user must rename the destination or skip the job.
- Every newly created CHD is fully verified before it becomes the final output.
- Source files are never modified or deleted.
- One job runs at a time. The active `chdman` process may use multiple CPU cores internally.
- CD extraction creates one BIN by default; split BIN per track is an advanced option.
- `verify --fix` is not offered because it mutates input.
- No runtime telemetry or network access is required.

## 3. Experience and visual design

The main experience is a smart workbench. Dropped or selected input is inspected first, then Hunk presents only the operations valid for that source.

### Main layout

- A compact navigation rail exposes Workbench and History.
- The workbench contains an import surface, source/job list, and contextual inspector.
- The inspector shows dependencies, tracks, media type, source size, destination, validation messages, and available actions.
- On narrow windows, the inspector becomes a drawer and primary actions remain visible without horizontal scrolling.
- History exposes status, timestamps, input/output paths, size savings, logs, retry, and removal from history.

### Signature element

A segmented track band represents data, audio, and subchannel areas. During processing, progress moves across the same band. Track types must also use labels or patterns so color is never the sole distinction.

### Design tokens

- Porcelain: `#F4F7F6`
- Basalt: `#17242B`
- Alloy: `#D9E2E1`
- Disc Blue: `#3F7CAC`
- Oxide Teal: `#2F8F83`
- Audio Amber: `#D59B45`
- Headings: Barlow Semi Condensed
- Interface text: Inter
- Hashes and logs: JetBrains Mono

Fonts must be redistributed only with their license files. The UI supports system-aware light and dark themes, visible keyboard focus, sufficient contrast, and reduced motion.

## 4. Technical architecture

### Stack

- Tauri 2 desktop shell.
- Rust backend for discovery, parsing, validation, job orchestration, process lifecycle, persistence, and filesystem mutations.
- Svelte 5, TypeScript, and Vite frontend.
- SQLite owned exclusively by the Rust backend.
- Custom design tokens and focused UI components instead of a large component framework.
- A pinned, bundled `chdman` executable built from an official MAME source tag during packaging.

The exact MAME tag is selected and recorded during the sidecar milestone after its license, Linux build, supported operations, and output format have passed integration tests. It must never float to an unpinned latest version.

### Core domain types

- `SourceSet`: primary file, referenced files, media kind, tracks, total size, and validation problems.
- `MediaKind`: `Cd`, `Dvd`, or `UnknownOptical` before an ISO choice.
- `Operation`: `CreateCd`, `CreateDvd`, `ExtractCd`, `ExtractDvd`, `Verify`, or `Info`.
- `JobSpec`: immutable source snapshot, operation, destination policy, and validated advanced options.
- `JobState`: `Queued`, `Preflight`, `Running`, `Verifying`, `Completed`, `Failed`, `Cancelled`, `Interrupted`, or `Blocked`.
- `JobProgress`: phase, optional percentage, processed bytes when available, elapsed time, and current message.
- `ChdInfo`: format version, media kind, codecs, logical/compressed sizes, ratio, hunk/unit data, hashes, tracks, and read-only metadata.

### Backend boundaries

- The frontend calls narrow typed commands for importing, scanning, queue actions, history, and settings.
- The frontend receives typed scan, progress, job-state, and queue-state events.
- The webview receives no general shell access and no unrestricted filesystem API.
- Rust constructs process argument arrays and never invokes a user-derived shell command.
- `chdman` version and expected capabilities are checked before the first operation.
- Unknown progress output falls back to indeterminate progress instead of guessing completion.

### Persistence

- SQLite stores preferences and the latest 100 job records.
- Logs are bounded and associated with job records.
- A job left active after a crash is marked `Interrupted` on the next start and requires manual retry.
- Source content, hashes of source content, covers, and library metadata are not indexed in 0.1.

## 5. File and process safety

Every mutating operation follows this sequence:

1. Validate source-set completeness and readability.
2. Validate destination policy, collisions, permissions, and available space.
3. Create a uniquely named Hunk-owned temporary output on the destination filesystem.
4. Run the appropriate `chdman` operation using an argument array.
5. For creation, run a full `chdman verify` against the temporary CHD.
6. Atomically rename the verified temporary output to the final destination.
7. Record the result and size information in history.

On failure or cancellation, Hunk may remove only temporary files it created and can positively identify. It must leave all sources and pre-existing destinations untouched. Closing the app with an active job requires confirmation.

CUE and GDI references are resolved relative to their descriptor directory. Both slash styles are accepted, while absolute paths, lexical parent escapes, and symlinks resolving outside that directory are validation errors. Recursive discovery does not follow directory symlinks.

## 6. Implementation milestones

Each milestone should produce a focused, public-ready Conventional Commit. Split a milestone only when necessary to keep commits reviewable and buildable.

### M0 — Repository foundation

- [x] Configure repository-local GitHub noreply commit identity before the first commit.
- [x] Add ignore rules covering local media, generated output, application state, and build artifacts.
- [x] Scaffold Tauri, Rust, Svelte, TypeScript, and Vite.
- [x] Add GPL-3.0-or-later licensing, lockfiles, base CI, and a launchable empty window.
- [x] Add English and German README entry points.

Target commit: `chore: initialize Hunk workspace`

### M1 — Source discovery

- [x] Implement recursive discovery and source-set de-duplication.
- [x] Parse CUE and GDI files without shell/path assumptions.
- [x] Detect ISO and CHD primary files.
- [x] Validate missing, duplicate, escaping, unreadable, and malformed references.
- [x] Cover spaces, brackets, Unicode, multiple BIN files, and mixed data/audio tracks.

Target commit: `feat(core): discover and validate optical images`

### M2 — chdman integration

- [ ] Pin and reproducibly build the approved official MAME source tag.
- [ ] Add version/capability checks.
- [ ] Build safe commands for all six 0.1 operations.
- [ ] Parse information, progress, verification results, and actionable errors.
- [ ] Add golden and process-level integration tests.

Target commit: `feat(core): integrate pinned chdman operations`

### M3 — Durable job engine

- [ ] Implement preflight, serial scheduling, pause/resume of queued work, cancellation, and retry.
- [ ] Implement temporary output, full verification, atomic publication, and cleanup invariants.
- [ ] Add SQLite settings and a 100-entry job history.
- [ ] Recover active jobs as interrupted after restart.

Target commit: `feat(core): add durable verified job queue`

### M4 — Workbench UI

- [ ] Build import, source list, contextual inspector, destination selection, and track band.
- [ ] Add create, extract, verify, and info flows.
- [ ] Add explicit ISO media selection and advanced options.
- [ ] Add queue and history views, conflict handling, logs, and human-readable errors.

Target commits:

- `feat(ui): build the Hunk workbench`
- `feat(ui): add conversion and management workflows`

### M5 — Localization and accessibility

- [ ] Complete German and English translation dictionaries.
- [ ] Localize dates, numbers, and sizes.
- [ ] Verify keyboard navigation, focus order, contrast, screen-reader labels, theme behavior, and reduced motion.

Target commit: `feat(app): add localization and accessibility`

### M6 — End-to-end validation

- [ ] Generate small redistributable fixtures at test time for CI.
- [ ] Provide an explicitly invoked local harness for the ignored `Test/` directory.
- [ ] Cover cancellation, crashes, low space, permissions, collisions, corrupt input, and malformed sidecar output.
- [ ] Confirm that no source is modified by success or failure paths.

Target commit: `test: add end-to-end conversion coverage`

### M7 — Linux packaging

- [ ] Produce Linux x86_64 Flatpak and AppImage artifacts.
- [ ] Add desktop entry, icons, AppStream metadata, third-party notices, and sidecar source/license compliance.
- [ ] Exercise Flatpak portals and document drag-and-drop limitations under sandboxing.
- [ ] Smoke-test Fedora KDE/Wayland and the selected oldest supported build baseline.

Target commit: `build: package Hunk for Linux`

### M8 — Public release preparation

- [ ] Complete README, contribution, security, architecture, development, and release documentation.
- [ ] Audit dependencies, bundled fonts, sidecar licenses, and source offers/notices.
- [ ] Audit the complete Git history for media, secrets, personal paths, generated binaries, and oversized files.
- [ ] Run all unit, integration, UI, packaging, and smoke tests.
- [ ] Align versions and move changelog entries from `Unreleased` to `0.1.0`.
- [ ] Prepare but do not push the `v0.1.0` tag without explicit approval.

Target commits:

- `docs: prepare public release documentation`
- `chore(release): prepare v0.1.0`

## 7. Test acceptance criteria

### Automated fixtures

- CUE/GDI parsing handles quoted names, Unicode, spaces, brackets, multi-file discs, and audio/data mixes.
- Missing or invalid references block a job before `chdman` starts.
- Command builders preserve arbitrary valid paths and reject invalid option combinations.
- Queue state remains consistent across pause, cancel, failure, retry, and restart.
- Temporary-file cleanup cannot target a source or pre-existing destination.
- Localization contains no missing keys and core workflows are keyboard-operable.

### Local representative data

The ignored local `Test/` directory currently provides three PlayStation source sets:

- Two independent Ace Combat single-track discs.
- One Castlevania mixed-mode disc with data and audio tracks stored in separate BIN files.

Acceptance requires discovery of exactly three source sets, successful creation and verification of all three CHDs, and a successful extract/recreate cycle for the mixed-mode disc. The tests must not modify or delete any file in `Test/`.

## 8. Git and publication policy

- `main` contains only commits suitable for later public push.
- `Cargo.lock` and `pnpm-lock.yaml` are tracked because Hunk is an application.
- ROM/disc data, local test data, sidecar binaries, packages, caches, logs, SQLite files, and private tooling state are never tracked.
- No remote or public repository is required during implementation.
- Before the first push, inspect tracked files, reachable history, object sizes, secrets, personal paths, licenses, package contents, and commit identity.
- Adding a remote, pushing, publishing packages, creating a repository, or creating a release always requires explicit user approval.

## 9. Documentation policy

- English is canonical for developer documentation.
- `README.md` and `README.de.md` provide equivalent English and German user entry points.
- This document contains accepted implementation detail.
- `ROADMAP.md` remains concise and user-facing.
- `CHANGELOG.md` contains delivered user-visible changes, not planned work.
- Architecture changes update this plan or the later architecture document in the same commit.
- User-visible behavior changes update `CHANGELOG.md` in the same commit.
- Installation or workflow changes update both README languages together.
- Markdown uses relative repository links and must not contain local absolute paths.
