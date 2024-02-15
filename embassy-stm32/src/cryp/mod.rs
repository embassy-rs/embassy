use embassy_hal_internal::{into_ref, PeripheralRef};

use crate::pac;
use crate::peripherals::CRYP;
use crate::rcc::sealed::RccPeripheral;
use crate::{interrupt, Peripheral};

const DES_BLOCK_SIZE: usize = 8; // 64 bits
const AES_BLOCK_SIZE: usize = 16; // 128 bits

pub struct Context<'c> {
    algo: Algorithm,
    mode: Mode,
    dir: Direction,
    last_block_processed: bool,
    aad_complete: bool,
    cr: u32,
    iv: [u32; 4],
    key: &'c [u8],
    csgcmccm: [u32; 8],
    csgcm: [u32; 8],
}

#[derive(PartialEq, Clone, Copy)]
pub enum Algorithm {
    AES,
    DES,
    TDES,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    ECB,
    CBC,
    CTR,
    GCM,
    GMAC,
    CCM,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    Encrypt,
    Decrypt,
}

/// Crypto Accelerator Driver
pub struct Cryp<'d, T: Instance> {
    _peripheral: PeripheralRef<'d, T>,
}

type InitVector<'v> = Option<&'v [u8]>;

impl<'d, T: Instance> Cryp<'d, T> {
    /// Create a new CRYP driver.
    pub fn new(peri: impl Peripheral<P = T> + 'd) -> Self {
        CRYP::enable_and_reset();
        into_ref!(peri);
        let instance = Self { _peripheral: peri };
        instance
    }

