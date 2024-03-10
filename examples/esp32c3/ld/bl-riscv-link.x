ENTRY(_start)

PROVIDE(_stext = ORIGIN(ROTEXT));
PROVIDE(_stack_start = ORIGIN(RWDATA) + LENGTH(RWDATA));
PROVIDE(_max_hart_id = 0);

PROVIDE(UserSoft = DefaultHandler);
PROVIDE(SupervisorSoft = DefaultHandler);
PROVIDE(MachineSoft = DefaultHandler);
PROVIDE(UserTimer = DefaultHandler);
PROVIDE(SupervisorTimer = DefaultHandler);
PROVIDE(MachineTimer = DefaultHandler);
PROVIDE(UserExternal = DefaultHandler);
PROVIDE(SupervisorExternal = DefaultHandler);
PROVIDE(MachineExternal = DefaultHandler);

PROVIDE(DefaultHandler = DefaultInterruptHandler);
PROVIDE(ExceptionHandler = DefaultExceptionHandler);

PROVIDE(__post_init = default_post_init);

/* A PAC/HAL defined routine that should initialize custom interrupt controller if needed. */
PROVIDE(_setup_interrupts = default_setup_interrupts);

/* # Multi-processing hook function
   fn _mp_hook() -> bool;

   This function is called from all the harts and must return true only for one hart,
   which will perform memory initialization. For other harts it must return false
   and implement wake-up in platform-dependent way (e.g. after waiting for a user interrupt).
*/
PROVIDE(_mp_hook = default_mp_hook);

/* # Start trap function override
  By default uses the riscv crates default trap handler
  but by providing the `_start_trap` symbol external crates can override.
*/
PROVIDE(_start_trap = default_start_trap);

/* esp32c3 fixups */
SECTIONS {
  .text.dummy (NOLOAD) :
  {
    /* This section is intended to make _stext address work */
    . = ABSOLUTE(_stext);
  } > ROTEXT
}
INSERT BEFORE .text_init;

SECTIONS {
  /* These symbols/functions need to be near eachother, group them together at the start of text */
  .text_init _stext : ALIGN(4) 
  {
    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
  } > ROTEXT
}
INSERT BEFORE .text;

SECTIONS {
  .trap : ALIGN(4)
  {
    KEEP(*(.trap));
    *(.trap.*);
  } > RWTEXT
}
INSERT BEFORE .rwtext;

SECTIONS {
  /**
   * This dummy section represents the .text section but in rodata.
   * Thus, it must have its alignement and (at least) its size.
   */
  .text_dummy (NOLOAD):
  {
    /* Start at the same alignement constraint than .text */
    . = ALIGN(4);
    /* Create an empty gap as big as .text section */
    . = . + SIZEOF(.text) + SIZEOF(.text_init);
    /* Prepare the alignement of the section above. Few bytes (0x20) must be
     * added for the mapping header. */
    . = ALIGN(0x10000) + 0x20;
  } > RODATA
}
INSERT BEFORE .rodata;

SECTIONS {
  /* similar as text_dummy */
  .rwdata_dummy (NOLOAD) : {
    . = ALIGN(ALIGNOF(.rwtext));
    . = . + SIZEOF(.rwtext);
    . = . + SIZEOF(.rwtext.wifi);
    . = . + SIZEOF(.trap);
  } > RWDATA
}
INSERT BEFORE .data;

/* Must be called __global_pointer$ for linker relaxations to work. */
PROVIDE(__global_pointer$ = _data_start + 0x800);
/* end of esp32c3 fixups */

/* Shared sections - ordering matters */
INCLUDE "text.x"
INCLUDE "rodata.x"
INCLUDE "rwtext.x"
INCLUDE "rwdata.x"
INCLUDE "rtc_fast.x"
/* End of Shared sections */

INCLUDE "debug.x"

