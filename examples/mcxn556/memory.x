MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 1M
  /* Use RAMC only; do not assume aggregate SRAM is a contiguous free region. */
  RAM : ORIGIN = 0x20010000, LENGTH = 64K
}
