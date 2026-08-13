#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    printf 'Usage: %s --appimage|--flatpak PACKAGE\n' "${0}" >&2
    exit 2
fi

for required_tool in realpath timeout; do
    if ! command -v "${required_tool}" >/dev/null 2>&1; then
        printf 'Required smoke-test tool is missing: %s\n' "${required_tool}" >&2
        exit 1
    fi
done

readonly PACKAGE_KIND="${1}"
readonly PACKAGE_PATH="$(realpath -- "${2}")"
readonly SMOKE_SECONDS="${HUNK_SMOKE_SECONDS:-8}"
readonly SMOKE_DIRECTORY="$(mktemp -d)"
flatpak_test_installed=false

cleanup() {
    if [[ "${flatpak_test_installed}" == true ]]; then
        flatpak --user uninstall --noninteractive --delete-data app.hunk.Hunk >/dev/null
    fi
    rm -rf -- "${SMOKE_DIRECTORY}"
}
trap cleanup EXIT

if [[ ! -f "${PACKAGE_PATH}" ]]; then
    printf 'Package does not exist: %s\n' "${PACKAGE_PATH}" >&2
    exit 1
fi

run_with_timeout() {
    set +e
    timeout --signal=TERM "${SMOKE_SECONDS}" "${@}"
    local status=$?
    set -e
    if [[ ${status} -ne 0 && ${status} -ne 124 ]]; then
        printf 'Package exited unexpectedly with status %s.\n' "${status}" >&2
        return "${status}"
    fi
}

case "${PACKAGE_KIND}" in
    --appimage)
        APPIMAGE_EXTRACT_AND_RUN=1 \
            XDG_CACHE_HOME="${SMOKE_DIRECTORY}/cache" \
            XDG_CONFIG_HOME="${SMOKE_DIRECTORY}/config" \
            XDG_DATA_HOME="${SMOKE_DIRECTORY}/data" \
            run_with_timeout "${PACKAGE_PATH}"
        ;;
    --flatpak)
        if ! command -v flatpak >/dev/null 2>&1; then
            printf 'Required smoke-test tool is missing: flatpak\n' >&2
            exit 1
        fi
        if flatpak info app.hunk.Hunk >/dev/null 2>&1; then
            printf 'Refusing to disturb an existing Hunk Flatpak installation.\n' >&2
            exit 1
        fi
        flatpak --user install --noninteractive --no-deps "${PACKAGE_PATH}"
        flatpak_test_installed=true
        run_with_timeout flatpak run app.hunk.Hunk
        ;;
    *)
        printf 'Unknown package kind: %s\n' "${PACKAGE_KIND}" >&2
        exit 2
        ;;
esac

printf 'Package remained healthy for the %s-second launch smoke window.\n' "${SMOKE_SECONDS}"
