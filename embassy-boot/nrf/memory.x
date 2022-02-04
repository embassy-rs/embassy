MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH                             : ORIGIN = 0x000f9000, LENGTH = 16K
  BOOTSTATE                         : ORIGIN = 0x000ff000, LENGTH = 4K
  RAM                         (rwx) : ORIGIN = 0x20000008, LENGTH = 0x2fff8
  uicr_bootloader_start_address (r) : ORIGIN = 0x10001014, LENGTH = 0x4
}

SECTIONS
{
  .bootstate :
  {
    KEEP(*(SORT(.bootstate*)))
  } > BOOTSTATE

  .uicr_bootloader_start_address :
  {
    KEEP(*(SORT(.uicr_bootloader_start_address*)))
  } > uicr_bootloader_start_address
}
