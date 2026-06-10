MEMORY
{
  FLASH_ZERO       : ORIGIN = 0x10000000 , LENGTH = 0
  /* Bootloader code that will have jumped to the start of FLASH  */
  BOOTLOADER       : ORIGIN = 0x10000000, LENGTH = 128K
  BOOTLOADER_STATE : ORIGIN = 0x10020000, LENGTH = 4K
  /* The binary that will actually run */
  FLASH            : ORIGIN = 0x10021000, LENGTH = 512K
  /* New binary to swap in, or the previous binary to revert to if it failed.
     Needs to be at least one page larger than FLASH  */
  DFU              : ORIGIN = ORIGIN(FLASH) + LENGTH(FLASH), LENGTH = LENGTH(FLASH) + 4K
  RAM   : ORIGIN = 0x20000000, LENGTH = 512K
}

SECTIONS {
    /* ### Boot ROM info
     *
     * Goes after .vector_table, to keep it in the first 4K of flash
     * where the Boot ROM (and picotool) can find it
     */
    .start_block : ALIGN(4)
    {
        __start_block_addr = .;
        KEEP(*(.start_block));
        KEEP(*(.boot_info));
    } > FLASH

} INSERT AFTER .vector_table;

/* move .text to start /after/ the boot info */
_stext = ADDR(.start_block) + SIZEOF(.start_block);

SECTIONS {
    /* ### Boot ROM extra info
     *
     * Goes after everything in our program, so it can contain a signature.
     */
    .end_block : ALIGN(4)
    {
        __end_block_addr = .;
        KEEP(*(.end_block));
    } > FLASH

} INSERT AFTER .uninit;

PROVIDE(start_to_end = __end_block_addr - __start_block_addr);
PROVIDE(end_to_start = __start_block_addr - __end_block_addr);

/* These will be read by `FirmwareUpdaterConfig::from_linkerfile_blocking`.
   Mind the subtracted FLASH_ZERO: they are offsets in flash, not absolute addresses. */

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH_ZERO);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH_ZERO);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(FLASH_ZERO);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(FLASH_ZERO);
