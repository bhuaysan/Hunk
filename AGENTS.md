# Hunk repository instructions

## Canonical project context

- Read `docs/IMPLEMENTATION_PLAN.md` before starting implementation work.
- Treat that document as the canonical product and implementation specification.
- Keep `ROADMAP.md` concise and user-facing; do not duplicate implementation details there.
- Record user-visible changes under `Unreleased` in `CHANGELOG.md`.
- If implementation requires changing an accepted decision, update the implementation plan in the same commit and explain the reason.

## Working agreements

- Keep every commit suitable for a future public repository.
- Use Conventional Commits with focused changes.
- Include relevant tests and documentation in the same commit as a behavior change.
- Run the checks relevant to changed Rust, TypeScript, Svelte, packaging, or Markdown files before committing.
- Preserve user files. Hunk must never delete or modify source disc images.
- Do not overwrite an existing output file.
- Pass paths to child processes as argument arrays; never interpolate user paths into shell commands.
- Keep the webview behind narrow typed Tauri commands. Do not grant it general shell or filesystem access.

## Files that must never be committed

- ROMs or disc images, including ISO, BIN, CUE, GDI, RAW, IMG, TOC, and CHD test data.
- The local `Test/` directory or any contents derived from it.
- Generated `chdman` binaries, application packages, build output, caches, logs, databases, crash dumps, or temporary conversion output.
- Tokens, credentials, `.env` files, personal paths, private notes, `.agents/`, or `.codex/`.

Use generated, redistributable fixtures for automated tests. The local `Test/` directory may only be used by explicitly invoked local end-to-end tests.

## External actions

- Do not add a Git remote, push commits, publish packages, create a public repository, or create a release without explicit user approval.
- Before the first public push, audit the complete reachable Git history, not only the working tree.
- Before the first commit, use a repository-local GitHub noreply email so a private email address is not embedded in public history.
