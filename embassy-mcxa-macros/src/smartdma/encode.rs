// Instruction encoding reference: NXP's fsl_smartdma_prv.h,
// mcuxsdk-core b2cee0c457e7e59bd85379fe835076f864260b93. This is a native Rust
// encoder with checked operands, not the vendor C-macro assembler. See README.md
// for source sections, supported forms, and integration/verification limits.

use std::collections::BTreeMap;

use super::parse::{Source, Statement, identifier, integer};

pub(super) enum RelocationKind {
    Absolute32,
    Goto,
}

pub(super) struct Encoded {
    pub word: u32,
    pub relocation: Option<(u32, RelocationKind)>,
}

struct Operands<'a, 's> {
    statement: &'a Statement,
    source: &'a Source<'s>,
}

impl Operands<'_, '_> {
    fn error(&self, message: impl std::fmt::Display) -> syn::Error {
        self.source.error(self.statement.location, message)
    }

    fn count(&self, expected: usize) -> syn::Result<()> {
        let actual = self.statement.operands.len();
        if actual != expected {
            return Err(self.error(format!("expected {expected} operands, found {actual}")));
        }
        Ok(())
    }

    // Called only after count() succeeds for this instruction form.
    fn register(&self, index: usize) -> syn::Result<u32> {
        let operand = &self.statement.operands[index];
        let register = match operand.to_ascii_lowercase().as_str() {
            "r0" => 0,
            "r1" => 1,
            "r2" => 2,
            "r3" => 3,
            "r4" => 4,
            "r5" => 5,
            "r6" => 6,
            "r7" => 7,
            "gpo" => 8,
            "gpd" => 9,
            "cfs" => 10,
            "cfm" => 11,
            "sp" => 12,
            "pc" => 13,
            "gpi" => 14,
            "ra" => 15,
            _ => {
                return Err(self.error(format!(
                    "invalid register `{operand}`; expected R0..R7 or a named EZH register"
                )));
            }
        };
        Ok(register)
    }

    fn immediate(&self, index: usize, minimum: i64, maximum: i64) -> syn::Result<i64> {
        let operand = &self.statement.operands[index];
        let value = integer(operand).map_err(|error| self.error(format!("`{operand}`: {error}")))?;
        if !(minimum..=maximum).contains(&value) {
            return Err(self.error(format!("immediate `{operand}` must be in {minimum}..={maximum}")));
        }
        Ok(value)
    }

    fn target(&self, labels: &BTreeMap<String, u32>) -> syn::Result<u32> {
        let name = &self.statement.operands[0];
        if !identifier(name) {
            return Err(self.error("expected a program-local label"));
        }
        labels
            .get(name)
            .copied()
            .ok_or_else(|| self.error(format!("undefined label `{name}`")))
    }

    fn destination(&self, index: usize) -> syn::Result<u32> {
        let register = self.register(index)?;
        // AN14650 Table 4 describes GPI as read-only. Its later suggestion to
        // reuse unused GPI as a GPR is inconsistent; do not promise writable
        // GPI until an applicable source resolves that conflict.
        if register == 14 {
            return Err(self.error("GPI is read-only; using it as a destination is not supported"));
        }
        Ok(register)
    }

    fn ordinary_condition(&self, condition: u32) -> syn::Result<()> {
        if condition == 10 || condition == 11 {
            return Err(self.error("this instruction does not support SPO/UNS or SNE/NZS conditions"));
        }
        Ok(())
    }
}

