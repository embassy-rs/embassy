MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH : ORIGIN = 0x000fc000, LENGTH = 12K
  BOOTLOADER: ORIGIN = 0x000ff000, LENGTH = 4K
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
  uicr_bootloader_start_address (r) : ORIGIN = 0x10001014, LENGTH = 0x4
}

SECTIONS
{
  .bootloader :
  {
    /* Boot magic */
    LONG(0xDAADD00D);
  } > BOOTLOADER

  .uicr_bootloader_start_address :
  {
    KEEP(*(SORT(.uicr_bootloader_start_address*)))
  } > uicr_bootloader_start_address
}
