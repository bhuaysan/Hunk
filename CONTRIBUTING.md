# Contributing to Hunk

Thank you for helping improve Hunk. English is the canonical language for developer documentation;
user-facing interface changes must remain complete in both English and German.

## Before starting

- Search existing issues and pull requests before proposing duplicate work.
- Discuss changes that alter the accepted product scope or safety model before implementation.
- Report vulnerabilities privately according to [SECURITY.md](SECURITY.md), not in a public issue.
- Read [docs/IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for the canonical specification and
  [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for trust boundaries.

## Development workflow

1. Follow the setup in [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md).
2. Create a focused branch and keep commits reviewable.
3. Use Conventional Commit subjects such as `fix(core): preserve destination on collision`.
4. Add tests and documentation with behavior changes. Add user-visible changes under `Unreleased`
   in [CHANGELOG.md](CHANGELOG.md).
5. Run the checks relevant to every changed Rust, TypeScript, Svelte, packaging, or Markdown file.

Pull requests should explain the user problem, the chosen behavior, safety implications, and the
checks performed. Keep unrelated refactors out of a behavior change.

## Non-negotiable safety rules

- Never delete or modify source disc images.
- Never overwrite an existing output.
- Pass user paths to child processes as individual arguments; do not construct shell commands.
- Keep filesystem, process, and SQLite access in Rust behind narrow typed Tauri commands.
- Remove only Hunk-owned temporary files that can be positively identified.
- Do not weaken descriptor path validation or follow directory symlinks during discovery.

Tests must use generated, redistributable fixtures. ROMs, disc images, local `Test/` contents,
generated `chdman` binaries, packages, logs, databases, credentials, and private tooling state must
never be committed.

## Pull request checklist

- The change matches the implementation plan or updates it with a documented reason.
- Source and pre-existing destination files remain untouched in success and failure paths.
- English and German user-facing copy stay in sync.
- Relevant automated tests pass and ignored real-media tests were run only when explicitly needed.
- README, architecture, packaging, security, and changelog documentation are updated where relevant.
- The diff contains no generated artifacts, secrets, personal paths, or unrelated formatting churn.

By contributing, you agree that your contribution is licensed under GPL-3.0-or-later, the same terms
as Hunk.
