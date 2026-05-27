/* Memory layout for STM32N6 application
 *
 * The application runs from memory-mapped external NOR flash via XSPI2.
 * The FSBL enables memory-mapped mode before jumping here.
 *
 * FLASH = ACTIVE partition (0x400 offset for signed image header).
 * RAM in SRAM starting at 0x34100000.
 */
MEMORY
{
  FLASH              : ORIGIN = 0x70100400, LENGTH = 2043K
  RAM                : ORIGIN = 0x34100000, LENGTH = 512K
  BOOTLOADER         : ORIGIN = 0x70000000, LENGTH = 1024K
  BOOTLOADER_STATE   : ORIGIN = 0x70500000, LENGTH = 12K
  DFU                : ORIGIN = 0x70300000, LENGTH = 2048K
}

/* embassy-boot partition symbols — offsets relative to external flash base */
__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(BOOTLOADER);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(BOOTLOADER);

/* Max firmware size for DFU overflow check: ACTIVE = DFU minus one 4K sector.
 * embassy-boot reserves the extra sector for swap bookkeeping. */
__dfu_max_fw_size = LENGTH(DFU) - 4K;
