#!/usr/bin/env bash
# Optional helper that sets BINDGEN_EXTRA_CLANG_ARGS by querying the active
# arm-none-eabi-gcc for its libc header path. Only needed if the default in
# .cargo/config.toml (newlib at /usr/include/newlib, the Debian/Ubuntu layout)
# does not match your host:
#
#   source scripts/lvgl-env.sh
#   cargo build --bin lvgl_touch_can --features lvgl,touch
#
# Standard `apt install gcc-arm-none-eabi` users do NOT need this — plain
# `cargo build` works.

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "Source this script, do not execute it:" >&2
    echo "  source scripts/lvgl-env.sh" >&2
    exit 1
fi

set -u

find_gcc() {
    if [[ -n "${ARM_NONE_EABI_GCC:-}" && -x "${ARM_NONE_EABI_GCC}" ]]; then
        printf '%s\n' "${ARM_NONE_EABI_GCC}"
        return 0
    fi
    if command -v arm-none-eabi-gcc >/dev/null 2>&1; then
        command -v arm-none-eabi-gcc
        return 0
    fi
    local pio="${HOME}/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc"
    if [[ -x "${pio}" ]]; then
        printf '%s\n' "${pio}"
        return 0
    fi
    return 1
}

GCC="$(find_gcc)" || {
    echo "lvgl-env: arm-none-eabi-gcc not found." >&2
    echo "  Install gcc-arm-none-eabi (Debian/Ubuntu: apt install gcc-arm-none-eabi)," >&2
    echo "  or set ARM_NONE_EABI_GCC to your toolchain's arm-none-eabi-gcc." >&2
    return 1
}

GCC_DIR="$(dirname "${GCC}")"
GCC_VERSION="$("${GCC}" -dumpversion)"

# Look for any of the common ARM libc header locations in priority order. The
# first directory that contains string.h wins. Toolchain-relative paths come
# first so a self-contained ARM tarball / PlatformIO install resolves to its
# own bundled headers instead of accidentally picking up a system libc.
INCLUDE_DIR=""
for dir in \
    "${GCC_DIR}/../arm-none-eabi/include" \
    "${GCC_DIR}/../lib/gcc/arm-none-eabi/${GCC_VERSION}/include" \
    "${HOME}/.platformio/packages/toolchain-gccarmnoneeabi/arm-none-eabi/include" \
    "/usr/lib/arm-none-eabi/include" \
    "/usr/arm-none-eabi/include" \
    "/usr/include/newlib" \
    "/usr/lib/picolibc/arm-none-eabi/include"
do
    if [[ -f "${dir}/string.h" ]]; then
        INCLUDE_DIR="${dir}"
        break
    fi
done

if [[ -z "${INCLUDE_DIR}" ]]; then
    echo "lvgl-env: could not locate <string.h> for ${GCC}" >&2
    echo "  Debian/Ubuntu: sudo apt install libnewlib-dev   # or picolibc-arm-none-eabi" >&2
    echo "  ARM tarball / PlatformIO: ensure the toolchain ships an arm-none-eabi/include/ dir" >&2
    echo "  Or set BINDGEN_EXTRA_CLANG_ARGS yourself, e.g.:" >&2
    echo "    export BINDGEN_EXTRA_CLANG_ARGS='--target=thumbv8m.main-none-eabihf -mcpu=cortex-m33 -mthumb -isystem /path/to/include'" >&2
    return 1
fi

export BINDGEN_EXTRA_CLANG_ARGS="--target=thumbv8m.main-none-eabihf -mcpu=cortex-m33 -mthumb -isystem ${INCLUDE_DIR}"

echo "lvgl-env: GCC=${GCC}"
echo "lvgl-env: BINDGEN_EXTRA_CLANG_ARGS=${BINDGEN_EXTRA_CLANG_ARGS}"
