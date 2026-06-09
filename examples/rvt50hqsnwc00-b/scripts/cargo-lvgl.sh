#!/usr/bin/env bash
# Wrapper that sources lvgl-env.sh and runs cargo with optional PlatformIO CC override.
#
#   ./scripts/cargo-lvgl.sh build --bin lvgl_touch_can --features lvgl,touch
#   ./scripts/cargo-lvgl.sh run --bin lvgl_touch_can --features lvgl,touch

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=/dev/null
source "${ROOT}/scripts/lvgl-env.sh"

if [[ -n "${ARM_NONE_EABI_GCC:-}" && -x "${ARM_NONE_EABI_GCC}" ]]; then
    GCC="${ARM_NONE_EABI_GCC}"
elif command -v arm-none-eabi-gcc >/dev/null 2>&1; then
    GCC="$(command -v arm-none-eabi-gcc)"
else
    GCC="${HOME}/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc"
fi
GCC_DIR="$(dirname "${GCC}")"

exec env \
    "CC_thumbv8m.main-none-eabihf=${GCC}" \
    "AR_thumbv8m.main-none-eabihf=${GCC_DIR}/arm-none-eabi-ar" \
    "CFLAGS_thumbv8m.main-none-eabihf=-mcpu=cortex-m33 -mthumb -mfloat-abi=hard -mfpu=fpv5-sp-d16" \
    "BINDGEN_EXTRA_CLANG_ARGS=${BINDGEN_EXTRA_CLANG_ARGS}" \
    cargo "$@"
