/* Linker script for the nRF54L15 */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  RAM : ORIGIN = 0x20020000, LENGTH = 128K - 512 /* coprocessor space */

  /* TODO: Figure out why this is necessary. If the heap is not specfied explicitly,
   * an error is raised that the heap is not aligned to 4 bytes.
   */
  REGION_HEAP : ORIGIN = 0x2003f000, LENGTH = 0xf000
}



REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_STACK", RAM);
