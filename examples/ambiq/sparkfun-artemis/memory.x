MEMORY
{
  /* Apollo3 typically has 1MB Flash and 384KB RAM. 
     Note: If you are using a bootloader (like SparkFun Artemis), 
     you may need to offset the FLASH origin to 0x0000C000 or 0x00010000. */
  FLASH (rx) : ORIGIN = 0x00010000, LENGTH = 960K
  RAM (rwx)  : ORIGIN = 0x10000000, LENGTH = 384K
}
