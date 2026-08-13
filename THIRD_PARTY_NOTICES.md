# Third-party notices

Hunk bundles a separately built `chdman` executable from MAME. Hunk's own license does not replace
the notices and licenses that apply to MAME and its bundled components.

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
