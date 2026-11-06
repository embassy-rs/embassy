//! `f64` compiler intrinsics backed by the RP235x double-precision coprocessor (DCP).
//!
//! The Cortex-M33 FPU on RP235x only supports single precision, so by default every `f64`
//! operation is a call into the software routines in `compiler-builtins`. Each Cortex-M33 core
//! also has a DCP (coprocessor 4), which accelerates double-precision arithmetic. See section
//! 3.6.2 "Double-precision coprocessor (DCP)" of the RP2350 datasheet.
//!
//! This module overrides the `__aeabi_d*` functions that the compiler calls for `f64` arithmetic,
//! comparisons and `i32`/`u32`/`f32` conversions with the canned DCP instruction sequences from
//! the Pico SDK.
//!
//! Like the SDK, every function first checks the DCP's "engaged" flag. If it's set, the function
//! has pre-empted another canned sequence on the same core (e.g. it's running in an interrupt
//! handler), so it saves the coprocessor state on the stack and restores it on return. This makes
//! the functions safe to use from any context.
//!
//! Differences from `compiler-builtins`: denormal inputs and outputs are flushed to zero.
//! Zeroes, infinities and NaNs are handled, and rounding is to nearest, ties to even.
//!
//! The DCP must be enabled in CPACR before any of these functions run. `__pre_init` does this for
//! core 0 and `multicore::spawn_core1` does it for core 1.
//!
//! The assembly in this module was taken from the Pico SDK 2.3.1, with each instruction written
//! out in place of the SDK's assembler macros:
//!
//! - <https://github.com/raspberrypi/pico-sdk/blob/079c6f39023649b154152db30f1d781e884879bc/src/rp2_common/pico_double/double_aeabi_dcp.S>
//! - <https://github.com/raspberrypi/pico-sdk/blob/079c6f39023649b154152db30f1d781e884879bc/src/rp2_common/hardware_dcp/include/hardware/dcp_canned.inc.S>
//! - <https://github.com/raspberrypi/pico-sdk/blob/079c6f39023649b154152db30f1d781e884879bc/src/rp2_common/hardware_dcp/include/hardware/dcp_instr.inc.S>
//!
//! The comment on each coprocessor instruction gives its name in the SDK and in section 3.6.2.5
//! of the datasheet.
//!
//! The following copyright notice and BSD-3-Clause license terms apply to the assembly:
//!
//! ```text
//! Copyright (c) 2020 Raspberry Pi (Trading) Ltd. (dcp_canned.inc.S, dcp_instr.inc.S)
//! Copyright (c) 2024 Raspberry Pi (Trading) Ltd. (double_aeabi_dcp.S)
//!
//! SPDX-License-Identifier: BSD-3-Clause
//!
//! Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//! following conditions are met:
//!
//! 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//!    disclaimer.
//!
//! 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following
//!    disclaimer in the documentation and/or other materials provided with the distribution.
//!
//! 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products
//!    derived from this software without specific prior written permission.
//!
//! THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//! INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//! DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//! SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//! SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//! WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
//! THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//! ```

/// Defines a global function, in its own section, that runs `$body` with the DCP state saved and
/// restored if needed. See `dcp_func_in_section`.
macro_rules! dcp_func {
    ($name:literal, $($body:literal),* $(,)?) => {
        concat!(
            ".section .text.", $name, ",\"ax\",%progbits\n",
            dcp_func_in_section!($name, $($body),*),
        )
    };
}

