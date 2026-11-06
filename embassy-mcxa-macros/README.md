# Embassy MCXA compile-time assembly

This crate implements the host-side procedural macros behind
`embassy_mcxa::smartdma_asm!` and `embassy_mcxa::smartdma_asm_file!`. Applications
depend on **`embassy-mcxa`**, not this crate directly. There is no C toolchain,
external assembler executable, runtime parser, or assembler allocation on the MCU.

The macros currently produce **unloaded program descriptions only**. They do not
reserve SRAM, configure pins, initialize a stack, launch SmartDMA, or provide
completion/cancellation. The HAL's separate `smartdma-waveform` examples exercise
loading and execution. Encodings do not prove memory safety or waveform timing: SmartDMA is
an independent bus master, not an RP PIO-style memory sandbox.

## Inline and file-based source

```rust,ignore
use embassy_mcxa::smartdma::program::Program;

const PROGRAM: Program = embassy_mcxa::smartdma_asm!(r#"
    .entry start
start:
    load_imm r0, 7
again:
    sub_imms r0, r0, 1
    goto.nz again
    hold
"#);

const FILE_PROGRAM: Program =
    embassy_mcxa::smartdma_asm_file!("src/program.ezh");
```

File paths resolve relative to the consumer's `CARGO_MANIFEST_DIR`, not the Rust
source file or shell working directory. Files must be UTF-8. An emitted
`core::include_str!` records the file dependency for incremental recompilation.
Renaming the HAL dependency works through hygienic `$crate` forwarding.

Programs hold static slices of 32-bit words, exported `Entry` descriptions, and
`Relocation` records. All metadata offsets are **word offsets**, not byte offsets.
These are not executable pointers or loaded-entry handles. Applications can
inspect metadata or construct raw images with `Program::from_parts`; that
constructor does not validate metadata. A future loader must validate it.

## Source language

- One statement per line, optionally preceded by a label (`again:`).
- Comments start with `//` or `;` and continue to the end of the line.
- ASCII identifiers: `[A-Za-z_][A-Za-z0-9_]*`. Labels are case-sensitive.
- Mnemonics, registers, conditions and directive names are case-insensitive.
- Operands are comma-separated; bracketed ARM addressing and C `E_*(...)`
  syntax are not accepted.
- Decimal, `0x` hexadecimal and `0b` binary integers, optionally negative, with
  internal underscores. Each operand has its own checked range. No expressions,
  constants, nested includes, implicit literal pools, or silent masking.
- Registers: `r0` through `r7`, `gpo`, `gpd`, `cfs`, `cfm`, `sp`, `pc`, `gpi`, `ra`.
  GPI may be read, but not written or used with pointer writeback. The cookbook
  calls it read-only while elsewhere suggesting reuse as a GPR; this frontend
  conservatively follows the read-only description.

Directives:

| Directive | Meaning |
|---|---|
| `.entry label` | Export a defined label naming a word in the image. |
| `.word value` | Emit one raw word, `0..=4294967295`; infer no relocation. |
| `.addr label` | Emit a zero placeholder and an `Absolute32` relocation. |

A label at the end of the image may be referenced by `.addr` as a one-past-end
address, but cannot be an entry or immediate branch target. `.entry` does not
classify the referenced word as code rather than data; arbitrary-program
execution remains a separate concern.

Duplicate labels/exports, missing symbols, empty images, invalid operands,
overflow and unsupported forms produce compile errors with source name and
1-based assembly line/column. The enclosing Rust literal is highlighted; assembly
characters do not have individual Rust token spans.

## Supported instructions

Names correspond to NXP macros without `E_`. **Every instruction emits exactly
one word.** There are no hidden scratch registers, veneers or multi-word
pseudo-instructions. Only the forms listed below are supported.

For conditional forms, append `.condition`, for example `mov.nz r1, r2`.
Conditions: `eu` (default), `ze`, `nz`, `po`, `ne`, `az`, `zb`, `ca`, `nc`, `cz`,
`spo`/`uns`, `sne`/`nzs`, `nbs`, `nex`, `bs`, `ex`. These are **EZH conditions**,
not ARM condition names: notably `ne` is the negative condition, not ARM's
not-equal. MOV/MVN, immediate loads, immediate-bit operations, and memory accesses
reject `spo`/`uns` and `sne`/`nzs`.

In this table, `d`, `a`, `b`, `p`, `v`, `end` and `count` are registers;
`iN` is an N-bit signed integer; `uN` is an N-bit unsigned integer.

