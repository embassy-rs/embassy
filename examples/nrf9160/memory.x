MEMORY
{
  /* Assumes Secure Partition Manager (SPM) flashed at the start */
  SPM                      : ORIGIN = 0x00000000, LENGTH = 320K
  FLASH                    : ORIGIN = 0x00050000, LENGTH = 704K
  RAM                      : ORIGIN = 0x20018000, LENGTH = 160K
}
