MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* Use the first half of flash */
  FLASH                             : ORIGIN = 0x08000000, LENGTH = 128K
  /* use the first 1K of flash for shared data */
  SHARED_RAM                  (rwx) : ORIGIN = 0x20000000, LENGTH = 1K
  /* use the first half of RAM */
  RAM                         (rwx) : ORIGIN = 0x20000400, LENGTH = 32K - 1K
}

SECTIONS
{
    .shared_data :
    {
        *(.shared_data)
    } > SHARED_RAM
}
