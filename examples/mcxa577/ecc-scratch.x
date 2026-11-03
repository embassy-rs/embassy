/* Ordinary RAM excludes the entire ECC bank for this binary. */
MEMORY
{
  RAM : ORIGIN = 0x20010000, LENGTH = 576K
  ECC_SCRATCH : ORIGIN = 0x20000000, LENGTH = 8
}
SECTIONS
{
  .ecc_scratch ORIGIN(ECC_SCRATCH) (NOLOAD) :
  {
    __ecc_scratch = .;
    KEEP(*(.ecc_scratch));
  } > ECC_SCRATCH
}
ASSERT(ADDR(.ecc_scratch) == 0x20000000 && SIZEOF(.ecc_scratch) == 8,
       "ECC scratch must reserve exactly two aligned words");
ASSERT(ORIGIN(RAM) >= 0x20010000, "Ordinary RAM must exclude ECC SRAM");
