#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIRECTORY="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPOSITORY_DIRECTORY="$(cd -- "${SCRIPT_DIRECTORY}/.." && pwd)"
# shellcheck source=./mame-pin.sh
source "${SCRIPT_DIRECTORY}/mame-pin.sh"

readonly SIDECAR_DIRECTORY="${REPOSITORY_DIRECTORY}/src-tauri/binaries"
readonly SIDECAR_BINARY="${SIDECAR_DIRECTORY}/chdman-x86_64-unknown-linux-gnu"
readonly SOURCE_ARCHIVE="${SIDECAR_DIRECTORY}/mame-${MAME_TAG}-source.tar.gz"
readonly COMPLIANCE_DIRECTORY="${SIDECAR_DIRECTORY}/mame-${MAME_TAG}-compliance"
readonly BUNDLE_DIRECTORY="${REPOSITORY_DIRECTORY}/src-tauri/target/release/bundle"
readonly DEB_DIRECTORY="${BUNDLE_DIRECTORY}/deb"
readonly APPIMAGE_DIRECTORY="${BUNDLE_DIRECTORY}/appimage"
readonly FLATPAK_OUTPUT_DIRECTORY="${BUNDLE_DIRECTORY}/flatpak"
readonly FLATPAK_OUTPUT="${FLATPAK_OUTPUT_DIRECTORY}/Hunk_x86_64.flatpak"
readonly CHECKSUM_FILE="${BUNDLE_DIRECTORY}/SHA256SUMS"
readonly FLATPAK_PAYLOAD="${REPOSITORY_DIRECTORY}/packaging/hunk-linux.tar.gz"
readonly FLATPAK_MANIFEST="${REPOSITORY_DIRECTORY}/packaging/app.hunk.Hunk.yml"

for required_tool in dpkg-deb file find flatpak grep pnpm sed sha256sum tar uname; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required packaging tool is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

if command -v flatpak-builder >/dev/null 2>&1; then
    if ! command -v eu-strip >/dev/null 2>&1; then
        printf 'Native flatpak-builder requires eu-strip from elfutils.\n' >&2
        exit 1
    fi
    flatpak_builder=(flatpak-builder)
elif flatpak info org.flatpak.Builder >/dev/null 2>&1; then
    flatpak_builder=(
        flatpak run
        "--filesystem=${REPOSITORY_DIRECTORY}"
        org.flatpak.Builder
    )
else
    printf 'flatpak-builder or the org.flatpak.Builder Flatpak is required.\n' >&2
    exit 1
fi

if [[ "$(uname -m)" != "x86_64" ]]; then
    printf 'Linux packages must be built natively on x86_64.\n' >&2
    exit 1
fi

sidecar_exists=false
source_exists=false
compliance_exists=false
[[ -f "${SIDECAR_BINARY}" ]] && sidecar_exists=true
[[ -f "${SOURCE_ARCHIVE}" ]] && source_exists=true
[[ -d "${COMPLIANCE_DIRECTORY}" ]] && compliance_exists=true

if [[ "${sidecar_exists}" == false && "${source_exists}" == false && "${compliance_exists}" == false ]]; then
    "${SCRIPT_DIRECTORY}/build-chdman.sh"
elif [[ "${sidecar_exists}" == true && "${source_exists}" == false && "${compliance_exists}" == false ]]; then
    "${SCRIPT_DIRECTORY}/prepare-mame-compliance.sh" "${SIDECAR_DIRECTORY}"
elif [[ "${sidecar_exists}" != true || "${source_exists}" != true || "${compliance_exists}" != true ]]; then
    printf 'The generated sidecar, source archive, and license directory are incomplete.\n' >&2
    printf 'Refusing to replace or combine an ambiguous generated bundle in %s.\n' \
        "${SIDECAR_DIRECTORY}" >&2
    exit 1
fi

if [[ -e "${FLATPAK_PAYLOAD}" || -L "${FLATPAK_PAYLOAD}" ]]; then
    printf 'Refusing to overwrite existing Flatpak payload: %s\n' "${FLATPAK_PAYLOAD}" >&2
    exit 1
