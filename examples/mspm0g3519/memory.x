MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 512K
  /* Select non-parity range of SRAM due to SRAM_ERR_01 errata in SLAZ758 */
  RAM   : ORIGIN = 0x20200000, LENGTH = 128K
}
