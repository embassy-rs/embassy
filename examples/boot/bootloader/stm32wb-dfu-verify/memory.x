/* 
N.B. this memory map is different from the standard stm32wb-dfu example since the
verification code requires more bootloader space. Thus, to use this bootloader
example together with the standard "application" example, you would need to update
memory.x for the application to match this memory map.
*/

MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH                             : ORIGIN = 0x08000000, LENGTH = 48K
  BOOTLOADER_STATE                  : ORIGIN = 0x0800C000, LENGTH = 4K
  ACTIVE                            : ORIGIN = 0x0800D000, LENGTH = 120K
  DFU                               : ORIGIN = 0x0802B000, LENGTH = 120K
  RAM                         (rwx) : ORIGIN = 0x20000000, LENGTH = 16K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(FLASH);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(FLASH);
