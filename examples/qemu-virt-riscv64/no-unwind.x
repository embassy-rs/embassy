/* Discard `.eh_frame` and `.eh_frame_hdr`.
 *
 * These sections are emitted by the prebuilt `libcore` and `compiler_builtins`
 * rlibs the toolchain ships, regardless of `panic = "abort"` or
 * `-C force-unwind-tables=no`. In a `no_std`, `panic = "abort"` binary
 * nothing walks them, so they're dead weight.
 *
 * Dropping them here also sidesteps a hard linker error on `riscv64` when
 * `.text` lives at a high address: `riscv-rt`'s `link.x` places these
 * sections at the default VMA (~0), so `R_RISCV_32_PCREL` fixups inside
 * `.eh_frame` cannot reach `.text` at `0x80000000` (= 2 GiB, the edge of
 * signed 32-bit) and the link fails.
 *
 * Passed to the linker via `-T` *before* `link.x` so `/DISCARD/` removes
 * the input sections before `riscv-rt`'s script can place them.
 */
SECTIONS {
  /DISCARD/ : {
    *(.eh_frame .eh_frame.*)
    *(.eh_frame_hdr)
  }
}
