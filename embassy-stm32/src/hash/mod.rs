//! Hash generator (HASH)
use core::cmp::min;

use embassy_hal_internal::{into_ref, PeripheralRef};
use stm32_metapac::hash::regs::*;

use crate::pac::HASH as PAC_HASH;
use crate::peripherals::HASH;
use crate::rcc::sealed::RccPeripheral;
use crate::Peripheral;

#[cfg(hash_v1)]
const NUM_CONTEXT_REGS: usize = 51;
#[cfg(hash_v2)]
const NUM_CONTEXT_REGS: usize = 54;

const HASH_BUFFER_LEN: usize = 68;
const DIGEST_BLOCK_SIZE: usize = 64;

///Hash algorithm selection
#[derive(PartialEq)]
pub enum Algorithm {
    /// SHA-1 Algorithm
    SHA1 = 0,
    /// MD5 Algorithm
    MD5 = 1,
    #[cfg(hash_v2)]
    /// SHA-224 Algorithm
    SHA224 = 2,
    #[cfg(hash_v2)]
    /// SHA-256 Algorithm
    SHA256 = 3,
}

/// Input data width selection
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum DataType {
    ///32-bit data, no data is swapped.
    Width32 = 0,
    ///16-bit data, each half-word is swapped.
    Width16 = 1,
    ///8-bit data, all bytes are swapped.
    Width8 = 2,
    ///1-bit data, all bits are swapped.
    Width1 = 3,
}

/// Stores the state of the HASH peripheral for suspending/resuming
/// digest calculation.
pub struct Context {
    first_word_sent: bool,
    buffer: [u8; HASH_BUFFER_LEN],
    buflen: usize,
    algo: Algorithm,
    format: DataType,
    imr: u32,
    str: u32,
    cr: u32,
    csr: [u32; NUM_CONTEXT_REGS],
}

/// HASH driver.
pub struct Hash<'d> {
    _peripheral: PeripheralRef<'d, HASH>,
}

impl<'d> Hash<'d> {
    /// Instantiates, resets, and enables the HASH peripheral.
    pub fn new(peripheral: impl Peripheral<P = HASH> + 'd) -> Self {
        HASH::enable_and_reset();
        into_ref!(peripheral);
        let instance = Self {
            _peripheral: peripheral,
        };
        instance
    }

    /// Starts computation of a new hash and returns the saved peripheral state.
    pub fn start(&mut self, algorithm: Algorithm, format: DataType) -> Context {
        // Define a context for this new computation.
        let mut ctx = Context {
            first_word_sent: false,
            buffer: [0; 68],
            buflen: 0,
            algo: algorithm,
            format: format,
            imr: 0,
            str: 0,
            cr: 0,
            csr: [0; NUM_CONTEXT_REGS],
        };

        // Set the data type in the peripheral.
        PAC_HASH.cr().modify(|w| w.set_datatype(ctx.format as u8));

        // Select the algorithm.
        let mut algo0 = false;
        let mut algo1 = false;
        if ctx.algo == Algorithm::MD5 || ctx.algo == Algorithm::SHA256 {
            algo0 = true;
        }
        if ctx.algo == Algorithm::SHA224 || ctx.algo == Algorithm::SHA256 {
            algo1 = true;
        }
        PAC_HASH.cr().modify(|w| w.set_algo0(algo0));
        PAC_HASH.cr().modify(|w| w.set_algo1(algo1));
        PAC_HASH.cr().modify(|w| w.set_init(true));

        // Store and return the state of the peripheral.
        self.store_context(&mut ctx);
        ctx
    }

