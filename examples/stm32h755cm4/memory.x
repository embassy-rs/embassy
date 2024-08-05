MEMORY
{
    FLASH    : ORIGIN = 0x08100000, LENGTH = 1024K /* BANK_2 */
    RAM      : ORIGIN = 0x10000000, LENGTH = 128K  /* SRAM1 */
    RAM_D3   : ORIGIN = 0x38000000, LENGTH = 64K   /* SRAM4 */
}

SECTIONS
{
    .ram_d3 :
    {
        *(.ram_d3.shared_data)
        *(.ram_d3)
    } > RAM_D3
}