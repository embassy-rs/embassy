/* Provides information about the memory layout of the device */
MEMORY {
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
}
