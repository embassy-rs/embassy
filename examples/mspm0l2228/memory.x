MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  /* Select non-parity range of SRAM due to SRAM_ERR_01 errata in SLAZ758 */
  RAM   : ORIGIN = 0x20200000, LENGTH = 32K
}
