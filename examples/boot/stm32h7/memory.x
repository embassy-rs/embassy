MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  BOOTLOADER                        : ORIGIN = 0x08000000, LENGTH = 128K
  BOOTLOADER_STATE                  : ORIGIN = 0x08020000, LENGTH = 128K
  FLASH                             : ORIGIN = 0x08040000, LENGTH = 256K
  DFU                               : ORIGIN = 0x08100000, LENGTH = 512K
  RAM                         (rwx) : ORIGIN = 0x24000000, LENGTH = 368K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(BOOTLOADER);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(BOOTLOADER);
