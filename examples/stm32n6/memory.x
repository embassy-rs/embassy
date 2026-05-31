/* STM32N6x7 AXI SRAM map (secure alias used throughout; non-secure 0x24xxxxxx is
 * an alias of the same RAM).
 *
 *   FLEXRAM  0x34000000   400 KB
 *   AXISRAM1 0x34064000   624 KB   (needs RCC.memenr.axisram1en = 1; off at reset)
 *   AXISRAM2 0x34100000  1024 KB   (enabled by boot ROM — always safe)
 *   AXISRAM3 0x34200000   448 KB
 *   AXISRAM4 0x34270000   448 KB
 *   AXISRAM5 0x342E0000   448 KB
 *   AXISRAM6 0x34350000   448 KB
 *   NPURAM   0x343C0000   256 KB
 *   VENCRAM  0x34400000   128 KB
 *
 * FLASH + RAM are compacted to the top of AXISRAM2 so the lower 768 KB of
 * that bank is contiguous and free — enough to hold a full 800x480 RGB565
 * framebuffer (750 KB) in a single bank, avoiding DMA2D's cross-bank
 * penalty. AXISRAM3..6 remains free for additional framebuffers / large
 * allocations (e.g. a triple-buffer FB2 at 0x342E0000).
 */
MEMORY
{
  FLASH : ORIGIN = 0x341A0000, LENGTH = 256K
  RAM   : ORIGIN = 0x341E0000, LENGTH = 128K
}