/// Defines a global function in the current section that runs `$body` with the DCP state saved
/// and restored if needed.
///
/// This mirrors `saving_func` in the SDK's `double_aeabi_dcp.S`. The entry point tests the
/// engaged flag (`PCMP apsr_nzcv`). If it's clear, it falls through to the body. If it's set, it
/// branches back to a prologue that calls `__embassy_rp_dcp_save_state`, which saves the state,
/// calls the body (the `b 2f` after the `bl`) and restores the state once the body returns.
///
/// The body must return with `bx lr` (or `pop {.., pc}`), and may not use local labels `1` or `2`.
macro_rules! dcp_func_in_section {
    ($name:literal, $($body:literal),* $(,)?) => {
        concat!(
            ".global ", $name, "\n",
            ".type ", $name, ",%function\n",
            ".p2align 2\n",
            "1:\n",
            "  push {lr}\n",
            "  bl __embassy_rp_dcp_save_state\n",
            "  b 2f\n",
            ".thumb_func\n",
            $name, ":\n",
            "  mrc2 p4, #0, apsr_nzcv, c0, c0, #1\n", // PCMP apsr_nzcv: N = engaged
            "  bmi 1b\n",
            "2:\n",
            $($body, "\n",)*
            ".size ", $name, ", . - ", $name, "\n",
        )
    };
}

