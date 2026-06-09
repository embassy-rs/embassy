#!/usr/bin/env bash
# Source before building LVGL examples:
#   source scripts/lvgl-env.sh
#   cargo build --bin lvgl_touch_can --features lvgl,touch
#
# lvgl-sys uses bindgen/clang to parse LVGL headers. Clang needs the ARM
# newlib/picolibc include path (string.h, stddef.h, …).

set -euo pipefail

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "Source this script, do not execute it:" >&2
    echo "  source scripts/lvgl-env.sh" >&2
    exit 1
fi

find_gcc() {
    if [[ -n "${ARM_NONE_EABI_GCC:-}" && -x "${ARM_NONE_EABI_GCC}" ]]; then
        echo "${ARM_NONE_EABI_GCC}"
        return 0
    fi
    if command -v arm-none-eabi-gcc >/dev/null 2>&1; then
        command -v arm-none-eabi-gcc
        return 0
    fi
    local pio="${HOME}/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc"
    if [[ -x "${pio}" ]]; then
        echo "${pio}"
        return 0
    fi
    return 1
}

add_isystem_if_string_h() {
    local dir="$1"
    if [[ -n "${dir}" && -f "${dir}/string.h" ]]; then
        BINDGEN_INCLUDES+=("-isystem" "${dir}")
    fi
}

GCC="$(find_gcc)" || {
    echo "lvgl-env: arm-none-eabi-gcc not found." >&2
    echo "  Install gcc-arm-none-eabi (+ picolibc-arm-none-eabi on Debian/Ubuntu)," >&2
    echo "  or set ARM_NONE_EABI_GCC to your toolchain's arm-none-eabi-gcc." >&2
    return 1
}

GCC_DIR="$(dirname "${GCC}")"
GCC_VERSION="$("${GCC}" -dumpversion)"
STRING_H="$("${GCC}" -print-file-name=include/string.h)"

BINDGEN_INCLUDES=(
    "--target=thumbv8m.main-none-eabihf"
    "-mcpu=cortex-m33"
    "-mthumb"
)

if [[ -f "${STRING_H}" ]]; then
    add_isystem_if_string_h "$(dirname "${STRING_H}")"
else
    # GCC internal includes (stdint.h, …)
    for dir in \
        "${GCC_DIR}/../lib/gcc/arm-none-eabi/${GCC_VERSION}/include" \
        "${GCC_DIR}/../arm-none-eabi/include" \
        "/usr/lib/gcc/arm-none-eabi/${GCC_VERSION}/include" \
        "/usr/lib/picolibc/arm-none-eabi/include" \
        "/usr/lib/picolibc/arm-none-eabi/include/release" \
        "${HOME}/.platformio/packages/toolchain-gccarmnoneeabi/arm-none-eabi/include" \
        "${HOME}/.platformio/packages/toolchain-gccarmnoneeabi/lib/gcc/arm-none-eabi/${GCC_VERSION}/include"
    do
        add_isystem_if_string_h "${dir}"
    done
fi

if [[ ${#BINDGEN_INCLUDES[@]} -le 3 ]]; then
    echo "lvgl-env: could not locate string.h for ${GCC}" >&2
    echo "  On Debian/Ubuntu: sudo apt install gcc-arm-none-eabi picolibc-arm-none-eabi" >&2
    echo "  Or point ARM_NONE_EABI_GCC at a full PlatformIO toolchain." >&2
    return 1
fi

# Bash cannot export CC_thumbv8m.* names (dots). Those stay in .cargo/config.toml.
# If arm-none-eabi-gcc is not on PATH, set CC_thumbv8m.main-none-eabihf there manually.
export BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_INCLUDES[*]}"

echo "lvgl-env: using ${GCC} for include discovery"
echo "lvgl-env: BINDGEN_EXTRA_CLANG_ARGS=${BINDGEN_EXTRA_CLANG_ARGS}"
