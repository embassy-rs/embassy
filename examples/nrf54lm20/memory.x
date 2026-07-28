MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 2036K

  /*
   * Actual RAM size is 512K, but:
   *
   * 0x2007df40 - 0x2007fe40: VPR saved context
   * 0x2007ff00 - 0x2007ffff: Protected RAM
   */
  RAM : ORIGIN = 0x20000000, LENGTH = 0x7fd40
}
