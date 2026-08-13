#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
# shellcheck source=./mame-pin.sh
source "${SCRIPT_DIRECTORY}/mame-pin.sh"

if [[ $# -ne 1 ]]; then
    printf 'Usage: %s OUTPUT_DIRECTORY\n' "${0}" >&2
    exit 2
fi

readonly OUTPUT_DIRECTORY="${1}"
readonly SOURCE_ARCHIVE="${OUTPUT_DIRECTORY}/mame-${MAME_TAG}-source.tar.gz"
readonly COMPLIANCE_DIRECTORY="${OUTPUT_DIRECTORY}/mame-${MAME_TAG}-compliance"

for required_tool in curl sha256sum tar install cp; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required compliance tool is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

for output_path in "${SOURCE_ARCHIVE}" "${COMPLIANCE_DIRECTORY}"; do
    if [[ -e "${output_path}" || -L "${output_path}" ]]; then
        printf 'Refusing to overwrite existing output: %s\n' "${output_path}" >&2
        exit 1
    fi
done

readonly WORK_DIRECTORY="$(mktemp -d)"
readonly DOWNLOADED_ARCHIVE="${WORK_DIRECTORY}/mame-${MAME_COMMIT}.tar.gz"
readonly EXTRACT_DIRECTORY="${WORK_DIRECTORY}/notices"
archive_published=false
compliance_published=false

cleanup() {
    readonly status=$?
    if [[ ${status} -ne 0 ]]; then
        if [[ "${compliance_published}" == true ]]; then
            rm -rf -- "${COMPLIANCE_DIRECTORY}"
        fi
        if [[ "${archive_published}" == true ]]; then
            rm -f -- "${SOURCE_ARCHIVE}"
        fi
    fi
    rm -rf -- "${WORK_DIRECTORY}"
}
trap cleanup EXIT

printf 'Downloading MAME compliance source %s (%s)\n' "${MAME_TAG}" "${MAME_COMMIT}"
curl --fail --location --proto '=https' --tlsv1.2 \
    --output "${DOWNLOADED_ARCHIVE}" "${MAME_ARCHIVE_URL}"
printf '%s  %s\n' "${MAME_ARCHIVE_SHA256}" "${DOWNLOADED_ARCHIVE}" |
    sha256sum --check --status

mkdir -- "${EXTRACT_DIRECTORY}"
tar --extract --gzip --file "${DOWNLOADED_ARCHIVE}" \
    --directory "${EXTRACT_DIRECTORY}" --strip-components=1 \
    "mame-${MAME_COMMIT}/COPYING" "mame-${MAME_COMMIT}/docs/legal"

mkdir --parents -- "${OUTPUT_DIRECTORY}"
install --mode 0644 -- "${DOWNLOADED_ARCHIVE}" "${SOURCE_ARCHIVE}"
archive_published=true
mkdir -- "${COMPLIANCE_DIRECTORY}"
compliance_published=true
install --mode 0644 -- "${EXTRACT_DIRECTORY}/COPYING" "${COMPLIANCE_DIRECTORY}/COPYING"
cp --archive -- "${EXTRACT_DIRECTORY}/docs" "${COMPLIANCE_DIRECTORY}/docs"
install --mode 0644 -- \
    "${SCRIPT_DIRECTORY}/build-chdman.sh" \
    "${COMPLIANCE_DIRECTORY}/build-chdman.sh"
install --mode 0644 -- \
    "${SCRIPT_DIRECTORY}/mame-pin.sh" \
    "${COMPLIANCE_DIRECTORY}/mame-pin.sh"
install --mode 0644 -- \
    "${REPOSITORY_DIRECTORY}/docs/CHDMAN.md" \
    "${COMPLIANCE_DIRECTORY}/CHDMAN.md"

printf 'Prepared MAME source and license bundle: %s\n' "${OUTPUT_DIRECTORY}"
