/* Simple RAM execution linker script for MCXA276 */
MEMORY
{
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* Pull in device default interrupt symbol aliases (e.g., CMC = DefaultHandler) */
INCLUDE device.x


/* Provide core exception weak aliases if not supplied by cortex-m-rt's link.x */
PROVIDE(NonMaskableInt = DefaultHandler);
PROVIDE(HardFault = DefaultHandler);
PROVIDE(MemoryManagement = DefaultHandler);
PROVIDE(BusFault = DefaultHandler);
PROVIDE(UsageFault = DefaultHandler);
PROVIDE(SecureFault = DefaultHandler);
PROVIDE(SVCall = DefaultHandler);
PROVIDE(DebugMonitor = DefaultHandler);
PROVIDE(PendSV = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);

/* In RAM-run we have no FLASH sidata; copy from sdata */
__sidata = __sdata;

/* Ensure the PAC interrupt table is kept */
EXTERN(__INTERRUPTS);


/* Pull in defmt's linker script to generate the defmt table that host decoders expect */
INCLUDE defmt.x

ENTRY(Reset)
EXTERN(VECTOR_TABLE)
EXTERN(Reset)
EXTERN(main)

/* Define _stack_start at end of RAM BEFORE it's used in vector table */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS
{
  .vector_table ORIGIN(RAM) :
  {
    /* Slot 0: Initial stack pointer - use our explicitly set _stack_start */
    LONG(_stack_start);
    /* Slot 1: Reset vector - address of Reset function with Thumb bit set */
    LONG(Reset | 1);
    /* Cortex-M33 core exceptions (slots 2-14) */
    KEEP(*(.vector_table.exceptions));
    /* Peripheral interrupt vectors provided by PAC (slots 16+) */
    KEEP(*(.vector_table.interrupts));
  } > RAM

  .text :
  {
    KEEP(*(.text.Reset));
    KEEP(*(.text.main));
    *(.text .text.*);
  } > RAM

  /* Keep defmt table and fragments so host decoders can find metadata */
  .defmt :
  {
    KEEP(*(.defmt));
    KEEP(*(.defmt.*));
  } > RAM

  .rodata :
  {
    *(.rodata .rodata.*);
  } > RAM

  .data :
  {
    . = ALIGN(4);
    __sdata = .;
    *(.data .data.*);
    . = ALIGN(4);
    __edata = .;
  } > RAM

  /* Ensure RTT control block with "SEGGER RTT" signature is loaded to RAM */
  .rtt :
  {
    KEEP(*(.rtt));
  } > RAM

  /* Place uninitialized buffers (like defmt-rtt) in RAM; load is fine for RAM-run */
  .uninit :
  {
    *(.uninit .uninit.*);
  } > RAM

  .bss :
  {
    . = ALIGN(4);
    __sbss = .;
    *(.bss .bss.*);
    . = ALIGN(4);
    __ebss = .;
  } > RAM

  /* Discard exception unwinding info */
  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
  }
}
