#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./mame-pin.sh
source "${SCRIPT_DIRECTORY}/mame-pin.sh"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
readonly OUTPUT_DIRECTORY="${1:-${REPOSITORY_DIRECTORY}/src-tauri/binaries}"
readonly OUTPUT_BINARY="${OUTPUT_DIRECTORY}/chdman-x86_64-unknown-linux-gnu"
readonly OUTPUT_SOURCE_ARCHIVE="${OUTPUT_DIRECTORY}/mame-${MAME_TAG}-source.tar.gz"
readonly OUTPUT_COMPLIANCE_DIRECTORY="${OUTPUT_DIRECTORY}/mame-${MAME_TAG}-compliance"
readonly BUILD_JOBS="${HUNK_BUILD_JOBS:-$(getconf _NPROCESSORS_ONLN)}"

for required_tool in curl sha256sum tar make gcc g++ install; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required build tool is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

for output_path in \
    "${OUTPUT_BINARY}" \
    "${OUTPUT_SOURCE_ARCHIVE}" \
    "${OUTPUT_COMPLIANCE_DIRECTORY}"; do
    if [[ -e "${output_path}" || -L "${output_path}" ]]; then
        printf 'Refusing to overwrite existing output: %s\n' "${output_path}" >&2
        exit 1
    fi
done

readonly WORK_DIRECTORY="$(mktemp -d)"
readonly COMPLIANCE_OUTPUT_DIRECTORY="${WORK_DIRECTORY}/compliance"
readonly ARCHIVE_PATH="${COMPLIANCE_OUTPUT_DIRECTORY}/mame-${MAME_TAG}-source.tar.gz"
readonly SOURCE_DIRECTORY="${WORK_DIRECTORY}/mame"
trap 'rm -rf -- "${WORK_DIRECTORY}"' EXIT

"${SCRIPT_DIRECTORY}/prepare-mame-compliance.sh" "${COMPLIANCE_OUTPUT_DIRECTORY}"

mkdir -- "${SOURCE_DIRECTORY}"
tar --extract --gzip --file "${ARCHIVE_PATH}" \
    --directory "${SOURCE_DIRECTORY}" --strip-components=1

configuration_arguments=(
    REGENIE=1
    TOOLS=1
    EMULATOR=0
    NO_OPENGL=1
    USE_QTDEBUG=0
    NO_X11=1
    NO_USE_XINPUT=1
    NO_USE_MIDI=1
    NO_USE_PORTAUDIO=1
    NO_USE_PULSEAUDIO=1
    NO_USE_PIPEWIRE=1
    OVERRIDE_CC=gcc
    OVERRIDE_CXX=g++
)
readonly PROJECT_DIRECTORY="${SOURCE_DIRECTORY}/build/projects/sdl/mame/gmake-linux"
make --directory "${SOURCE_DIRECTORY}" "${configuration_arguments[@]}" \
    build/generated/version.cpp build/projects/sdl/mame/gmake-linux/Makefile
make --directory "${PROJECT_DIRECTORY}" config=release "-j${BUILD_JOBS}" chdman

mkdir --parents -- "${OUTPUT_DIRECTORY}"
install --mode 0644 -- "${ARCHIVE_PATH}" "${OUTPUT_SOURCE_ARCHIVE}"
cp --archive -- \
    "${COMPLIANCE_OUTPUT_DIRECTORY}/mame-${MAME_TAG}-compliance" \
    "${OUTPUT_COMPLIANCE_DIRECTORY}"
install --mode 0755 -- "${SOURCE_DIRECTORY}/chdman" "${OUTPUT_BINARY}"
printf 'Built approved chdman sidecar: %s\n' "${OUTPUT_BINARY}"
