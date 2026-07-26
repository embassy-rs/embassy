/* QEMU `virt` machine: 128 MB of RAM at 0x80000000.
 * With `-bios none`, the kernel ELF is loaded directly at the start of RAM
 * and execution begins at its entry point (provided by riscv-rt).
 *
 * Use only the first 16 MB for our binary. QEMU places the device tree blob
 * near the top of the configured RAM, so claiming all 128 MB for `.stack`
 * causes a ROM-region overlap with the FDT.
 */
MEMORY
{
  RAM : ORIGIN = 0x80000000, LENGTH = 16M
}

REGION_ALIAS("REGION_TEXT", RAM);
REGION_ALIAS("REGION_RODATA", RAM);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

_stack_start = ORIGIN(RAM) + LENGTH(RAM) - 8;
