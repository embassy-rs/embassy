/* Teaching example only: isolate data from executable/read-only/load sections. */
SECTIONS
{
  .mbc_data : ALIGN(16384)
  {
    KEEP(*(.mbc_data));
  } > FLASH
} INSERT AFTER .rodata;

ASSERT(ADDR(.mbc_data) % 16384 == 0, "MBC settings must start on a block boundary");
ASSERT(SIZEOF(.mbc_data) == 16384, "MBC settings must occupy whole blocks");
