#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
readonly TAURI_CONFIGURATION="${REPOSITORY_DIRECTORY}/src-tauri/tauri.conf.json"

for required_tool in desktop-file-validate appstreamcli; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required metadata validator is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

desktop-file-validate "${REPOSITORY_DIRECTORY}/packaging/app.hunk.Hunk.desktop"

if ! grep -Fq \
    '"/usr/share/icons/hicolor/512x512/apps/app.hunk.Hunk.png": "icons/icon.png"' \
    "${TAURI_CONFIGURATION}"; then
    printf 'The AppImage desktop entry requires its matching app.hunk.Hunk icon.\n' >&2
    exit 1
fi

appstreamcli validate --no-net \
    "${REPOSITORY_DIRECTORY}/packaging/app.hunk.Hunk.metainfo.xml"

bash -n \
    "${REPOSITORY_DIRECTORY}/scripts/build-chdman.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/build-linux-packages.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/mame-pin.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/prepare-mame-compliance.sh" \
    "${REPOSITORY_DIRECTORY}/scripts/smoke-test-linux-package.sh"

printf 'Linux packaging metadata and scripts are valid.\n'
