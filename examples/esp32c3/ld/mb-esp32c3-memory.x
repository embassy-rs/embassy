MEMORY
{
    /*
        https://github.com/espressif/esptool/blob/ed64d20b051d05f3f522bacc6a786098b562d4b8/esptool/targets/esp32c3.py#L78-L90
        MEMORY_MAP = [[0x00000000, 0x00010000, "PADDING"],
                  [0x3C000000, 0x3C800000, "DROM"],
                  [0x3FC80000, 0x3FCE0000, "DRAM"],
                  [0x3FC88000, 0x3FD00000, "BYTE_ACCESSIBLE"],
                  [0x3FF00000, 0x3FF20000, "DROM_MASK"],
                  [0x40000000, 0x40060000, "IROM_MASK"],
                  [0x42000000, 0x42800000, "IROM"],
                  [0x4037C000, 0x403E0000, "IRAM"],
                  [0x50000000, 0x50002000, "RTC_IRAM"],
                  [0x50000000, 0x50002000, "RTC_DRAM"],
                  [0x600FE000, 0x60100000, "MEM_INTERNAL2"]]
    */

    /* The origin values for "metadata" and "ROM" memory regions are the actual
     * load addresses.
     *
     * NOTE: The memory region starting from 0x0 with 0x20 length is reserved
     * for the MCUboot header, which will be prepended to the binary file by
     * the "imgtool" during the signing of firmware image.
     */
    metadata : ORIGIN = 0x20, LENGTH = 0x20
    ROM : ORIGIN = 0x40, LENGTH = 0x400000 - 0x40

    /* 400K of on soc RAM, 16K reserved for cache */
    ICACHE : ORIGIN = 0x4037C000,  LENGTH = 0x4000
    /* Instruction RAM */
    IRAM : ORIGIN = 0x4037C000 + 0x4000, LENGTH = 400K - 0x4000
    /* Data RAM */
    DRAM : ORIGIN = 0x3FC80000, LENGTH = 0x50000

    /* External flash */
    /* Instruction ROM */
    IROM : ORIGIN =   0x42000000, LENGTH = 0x400000
    /* Data ROM */
    /* The DROM segment origin is offset by 0x40 for mirroring the actual ROM
     * image layout:
     *    0x0  - 0x1F : MCUboot header
     *    0x20 - 0x3F : Application image metadata section
     *    0x40 onwards: ROM code and data
     * This is required to meet the following constraint from the external
     * flash MMU:
     *    VMA % 64KB == LMA % 64KB
     * i.e. the lower 16 bits of both the virtual address (address seen by the
     * CPU) and the load address (physical address of the external flash) must
     * be equal.
     */
    DROM : ORIGIN = 0x3C000000 + 0x40, LENGTH = 0x400000 - 0x40

    /* RTC fast memory (executable). Persists over deep sleep. */
    RTC_FAST : ORIGIN = 0x50000000, LENGTH = 0x2000 /*- ESP_BOOTLOADER_RESERVE_RTC*/
}

REGION_ALIAS("REGION_TEXT", IROM);
REGION_ALIAS("REGION_RODATA", DROM);

REGION_ALIAS("REGION_DATA", DRAM);
REGION_ALIAS("REGION_BSS", DRAM);
REGION_ALIAS("REGION_STACK", DRAM);

REGION_ALIAS("REGION_RWTEXT", IRAM);
REGION_ALIAS("REGION_RTC_FAST", RTC_FAST);
