# Optional helper that sets BINDGEN_EXTRA_CLANG_ARGS by querying the active
# arm-none-eabi-gcc for its libc header path. fish-shell port of
# scripts/lvgl-env.sh — same discovery order, same output env var.
#
# Usage:
#   source scripts/lvgl-env.fish
#   cargo build --bin lvgl_touch_can --features lvgl,touch
#
# Pin a path into your local .cargo/config.toml [env] block to skip this step.

function __lvgl_find_gcc
    if set -q ARM_NONE_EABI_GCC; and test -x "$ARM_NONE_EABI_GCC"
        echo $ARM_NONE_EABI_GCC
        return 0
    end
    if command -q arm-none-eabi-gcc
        command -v arm-none-eabi-gcc
        return 0
    end
    set -l pio "$HOME/.platformio/packages/toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc"
    if test -x $pio
        echo $pio
        return 0
    end
    return 1
end

set -l gcc (__lvgl_find_gcc)
or begin
    echo "lvgl-env: arm-none-eabi-gcc not found." >&2
    echo "  Arch: sudo pacman -S arm-none-eabi-gcc arm-none-eabi-newlib" >&2
    echo "  Debian/Ubuntu: sudo apt install gcc-arm-none-eabi libnewlib-dev" >&2
    echo "  Or set ARM_NONE_EABI_GCC to your toolchain's arm-none-eabi-gcc." >&2
    functions -e __lvgl_find_gcc
    return 1
end

set -l gcc_dir (dirname $gcc)
set -l gcc_version ($gcc -dumpversion)

set -l include_dir ""
for dir in \
    "$gcc_dir/../arm-none-eabi/include" \
    "$gcc_dir/../lib/gcc/arm-none-eabi/$gcc_version/include" \
    "$HOME/.platformio/packages/toolchain-gccarmnoneeabi/arm-none-eabi/include" \
    "/usr/lib/arm-none-eabi/include" \
    "/usr/arm-none-eabi/include" \
    "/usr/include/newlib" \
    "/usr/lib/picolibc/arm-none-eabi/include"
    if test -f "$dir/string.h"
        set include_dir $dir
        break
    end
end

if test -z "$include_dir"
    echo "lvgl-env: could not locate <string.h> for $gcc" >&2
    echo "  Install your toolchain's libc headers (newlib / picolibc), or" >&2
    echo "  set BINDGEN_EXTRA_CLANG_ARGS yourself, e.g.:" >&2
    echo "    set -gx BINDGEN_EXTRA_CLANG_ARGS '--target=thumbv8m.main-none-eabihf -mcpu=cortex-m33 -mthumb -isystem /path/to/include'" >&2
    functions -e __lvgl_find_gcc
    return 1
end

set -gx BINDGEN_EXTRA_CLANG_ARGS "--target=thumbv8m.main-none-eabihf -mcpu=cortex-m33 -mthumb -isystem $include_dir"

echo "lvgl-env: GCC=$gcc"
echo "lvgl-env: BINDGEN_EXTRA_CLANG_ARGS=$BINDGEN_EXTRA_CLANG_ARGS"

functions -e __lvgl_find_gcc
