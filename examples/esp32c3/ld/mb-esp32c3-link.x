INCLUDE memory.x

SECTIONS
{
  .metadata :
  {
    /* Magic for load header */

    LONG(0xace637d3)

    /* Application entry point address */

    KEEP(*(.entry_addr))

    /* IRAM metadata:
     * - Destination address (VMA) for IRAM region
     * - Flash offset (LMA) for start of IRAM region
     * - Size of IRAM region
     */

    LONG(ADDR(.rwtext))
    LONG(LOADADDR(.rwtext))
    LONG(SIZEOF(.rwtext))

    /* DRAM metadata:
     * - Destination address (VMA) for DRAM region
     * - Flash offset (LMA) for start of DRAM region
     * - Size of DRAM region
     */

    LONG(ADDR(.data))
    LONG(LOADADDR(.data))
    LONG(SIZEOF(.data))
  } > metadata
}

INCLUDE riscv-link.x

_image_drom_vma = ADDR(.rodata);
_image_drom_lma = LOADADDR(.rodata);
_image_drom_size = LOADADDR(.rodata) + SIZEOF(.rodata) - _image_drom_lma;

_image_irom_vma = ADDR(.text);
_image_irom_lma = LOADADDR(.text);
_image_irom_size = LOADADDR(.text) + SIZEOF(.text) - _image_irom_lma;
