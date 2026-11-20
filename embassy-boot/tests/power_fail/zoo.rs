//! The flash families, one entry per set of rules. Sources are the reference manuals and datasheets.

use super::flash::{Model, Rule};

macro_rules! model {
    ($(#[$doc:meta])* $name:ident, $source:literal, write $write:expr, erase $erase:expr, unit $unit:expr,
     erased $erased:expr, $rule:expr, ecc $ecc:expr $(, $key:ident = $value:expr)*) => {
        $(#[$doc])*
        pub struct $name;
        impl Model for $name {
            const NAME: &'static str = stringify!($name);
            const SOURCE: &'static str = $source;
            const WRITE_SIZE: usize = $write;
            const ERASE_SIZE: usize = $erase;
            const UNIT: usize = $unit;
            const ERASED: u8 = $erased;
            const RULE: Rule = $rule;
            const ECC: bool = $ecc;
            $(const $key: model!(@type $key) = $value;)*
        }
    };
    (@type SECTORS) => { &'static [usize] };
    (@type MAX_WRITES) => { usize };
    (@type BLOCK_WRITES) => { (usize, usize) };
}

model!(
    /// STM32F0/F1/F3: half-words; a programmed one is skipped with PGERR, except 0x0000. 1 KB pages (2 KB on
    /// high density, F3, F07x/F09x: same rules).
    F1, "RM0008 Rev 21 / PM0075 Rev 2 §2.3.3, PM0068 Rev 2 §2.3.3, RM0091 Rev 10 §3.2.2, RM0316 Rev 10 §4.2.3", write 2, erase 1024, unit 2, erased 0xFF, Rule::OnceOrZero, ecc false
);
model!(
    /// STM32F2/F4: words (PSIZE x32), no erased check ("Successive write operations are possible ... when
    /// changing bits from '1' to '0'"), sectors 4x16, 1x64, 7x128 KB. Scaled down 64 times (256 B, 1 KB, 2 KB)
    /// with the driver's erase size the biggest sector, as embassy-stm32's whole-flash `Flash` has: one erase of
    /// the first 2 KB is five sector erases.
    F4, "RM0090 Rev 22 §3.3 (sectors), §3.6.4 (programming); PM0059 Rev 5 §2.5.4", write 4, erase 2048, unit 4, erased 0xFF, Rule::BitsToZero, ecc false,
    SECTORS = &[256, 256, 256, 256, 1024, 2048]
);
model!(
    /// STM32F7, single bank: words, no erased check, sectors 4x32, 1x128, 7x256 KB, scaled down 64 times.
    F7, "RM0410 Rev 5 §3.3.1 (organization), §3.3.8 (programming); RM0385 Rev 9", write 4, erase 4096, unit 4, erased 0xFF, Rule::BitsToZero, ecc false,
    SECTORS = &[512, 512, 512, 512, 2048, 4096]
);
model!(
    /// STM32L0 categories 1, 2, 5: erases to 0x00, a write to a word that is not zero is not performed
    /// (NOTZEROERR). 128 B pages.
    L0, "RM0377 Rev 10 §3.3.1 (organization), §3.3.4 (writing/erasing), §3.7.7 (FLASH_SR NOTZEROERR)", write 4, erase 128, unit 4, erased 0x00, Rule::NotZeroRefused, ecc false
);
model!(
    /// STM32L0 category 3: erases to 0x00, a write over programmed data ORs into it. 128 B pages.
    L0c3, "RM0377 Rev 10 §3.3.4 (category 3: a write over a word that is not zero ORs data and ECC)", write 4, erase 128, unit 4, erased 0x00, Rule::Or, ecc false
);
model!(
    /// STM32L1: erases to 0x00 (stated for data EEPROM, inferred for program memory); a write over programmed
    /// data is undocumented, modelled as OR. 256 B pages.
    L1, "RM0038 Rev 18 §3.2 (organization), §3.4.3 (programming), §3.9.7 (FLASH_SR: no not-zero flag)", write 4, erase 256, unit 4, erased 0x00, Rule::Or, ecc false
);
model!(
    /// STM32G0 (and WB, WL, with the same erratum): double words with ECC, PROGERR unless erased or all zero;
    /// all zeros over a word programmed to all ones fails too. 2 KB pages.
    G0, "RM0444 Rev 6 §3.3.3 (ECC), §3.3.6, §3.3.8 (PROGERR); ES0548 Rev 4 §2.2.3, ES0418 Rev 6 §2.2.5 (errata)", write 8, erase 2048, unit 8, erased 0xFF, Rule::OnceOrZeroErrata, ecc true
);
model!(
    /// STM32G4 category 3 single bank, L4, L4+: double words with ECC, PROGERR unless erased or all zero.
    /// 4 KB pages.
    G4, "RM0440 Rev 9 §3.3.1, §3.3.2 (ECC), §3.3.7 (PROGERR); RM0351 Rev 11, RM0432 Rev 9 (L4, L4+)", write 8, erase 4096, unit 8, erased 0xFF, Rule::OnceOrZero, ecc true
);
model!(
    /// G4 as above, but with all zeros over all zeros refused, as AN4894 reads: the manuals don't say, so both
    /// readings run.
    G4zz, "AN4894 Rev 12 (PROGERR 'if the flash memory line was already at 0'); RM0440 Rev 9 §3.3.7 silent on it", write 8, erase 4096, unit 8, erased 0xFF, Rule::ZeroOnce, ecc true
);
model!(
    /// STM32C0: double words, no ECC, PROGERR unless erased or all zero. 2 KB pages.
    C0, "RM0490 Rev 6 §4.2 ('no ECC'), §4.3.6 (PROGERR)", write 8, erase 2048, unit 8, erased 0xFF, Rule::OnceOrZero, ecc false
);
model!(
    /// STM32U5, WBA: quad words with ECC; PROGERR if not erased, or already fully programmed to 0, or the new
    /// value is not all zero. 8 KB pages.
    U5, "RM0456 Rev 7 §7.3.1, §7.3.2 (ECC), §7.3.7, §7.3.9 (PROGERR); RM0493 Rev 7 (WBA)", write 16, erase 8192, unit 16, erased 0xFF, Rule::ZeroOnce, ecc true
);
model!(
    /// STM32L5, U3: double words with ECC, the U5 rule. 2 KB pages (L5 dual bank; 4 KB on U3 and L5 single bank).
    L5, "RM0438 Rev 8 §6.3.1, §6.3.2 (ECC), §6.3.7, §6.3.8 (PROGERR); RM0487 Rev 3 (U3)", write 8, erase 2048, unit 8, erased 0xFF, Rule::ZeroOnce, ecc true
);
model!(
    /// STM32H742/743/750/753: 256-bit flash words with ECC; a program over a used word is never refused and leaves
    /// it faulting (zeros over a programmed word included: inferred, no vendor sentence either way). 128 KB
    /// sectors, scaled down to 8 KB.
    H7, "RM0433 Rev 8 §4.3.4, §4.3.9 (overwrite 'not recommended ... inconsistent ECC'); AN5342 Rev 8 App. A", write 32, erase 8192, unit 32, erased 0xFF, Rule::EccOverwrite, ecc true
);
model!(
    /// STM32H7A3/B0, H7R/S, H5, C5: 128-bit flash words, the H7 rule. 8 KB sectors.
    H7ab, "RM0455 Rev 13 §4.3.4, §4.3.9; RM0477 Rev 10 (H7R/S), RM0481 Rev 5 (H5), RM0522 Rev 1 (C5)", write 16, erase 8192, unit 16, erased 0xFF, Rule::EccOverwrite, ecc true
);
model!(
    /// nRF52, nRF5340 application core, nRF91: words, only 0 bits are written, n_WRITE = 2 writes per word
    /// between erases. 4 KB pages.
    Nrf52, "nRF52840 PS v1.11 §4.3.1 (and nRF52805/810/811/820/833 §4.3.1, nRF5340 PS v1.6, nRF9160 PS v2.2)", write 4, erase 4096, unit 4, erased 0xFF, Rule::BitsToZero, ecc false, MAX_WRITES = 2
);
model!(
    /// nRF52832: as nRF52, the limit is per 512 B block instead of per word. 4 KB pages.
    Nrf52832, "nRF52832 PS v1.9 §8.2, §11.1 (nWRITE,BLOCK: 181 writes per 512 B block between erases)", write 4, erase 4096, unit 4, erased 0xFF, Rule::BitsToZero, ecc false, BLOCK_WRITES = (512, 181)
);
model!(
    /// nRF5340 network core: as nRF52 with 2 KB pages.
    Nrf53net, "nRF5340 PS v1.6 Table 5 (network core: 2 KB pages), §7.21.1 (n_WRITE = 2)", write 4, erase 2048, unit 4, erased 0xFF, Rule::BitsToZero, ecc false, MAX_WRITES = 2
);
model!(
    /// nRF54L15/L10/L05, nRF54LM20 RRAM: 16 B lines, any value over anything; embassy-nrf erases by writing 0xFF
    /// in 4 KB pages. A cut leaves a line part old, part new, by 32-bit words (embassy-nrf commits a line as four
    /// word stores; inferred). ECC corrects up to 2 bits per line; uncorrectable errors are not modelled.
    Rram, "nRF54L15 Datasheet v1.0 §4.2.6, §4.2.6.2 ('Write and overwrite without erasing', 128-bit lines)", write 16, erase 4096, unit 16, erased 0xFF, Rule::Overwrite, ecc false
);
model!(
    /// RP2040/RP2350 with QSPI NOR (W25Q16JV on Pico, W25Q32RV on Pico 2), and any external SPI NOR: bytes
    /// (embassy-rp pads to 256 B pages with 0xFF), bits only to 0, 4 KB sectors.
    Qspi, "RP2040 Datasheet §2.8.3.1.3, RP2350 Datasheet (boot ROM flash functions); W25Q16JV Rev J §9.2.13, §9.3", write 1, erase 4096, unit 1, erased 0xFF, Rule::BitsToZero, ecc false
);

/// What the test expects of an entry in this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    /// Every scenario must pass (known classes aside).
    Pass,
    /// A known finding: the test passes only if the sweep reproduces it.
    Finding(&'static str),
    /// Not a configuration anyone can build (erase value of the chip vs `flash-erase-zero`).
    Skip,
}

const ERASE_ZERO: bool = cfg!(feature = "flash-erase-zero");

/// The expectation for a model with erase value `erased` in this build.
pub fn expect(name: &str, erased: u8) -> Expect {
    let zero_chip = erased == 0x00;
    match (zero_chip, ERASE_ZERO) {
        (false, false) => Expect::Pass,
        (false, true) => Expect::Skip,
        (true, false) => Expect::Finding(
            "flash erases to 0x00 but embassy-boot is built without `flash-erase-zero`, as the L0/L1 boot examples are",
        ),
        (true, true) => {
            if name.starts_with("L0") && name != "L0c3" {
                Expect::Finding(
                    "with `flash-erase-zero`, the magic is cleared with 0xFF over 0xF0, which the L0 refuses",
                )
            } else {
                Expect::Finding(
                    "with `flash-erase-zero`, the magic is cleared with 0xFF over 0xF0, which ORs into it (L1: undocumented)",
                )
            }
        }
    }
}

/// Something to run once per model type.
pub trait Visit {
    type Out;
    fn visit<M: Model>(self) -> Self::Out;
}

/// The zoo: entry `i` (taken modulo the size, so any byte picks one).
pub fn with_model<V: Visit>(i: usize, v: V) -> V::Out {
    match i % COUNT {
        0 => v.visit::<F1>(),
        1 => v.visit::<G0>(),
        2 => v.visit::<Qspi>(),
        3 => v.visit::<L0>(),
        4 => v.visit::<L1>(),
        5 => v.visit::<F4>(),
        6 => v.visit::<F7>(),
        7 => v.visit::<L0c3>(),
        8 => v.visit::<G4>(),
        9 => v.visit::<G4zz>(),
        10 => v.visit::<C0>(),
        11 => v.visit::<U5>(),
        12 => v.visit::<L5>(),
        13 => v.visit::<H7>(),
        14 => v.visit::<H7ab>(),
        15 => v.visit::<Nrf52>(),
        16 => v.visit::<Nrf52832>(),
        17 => v.visit::<Nrf53net>(),
        _ => v.visit::<Rram>(),
    }
}

/// Number of entries in [`with_model`].
pub const COUNT: usize = 19;
