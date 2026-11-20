MEMORY
{
  /* Bank 1, secure alias. The last 32 bytes are the NSC veneer. */
  FLASH : ORIGIN = 0x0C000000, LENGTH = 1024K
  /* SRAM2, secure alias. SRAM1 is given to the non-secure image. */
  RAM   : ORIGIN = 0x30070000, LENGTH = 64K
}