fi
if [[ -e "${FLATPAK_OUTPUT}" || -L "${FLATPAK_OUTPUT}" ]]; then
    printf 'Refusing to overwrite existing Flatpak: %s\n' "${FLATPAK_OUTPUT}" >&2
    exit 1
fi
if [[ -e "${CHECKSUM_FILE}" || -L "${CHECKSUM_FILE}" ]]; then
    printf 'Refusing to overwrite existing checksum file: %s\n' "${CHECKSUM_FILE}" >&2
    exit 1
fi
if find "${APPIMAGE_DIRECTORY}" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null | grep -q .; then
    printf 'Refusing to reuse a non-empty AppImage output directory: %s.\n' \
        "${APPIMAGE_DIRECTORY}" >&2
    exit 1
fi
if find "${DEB_DIRECTORY}" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null | grep -q .; then
    printf 'Refusing to reuse a non-empty Debian staging directory: %s.\n' "${DEB_DIRECTORY}" >&2
    exit 1
fi

readonly WORK_DIRECTORY="$(mktemp -d)"
cleanup() {
    rm -rf -- "${WORK_DIRECTORY}"
    rm -f -- "${FLATPAK_PAYLOAD}"
}
trap cleanup EXIT

"${SCRIPT_DIRECTORY}/check-linux-packaging.sh"
mkdir --parents -- "${REPOSITORY_DIRECTORY}/src-tauri/target/ccache"
CCACHE_DIR="${REPOSITORY_DIRECTORY}/src-tauri/target/ccache" \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    LDAI_NO_APPSTREAM=1 \
    NO_STRIP=1 \
    XDG_CACHE_HOME="${REPOSITORY_DIRECTORY}/src-tauri/target/xdg-cache" \
    pnpm --dir "${REPOSITORY_DIRECTORY}" tauri build \
        --config "${REPOSITORY_DIRECTORY}/src-tauri/tauri.bundle.conf.json" \
        --bundles appimage,deb

mapfile -t deb_packages < <(find "${DEB_DIRECTORY}" -maxdepth 1 -type f -name '*.deb' -print)
if [[ ${#deb_packages[@]} -ne 1 ]]; then
    printf 'Expected exactly one generated Debian staging package, found %s.\n' \
        "${#deb_packages[@]}" >&2
    exit 1
fi

mkdir -- "${WORK_DIRECTORY}/payload"
dpkg-deb --extract "${deb_packages[0]}" "${WORK_DIRECTORY}/payload"
tar --create --gzip --file "${FLATPAK_PAYLOAD}" \
    --directory "${WORK_DIRECTORY}/payload" usr

"${flatpak_builder[@]}" --user --disable-cache --force-clean \
    --repo="${WORK_DIRECTORY}/repository" \
    "${WORK_DIRECTORY}/build" "${FLATPAK_MANIFEST}"
mkdir --parents -- "${FLATPAK_OUTPUT_DIRECTORY}"
flatpak build-bundle \
    --runtime-repo="https://flathub.org/repo/flathub.flatpakrepo" \
    "${WORK_DIRECTORY}/repository" "${FLATPAK_OUTPUT}" app.hunk.Hunk

mapfile -t appimages < <(find "${APPIMAGE_DIRECTORY}" -maxdepth 1 -type f -name '*.AppImage' -print)
if [[ ${#appimages[@]} -ne 1 ]]; then
    printf 'Expected exactly one generated AppImage, found %s.\n' "${#appimages[@]}" >&2
    exit 1
fi

sha256sum "${appimages[0]}" "${FLATPAK_OUTPUT}" "${SOURCE_ARCHIVE}" |
    sed "s#${REPOSITORY_DIRECTORY}/##" >"${CHECKSUM_FILE}"

printf 'Built AppImage: %s\n' "${appimages[0]}"
printf 'Built Flatpak: %s\n' "${FLATPAK_OUTPUT}"
printf 'Accompanying sidecar source: %s\n' "${SOURCE_ARCHIVE}"
