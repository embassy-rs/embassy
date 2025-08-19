/****************************************************************************
 * boards/arm/cxd56xx/spresense/scripts/ramconfig-new.ld
 *
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.  The
 * ASF licenses this file to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance with the
 * License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
 * License for the specific language governing permissions and limitations
 * under the License.
 *
 ****************************************************************************/

MEMORY
{
    ram (rwx)  : ORIGIN = 0x0d000000, LENGTH = 1536K
    gnssram (rwx) : ORIGIN = 0x09000000, LENGTH = 640K
}

OUTPUT_ARCH(arm)
ENTRY(__start)         /* Treat __start as the anchor for dead code stripping */
EXTERN(_vectors)       /* Force the vectors to be included in the output */
EXTERN(__stack)        /* Force the __stack to be included in the output */
SECTIONS
{
    /* GNSS memory */

    .gnssram.text : {
        _sgnsstext = ABSOLUTE(.);

        /* Possible to locate text of any object file.
         * *libxxx.a:*.o(.text .text.*)
         * *libxxx.a:*.o(.rodata .rodata.*)
         */

    } > gnssram

    .gnssram.data . : ALIGN(4) {
        /* Possible to locate data of any object file.
         * *libxxx.a:*.o(.data .data.*)
         */

    } > gnssram

    .gnssram.bss . (NOLOAD) : {
        . = ALIGN(4);
        _gnssramsbss = ABSOLUTE(.);

        /* Possible to locate bss of any object file.
         * *libxxx.a:*.o(.bss .bss.*)
         * *libxxx.a:*.o(COMMON)
         */

    } > gnssram

    . = ALIGN(4);
    _gnssramebss = ABSOLUTE(.);

    /* Whatever is left from the GNSS memory is used as a special heap. */

    _sgnssheap = ABSOLUTE(.);
    _egnssheap = ORIGIN(gnssram) + LENGTH(gnssram);

    ASSERT(_sgnssheap < _egnssheap, "Error: Out of memory")

    /* Application memory */

    .text : {
        _stext = ABSOLUTE(.);
        *(.vectors)
        *(.text .text.*)
        *(.fixup)
        *(.gnu.warning)
        *(.rodata .rodata.*)
        *(.gnu.linkonce.t.*)
        *(.glue_7)
        *(.glue_7t)
        *(.got)
        *(.gcc_except_table)
        *(.gnu.linkonce.r.*)
        _image_modlist_base = .;
        KEEP(*(.modulelist))
        _image_modlist_limit = .;
        _etext = ABSOLUTE(.);
    } > ram

    .init_section : {
        _sinit = ABSOLUTE(.);
        KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)))
        KEEP(*(.init_array .ctors))
        _einit = ABSOLUTE(.);
    } > ram

    .ARM.extab : {
        *(.ARM.extab*)
    } > ram

    __exidx_start = ABSOLUTE(.);
    .ARM.exidx : {
        *(.ARM.exidx*)
    } > ram
    __exidx_end = ABSOLUTE(.);

    _eronly = ABSOLUTE(.);

    /* Map .data and .bss sections to system bus */

     . = ALIGN(16);
     . |= 0x20000000;

    .data . : ALIGN(16) {
        _sdata = ABSOLUTE(.);
        *(.data .data.*)
        *(.gnu.linkonce.d.*)
        CONSTRUCTORS
        . = ALIGN(4);
        _edata = ABSOLUTE(.);
    } AT > ram

    .bss . : {            /* BSS              */
        _sbss = ABSOLUTE(.);
        *(.bss .bss.*)
        *(.gnu.linkonce.b.*)
        *(COMMON)
        . = ALIGN(4);
        _ebss = ABSOLUTE(.);
    } AT > ram

    /* __stack symbol is referred from mkspk tool
     * and means the end address of heap region */
    PROVIDE(__stack = ORIGIN(ram) + LENGTH(ram) + 0x20000000);
    __stack -= DEFINED(__reserved_ramsize) ? __reserved_ramsize : 0;

    ASSERT(_ebss < __stack, "Error: Out of memory")

    /* Stabs debugging sections.    */
    .stab 0 : { *(.stab) }
    .stabstr 0 : { *(.stabstr) }
    .stab.excl 0 : { *(.stab.excl) }
    .stab.exclstr 0 : { *(.stab.exclstr) }
    .stab.index 0 : { *(.stab.index) }
    .stab.indexstr 0 : { *(.stab.indexstr) }
    .comment 0 : { *(.comment) }
    .debug_abbrev 0 : { *(.debug_abbrev) }
    .debug_info 0 : { *(.debug_info) }
    .debug_line 0 : { *(.debug_line) }
    .debug_pubnames 0 : { *(.debug_pubnames) }
    .debug_aranges 0 : { *(.debug_aranges) }
}