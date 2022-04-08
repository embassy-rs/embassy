/*
    The size of this file must be exactly the same as in other memory_xx.x files.
    Memory size for STM32WB55xC with 256K FLASH
*/

MEMORY
{
    FLASH (rx)                 : ORIGIN = 0x08000000, LENGTH = 256K
    RAM (xrw)                  : ORIGIN = 0x20000004, LENGTH = 191K
    RAM_SHARED (xrw)           : ORIGIN = 0x20030000, LENGTH = 10K
}

/* Place stack at the end of SRAM1 */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/*
 * Scatter the mailbox interface memory sections in shared memory
 */
SECTIONS {
    TL_REF_TABLE                     (NOLOAD) : { *(TL_REF_TABLE) } >RAM_SHARED

    TL_DEVICE_INFO_TABLE    0x2003001c (NOLOAD) : { *(TL_DEVICE_INFO_TABLE) } >RAM_SHARED
    TL_BLE_TABLE            0x2003003c (NOLOAD) : { *(TL_BLE_TABLE) } >RAM_SHARED
    TL_THREAD_TABLE         0x2003004c (NOLOAD) : { *(TL_THREAD_TABLE) } >RAM_SHARED
    TL_SYS_TABLE            0x20030058 (NOLOAD) : { *(TL_SYS_TABLE) } >RAM_SHARED
    TL_MEM_MANAGER_TABLE    0x20030060 (NOLOAD) : { *(TL_MEM_MANAGER_TABLE) } >RAM_SHARED
    TL_TRACES_TABLE         0x2003007c (NOLOAD) : { *(TL_TRACES_TABLE) } >RAM_SHARED
    TL_MAC_802_15_4_TABLE   0x20030080 (NOLOAD) : { *(TL_MAC_802_15_4_TABLE) } >RAM_SHARED

    HCI_ACL_DATA_BUFFER     0x20030a08 (NOLOAD) : { *(HCI_ACL_DATA_BUFFER) } >RAM_SHARED
    BLE_CMD_BUFFER          0x200308fc (NOLOAD) : { *(BLE_CMD_BUFFER) } >RAM_SHARED
    BLE_SPARE_EVT_BUF       0x200301a8 (NOLOAD) : { *(BLE_SPARE_EVT_BUF) } >RAM_SHARED
    SYS_SPARE_EVT_BUF       0x200302b4 (NOLOAD) : { *(SYS_SPARE_EVT_BUF) } >RAM_SHARED
    EVT_POOL                0x200303c0 (NOLOAD) : { *(EVT_POOL) } >RAM_SHARED
    SYS_CMD_BUF             0x2003009c (NOLOAD) : { *(SYS_CMD_BUF) } >RAM_SHARED
    SYSTEM_EVT_QUEUE        0x20030b28 (NOLOAD) : { *(SYSTEM_EVT_QUEUE) } >RAM_SHARED
    EVT_QUEUE               0x20030b10 (NOLOAD) : { *(EVT_QUEUE) } >RAM_SHARED
    CS_BUFFER               0x20030b18 (NOLOAD) : { *(CS_BUFFER) } >RAM_SHARED
    TRACES_EVT_QUEUE        0x20030094 (NOLOAD) : { *(TRACES_EVT_QUEUE) } >RAM_SHARED
    FREE_BUF_QUEUE          0x2003008c (NOLOAD) : { *(FREE_BUF_QUEUE) } >RAM_SHARED
}
