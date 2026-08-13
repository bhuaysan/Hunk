# Architecture

Hunk is a Tauri 2 desktop application with a Svelte 5 webview and a Rust backend. The architecture
keeps untrusted paths and process output out of the browser security boundary while preserving a
typed, event-driven user interface.

## Component boundaries

```text
Svelte workbench
  | typed invoke calls / typed events
  v
Tauri command boundary
  |-- discovery and descriptor validation
  |-- durable serial job engine ---- SQLite history and settings
  |-- safe command construction ---- pinned chdman child process
  `-- temporary output and collision-safe publication
```

The frontend may open native dialogs, receive desktop drag-and-drop paths, call the commands defined
in `src-tauri/src/lib.rs`, and listen for queue, job, progress, and close-request events. It has no
general shell API, filesystem API, SQL access, or arbitrary command bridge. The capability file
grants only Tauri core defaults and the native open dialog.

## Backend modules

| Area        | Responsibility                                                                                                         |
| ----------- | ---------------------------------------------------------------------------------------------------------------------- |
| `discovery` | Recursively find primary inputs, parse CUE/GDI, resolve dependencies, and reject escapes or malformed references.      |
| `domain`    | Shared serializable media, source, track, progress, and CHD information types.                                         |
| `chdman`    | Validate the pinned sidecar, build argument arrays, parse progress/info/verification, and classify errors.             |
| `jobs`      | Preflight, schedule one job at a time, cancel/retry, verify creation, publish output, bound logs, and persist records. |
| `lib.rs`    | Expose the narrow Tauri command/event surface and resolve packaged resources.                                          |

SQLite is opened only by `JobStore`. It stores preferences and at most the latest 100 terminal job
records. An active record found on startup becomes `Interrupted` and requires an explicit retry.

## Source and descriptor trust

Imported CUE and GDI references are resolved relative to the descriptor directory. Both slash styles
are accepted, but absolute references, lexical parent escapes, and symlinks resolving outside the
descriptor directory are rejected. Recursive discovery does not follow directory symlinks.
Referenced track files are dependencies of their descriptor, not separate jobs.

ISO media type is intentionally unresolved until the user chooses CD or DVD. Hunk does not infer it
from file size. CHD media type is obtained through `chdman info` where an operation requires it.

## Job lifecycle and publication

A mutating job moves through queued, preflight, running, and—when creating a CHD—verifying phases.
Preflight revalidates the immutable source snapshot, destination collision and permissions, available
space, and option combinations immediately before process launch.

Hunk writes only uniquely named temporary output on the destination filesystem. It passes paths as
individual process arguments and never offers force-overwrite or mutating verification flags. New
CHDs receive a full `chdman verify` before publication.

Publication creates a hard link at the final name and then removes the temporary name. Link creation
fails if the destination already exists, avoiding rename APIs that may replace files. Multi-file CD
extraction publishes BIN files before the CUE descriptor; a partial publication rolls back only the
links created by that job. Failure and cancellation cleanup is restricted to tokenized paths recorded
as owned by the job.

## Packaging boundary

AppImage bundles the application and pinned MAME 0.289 `chdman` sidecar. Flatpak adds an application
sandbox but grants the Rust backend host filesystem access because a selected descriptor alone does
not grant its sibling tracks or a writable adjacent destination. Native dialogs still use desktop
portals. This permission does not expand the webview capability set.

Hunk performs no telemetry, update checks, or other runtime network requests. Build and packaging
workflows use the network only to fetch locked dependencies, the pinned MAME source archive, and the
Flatpak runtime.

The accepted product details and invariants remain canonical in
[IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).
