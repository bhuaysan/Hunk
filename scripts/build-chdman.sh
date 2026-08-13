#!/usr/bin/env bash
set -euo pipefail

readonly MAME_TAG="mame0289"
readonly MAME_COMMIT="f34f02505e32c1993c6a782b6814232cbfc74e36"
readonly MAME_ARCHIVE_SHA256="17d50a6effe503e5cd23818daf42ee2a60f471d1cda41c13e0e7cc4ae78c5e11"
readonly MAME_ARCHIVE_URL="https://github.com/mamedev/mame/archive/${MAME_COMMIT}.tar.gz"
readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
readonly OUTPUT_DIRECTORY="${1:-${REPOSITORY_DIRECTORY}/src-tauri/binaries}"
readonly OUTPUT_BINARY="${OUTPUT_DIRECTORY}/chdman-x86_64-unknown-linux-gnu"
readonly BUILD_JOBS="${HUNK_BUILD_JOBS:-$(getconf _NPROCESSORS_ONLN)}"

for required_tool in curl sha256sum tar make gcc g++ install; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required build tool is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

if [[ -e "${OUTPUT_BINARY}" || -L "${OUTPUT_BINARY}" ]]; then
    printf 'Refusing to overwrite existing output: %s\n' "${OUTPUT_BINARY}" >&2
    exit 1
fi

readonly WORK_DIRECTORY="$(mktemp -d)"
readonly ARCHIVE_PATH="${WORK_DIRECTORY}/mame-${MAME_COMMIT}.tar.gz"
readonly SOURCE_DIRECTORY="${WORK_DIRECTORY}/mame"
trap 'rm -rf -- "${WORK_DIRECTORY}"' EXIT

printf 'Downloading MAME %s (%s)\n' "${MAME_TAG}" "${MAME_COMMIT}"
curl --fail --location --proto '=https' --tlsv1.2 \
    --output "${ARCHIVE_PATH}" "${MAME_ARCHIVE_URL}"
printf '%s  %s\n' "${MAME_ARCHIVE_SHA256}" "${ARCHIVE_PATH}" | sha256sum --check --status

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
install --mode 0755 -- "${SOURCE_DIRECTORY}/chdman" "${OUTPUT_BINARY}"
printf 'Built approved chdman sidecar: %s\n' "${OUTPUT_BINARY}"
