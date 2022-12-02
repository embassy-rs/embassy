MEMORY {
    BOOT2             : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH             : ORIGIN = 0x10000100, LENGTH = 24K - 0x100
    BOOTLOADER_STATE  : ORIGIN = 0x10006000, LENGTH = 4K
    ACTIVE            : ORIGIN = 0x10008000, LENGTH = 32K
    DFU               : ORIGIN = 0x10010000, LENGTH = 36K
    RAM               : ORIGIN = 0x20000000, LENGTH = 256K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(FLASH);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(FLASH);
