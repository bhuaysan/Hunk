# Release process

Releases are deliberate maintainer actions. CI and packaging workflows build artifacts but do not
create a GitHub release, publish packages, or push tags.

## 1. Prepare the tree

Start from a clean `main` branch. Confirm the implementation plan milestone is complete, update both
READMEs and user documentation, move changelog entries from `Unreleased` to the release version and
date, and align all version fields:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`;
- the version shown by the frontend; and
- `packaging/app.hunk.Hunk.metainfo.xml`.

Regenerate lockfiles only when dependency resolution intentionally changes. Review the complete diff
and use focused Conventional Commits.

## 2. Audit source and dependencies

Run the dependency/license procedure in [DEPENDENCIES.md](DEPENDENCIES.md). Then audit every reachable
Git object, not only the working tree:

```sh
git fsck --full
git log --all --format='%H %an <%ae>'
git log --all --name-only --format= | sort -u
git rev-list --objects --all | \
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | \
  sort -k3 -n
```

Inspect paths and blob contents for disc/ROM data, `Test/`, generated binaries/packages, logs,
databases, caches, secrets, tokens, private keys, private email addresses, personal paths, and private
tooling state. Stop and clean the reachable history before any public push if one is found.

## 3. Run validation

Run the complete normal suite from [DEVELOPMENT.md](DEVELOPMENT.md), both approved-sidecar ignored
tests from [CHDMAN.md](CHDMAN.md), and the explicitly invoked local representative-media harness when
the approved `Test/` set is available. Verify all source hashes before and after.

Build both Linux packages through the canonical Ubuntu 22.04 workflow. Smoke-test AppImage and
Flatpak under its X11 fallback, then complete the Fedora KDE/Wayland and Ubuntu 22.04 manual launch
and Flatpak portal checklists in [LINUX_PACKAGING.md](LINUX_PACKAGING.md).

## 4. Inspect artifacts

Verify package metadata, permissions, executable architecture, desktop integration, and checksums.
Inspect both package payloads and confirm they contain:

- Hunk's GPL license and third-party notice;
- the approved `chdman` executable;
- MAME's `COPYING`, complete `docs/legal`, pin record, sidecar documentation, and build recipe; and
- no caches, logs, databases, temporary outputs, local paths, or test media.

The release artifact set must contain exactly one AppImage, one Flatpak bundle, `SHA256SUMS`, and the
verified `mame-mame0289-source.tar.gz`. Keep all four together.

## 5. Tag and publish

Prepare an annotated tag only after the release commit passes every check:

```sh
git tag -a v0.1.0 -m 'Hunk 0.1.0'
git show --stat --decorate v0.1.0
```

Creating the local tag does not authorize pushing it. Obtain explicit approval before:

```sh
git push origin main
git push origin v0.1.0
```

After the tag is public, create a GitHub release from that exact tag and attach the four-artifact set.
Use the matching changelog section as release notes, identify the AppImage baseline and Flatpak host
filesystem permission, and link the security policy and third-party notices. Do not mark the release
complete until the public tag, source archive, packages, and checksums can all be downloaded and
verified independently.

## 6. Post-release

Restore an empty `Unreleased` section, verify installation from public downloads on both supported
test environments, and record any packaging or documentation correction as a new commit. Never
replace an existing published artifact silently; publish a new version when bits change.