| Forms | Operands | Conditional? |
|---|---|---|
| `nop`, `wait_for_beat` | none | No |
| `int_trigger` | nonzero `u24` interrupt mask | No |
| `goto`, `gotol` | program-local label | Yes |
| `goto_reg`, `goto_regl` | address register | Yes |
| `mov`, `movs`, `mvn`, `mvns` | `d, a` | Yes |
| `load_imm`, `load_imms` | `d, i11` | Yes |
| `load_simm`, `load_simms` | `d, i11, u5` shift | Yes |
| `add`, `sub`, `adc`, `sbc`, `or`, `and`, `xor`; append `s` to set flags | `d, a, b` | Yes |
| Same ALU names with `_imm` or `_imms` (sets flags) | `d, a, i12` | Yes |
| `ldr`, `ldrb`, `ldrbs`; optionally append `_pre` or `_post` | `d, p, i8` offset | Yes |
| `str`, `strb`; optionally append `_pre` or `_post` | **`p, v, i8`** offset | Yes |
| `per_read`, `per_write` | `d` or `v`, peripheral byte offset | Yes |
| `lsl`, `lsr`, `asr`, `ror`; append `s` to set flags | `d, a, u5` shift | Yes |
| `btst_imm`, `bclr_imm`, `bset_imm`, `btog_imm`; append `s` to set flags | `d, a, u5` bit | Yes |
| `tight_loop` | `end, count` | Yes |
| `hold` | none | Yes |
| `modify_gpo_byte` | `u8` AND mask, `u8` OR mask, `u8` XOR mask | No |
| `heart_rythm_imm` | `u16` raw reload | No |
| `heart_rythm` | reload register | No |
| `synch_all_to_beat` | `0` or `1` | No |

Important semantics and restrictions:

- `goto`/`gotol` emit a `Goto` relocation with zero target bits. Their words are
  **not ready to execute** until relocated. The loader must verify word alignment,
  address arithmetic and the target region against the runtime PC; it must not
  silently truncate an address. `gotol`/`goto_regl` save a return address in RA.
- A branch conditioned on `uns`/`spo` is scheduled: the following two instructions
  are not flushed. `nzs`/`sne` schedules a branch when NZ is set. No delay slots
  are inserted automatically. Ordinary branches flush the pipeline.
- When a set-flag operation writes PC, the flag-update bit requests a scheduled
  branch instead: the following two instructions are not flushed. It does not
  update arithmetic flags. This also applies to the MOV and immediate-bit forms.
- Memory offsets are in **words for word accesses, bytes for byte accesses**.
  Store operands retain NXP's pointer-first order. `ldrbs` sign-extends a byte;
  its `s` does not mean set flags. Post-update of PC, a post-updated load whose
  destination equals its pointer, and byte loads into CFS are rejected. Reading
  through CFS uses zero for its bottom eight pointer bits (hardware behavior).
- Peripheral operations accept a **byte offset** in `0..=0xFFFFC`, aligned to 4,
  from `0x40000000`, matching the pinned SDK macro. Do not pass an absolute
  address. The cookbook's larger-region wording is not generalized here.
- `modify_gpo_byte` affects only the bottom eight GPO bits, not arbitrary pins.
- `tight_loop` executes the loop one more time than the programmed count; the
  instruction immediately following its setup executes only once.
- Initialize the heartbeat before waiting for a beat. Reload values are raw
  hardware operands; no counter off-by-one or nanosecond conversion is assumed.
- `hold` is not a proof of bus quiescence or a general completion protocol.
- Loads can stall, even when instruction issue takes one cycle. Assembly alone
  does not establish a timing budget under bus contention.

Not supported yet: GOSUB, PUSH/POP aliases, vectored/accelerated HOLD, register
offset loads/stores, register-specified shifts, fused ALU shifts/flips/inversion,
ANDOR, and the remaining vendor mnemonic combinations. `.word` permits explicit
raw encodings but neither diagnoses their operands nor infers address fixups.

## Encoding reference and license

The authoritative encoding reference is NXP's **EZH Architecture B** header:

- Repository: <https://github.com/nxp-mcuxpresso/mcuxsdk-core>
- Revision: `b2cee0c457e7e59bd85379fe835076f864260b93`
- File: [`drivers/smartdma/fsl_smartdma_prv.h`](https://github.com/nxp-mcuxpresso/mcuxsdk-core/blob/b2cee0c457e7e59bd85379fe835076f864260b93/drivers/smartdma/fsl_smartdma_prv.h)
- SHA-256: `effe065b8eae9b874e470db7ffe8e68c4f31fd7592d5774947c7763b5ab22424`
- Sections: registers/conditions 83–118; branch/move/immediate 154–193;
  memory/peripheral 212–288; base ALU definitions 295–1140; immediate shifts
  1253–1540; immediate bits 3234–3255; loop/hold/GPIO/heartbeat 3260–3332.

Semantics and operand restrictions additionally follow NXP **AN14650, Rev. 1.0,
2025-05-28**, especially §§4.2, 5.1–5.9, 5.14–5.22 and 5.24–5.28. This is not a
claim of complete ISA coverage or per-chip execution conformance. SDK integration
supports MCXA relevance; silicon behavior still requires hardware validation.

This crate uses MIT OR Apache-2.0, following `embassy-mcxa` and the repository
root licenses. The vendor header is a reproducible encoding reference, **not** a downloaded
build dependency. No third-party assembler or machine-readable tables are
vendored. Future changes should check this pinned reference against NXP fixes;
in particular, older mirrored headers miss a register-shift mask correction.