pub(super) fn encode(
    statement: &Statement,
    labels: &BTreeMap<String, u32>,
    image_words: u32,
    source: &Source<'_>,
) -> syn::Result<Encoded> {
    let operands = Operands { statement, source };
    // A directive's leading dot is not a condition separator.
    let (mnemonic, suffix) = if statement.mnemonic.starts_with('.') {
        (statement.mnemonic.as_str(), None)
    } else {
        statement
            .mnemonic
            .split_once('.')
            .map_or((statement.mnemonic.as_str(), None), |(name, condition)| {
                (name, Some(condition))
            })
    };
    let condition = match suffix {
        None | Some("eu") => 0,
        Some("ze") => 1,
        Some("nz") => 2,
        Some("po") => 3,
        Some("ne") => 4,
        Some("az") => 5,
        Some("zb") => 6,
        Some("ca") => 7,
        Some("nc") => 8,
        Some("cz") => 9,
        Some("spo" | "uns") => 10,
        Some("sne" | "nzs") => 11,
        Some("nbs") => 12,
        Some("nex") => 13,
        Some("bs") => 14,
        Some("ex") => 15,
        Some(condition) => return Err(operands.error(format!("unknown condition `{condition}`"))),
    };
    let unconditional = matches!(
        mnemonic,
        ".word"
            | ".addr"
            | "nop"
            | "int_trigger"
            | "modify_gpo_byte"
            | "heart_rythm_imm"
            | "heart_rythm"
            | "synch_all_to_beat"
            | "wait_for_beat"
    );
    if unconditional && suffix.is_some() {
        return Err(operands.error(format!("`{mnemonic}` does not accept a condition suffix")));
    }
    let mut relocation = None;
    let word = match mnemonic {
        ".word" => {
            operands.count(1)?;
            operands.immediate(0, 0, u32::MAX.into())? as u32
        }
        ".addr" => {
            operands.count(1)?;
            relocation = Some((operands.target(labels)?, RelocationKind::Absolute32));
            0
        }
        // NXP header lines 154-167. Immediate branch address bits remain zero
        // until a loader validates and applies the explicit relocation.
        "nop" => {
            operands.count(0)?;
            0x12
        }
        "int_trigger" => {
            operands.count(1)?;
            0x14 | ((operands.immediate(0, 1, 0xff_ffff)? as u32) << 8)
        }
        "goto" | "gotol" => {
            operands.count(1)?;
            let target = operands.target(labels)?;
            // A branch must name an actual word, not the end-of-image marker.
            // parse() checks the program length fits u32 before encoding.
            if target >= image_words {
                return Err(operands.error("branch target points past the image"));
            }
            relocation = Some((target, RelocationKind::Goto));
            0x15 | (1 << 9) | (u32::from(mnemonic == "gotol") << 10)
        }
        "goto_reg" | "goto_regl" => {
            operands.count(1)?;
            0x15 | (operands.register(0)? << 14) | (u32::from(mnemonic == "goto_regl") << 10)
        }
        // NXP header lines 172-193. Signed 11-bit immediate, optional shift.
        "mov" | "movs" | "mvn" | "mvns" => {
            operands.count(2)?;
            operands.ordinary_condition(condition)?;
            (operands.destination(0)? << 10)
                | (operands.register(1)? << 14)
                | (u32::from(mnemonic.ends_with('s')) << 9)
                | (u32::from(mnemonic.starts_with("mvn")) << 31)
        }
        "load_imm" | "load_imms" | "load_simm" | "load_simms" => {
            let shifted = mnemonic.starts_with("load_simm");
            operands.count(if shifted { 3 } else { 2 })?;
            operands.ordinary_condition(condition)?;
            let immediate = operands.immediate(1, -1024, 1023)? as u32 & 0x7ff;
            let shift = if shifted {
                operands.immediate(2, 0, 31)? as u32
            } else {
                0
            };
            (1 << 18)
                | (operands.destination(0)? << 10)
                | (immediate << 20)
                | ((shift & 0xf) << 14)
                | ((shift >> 4) << 19)
                | (u32::from(mnemonic.ends_with('s')) << 9)
        }
        // NXP header base ALU definitions: ADD 295, SUB 431, ADC 568,
        // SBC 705, OR 842, AND 979, XOR 1116 and their immediate forms.
        "add" | "adds" | "add_imm" | "add_imms" | "sub" | "subs" | "sub_imm" | "sub_imms" | "adc" | "adcs"
        | "adc_imm" | "adc_imms" | "sbc" | "sbcs" | "sbc_imm" | "sbc_imms" | "or" | "ors" | "or_imm" | "or_imms"
        | "and" | "ands" | "and_imm" | "and_imms" | "xor" | "xors" | "xor_imm" | "xor_imms" => {
            operands.count(3)?;
            let opcode = match &mnemonic[..3.min(mnemonic.len())] {
                "add" => 0x6,
                "sub" => 0x8,
                "adc" => 0x9,
                "sbc" => 0xa,
                "and" => 0xd,
                "xor" => 0xe,
                _ => 0xc, // OR / ORS / OR_IMM / OR_IMMS.
            };
            let operand = if mnemonic.contains("_imm") {
                (1 << 18) | ((operands.immediate(2, -2048, 2047)? as u32 & 0xfff) << 20)
            } else {
                operands.register(2)? << 20
            };
            opcode
                | (operands.destination(0)? << 10)
                | (operands.register(1)? << 14)
                | operand
                | (u32::from(mnemonic.ends_with('s')) << 9)
        }
        // NXP header lines 212-258. Preserve vendor operand order:
        // LDR destination,pointer,offset; STR pointer,data,offset.
        "ldr" | "ldrb" | "ldrbs" | "ldr_pre" | "ldrb_pre" | "ldrbs_pre" | "ldr_post" | "ldrb_post" | "ldrbs_post"
        | "str" | "strb" | "str_pre" | "strb_pre" | "str_post" | "strb_post" => {
            operands.count(3)?;
            operands.ordinary_condition(condition)?;
            let load = mnemonic.starts_with("ldr");
            let byte = mnemonic.starts_with("ldrb") || mnemonic.starts_with("strb");
            let post = mnemonic.ends_with("_post");
            let update = post || mnemonic.ends_with("_pre");
            let first = if load {
                operands.destination(0)?
            } else {
                operands.register(0)?
            };
            let second = operands.register(1)?;
            let pointer = if load { second } else { first };
            if update && pointer == 14 {
                return Err(operands.error("GPI is read-only; pointer writeback is not supported"));
            }
            if post && pointer == 13 {
                return Err(operands.error("post-update of PC is not supported"));
            }
            if load && post && first == pointer {
                return Err(operands.error("post-updated load pointer must differ from its destination"));
            }
            if load && byte && first == 10 {
                return Err(operands.error("CFS cannot be the destination of a byte load"));
            }
            let offset = operands.immediate(2, -128, 127)? as u32 & 0xff;
            let base = if load {
                0x1 | (first << 10)
                    | (second << 14)
                    | (u32::from(update) << 20)
                    | (u32::from(mnemonic.starts_with("ldrbs")) << 21)
            } else {
                0x2 | (first << 14) | (second << 20) | (u32::from(update) << 10)
            };
            base | (u32::from(!byte) << 18) | (u32::from(post) << 19) | (offset << 24)
        }
        // NXP header lines 280-288. This frontend deliberately accepts the
        // vendor macro's 20-bit BYTE offset, with two low bits zero, not the
        // cookbook's ambiguous larger-window wording. Never silently mask it.
        "per_read" | "per_write" => {
            operands.count(2)?;
            operands.ordinary_condition(condition)?;
            let register = if mnemonic == "per_read" {
                operands.destination(0)?
            } else {
                operands.register(0)?
            };
            let offset = operands.immediate(1, 0, 0xf_fffc)? as u32;
            if offset & 3 != 0 {
                return Err(operands.error("peripheral byte offset must be 4-byte aligned"));
            }
            if mnemonic == "per_read" {
                0x4 | (register << 10) | (offset << 12)
            } else {
                0x5 | (register << 20) | ((offset & 0xff000) << 12) | ((offset & 0xffc) << 8)
            }
        }
        // NXP header lines 1253-1258,1385-1388,1485-1488.
        "lsl" | "lsls" | "lsr" | "lsrs" | "asr" | "asrs" | "ror" | "rors" => {
            operands.count(3)?;
            let right = mnemonic.starts_with("lsr") || mnemonic.starts_with("ror");
            let arithmetic_or_rotate = mnemonic.starts_with("asr") || mnemonic.starts_with("ror");
            0x10 | (operands.destination(0)? << 10)
                | (operands.register(1)? << 20)
                | ((operands.immediate(2, 0, 31)? as u32) << 24)
                | (u32::from(right) << 19)
                | (u32::from(arithmetic_or_rotate) << 18)
                | (u32::from(mnemonic.ends_with('s')) << 9)
        }
        // NXP header lines 3234-3255.
        "btst_imm" | "btst_imms" | "bclr_imm" | "bclr_imms" | "bset_imm" | "bset_imms" | "btog_imm" | "btog_imms" => {
            operands.count(3)?;
            operands.ordinary_condition(condition)?;
            let operation = match &mnemonic[..4] {
                "bclr" => 1,
                "btst" => 5,
                "bset" => 6,
                _ => 7,
            };
            0x18 | (operands.destination(0)? << 10)
                | (operands.register(1)? << 14)
                | ((operands.immediate(2, 0, 31)? as u32) << 24)
                | (operation << 29)
                | (u32::from(mnemonic.ends_with('s')) << 9)
        }
        // NXP header lines 3260-3268,3324-3332.
        "tight_loop" => {
            operands.count(2)?;
            0x1a | (operands.register(0)? << 14) | (operands.register(1)? << 20)
        }
        "hold" => {
            operands.count(0)?;
            0x1c | (13 << 10) | (1 << 15)
        }
        "modify_gpo_byte" => {
            operands.count(3)?;
            0x1e | ((operands.immediate(0, 0, 255)? as u32) << 8)
                | ((operands.immediate(1, 0, 255)? as u32) << 16)
                | ((operands.immediate(2, 0, 255)? as u32) << 24)
        }
        "heart_rythm_imm" => {
            operands.count(1)?;
            0x32 | ((operands.immediate(0, 0, 65535)? as u32) << 16)
        }
        "heart_rythm" => {
            operands.count(1)?;
            0x32 | (operands.register(0)? << 14) | (1 << 9)
        }
        "synch_all_to_beat" => {
            operands.count(1)?;
            0x52 | ((operands.immediate(0, 0, 1)? as u32) << 31)
        }
        "wait_for_beat" => {
            operands.count(0)?;
            0x72
        }
        _ => return Err(operands.error(format!("unsupported instruction or directive `{mnemonic}`"))),
    };
    Ok(Encoded {
        word: word | (condition << 5),
        relocation,
    })
}
