#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
readonly DEFAULT_CHDMAN="${REPOSITORY_DIRECTORY}/src-tauri/binaries/chdman-x86_64-unknown-linux-gnu"
readonly CHDMAN_PATH="${HUNK_CHDMAN:-${DEFAULT_CHDMAN}}"

if [[ ! -d "${REPOSITORY_DIRECTORY}/Test" ]]; then
    printf 'The ignored local Test/ directory is missing.\n' >&2
    exit 1
fi
if [[ ! -x "${CHDMAN_PATH}" ]]; then
    printf 'Approved chdman is missing or not executable: %s\n' "${CHDMAN_PATH}" >&2
    printf 'Build it with ./scripts/build-chdman.sh or set HUNK_CHDMAN.\n' >&2
    exit 1
fi

cd -- "${REPOSITORY_DIRECTORY}"
HUNK_CHDMAN="${CHDMAN_PATH}" cargo test --manifest-path src-tauri/Cargo.toml \
    --test end_to_end local_representative_media -- --ignored --exact --nocapture
