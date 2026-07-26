MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* Use the second half of flash */
  FLASH                             : ORIGIN = 0x08020000, LENGTH = 128K
  /* use the first 1K of flash from teh first half of ram for shared data */
  SHARED_RAM                  (rwx) : ORIGIN = 0x20000000, LENGTH = 1K
  /* use the second half of flash */
  RAM                         (rwx) : ORIGIN = 0x20008000, LENGTH = 32K
}

SECTIONS
{
    .shared_data :
    {
        /* force ordering of SHARED_DATA followed by LED_STATE */
        *(.shared_data.0)
        *(.shared_data.1)
        *(.shared_data)
    } > SHARED_RAM
}
