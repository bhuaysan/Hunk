#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"

for required_tool in desktop-file-validate appstreamcli; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required metadata validator is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

desktop-file-validate "${REPOSITORY_DIRECTORY}/packaging/app.hunk.Hunk.desktop"

# A public homepage is intentionally added only during M8, after a public location is approved.
# AppStream 0.15 (the Ubuntu 22.04 baseline) predates per-tag severity overrides.
if appstreamcli validate --help 2>&1 | grep -q -- '--override'; then
    appstreamcli validate --no-net \
        --override=url-homepage-missing=pedantic \
        "${REPOSITORY_DIRECTORY}/packaging/app.hunk.Hunk.metainfo.xml"
else
    appstreamcli validate --no-net \
        "${REPOSITORY_DIRECTORY}/packaging/app.hunk.Hunk.metainfo.xml"
fi

bash -n \
    "${REPOSITORY_DIRECTORY}/scripts/build-chdman.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/build-linux-packages.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/mame-pin.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/prepare-mame-compliance.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/smoke-test-linux-package.sh"

printf 'Linux packaging metadata and scripts are valid.\n'
