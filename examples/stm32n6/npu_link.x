/* Extra sections for `npu_mobilenet`'s big NPU assets — see memory.x for why
 * these can't just live in the ordinary FLASH/RAM regions.
 *
 * `.npu_blob` is real, loaded content (EC_BLOB's `.rodata`): probe-rs writes
 * it to NPUMEM at download time, same as any other loadable section.
 * `.npu_uninit` is NOLOAD (BLOB_STORAGE's backing array): it must NOT be
 * part of the Reset handler's zero-init loop, since that runs before
 * `enable_all_sram()` clocks AXISRAM3 on; the code that actually fills it
 * (deep in `main()`) already writes real data there itself.
 */
SECTIONS
{
  .npu_blob : ALIGN(4)
  {
    *(.npu_blob);
  } > NPUMEM

  .npu_uninit (NOLOAD) : ALIGN(8)
  {
    *(.npu_uninit);
  } > NPUMEM
}
/* Must come after `.got` (the last section in cortex-m-rt's link.x), not
 * after `.uninit`: inserting here advances the link-time location counter,
 * and cortex-m-rt computes `_stack_end` as wherever that counter sits right
 * after `.uninit` — inserting there would push `_stack_end` up into NPUMEM,
 * past `_stack_start`, and fail cortex-m-rt's own sanity ASSERT.
 */
INSERT AFTER .got;
