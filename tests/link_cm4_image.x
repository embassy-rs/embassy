/* Places the embedded secondary-core (CM4) image at the address its boot
 * option byte (BOOT_ADD0) points to, so that the core self-starts it after
 * every reset. Used by the `hsem_secondary` test (see src/bin/hsem_secondary.rs). */
SECTIONS
{
  .cm4_image 0x08100000 :
  {
    KEEP(*(.cm4_image))
  } > FLASH
} INSERT AFTER .rodata;
