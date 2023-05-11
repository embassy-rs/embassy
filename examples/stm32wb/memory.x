/*
    The size of this file must be exactly the same as in other memory_xx.x files.
    Memory size for STM32WB55xC with 256K FLASH
*/

MEMORY
{
    FLASH (rx)                 : ORIGIN = 0x08000000, LENGTH = 256K
    RAM (xrw)                  : ORIGIN = 0x20000000, LENGTH = 192K
    RAM_SHARED (xrw)           : ORIGIN = 0x20030000, LENGTH = 10K
}

/* 
    Memory size for STM32WB55xC with 512K FLASH

    MEMORY
    {
        FLASH (rx)                 : ORIGIN = 0x08000000, LENGTH = 512K
        RAM (xrw)                  : ORIGIN = 0x20000008, LENGTH = 0x2FFF8
        RAM_SHARED (xrw)           : ORIGIN = 0x20030000, LENGTH = 10K
    }
*/

/* Place stack at the end of SRAM1 */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/*
 * Scatter the mailbox interface memory sections in shared memory
 */
SECTIONS {
    TL_REF_TABLE                     (NOLOAD) : { *(TL_REF_TABLE) } >RAM_SHARED

    MB_MEM1 (NOLOAD)                          : { *(MB_MEM1) } >RAM_SHARED
    MB_MEM2 (NOLOAD)                          : { _sMB_MEM2 = . ; *(MB_MEM2) ; _eMB_MEM2 = . ; } >RAM_SHARED
}
