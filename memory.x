/* Memory layout for MCXA276 - RAM execution with cortex-m-rt */
MEMORY
{
  /* FLASH and RAM overlap for RAM-execution experiments. */
  FLASH : ORIGIN = 0x20000000, LENGTH = 128K
  /* RAM overlaps FLASH */
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Leave symbol and section boundary definitions to cortex-m-rt's link.x. */