core::arch::global_asm!(
    concat!(
        ".syntax unified\n",
        ".thumb\n",
        // `smmlar` and `smmulr` need the DSP extension, which the thumbv8m.main target doesn't enable.
        ".cpu cortex-m33\n",
        // ============== STATE SAVE AND RESTORE ===============
        //
        // Called with the caller's lr pushed, and lr pointing at the body to run.
        ".section .text.__embassy_rp_dcp_save_state,\"ax\",%progbits\n",
        ".type __embassy_rp_dcp_save_state,%function\n",
        ".thumb_func\n",
        "__embassy_rp_dcp_save_state:\n",
        "  sub sp, #24\n",
        "  push {r0, r1}\n",
        "  mrrc2 p4, #0, r0, r1, c8\n", // PXMD r0, r1
        "  strd r0, r1, [sp, #8 + 0]\n",
        "  mrrc2 p4, #0, r0, r1, c9\n", // PYMD r0, r1
        "  strd r0, r1, [sp, #8 + 8]\n",
        "  mrrc p4, #0, r0, r1, c10\n", // REFD r0, r1
        "  strd r0, r1, [sp, #8 + 16]\n",
        "  pop {r0, r1}\n",
        "  blx lr\n", // run the body, which returns here
        "  pop {r12, r14}\n",
        "  mcrr p4, #0, r12, r14, c0\n", // WXMD r12, r14
        "  pop {r12, r14}\n",
        "  mcrr p4, #0, r12, r14, c1\n", // WYMD r12, r14
        "  pop {r12, r14}\n",
        "  mcrr p4, #0, r12, r14, c2\n", // WEFD r12, r14
        "  pop {pc}\n",
        ".size __embassy_rp_dcp_save_state, . - __embassy_rp_dcp_save_state\n",
        // ============== ARITHMETIC FUNCTIONS ===============
        dcp_func!(
            "__aeabi_dadd",
            "  mcrr p4, #1, r0, r1, c0",    // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",    // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #0", // ADD1
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #1, r0, r1, c0",    // RDDA r0, r1
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_dsub",
            "  mcrr p4, #1, r0, r1, c0",    // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",    // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #1", // SUB1
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #3, r0, r1, c0",    // RDDS r0, r1
            "  bx lr",
        ),
        // LLVM doesn't currently call `__aeabi_drsub` or the `__aeabi_cd*` comparisons below. They're
        // here so every RTABI `f64` helper has a DCP version, whatever the compiler emits in future.
        // The linker drops them when they're unused.
        dcp_func!(
            "__aeabi_drsub",
            "  mcrr p4, #1, r2, r3, c0",    // WXUP r2, r3
            "  mcrr p4, #1, r0, r1, c1",    // WYUP r0, r1
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #1", // SUB1
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #3, r0, r1, c0",    // RDDS r0, r1
            "  bx lr",
        ),
        // The DCP has no multiplier, so the mantissa product is computed on the CPU.
        dcp_func!(
            "__aeabi_dmul",
            "  push {r4, r14}",
            "  mcrr p4, #1, r0, r1, c0", // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1", // WYUP r2, r3
            "  mrrc p4, #0, r0, r1, c4", // RXMS r0, r1, 0
            "  mrrc p4, #0, r2, r3, c5", // RYMS r2, r3, 0
            "  umull r4, r12, r0, r2",
            "  movs r14, #0",
            "  umlal r12, r14, r0, r3",
            "  umlal r12, r14, r1, r2",
            "  mcrr p4, #2, r4, r12, c0", // WXMS r4, r12
            "  movs r4, #0",
            "  umlal r14, r4, r1, r3",
            "  mcrr p4, #3, r14, r4, c0",   // WXMO r14, r4
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #5, r0, r1, c0",    // RDDM r0, r1
            "  pop {r4, pc}",
        ),
        // Correctly rounded division, refining the DCP's reciprocal approximation on the CPU.
        dcp_func!(
            "__aeabi_ddiv",
            "  mcrr p4, #1, r0, r1, c0", // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1", // WYUP r2, r3
            "  mrrc p4, #2, r0, r1, c1", // RYMR r0, r1
            "  umull r1, r2, r0, r1",
            "  mvn r2, r2, lsl #2",
            "  smmlar r0, r2, r0, r0",
            "  smmulr r2, r2, r2",
            "  smmlar r0, r2, r0, r0",
            "  sub r12, r0, r0, lsr #31",
            "  mrrc p4, #0, r2, r3, c4", // RXMS r2, r3, 0
            "  smmulr r1, r12, r3",
            "  mrrc p4, #1, r2, r3, c5", // RYMS r2, r3, 1
            "  umull r2, r0, r1, r2",
            "  mla r0, r1, r3, r0",
            "  mrrc p4, #4, r2, r3, c4", // RXMS r2, r3, 4
            "  sub r0, r2, r0",
            "  smmulr r2, r0, r12",
            "  mov r3, r1, lsr #4",
            "  adds r0, r2, r1, lsl #28",
            "  adc r1, r3, r2, asr #31",
            "  orr r0, r0, r1, lsr #21",
            "  bic r0, r0, r1, lsr #22",
            "  bic r0, r0, #7",
            "  mrrc p4, #7, r2, r3, c5", // RYMS r2, r3, 7
            "  umull r3, r12, r0, r2",
            "  mrrc p4, #7, r2, r3, c5", // RYMS r2, r3, 7
            "  mla r12, r0, r3, r12",
            "  mla r12, r1, r2, r12",
            "  mrrc p4, #0, r2, r3, c4", // RXMS r2, r3, 0
            "  sub r12, r12, r2, lsl #18",
            "  orr r0, r0, r12, lsr #29",
            "  sub r0, r0, #1",
            "  mcrr p4, #4, r0, r1, c0",    // WXDD r0, r1
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #7, r0, r1, c0",    // RDDD r0, r1
            "  bx lr",
        ),
        // ============== CONVERSION FUNCTIONS ===============
        dcp_func!(
            "__aeabi_d2f",
            "  mcrr p4, #1, r0, r1, c0",    // WXUP r0, r1
            "  cdp p4, #8, c0, c0, c2, #1", // NRDF
            "  mrc p4, #0, r0, c0, c2, #5", // RDFG r0
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_i2d",
            "  mcrr p4, #7, r0, r0, c0",    // WXIC r0, r0
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #1", // SUB1
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #3, r0, r1, c0",    // RDDS r0, r1
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_ui2d",
            "  mcrr p4, #6, r0, r0, c0",    // WXUC r0, r0
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #1", // SUB1
            "  cdp p4, #8, c0, c0, c0, #1", // NRDD
            "  mrrc p4, #3, r0, r1, c0",    // RDDS r0, r1
            "  bx lr",
        ),
        // Truncates towards zero.
        dcp_func!(
            "__aeabi_d2iz",
            "  mcrr p4, #8, r0, r1, c0",    // WXDC r0, r1
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #0", // ADD1
            "  cdp p4, #8, c0, c0, c0, #2", // NTDC
            "  mrc p4, #0, r0, c0, c3, #0", // RDIC r0
            "  bx lr",
        ),
        // Truncates towards zero.
        dcp_func!(
            "__aeabi_d2uiz",
            "  mcrr p4, #8, r0, r1, c0",    // WXDC r0, r1
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  cdp p4, #1, c0, c0, c1, #0", // ADD1
            "  cdp p4, #8, c0, c0, c0, #2", // NTDC
            "  mrc p4, #0, r0, c0, c3, #1", // RDUC r0
            "  bx lr",
        ),
        // ============== COMPARISON FUNCTIONS ===============
        //
        // RCMP sets the flags from comparing x and y: Z if x == y, C if x >= y, and V if unordered.
        dcp_func!(
            "__aeabi_dcmpun",
            "  mcrr p4, #1, r0, r1, c0",    // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",    // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  mrc p4, #0, r0, c0, c0, #1", // RCMP r0
            "  ubfx r0, r0, #28, #1",       // V
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_dcmpeq",
            "  mcrr p4, #1, r0, r1, c0",    // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",    // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0", // ADD0
            "  mrc p4, #0, r0, c0, c0, #1", // RCMP r0
            "  ubfx r0, r0, #30, #1",       // Z
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_dcmplt",
            "  mcrr p4, #1, r2, r3, c0",           // WXUP r2, r3
            "  mcrr p4, #1, r0, r1, c1",           // WYUP r0, r1
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  ite hi",
            "  movhi r0, #1",
            "  movls r0, #0",
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_dcmple",
            "  mcrr p4, #1, r2, r3, c0",           // WXUP r2, r3
            "  mcrr p4, #1, r0, r1, c1",           // WYUP r0, r1
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  ite hs",
            "  movhs r0, #1",
            "  movlo r0, #0",
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_dcmpge",
            "  mcrr p4, #1, r0, r1, c0",           // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",           // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  ite hs",
            "  movhs r0, #1",
            "  movlo r0, #0",
            "  bx lr",
        ),
        dcp_func!(
            "__aeabi_dcmpgt",
            "  mcrr p4, #1, r0, r1, c0",           // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",           // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  ite hi",
            "  movhi r0, #1",
            "  movls r0, #0",
            "  bx lr",
        ),
        // The flag-returning comparisons only set the flags. If the operands are unordered, the
        // result must be C set and Z clear, which the shared tail does with r12 (which they may
        // clobber). As in the SDK, they share one section so they can share the tail.
        ".section .text.__aeabi_cdcmp,\"ax\",%progbits\n",
        dcp_func_in_section!(
            "__aeabi_cdrcmple",
            "  mcrr p4, #1, r2, r3, c0",           // WXUP r2, r3
            "  mcrr p4, #1, r0, r1, c1",           // WYUP r0, r1
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  bvs .L__embassy_rp_dcp_cmp_nan",
            "  bx lr",
        ),
        dcp_func_in_section!(
            "__aeabi_cdcmple",
            "  mcrr p4, #1, r0, r1, c0",           // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",           // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  bvs .L__embassy_rp_dcp_cmp_nan",
            "  bx lr",
        ),
        dcp_func_in_section!(
            "__aeabi_cdcmpeq",
            "  mcrr p4, #1, r0, r1, c0",           // WXUP r0, r1
            "  mcrr p4, #1, r2, r3, c1",           // WYUP r2, r3
            "  cdp p4, #0, c0, c0, c1, #0",        // ADD0
            "  mrc p4, #0, apsr_nzcv, c0, c0, #1", // RCMP apsr_nzcv
            "  bvs .L__embassy_rp_dcp_cmp_nan",
            "  bx lr",
            ".L__embassy_rp_dcp_cmp_nan:",
            "  movs r12, #3",
            "  lsrs r12, #1", // set C, clear Z
            "  bx lr",
        ),
        ".text\n",
    ),
    options(raw)
);