    /// Restores the peripheral state using the given context,
    /// then updates the state with the provided data.
    pub fn update(&mut self, ctx: &mut Context, input: &[u8]) {
        let mut data_waiting = input.len() + ctx.buflen;
        if data_waiting < DIGEST_BLOCK_SIZE || (data_waiting < ctx.buffer.len() && !ctx.first_word_sent) {
            // There isn't enough data to digest a block, so append it to the buffer.
            ctx.buffer[ctx.buflen..ctx.buflen + input.len()].copy_from_slice(input);
            ctx.buflen += input.len();
            return;
        }

        //Restore the peripheral state.
        self.load_context(&ctx);

        let mut ilen_remaining = input.len();
        let mut input_start = 0;

        // Handle first block.
        if !ctx.first_word_sent {
            let empty_len = ctx.buffer.len() - ctx.buflen;
            let copy_len = min(empty_len, ilen_remaining);
            // Fill the buffer.
            if copy_len > 0 {
                ctx.buffer[ctx.buflen..ctx.buflen + copy_len].copy_from_slice(&input[0..copy_len]);
                ctx.buflen += copy_len;
                ilen_remaining -= copy_len;
                input_start += copy_len;
            }
            assert_eq!(ctx.buflen, HASH_BUFFER_LEN);
            self.accumulate(ctx.buffer.as_slice());
            data_waiting -= ctx.buflen;
            ctx.buflen = 0;
            ctx.first_word_sent = true;
        }

        if data_waiting < 64 {
            // There isn't enough data remaining to process another block, so store it.
            assert_eq!(ctx.buflen, 0);
            ctx.buffer[0..ilen_remaining].copy_from_slice(&input[input_start..input_start + ilen_remaining]);
            ctx.buflen += ilen_remaining;
        } else {
            let mut total_data_sent = 0;
            // First ingest the data in the buffer.
            let empty_len = DIGEST_BLOCK_SIZE - ctx.buflen;
            if empty_len > 0 {
                let copy_len = min(empty_len, ilen_remaining);
                ctx.buffer[ctx.buflen..ctx.buflen + copy_len]
                    .copy_from_slice(&input[input_start..input_start + copy_len]);
                ctx.buflen += copy_len;
                ilen_remaining -= copy_len;
                input_start += copy_len;
            }
            assert_eq!(ctx.buflen % 64, 0);
            self.accumulate(&ctx.buffer[0..64]);
            total_data_sent += ctx.buflen;
            ctx.buflen = 0;

            // Move any extra data to the now-empty buffer.
            let leftovers = ilen_remaining % 64;
            if leftovers > 0 {
                assert!(ilen_remaining >= leftovers);
                ctx.buffer[0..leftovers].copy_from_slice(&input[input.len() - leftovers..input.len()]);
                ctx.buflen += leftovers;
                ilen_remaining -= leftovers;
            }
            assert_eq!(ilen_remaining % 64, 0);

            // Hash the remaining data.
            self.accumulate(&input[input_start..input_start + ilen_remaining]);

            total_data_sent += ilen_remaining;
            assert_eq!(total_data_sent % 64, 0);
            assert!(total_data_sent >= 64);
        }

        // Save the peripheral context.
        self.store_context(ctx);
    }

    /// Computes a digest for the given context. A slice of the provided digest buffer is returned.
    /// The length of the returned slice is dependent on the digest length of the selected algorithm.
    pub fn finish<'a>(&mut self, mut ctx: Context, digest: &'a mut [u8; 32]) -> &'a [u8] {
        // Restore the peripheral state.
        self.load_context(&ctx);
        // Hash the leftover bytes, if any.
        self.accumulate(&ctx.buffer[0..ctx.buflen]);
        ctx.buflen = 0;

        //Start the digest calculation.
        PAC_HASH.str().write(|w| w.set_dcal(true));

        //Wait for completion.
        while !PAC_HASH.sr().read().dcis() {}

        //Return the digest.
        let digest_words = match ctx.algo {
            Algorithm::SHA1 => 5,
            Algorithm::MD5 => 4,
            Algorithm::SHA224 => 7,
            Algorithm::SHA256 => 8,
        };
        let mut i = 0;
        while i < digest_words {
            let word = PAC_HASH.hr(i).read();
            digest[(i * 4)..((i * 4) + 4)].copy_from_slice(word.to_be_bytes().as_slice());
            i += 1;
        }
        &digest[0..digest_words * 4]
    }

    fn accumulate(&mut self, input: &[u8]) {
        //Set the number of valid bits.
        let num_valid_bits: u8 = (8 * (input.len() % 4)) as u8;
        PAC_HASH.str().modify(|w| w.set_nblw(num_valid_bits));

        let mut i = 0;
        while i < input.len() {
            let mut word: [u8; 4] = [0; 4];
            let copy_idx = min(i + 4, input.len());
            word[0..copy_idx - i].copy_from_slice(&input[i..copy_idx]);
            PAC_HASH.din().write_value(u32::from_ne_bytes(word));
            i += 4;
        }
    }

    /// Save the peripheral state to a context.
    fn store_context(&mut self, ctx: &mut Context) {
        while !PAC_HASH.sr().read().dinis() {}
        ctx.imr = PAC_HASH.imr().read().0;
        ctx.str = PAC_HASH.str().read().0;
        ctx.cr = PAC_HASH.cr().read().0;
        let mut i = 0;
        while i < NUM_CONTEXT_REGS {
            ctx.csr[i] = PAC_HASH.csr(i).read();
            i += 1;
        }
    }

    /// Restore the peripheral state from a context.
    fn load_context(&mut self, ctx: &Context) {
        // Restore the peripheral state from the context.
        PAC_HASH.imr().write_value(Imr { 0: ctx.imr });
        PAC_HASH.str().write_value(Str { 0: ctx.str });
        PAC_HASH.cr().write_value(Cr { 0: ctx.cr });
        PAC_HASH.cr().modify(|w| w.set_init(true));
        let mut i = 0;
        while i < NUM_CONTEXT_REGS {
            PAC_HASH.csr(i).write_value(ctx.csr[i]);
            i += 1;
        }
    }
}
