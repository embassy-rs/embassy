/*
Memory configuration for the MPS3-AN536 machine.

See https://github.com/qemu/qemu/blob/master/hw/arm/mps3r.c
*/

MEMORY {
    QSPI : ORIGIN = 0x08000000, LENGTH = 8M
    BRAM : ORIGIN = 0x10000000, LENGTH = 512K
    DDR  : ORIGIN = 0x20000000, LENGTH = 1536M
}

REGION_ALIAS("VECTORS", QSPI);
REGION_ALIAS("CODE", QSPI);
REGION_ALIAS("DATA", BRAM);
REGION_ALIAS("STACKS", BRAM);
