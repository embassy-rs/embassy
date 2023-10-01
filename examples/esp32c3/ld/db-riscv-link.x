ENTRY(_start)

PROVIDE(_stext = ORIGIN(REGION_TEXT));
PROVIDE(_stack_start = ORIGIN(REGION_STACK) + LENGTH(REGION_STACK));
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

SECTIONS
{
  .text.dummy (NOLOAD) :
  {
    /* This section is intended to make _stext address work */
    . = ABSOLUTE(_stext);
  } > REGION_TEXT

  .text _stext :
  {
    /* Put reset handler first in .text section so it ends up as the entry */
    /* point of the program. */
    KEEP(*(.init));
    KEEP(*(.init.rust));
    KEEP(*(.text.abort));
    . = ALIGN(4);

    *(.text .text.*);
    _etext = .;
  } > REGION_TEXT

  _text_size = _etext - _stext + 8;
  .rodata ORIGIN(DROM) + _text_size : AT(_text_size)
  {
    _srodata = .;
    *(.srodata .srodata.*);
    *(.rodata .rodata.*);

    /* 4-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(4);
    _erodata = .;
  } > REGION_RODATA

  _rodata_size = _erodata - _srodata + 8;
  .data ORIGIN(DRAM) : AT(_text_size + _rodata_size)
  {
    _data_start = .;
    /* Must be called __global_pointer$ for linker relaxations to work. */
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);
    . = ALIGN(4);
    _data_end = .;
  } > REGION_DATA

  _data_size = _data_end - _data_start + 8;
  .rwtext ORIGIN(REGION_RWTEXT) + _data_size : AT(_text_size + _rodata_size + _data_size){
    _srwtext = .;
    KEEP(*(.trap));
    *(.trap.*);
    *(.rwtext);
    . = ALIGN(4);
    _erwtext = .;
  } > REGION_RWTEXT
  _rwtext_size = _erwtext - _srwtext + 8;

  .rwtext.dummy (NOLOAD):
  {
    /* This section is required to skip .rwtext area because REGION_RWTEXT
     * and REGION_BSS reflect the same address space on different buses.
     */
    . = ORIGIN(REGION_DATA) + _rwtext_size + 8 + SIZEOF(.data);
  } > REGION_DATA

  .bss (NOLOAD) :
  {
    _bss_start = .;
    *(.sbss .sbss.* .bss .bss.*);
    . = ALIGN(4);
    _bss_end = .;
  } > REGION_BSS

  /* ### .uninit */
  .uninit (NOLOAD) : ALIGN(4)
  {
    . = ALIGN(4);
    __suninit = .;
    *(.uninit .uninit.*);
    . = ALIGN(4);
    __euninit = .;
  } > REGION_BSS

  /* fictitious region that represents the memory available for the stack */
  .stack (NOLOAD) :
  {
    _estack = .;
    . = ABSOLUTE(_stack_start);
    _sstack = .;
  } > REGION_STACK

  .rtc_fast.text : AT(_text_size + _rodata_size + _data_size + _rwtext_size) {
    _srtc_fast_text = .;
    *(.rtc_fast.literal .rtc_fast.text .rtc_fast.literal.* .rtc_fast.text.*)
    . = ALIGN(4);
    _ertc_fast_text = .;
  } > REGION_RTC_FAST
  _fast_text_size = _ertc_fast_text - _srtc_fast_text + 8;

  .rtc_fast.data : AT(_text_size + _rodata_size + _data_size + _rwtext_size + _fast_text_size)
  {
    _rtc_fast_data_start = ABSOLUTE(.);
    *(.rtc_fast.data .rtc_fast.data.*)
    . = ALIGN(4);
    _rtc_fast_data_end = ABSOLUTE(.);
  } > REGION_RTC_FAST
  _rtc_fast_data_size = _rtc_fast_data_end - _rtc_fast_data_start + 8;

 .rtc_fast.bss (NOLOAD) : ALIGN(4)
  {
    _rtc_fast_bss_start = ABSOLUTE(.);
    *(.rtc_fast.bss .rtc_fast.bss.*)
    . = ALIGN(4);
    _rtc_fast_bss_end = ABSOLUTE(.);
  } > REGION_RTC_FAST

 .rtc_fast.noinit (NOLOAD) : ALIGN(4)
  {
    *(.rtc_fast.noinit .rtc_fast.noinit.*)
  } > REGION_RTC_FAST

  /* fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect
     relocatable code in the input files and raise an error if relocatable code
     is found */
  .got (INFO) :
  {
    KEEP(*(.got .got.*));
  }

  .eh_frame (INFO) : { KEEP(*(.eh_frame)) }
  .eh_frame_hdr (INFO) : { *(.eh_frame_hdr) }
}

PROVIDE(_sidata = _erodata + 8);
PROVIDE(_irwtext = ORIGIN(DROM) + _text_size + _rodata_size + _data_size);
PROVIDE(_irtc_fast_text = ORIGIN(DROM) + _text_size + _rodata_size + _data_size + _rwtext_size);
PROVIDE(_irtc_fast_data = ORIGIN(DROM) + _text_size + _rodata_size + _data_size + _rwtext_size + _fast_text_size);

/* Do not exceed this mark in the error messages above                                    | */
ASSERT(ORIGIN(REGION_TEXT) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_TEXT must be 4-byte aligned");

ASSERT(ORIGIN(REGION_RODATA) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_RODATA must be 4-byte aligned");

ASSERT(ORIGIN(REGION_DATA) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_DATA must be 4-byte aligned");

ASSERT(ORIGIN(REGION_TEXT) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_TEXT must be 4-byte aligned");

ASSERT(ORIGIN(REGION_STACK) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_STACK must be 4-byte aligned");

ASSERT(_stext % 4 == 0, "
ERROR(riscv-rt): `_stext` must be 4-byte aligned");

ASSERT(_data_start % 4 == 0 && _data_end % 4 == 0, "
BUG(riscv-rt): .data is not 4-byte aligned");

ASSERT(_sidata % 4 == 0, "
BUG(riscv-rt): the LMA of .data is not 4-byte aligned");

ASSERT(_bss_start % 4 == 0 && _bss_end % 4 == 0, "
BUG(riscv-rt): .bss is not 4-byte aligned");

ASSERT(_stext + SIZEOF(.text) < ORIGIN(REGION_TEXT) + LENGTH(REGION_TEXT), "
ERROR(riscv-rt): The .text section must be placed inside the REGION_TEXT region.
Set _stext to an address smaller than 'ORIGIN(REGION_TEXT) + LENGTH(REGION_TEXT)'");

ASSERT(SIZEOF(.got) == 0, "
.got section detected in the input files. Dynamic relocations are not
supported. If you are linking to C code compiled using the `gcc` crate
then modify your build script to compile the C code _without_ the
-fPIC flag. See the documentation of the `gcc::Config.fpic` method for
details.");

/* Do not exceed this mark in the error messages above                                    | */
