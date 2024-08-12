MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  FLASH                             : ORIGIN = 0x08000000, LENGTH = 256K
  SHARED_RAM                  (rwx) : ORIGIN = 0x20000000, LENGTH = 64
  RAM                         (rwx) : ORIGIN = 0x20000040, LENGTH = 64K - 64
}

SECTIONS
{
    .shared_data :
    {
        *(.shared_data)
    } > SHARED_RAM
}