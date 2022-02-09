MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  MBR                               : ORIGIN = 0x00000000, LENGTH = 4K
  SOFTDEVICE                        : ORIGIN = 0x00001000, LENGTH = 155648
  ACTIVE                            : ORIGIN = 0x00027000, LENGTH = 425984
  DFU                               : ORIGIN = 0x0008F000, LENGTH = 430080
  FLASH                             : ORIGIN = 0x000f9000, LENGTH = 24K
  BOOTLOADER_STATE                  : ORIGIN = 0x000ff000, LENGTH = 4K
  RAM                         (rwx) : ORIGIN = 0x20000008, LENGTH = 0x2fff8
  uicr_bootloader_start_address (r) : ORIGIN = 0x10001014, LENGTH = 0x4
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE);

__bootloader_active_start = ORIGIN(ACTIVE);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE);

__bootloader_dfu_start = ORIGIN(DFU);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU);

__bootloader_start = ORIGIN(FLASH);

SECTIONS
{
  .uicr_bootloader_start_address :
  {
    LONG(__bootloader_start)
  } > uicr_bootloader_start_address
}
