# Third-party notices

Hunk includes open-source Rust and JavaScript dependencies and bundles a separately built `chdman`
executable from MAME. Hunk's own license does not replace the notices and licenses that apply to
those works.

## Application dependencies

The exact Rust and JavaScript package versions are recorded in `src-tauri/Cargo.lock` and
`pnpm-lock.yaml`. Their declared licenses were audited for Hunk 0.1.0 and are compatible with
distribution under GPL-3.0-or-later. They include Apache-2.0, MIT, BSD, ISC, MPL-2.0, Unicode-3.0,
Zlib, 0BSD, CC0, MIT-0, and Unlicense alternatives. Copyright and license notices remain with the
corresponding upstream package sources.

The complete audit scope, security results, accepted transitive warnings, and reproduction procedure
are documented in [`docs/DEPENDENCIES.md`](docs/DEPENDENCIES.md). The public Hunk source tag and its
lockfiles identify the exact application sources corresponding to an official binary release.

Hunk does not bundle fonts. Barlow Semi Condensed, Inter, and JetBrains Mono are optional CSS family
preferences only; installed system fonts or generic fallbacks are used at runtime.

## MAME `chdman`

- Upstream: MAMEdev, [mamedev/mame](https://github.com/mamedev/mame)
- Approved source tag: `mame0289`
- Approved source commit: `f34f02505e32c1993c6a782b6814232cbfc74e36`
- Source archive SHA-256: `17d50a6effe503e5cd23818daf42ee2a60f471d1cda41c13e0e7cc4ae78c5e11`
- Primary license: GNU General Public License version 2; individual files and bundled components
  may carry the less restrictive licenses identified by their source headers and MAME's
  `docs/legal` directory.

MAME is Copyright © 1997–2026 MAMEdev and contributors. MAME is a registered trademark of Gregory
Ember. Other trademarks are the property of their respective owners. The software is provided
without warranty, including without implied warranties of merchantability or fitness for a
particular purpose.

Every official Linux artifact set contains the exact verified MAME source archive used for the
sidecar as `mame-mame0289-source.tar.gz`. The installed package also contains MAME's `COPYING` file
and full `docs/legal` directory, Hunk's sidecar documentation, pin record, and reproducible build
recipe under its `share/licenses/hunk/mame` directory. The repository copy of the recipe is
[`scripts/build-chdman.sh`](scripts/build-chdman.sh).

Recipients who obtained a Hunk package without its accompanying source archive can retrieve the
exact archive from the commit URL recorded above and verify it with the published SHA-256, or obtain
the archive from the same place that supplied the Hunk package.