    /// Start a new cipher operation.
    /// Key size must be 128, 192, or 256 bits.
    pub fn start<'c>(&self, key: &'c [u8], iv: InitVector, algo: Algorithm, mode: Mode, dir: Direction) -> Context<'c> {
        let mut ctx = Context {
            algo,
            mode,
            dir,
            last_block_processed: false,
            cr: 0,
            iv: [0; 4],
            key,
            csgcmccm: [0; 8],
            csgcm: [0; 8],
            aad_complete: false,
        };

        T::regs().cr().modify(|w| w.set_crypen(false));

        // Checks for correctness
        if algo == Algorithm::AES {
            let keylen = key.len() * 8;
            let ivlen;
            if let Some(iv) = iv {
                ivlen = iv.len() * 8;
            } else {
                ivlen = 0;
            }
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

        self.load_key(key);

        // Set data type to 8-bit. This will match software implementations.
        T::regs().cr().modify(|w| w.set_datatype(2));

        self.prepare_key(&ctx);

        if algo == Algorithm::AES {
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

        if mode == Mode::GCM {
            // GCM init phase
            T::regs().cr().modify(|w| w.set_gcm_ccmph(0));
            T::regs().cr().modify(|w| w.set_crypen(true));
            while T::regs().cr().read().crypen() {}
        }

        self.store_context(&mut ctx);

        ctx
    }

    // pub fn aad_blocking(&self, ctx: &mut Context, aad: &[u8]) {
    //     if ctx.aad_complete {
    //         panic!("Cannot update AAD after calling 'update'!")
    //     }
    //     if (ctx.mode != Mode::GCM) && (ctx.mode != Mode::GMAC) && (ctx.mode != Mode::CCM) {
    //         panic!("Associated data only valid for GCM, GMAC, and CCM modes.")
    //     }

    //     let mut header_size = 0;
    //     let mut header: [u8;]

    //     if aad.len() < 65280 {

    //     }

    //     // GCM header phase
    //     T::regs().cr().modify(|w| w.set_gcm_ccmph(1));
    //     T::regs().cr().modify(|w| w.set_crypen(true));
    // }

    pub fn update_blocking(&self, ctx: &mut Context, input: &[u8], output: &mut [u8], last_block: bool) {
        self.load_context(ctx);

        ctx.aad_complete = true;
        if last_block {
            ctx.last_block_processed = true;
        }

        let block_size;
        if ctx.algo == Algorithm::DES {
            block_size = 8;
        } else {
            block_size = 16;
        }
        let last_block_remainder = input.len() % block_size;

        // Perform checks for correctness.

        if ctx.mode == Mode::GMAC {
            panic!("GMAC works on header data only. Do not call this function for GMAC.");
        }
        if ctx.last_block_processed {
            panic!("The last block has already been processed!");
        }
        if input.len() != output.len() {
            panic!("Output buffer length must match input length.");
        }
        if !last_block {
            if last_block_remainder != 0 {
                panic!("Input length must be a multiple of {block_size} bytes.");
            }
        }
        if (ctx.mode == Mode::ECB) || (ctx.mode == Mode::CBC) {
            if last_block_remainder != 0 {
                panic!("Input must be a multiple of {block_size} bytes in ECB and CBC modes. Consider padding or ciphertext stealing.");
            }
        }

        // Load data into core, block by block.
        let num_full_blocks = input.len() / block_size;
        for block in 0..num_full_blocks {
            let mut index = block * block_size;
            let end_index = index + block_size;
            // Write block in
            while index < end_index {
                let mut in_word: [u8; 4] = [0; 4];
                in_word.copy_from_slice(&input[index..index + 4]);
                T::regs().din().write_value(u32::from_ne_bytes(in_word));
                index += 4;
            }
            let mut index = block * block_size;
            let end_index = index + block_size;
            // Block until there is output to read.
            while !T::regs().sr().read().ofne() {}
            // Read block out
            while index < end_index {
                let out_word: u32 = T::regs().dout().read();
                output[index..index + 4].copy_from_slice(u32::to_ne_bytes(out_word).as_slice());
                index += 4;
            }
        }

        // Handle the final block, which is incomplete.
        if last_block_remainder > 0 {
            if ctx.mode == Mode::GCM && ctx.dir == Direction::Encrypt {
                //Handle special GCM partial block process.
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().write(|w| w.set_algomode0(6));
                let iv1r = T::regs().csgcmccmr(7).read() - 1;
                T::regs().init(1).ivrr().write_value(iv1r);
                T::regs().cr().modify(|w| w.set_crypen(true));
            }

            let mut intermediate_data: [u8; 16] = [0; 16];
            let mut last_block: [u8; 16] = [0; 16];
            last_block.copy_from_slice(&input[input.len() - last_block_remainder..input.len()]);
            let mut index = 0;
            let end_index = block_size;
            // Write block in
            while index < end_index {
                let mut in_word: [u8; 4] = [0; 4];
                in_word.copy_from_slice(&last_block[index..index + 4]);
                T::regs().din().write_value(u32::from_ne_bytes(in_word));
                index += 4;
            }
            let mut index = 0;
            let end_index = block_size;
            // Block until there is output to read.
            while !T::regs().sr().read().ofne() {}
            // Read block out
            while index < end_index {
                let out_word: u32 = T::regs().dout().read();
                intermediate_data[index..index + 4].copy_from_slice(u32::to_ne_bytes(out_word).as_slice());
                index += 4;
            }

            // Handle the last block depending on mode.
            output[output.len() - last_block_remainder..output.len()]
                .copy_from_slice(&intermediate_data[0..last_block_remainder]);

            if ctx.mode == Mode::GCM && ctx.dir == Direction::Encrypt {
                //Handle special GCM partial block process.
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().write(|w| w.set_algomode0(8));
                T::regs().init(1).ivrr().write_value(2);
                T::regs().cr().modify(|w| w.set_crypen(true));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(3));
                let mut index = 0;
                let end_index = block_size;
                while index < end_index {
                    let mut in_word: [u8; 4] = [0; 4];
                    in_word.copy_from_slice(&intermediate_data[index..index + 4]);
                    T::regs().din().write_value(u32::from_ne_bytes(in_word));
                    index += 4;
                }
                for _ in 0..4 {
                    T::regs().dout().read();
                }
            }
        }
    }

    fn prepare_key(&self, ctx: &Context) {
        if ctx.algo == Algorithm::AES {
            if (ctx.mode == Mode::ECB) || (ctx.mode == Mode::CBC) {
                T::regs().cr().modify(|w| w.set_algomode0(7));
                T::regs().cr().modify(|w| w.set_crypen(true));
                while T::regs().sr().read().busy() {}
            }
        }
    }

    fn load_key(&self, key: &[u8]) {
        // Load the key into the registers.
        let mut keyidx = 0;
        let mut keyword: [u8; 4] = [0; 4];
        let keylen = key.len() * 8;
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
    }

    fn store_context(&self, ctx: &mut Context) {
        // Wait for data block processing to finish.
        while !T::regs().sr().read().ifem() {}
        while T::regs().sr().read().ofne() {}
        while T::regs().sr().read().busy() {}

        // Disable crypto processor.
        T::regs().cr().modify(|w| w.set_crypen(false));

        // Save the peripheral state.
        ctx.cr = T::regs().cr().read().0;
        ctx.iv[0] = T::regs().init(0).ivlr().read();
        ctx.iv[1] = T::regs().init(0).ivrr().read();
        ctx.iv[2] = T::regs().init(1).ivlr().read();
        ctx.iv[3] = T::regs().init(1).ivrr().read();
        for i in 0..8 {
            ctx.csgcmccm[i] = T::regs().csgcmccmr(i).read();
            ctx.csgcm[i] = T::regs().csgcmr(i).read();
        }
    }

    fn load_context(&self, ctx: &Context) {
        // Reload state registers.
        T::regs().cr().write(|w| w.0 = ctx.cr);
        T::regs().init(0).ivlr().write_value(ctx.iv[0]);
        T::regs().init(0).ivrr().write_value(ctx.iv[1]);
        T::regs().init(1).ivlr().write_value(ctx.iv[2]);
        T::regs().init(1).ivrr().write_value(ctx.iv[3]);
        for i in 0..8 {
            T::regs().csgcmccmr(i).write_value(ctx.csgcmccm[i]);
            T::regs().csgcmr(i).write_value(ctx.csgcm[i]);
        }
        self.load_key(ctx.key);

        // Prepare key if applicable.
        self.prepare_key(ctx);

        // Enable crypto processor.
        T::regs().cr().modify(|w| w.set_crypen(true));
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
