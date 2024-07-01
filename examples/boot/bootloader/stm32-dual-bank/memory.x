MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH                             : ORIGIN = 0x08000000, LENGTH = 128K
  BOOTLOADER_STATE                  : ORIGIN = 0x08020000, LENGTH = 128K
  ACTIVE                            : ORIGIN = 0x08040000, LENGTH = 512K
  DFU                               : ORIGIN = 0x08100000, LENGTH = 640K
  RAM                         (rwx) : ORIGIN = 0x24000000, LENGTH = 512K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(DFU);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(DFU);
