/* Memory layout for STM32N6 FSBL (First Stage Boot Loader)
 *
 * The boot ROM loads the signed FSBL from external flash at 0x70000000
 * into SRAM at 0x34180400, then executes it.
 *
 * Memory regions from GitHub issue #5626 (proven to work):
 * - RAM:              FSBL stack/data in SRAM
 * - FLASH:            FSBL code in SRAM (loaded by boot ROM)
 * - BOOTLOADER:       Signed FSBL binary on external NOR flash
 * - ACTIVE:           Application firmware on external NOR flash
 * - DFU:              Update staging area on external NOR flash
 * - BOOTLOADER_STATE: Swap/boot state tracking on external NOR flash
 */
MEMORY
{
  RAM                (rwx) : ORIGIN = 0x34100000, LENGTH = 513K
  FLASH                    : ORIGIN = 0x34180400, LENGTH = 507K
  BOOTLOADER               : ORIGIN = 0x70000000, LENGTH = 1024K
  ACTIVE                   : ORIGIN = 0x70100000, LENGTH = 2044K
  DFU                      : ORIGIN = 0x70300000, LENGTH = 2048K
  BOOTLOADER_STATE         : ORIGIN = 0x70500000, LENGTH = 12K
}

/* embassy-boot partition symbols — offsets relative to external flash base */
__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(BOOTLOADER);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(BOOTLOADER);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(BOOTLOADER);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(BOOTLOADER);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(BOOTLOADER);
