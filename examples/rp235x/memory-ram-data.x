/* Loaded by blinky_ram only (memory.x is shared and selects its regions).
 * Link .data at its load address, so the boot ROM loads it and there is nothing to copy. */
OVERWRITE_SECTIONS
{
  .data : ALIGN(4)
  {
    . = ALIGN(4);
    __sdata = .;
    *(.data .data.*);
    . = ALIGN(4);
  } > FLASH
}

/* link.x derived this from the old section; .data has one address now, so the startup copy is a no-op. */
__sidata = __sdata;
