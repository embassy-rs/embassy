MEMORY
{
  /* Lengths should match the values set in neorv32_tb. */
  IMEM : ORIGIN = 0x00000000, LENGTH = 32K
  DMEM : ORIGIN = 0x80000000, LENGTH = 8K
}

REGION_ALIAS("REGION_TEXT", IMEM);
REGION_ALIAS("REGION_RODATA", IMEM);
REGION_ALIAS("REGION_DATA", DMEM);
REGION_ALIAS("REGION_BSS", DMEM);
REGION_ALIAS("REGION_HEAP", DMEM);
REGION_ALIAS("REGION_STACK", DMEM);

/* There does not appear to be much harm in setting this to 1 even for single-hart configurations. */
_max_hart_id = 1;
