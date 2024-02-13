use embassy_hal_internal::{into_ref, PeripheralRef};
use pac::cryp::Init;

use crate::pac;
use crate::peripherals::CRYP;
use crate::rcc::sealed::RccPeripheral;
use crate::{interrupt, peripherals, Peripheral};

pub struct Context<'c> {
    key: &'c [u8],
}

#[derive(PartialEq)]
pub enum Algorithm {
    AES,
    DES,
    TDES,
}

#[derive(PartialEq)]
pub enum Mode {
    ECB,
    CBC,
    CTR,
    GCM,
    GMAC,
    CCM,
}

#[derive(PartialEq)]
pub enum Direction {
    Encrypt,
    Decrypt,
}

/// Crypto Accelerator Driver
pub struct Cryp<'d, T: Instance, In, Out> {
    _peripheral: PeripheralRef<'d, T>,
    indma: PeripheralRef<'d, In>,
    outdma: PeripheralRef<'d, Out>,
}

type InitVector<'v> = Option<&'v [u8]>;

impl<'d, T: Instance, In, Out> Cryp<'d, T, In, Out> {
    /// Create a new CRYP driver.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        indma: impl Peripheral<P = In> + 'd,
        outdma: impl Peripheral<P = Out> + 'd,
    ) -> Self {
        CRYP::enable_and_reset();
        into_ref!(peri, indma, outdma);
        let instance = Self {
            _peripheral: peri,
            indma: indma,
            outdma: outdma,
        };
        instance
    }

    /// Start a new cipher operation.
    /// Key size must be 128, 192, or 256 bits.
    pub fn start(key: &[u8], iv: InitVector, algo: Algorithm, mode: Mode, dir: Direction) -> Context {
        T::regs().cr().modify(|w| w.set_crypen(false));

        let keylen = key.len() * 8;
        let ivlen;
        if let Some(iv) = iv {
            ivlen = iv.len() * 8;
        } else {
            ivlen = 0;
        }

        // Checks for correctness
        if algo == Algorithm::AES {
            match keylen {
                128 => T::regs().cr().write(|w| w.set_keysize(0)),
                192 => T::regs().cr().write(|w| w.set_keysize(1)),
                256 => T::regs().cr().write(|w| w.set_keysize(2)),
                _ => panic!("Key length must be 128, 192, or 256 bits."),
            }

            if (mode == Mode::GCM) && (ivlen != 96) {
                panic!("IV length must be 96 bits for GCM.");
            } else if (mode == Mode::CBC) && (ivlen != 128) {
                panic!("IV length must be 128 bits for CBC.");
            } else if (mode == Mode::CCM) && (ivlen != 128) {
                panic!("IV length must be 128 bits for CCM.");
            } else if (mode == Mode::CTR) && (ivlen != 64) {
                panic!("IV length must be 64 bits for CTR.");
            } else if (mode == Mode::GCM) && (ivlen != 96) {
                panic!("IV length must be 96 bits for GCM.");
            } else if (mode == Mode::GMAC) && (ivlen != 96) {
                panic!("IV length must be 96 bits for GMAC.");
            }
        }

        // Load the key into the registers.
        let mut keyidx = 0;
        let mut keyword: [u8; 4] = [0; 4];
        if keylen > 192 {
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().key(0).klr().write_value(u32::from_be_bytes(keyword));
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().key(0).krr().write_value(u32::from_be_bytes(keyword));
        }
        if keylen > 128 {
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().key(1).klr().write_value(u32::from_be_bytes(keyword));
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().key(1).krr().write_value(u32::from_be_bytes(keyword));
        }
        if keylen > 64 {
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().key(2).klr().write_value(u32::from_be_bytes(keyword));
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().key(2).krr().write_value(u32::from_be_bytes(keyword));
        }
        keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
        keyidx += 4;
        T::regs().key(3).klr().write_value(u32::from_be_bytes(keyword));
        keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
        T::regs().key(3).krr().write_value(u32::from_be_bytes(keyword));

        // Set data type to 8-bit. This will match software implementations.
        T::regs().cr().modify(|w| w.set_datatype(2));

        if algo == Algorithm::AES {
            if (mode == Mode::ECB) || (mode == Mode::CBC) {
                T::regs().cr().modify(|w| w.set_algomode0(7));
                T::regs().cr().modify(|w| w.set_crypen(true));
                while T::regs().sr().read().busy() {}
            }

            match mode {
                Mode::ECB => T::regs().cr().modify(|w| w.set_algomode0(4)),
                Mode::CBC => T::regs().cr().modify(|w| w.set_algomode0(5)),
                Mode::CTR => T::regs().cr().modify(|w| w.set_algomode0(6)),
                Mode::GCM => T::regs().cr().modify(|w| w.set_algomode0(8)),
                Mode::GMAC => T::regs().cr().modify(|w| w.set_algomode0(8)),
                Mode::CCM => T::regs().cr().modify(|w| w.set_algomode0(9)),
            }
        } else if algo == Algorithm::DES {
            match mode {
                Mode::ECB => T::regs().cr().modify(|w| w.set_algomode0(2)),
                Mode::CBC => T::regs().cr().modify(|w| w.set_algomode0(3)),
                _ => panic!("Only ECB and CBC modes are valid for DES."),
            }
        } else if algo == Algorithm::TDES {
            match mode {
                Mode::ECB => T::regs().cr().modify(|w| w.set_algomode0(0)),
                Mode::CBC => T::regs().cr().modify(|w| w.set_algomode0(1)),
                _ => panic!("Only ECB and CBC modes are valid for TDES."),
            }
        }

        // Set encrypt/decrypt
        if dir == Direction::Encrypt {
            T::regs().cr().modify(|w| w.set_algodir(false));
        } else {
            T::regs().cr().modify(|w| w.set_algodir(true));
        }

        // Load the IV into the registers.
        if let Some(iv) = iv {
            let mut iv_idx = 0;
            let mut iv_word: [u8; 4] = [0; 4];
            iv_word.copy_from_slice(&iv[iv_idx..iv_idx + 4]);
            iv_idx += 4;
            T::regs().init(0).ivlr().write_value(u32::from_be_bytes(iv_word));
            iv_word.copy_from_slice(&iv[iv_idx..iv_idx + 4]);
            iv_idx += 4;
            if iv.len() >= 12 {
                T::regs().init(0).ivrr().write_value(u32::from_be_bytes(iv_word));
                iv_word.copy_from_slice(&iv[iv_idx..iv_idx + 4]);
                iv_idx += 4;
            }
            if iv.len() >= 16 {
                T::regs().init(1).ivlr().write_value(u32::from_be_bytes(iv_word));
                iv_word.copy_from_slice(&iv[iv_idx..iv_idx + 4]);
                T::regs().init(1).ivrr().write_value(u32::from_be_bytes(iv_word));
            }
        }

        // Flush in/out FIFOs
        T::regs().cr().modify(|w| w.fflush());

        let ctx = Context { key: key };

        ctx
    }
}

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        fn regs() -> pac::cryp::Cryp;
    }
}

/// RNG instance trait.
pub trait Instance: sealed::Instance + Peripheral<P = Self> + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this RNG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, rng, CRYP, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::cryp::Cryp {
                crate::pac::$inst
            }
        }
    };
);
