/* File originally from lpc55-hal repo: https://github.com/lpc55/lpc55-hal/blob/main/memory.x */ 
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K

  /* for use with standard link.x */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K

  /* would be used with proper link.x */
  /* needs changes to r0 (initialization code) */
  /* SRAM0 : ORIGIN = 0x20000000, LENGTH = 64K */
  /* SRAM1 : ORIGIN = 0x20010000, LENGTH = 64K */
  /* SRAM2 : ORIGIN = 0x20020000, LENGTH = 64K */
  /* SRAM3 : ORIGIN = 0x20030000, LENGTH = 64K */

  /* CASPER SRAM regions */
  /* SRAMX0: ORIGIN = 0x1400_0000, LENGTH = 4K /1* to 0x1400_0FFF *1/ */
  /* SRAMX1: ORIGIN = 0x1400_4000, LENGTH = 4K /1* to 0x1400_4FFF *1/ */

  /* USB1 SRAM regin */
  /* USB1_SRAM : ORIGIN = 0x40100000, LENGTH = 16K */

  /* To define our own USB RAM section in one regular */
  /* RAM, probably easiest to shorten length of RAM */
  /* above, and use this freed RAM section */

}

