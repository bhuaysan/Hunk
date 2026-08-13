# Linux packaging

Hunk produces x86_64 AppImage and Flatpak artifacts. Packaging is deliberately separate from a
release: the manually dispatched `Linux packages` workflow builds and tests artifacts but never
creates a release or publishes a package.

## Supported baseline and runtimes

- AppImage is built on Ubuntu 22.04, the selected oldest supported native baseline. Tauri 2 requires
  WebKitGTK 4.1, and building on an older glibc baseline keeps the resulting AppImage usable on newer
  distributions.
- Flatpak targets `org.gnome.Platform//50` and is built with the matching SDK. Its runtime supplies
  WebKitGTK independently of the host distribution.
- Linux packages are built natively for x86_64. Cross-built AppImages are not supported.

The `Linux packages` workflow is the canonical reproducible build. It installs pinned Node and pnpm
major versions, uses the Rust lockfile, builds the approved MAME 0.289 sidecar from its verified
commit archive, and runs both packages under an X11 fallback display before retaining them as one
workflow artifact. The build disables `linuxdeploy` stripping because its embedded older binutils
cannot process modern RELR sections on current Fedora hosts; Hunk's Rust release profile is already
configured to strip the application binary. AppImage's duplicate automatic AppStream check is
disabled while the canonical metadata is validated explicitly before every package build.

## Local package build

Install the normal development requirements plus `appstreamcli`, `desktop-file-validate`, `dpkg-deb`,
`file`, and `flatpak`. The canonical build uses the sandboxed `org.flatpak.Builder`; a
distribution-provided `flatpak-builder` is a fallback only when it is compatible with the selected
SDK. The manifest disables a separate debug extension because the copied release binaries are
already stripped. Install the builder, Flatpak runtime, and SDK:

```sh
flatpak install --user flathub org.flatpak.Builder org.gnome.Platform//50 org.gnome.Sdk//50
```

Keep these components in Flatpak's default user installation. The packaging script exposes that
installation read-only to the sandboxed builder so the pinned SDK remains discoverable on older
Flatpak hosts such as Ubuntu 22.04.

Then run:

```sh
./scripts/build-linux-packages.sh
```

The script refuses to replace an existing AppImage, Flatpak, Flatpak staging payload, sidecar,
source archive, or ambiguous partial compliance bundle. Generated files remain below
`src-tauri/target/` and `src-tauri/binaries/`, both of which are ignored by Git.

The output set contains:

- `bundle/appimage/*.AppImage`;
- `bundle/flatpak/Hunk_x86_64.flatpak`;
- `bundle/SHA256SUMS` covering both packages and the sidecar source archive;
- `src-tauri/binaries/mame-mame0289-source.tar.gz`; and
- the MAME `COPYING` and `docs/legal` files plus Hunk's pin, documentation, and build recipe installed
  into both packages.

Keep the source archive and package artifacts together when redistributing Hunk. See
[`THIRD_PARTY_NOTICES.md`](../THIRD_PARTY_NOTICES.md) for the exact upstream pin and license scope.

## Flatpak portals and filesystem access

The Tauri dialog plugin uses the desktop file chooser, which Flatpak routes through XDG Desktop
Portal. Hunk's webview still has no shell or filesystem API; imported paths go to narrow Rust
commands. The Flatpak grants the backend `--filesystem=host` because Hunk must resolve all tracks
referenced by CUE/GDI descriptors and publish output beside an arbitrary source. A grant for only the
selected document cannot reliably cover those sibling dependencies or the default output policy.

This broad filesystem permission is intentional and should be revisited if document portals gain a
way to grant complete source sets and writable sibling destinations. Hunk's own safety rules remain
in force: source files are never modified or deleted, existing output is never replaced, and only
positively identified temporary output may be cleaned up.

Flathub's manifest linter reports `finish-args-host-filesystem-access` for this permission. That is
the single expected packaging exception; it reflects Hunk's core local-file workflow and must be
explained during any later Flathub submission rather than hidden or weakened during artifact builds.

Drag and drop has these sandbox-specific limits:

- A desktop may provide a document-portal path for only the dropped descriptor. Its referenced track
  files may not be exported at matching relative paths. If validation reports missing dependencies,
  import the containing folder through the folder dialog.
- Remote or virtual files that are exposed as non-local URIs cannot be queued until the desktop
  portal provides a local path.
- Directory symlinks are never followed, inside or outside Flatpak.

Portal exercise checklist:

1. Start the installed Flatpak from the desktop menu under Wayland.
2. Import one file and one folder with the native dialogs; verify cancel returns without adding a
   source.
3. Choose a destination folder, queue a generated test fixture, and verify the source remains
   byte-identical.
4. Drop a local descriptor and a folder from the KDE file manager; confirm valid local paths import
   and a portal-only dependency failure remains actionable.
5. Repeat with the desktop portal service unavailable and confirm Hunk reports the dialog failure
   without gaining direct webview filesystem access.

## Smoke tests

For a graphical session, launch each generated artifact with the bounded smoke helper:

```sh
./scripts/smoke-test-linux-package.sh --appimage \
  src-tauri/target/release/bundle/appimage/Hunk_0.1.0_amd64.AppImage
./scripts/smoke-test-linux-package.sh --flatpak \
  src-tauri/target/release/bundle/flatpak/Hunk_x86_64.flatpak
```

The helper uses an isolated Flatpak user installation and removes it after the launch window. The
release checklist requires successful runs on Fedora KDE/Wayland and Ubuntu 22.04 (the package build
baseline), plus the portal exercise above.
