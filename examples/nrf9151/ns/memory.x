MEMORY
{
  /* Trusted Firmware-M (TF-M) is flashed at the start */
  FLASH                             : ORIGIN = 0x00008000, LENGTH = 0xf8000
  RAM                         (rwx) : ORIGIN = 0x2000C568, LENGTH = 0x33a98
}

