// SGI (Secure Generic Interface) driver for MCXA

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_mcxa::dma::DmaChannel;
use embassy_time::Instant;
use log::{error, info};
use nxp_pac::edma_0_tcd::vals::{Bwc, Dreq, Esg, Size, Start};

use crate::hash::HashOptions;

// ============================================================================
// SGI0 (Secure Generic Interface) constants and bitfields
// ============================================================================

// Base address
pub const SGI0_BASE: u32 = 0x400E_B000;

// Register offsets from base address
pub const SGI_DATIN0A_OFFSET: u32 = 0x200;
// Alias kept for compatibility/readability.
pub const DATAIN_START_OFFSET: u32 = SGI_DATIN0A_OFFSET;
pub const SGI_DATAIN_TOTAL_WORDS: usize = 16; // 16 words (64 bytes) total for DATIN0A-DATIN3D
pub const SGI_KEY0A_OFFSET: u32 = 0x240;
pub const SGI_KEYIN_TOTAL_WORDS: usize = 32; // 32 words (128 bytes) total for KEY0A-KEY7D
pub const SGI_DATOUT_START_OFFSET: u32 = 0x2C0;
pub const SGI_DATOUTA_OFFSET: u32 = SGI_DATOUT_START_OFFSET;
pub const SGI_DATOUTB_OFFSET: u32 = SGI_DATOUT_START_OFFSET + 0x04;
pub const SGI_DATOUTC_OFFSET: u32 = SGI_DATOUT_START_OFFSET + 0x08;
pub const SGI_DATOUTD_OFFSET: u32 = SGI_DATOUT_START_OFFSET + 0x0C;
pub const SGI_STATUS_OFFSET: u32 = 0xC00;
pub const SGI_COUNT_OFFSET: u32 = 0xC04;
pub const SGI_KEYCHK_OFFSET: u32 = 0xC08;
pub const SGI_CTRL_OFFSET: u32 = 0xD00;
pub const SGI_CTRL2_OFFSET: u32 = 0xD04;
pub const SGI_DUMMY_CTRL_OFFSET: u32 = 0xD08;
pub const SGI_SFR_SW_MASK_OFFSET: u32 = 0xD0C;
pub const SGI_SFRSEED_OFFSET: u32 = 0xD10;
pub const SGI_SHA2_CTRL_OFFSET: u32 = 0xD14;
pub const SGI_SHA_FIFO_OFFSET: u32 = 0xD18;
pub const SGI_CONFIG_OFFSET: u32 = 0xD1C;
pub const SGI_CONFIG2_OFFSET: u32 = 0xD20;
pub const SGI_AUTO_MODE_OFFSET: u32 = 0xD24;
pub const SGI_AUTO_DMA_CTRL_OFFSET: u32 = 0xD28;
pub const SGI_PRNG_SW_SEED_OFFSET: u32 = 0xD30;
pub const SGI_KEY_CTRL_OFFSET: u32 = 0xD40;
pub const SGI_KEY_WRAP_OFFSET: u32 = 0xD50;
pub const SGI_VERSION_OFFSET: u32 = 0xF08;
pub const SGI_ACCESS_ERR_OFFSET: u32 = 0xFC0;
pub const SGI_ACCESS_ERR_CLR_OFFSET: u32 = 0xFC4;
pub const SGI_INT_STATUS_OFFSET: u32 = 0xFE0;
pub const SGI_INT_ENABLE_OFFSET: u32 = 0xFE4;
pub const SGI_INT_STATUS_CLR_OFFSET: u32 = 0xFE8;
pub const SGI_INT_STATUS_SET_OFFSET: u32 = 0xFEC;
pub const SGI_MODULE_ID_OFFSET: u32 = 0xFFC;

// STATUS bits
pub const STATUS_SGI_BUSY: u32 = 1 << 0;
pub const STATUS_SGI_ERR: u32 = 1 << 1;
pub const STATUS_SGI_INT_PDONE: u32 = 1 << 0;
pub const STATUS_SGI_INT_EN: u32 = 1 << 0;
pub const STATUS_SGI_INT_CLR: u32 = 1 << 0;

// SGI_CTRL register bitfields
pub const SGI_CTRL_START: u32 = 1 << 0;
pub const SGI_CTRL_DECRYPT: u32 = 1 << 1;
pub const SGI_CTRL_AESKEYSZ_SHIFT: u32 = 2;
pub const SGI_CTRL_AESKEYSZ_128: u32 = 0 << 2;
pub const SGI_CTRL_AESKEYSZ_256: u32 = 2 << 2;
pub const SGI_CTRL_CRYPTO_OP_SHIFT: u32 = 4;
pub const SGI_CTRL_CRYPTO_OP_AES: u32 = 0 << 4;
pub const SGI_CTRL_CRYPTO_OP_DES: u32 = 1 << 4;
pub const SGI_CTRL_CRYPTO_OP_TDES: u32 = 2 << 4;
pub const SGI_CTRL_CRYPTO_OP_GFMUL: u32 = 3 << 4;
pub const SGI_CTRL_CRYPTO_OP_SHA2: u32 = 4 << 4;
pub const SGI_CTRL_CRYPTO_OP_CMAC: u32 = 5 << 4;
pub const SGI_CTRL_INSEL_SHIFT: u32 = 7;
pub const SGI_CTRL_INSEL_DATIN0: u32 = 0x0 << 7;
pub const SGI_CTRL_INSEL_DATIN1: u32 = 0x1 << 7;
pub const SGI_CTRL_INSEL_DATIN2: u32 = 0x2 << 7;
pub const SGI_CTRL_INSEL_DATIN3: u32 = 0x3 << 7;
pub const SGI_CTRL_INSEL_DATIN0_XOR_DATOUT: u32 = 0x4 << 7;
pub const SGI_CTRL_INSEL_DATIN1_XOR_DATOUT: u32 = 0x5 << 7;
pub const SGI_CTRL_INSEL_DATIN2_XOR_DATOUT: u32 = 0x6 << 7;
pub const SGI_CTRL_INSEL_DATIN3_XOR_DATOUT: u32 = 0x7 << 7;
pub const SGI_CTRL_INSEL_DATOUT: u32 = 0x8 << 7;
pub const SGI_CTRL_OUTSEL_SHIFT: u32 = 11;
pub const SGI_CTRL_OUTSEL_KERNEL: u32 = 0x0 << 11;
pub const SGI_CTRL_OUTSEL_KERNEL_XOR_DATIN0: u32 = 0x1 << 11;
pub const SGI_CTRL_OUTSEL_KERNEL_XOR_DATIN1: u32 = 0x2 << 11;
pub const SGI_CTRL_OUTSEL_KERNEL_XOR_DATIN2: u32 = 0x3 << 11;
pub const SGI_CTRL_OUTSEL_KERNEL_XOR_DATIN3: u32 = 0x4 << 11;
pub const SGI_CTRL_DATOUT_RES_SHIFT: u32 = 14;
pub const SGI_CTRL_DATOUT_RES_END_UP: u32 = 0x0 << 14;
pub const SGI_CTRL_DATOUT_RES_START_UP: u32 = 0x1 << 14;
pub const SGI_CTRL_DATOUT_RES_TRIGGER_UP: u32 = 0x2 << 14;
pub const SGI_CTRL_DATOUT_RES_NO_UP: u32 = 0x3 << 14;
pub const SGI_CTRL_AES_EN: u32 = 1 << 16;
pub const SGI_CTRL_DES_EN: u32 = 1 << 17;
pub const SGI_CTRL_GCM_EN: u32 = 1 << 18;
pub const SGI_CTRL_PRNG_EN: u32 = 1 << 19;
pub const SGI_CTRL_INKEYSEL_SHIFT: u32 = 20;
pub const SGI_CTRL_TDESKEY: u32 = 1 << 25;
pub const SGI_CTRL_AES_NO_KL: u32 = 1 << 26;
pub const SGI_CTRL_AES_SEL: u32 = 1 << 27;

// SGI_SHA2_CTRL register bitfields
pub const SGI_SHA2_CTRL_EN: u32 = 1 << 0;
pub const SGI_SHA2_CTRL_MODE_AUTO: u32 = 1 << 1;
pub const SGI_SHA2_CTRL_MODE_NORMAL: u32 = 0 << 1;
pub const SGI_SHA2_CTRL_SIZE_SHIFT: u32 = 2;
pub const SGI_SHA2_CTRL_SIZE_SHA384: u32 = 2 << 2;
pub const SGI_SHA2_CTRL_SIZE_SHA512: u32 = 3 << 2;
pub const SGI_SHA2_CTRL_LOW_LIM_SHIFT: u32 = 4;
pub const SGI_SHA2_CTRL_HIGH_LIM_SHIFT: u32 = 8;
pub const SGI_SHA2_CTRL_LOW_LIM: u32 = 0 << SGI_SHA2_CTRL_LOW_LIM_SHIFT; // FIFO low limit.
pub const SGI_SHA2_CTRL_HIGH_LIM_NORMAL_BLOCK: u32 = 7 << SGI_SHA2_CTRL_HIGH_LIM_SHIFT; // FIFO high limit for AUTO mode normal block (7 means 8 blocks, since limit is inclusive), 8 blocks * 4 words/block = 32 words = 128 bytes, which is the maximum block size.
pub const SGI_SHA2_CTRL_HIGH_LIM_NORMAL_RELOAD: u32 = 3 << SGI_SHA2_CTRL_HIGH_LIM_SHIFT; // FIFO high limit for AUTO mode reload block (3 means 4 blocks, since limit is inclusive), 4 blocks * 4 words/block = 16 words = 64 bytes.
pub const SGI_SHA2_CTRL_COUNT_EN: u32 = 1 << 12;
pub const SGI_SHA2_CTRL_HASH_RELOAD: u32 = 1 << 13;
pub const SGI_SHA2_CTRL_NO_HASH_RELOAD: u32 = 0 << 13;
pub const SGI_SHA2_CTRL_STOP: u32 = 1 << 14;
pub const SGI_SHA2_CTRL_NO_AUTO_INIT: u32 = 1 << 15;
pub const SGI_SHA2_CTRL_AUTO_INIT: u32 = 0 << 15;

// SGI_STATUS register bitfields
pub const SGI_STATUS_BUSY: u32 = 1 << 0;
pub const SGI_STATUS_OFLOW: u32 = 1 << 1;
pub const SGI_STATUS_PRNG_RDY: u32 = 1 << 2;
pub const SGI_STATUS_ERROR_SHIFT: u32 = 3;
pub const SGI_STATUS_ERROR_MASK: u32 = 0x7 << SGI_STATUS_ERROR_SHIFT;
pub const SGI_NO_ERROR: u32 = 0x5;
pub const SGI_STATUS_SHA2_BUSY: u32 = 1 << 6;
pub const SGI_STATUS_IRQ: u32 = 1 << 7;
pub const SGI_STATUS_SHA_FIFO_FULL: u32 = 1 << 8;
pub const SGI_STATUS_SHA_FIFO_LEVEL_SHIFT: u32 = 9;
pub const SGI_STATUS_SHA_FIFO_LEVEL_MASK: u32 = 0x3F << 9;
pub const SGI_STATUS_SHA_ERROR: u32 = 1 << 15;
pub const SGI_STATUS_KEY_READ_ERR: u32 = 1 << 16;
pub const SGI_STATUS_KEY_UNWRAP_ERR: u32 = 1 << 17;

// SGI_CTRL2 register bitfields
pub const SGI_CTRL2_FLUSH: u32 = 1 << 0;
pub const SGI_CTRL2_KEY_FLUSH: u32 = 1 << 1;
pub const SGI_CTRL2_DATIN_FLUSH: u32 = 1 << 2;
pub const SGI_CTRL2_INCR: u32 = 1 << 3;
pub const SGI_CTRL2_XORWR: u32 = 1 << 4;
pub const SGI_CTRL2_FLUSHWR: u32 = 1 << 5;
pub const SGI_CTRL2_INCR_CIN: u32 = 1 << 6;
pub const SGI_CTRL2_SMASKEN: u32 = 1 << 8;
pub const SGI_CTRL2_SMASKSTEP: u32 = 1 << 9;
pub const SGI_CTRL2_SMASKSW: u32 = 1 << 10;
pub const SGI_CTRL2_MOVEM_SHIFT: u32 = 12;
pub const SGI_CTRL2_MOVEM_MASK: u32 = 0xF << 12;
pub const SGI_CTRL2_KEYRES_SHIFT: u32 = 16;
pub const SGI_CTRL2_KEYRES_MASK: u32 = 0x1F << 16;
pub const SGI_CTRL2_RKEY: u32 = 1 << 21;
pub const SGI_CTRL2_BYTES_ORDER: u32 = 1 << 22;
pub const SGI_CTRL2_GCM_INXOR: u32 = 1 << 23;

// SHA output sizes
pub const SGI_HASH_OUTPUT_SIZE: usize = 64;
pub const MAX_BLOCK_SIZE: usize = 128; // Maximum block size for SHA-384/SHA-512

// SGI_AUTO_DMA_CTRL register bitfields
pub const SGI_AUTO_DMA_CTRL_IFE: u32 = 1 << 0; // Input FIFO DMA handshake enable - for non SHA operations only.
pub const SGI_AUTO_DMA_CTRL_OFE: u32 = 1 << 8; // Output FIFO DMA handshake enable - for non SHA operations only.

// ============================================================================
// SGI structs and helpers

/// SGI Error types
#[derive(Debug, Clone, Copy)]
pub enum SGIError {
    SGIInstanceAlreadyExists,
    Busy,
    SHA2Busy,
    ShaError,
    KeyReadError,
    KeyUnwrapError,
    BufferTooSmall,
    InvalidSize,
    Timeout,
    HardwareError,
    DmaError,
    HashingNotComplete,
}

// ============================================================================
// SGI driver interface and implementations

static SGI_EXISTS: AtomicBool = AtomicBool::new(false);
/// SGI (Secure Generic Interface) hardware abstraction
pub struct Sgi {
    base_addr: u32,
}

impl Drop for Sgi {
    fn drop(&mut self) {
        // Ensure the global singleton lock is always released, even if callers
        // return early via `?` before reaching an explicit `release()`.
        //
        // Keep Drop best-effort and non-blocking.
        if self.is_sha2_busy() {
            self.sgi_stop_sha2_cmd();
        }

        // Don't leave DMA handshakes enabled if the driver is dropped mid operation.
        self.set_auto_dma_handshake(false, None);

        if self.sgi_operation_error().is_err() || self.sgi_error().is_err() {
            // If errors occurred, busy gets de-asserted and no interrupt is generated.
            self.clear_errors();
        }
        self.release();
    }
}

pub(crate) fn is_expired(start: Instant, timeout: u64) -> bool {
    Instant::now().duration_since(start).as_micros() > timeout
}

impl Sgi {
    /// Create a new SGI instance with the default base address
    pub fn new() -> Result<Self, SGIError> {
        if SGI_EXISTS.swap(true, Ordering::SeqCst) {
            return Err(SGIError::SGIInstanceAlreadyExists); // Another instance already exists
        }
        Ok(Self { base_addr: SGI0_BASE })
    }

    pub(crate) fn release(&mut self) {
        SGI_EXISTS.store(false, Ordering::SeqCst);
    }

    #[inline(always)]
    fn read_reg(&self, offset: u32) -> u32 {
        unsafe { core::ptr::read_volatile((self.base_addr + offset) as *const u32) }
    }

    #[inline(always)]
    fn write_reg(&self, offset: u32, value: u32) {
        unsafe { core::ptr::write_volatile((self.base_addr + offset) as *mut u32, value) }
    }

    #[inline(always)]
    fn status(&self) -> u32 {
        self.read_reg(SGI_STATUS_OFFSET)
    }

    pub fn set_auto_dma_handshake(&mut self, input_fifo_enable: bool, output_fifo_enable: Option<bool>) {
        let mut value = 0u32;
        if input_fifo_enable {
            value |= SGI_AUTO_DMA_CTRL_IFE;
        }
        let output_enable = output_fifo_enable.unwrap_or(false); // Default to false if not provided
        if output_enable {
            value |= SGI_AUTO_DMA_CTRL_OFE;
        }

        // Reserved bits are written as 0.
        self.write_reg(SGI_AUTO_DMA_CTRL_OFFSET, value);
    }

    pub fn init_sgi_sha(&mut self, options: HashOptions) -> Result<(), SGIError> {
        if self.is_busy() || self.is_sha2_busy() {
            return Err(SGIError::Busy);
        }
        self.sgi_byte_order_big(options.byte_order_big)?; // Set byte order for input data.
        let mut fifo_low_lim = 0;
        let mut fifo_high_lim = 0;
        if options.op_mode == SGI_SHA2_CTRL_MODE_NORMAL {
            fifo_low_lim = SGI_SHA2_CTRL_LOW_LIM;
            fifo_high_lim = SGI_SHA2_CTRL_HIGH_LIM_NORMAL_BLOCK;
        }
        // Enable hash engine with FIFO limits for auto mode.
        self.write_reg(
            SGI_SHA2_CTRL_OFFSET,
            SGI_SHA2_CTRL_EN
                | options.op_mode
                | options.hash_mode
                | SGI_SHA2_CTRL_COUNT_EN
                | options.init
                | options.reload
                | fifo_low_lim
                | fifo_high_lim,
        );
        Ok(())
    }

    pub fn sgi_byte_order_big(&self, byte_order_big: bool) -> Result<(), SGIError> {
        if self.is_busy() || self.is_sha2_busy() {
            return Err(SGIError::Busy);
        }
        let ctrl2_val = self.read_reg(SGI_CTRL2_OFFSET);
        let new_val = if byte_order_big {
            ctrl2_val & !SGI_CTRL2_BYTES_ORDER
        } else {
            ctrl2_val | SGI_CTRL2_BYTES_ORDER
        };
        self.write_reg(SGI_CTRL2_OFFSET, new_val);
        Ok(())
    }

    pub fn start_sgi_hash(&mut self, options: HashOptions, data: &[u8]) -> Result<(), SGIError> {
        if self.is_busy() || self.is_sha2_busy() {
            return Err(SGIError::Busy);
        }

        if options.op_mode == SGI_SHA2_CTRL_MODE_NORMAL {
            self.fill_sha2_fifo_normal(data)?;
        }
        // Keep as a single write to the CTRL register.
        self.write_reg(SGI_CTRL_OFFSET, SGI_CTRL_CRYPTO_OP_SHA2 | SGI_CTRL_START);
        Ok(())
    }

    pub fn fill_sha2_fifo_normal(&mut self, data: &[u8]) -> Result<(), SGIError> {
        const MAX_FIFO_NORM_WORD_SIZE: usize = 32;
        // Max size = (4 datain +  8 keyin) banks *  4 words per bank = 48 words = 192 bytes;
        // but we set it to 32 words (128 bytes) to fit one full SHA2 block, since the hardware processes data in blocks
        // Filling more than one block at a time may not be supported or may require additional handling.
        // but realistically we can only fill one SHA2 block of 1024 bits (128 bytes) at a time, which is 32 words.
        if data.len() > MAX_FIFO_NORM_WORD_SIZE * 4 {
            return Err(SGIError::InvalidSize);
        }
        const STARTING_FIFO_WORD_OFFSET: u32 = SGI_DATIN0A_OFFSET; // Starting with DATIN0A
        for (i, chunk) in data.chunks_exact(4).enumerate() {
            let word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            self.write_reg(STARTING_FIFO_WORD_OFFSET + (i as u32 * 4), word);
        }
        Ok(())
    }

    pub fn fill_sha2_fifo(&mut self, options: HashOptions, data: &[u8], length: usize) -> Result<(), SGIError> {
        if options.op_mode == SGI_SHA2_CTRL_MODE_AUTO {
            self.fill_sha2_fifo_auto(data, length)?;
        }
        Ok(())
    }

    pub fn is_sha2_fifo_full(&self) -> bool {
        (self.status() & SGI_STATUS_SHA_FIFO_FULL) != 0
    }

    pub fn fill_sha2_fifo_auto(&mut self, data: &[u8], length: usize) -> Result<(), SGIError> {
        for chunk in data[..length].chunks_exact(4) {
            let word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            while self.is_sha2_fifo_full() {
                // Wait until there is space in the FIFO.
                core::hint::spin_loop();
            }
            self.write_reg(SGI_SHA_FIFO_OFFSET, word);
        }
        Ok(())
    }

    pub fn fill_sha2_fifo_dma_start(
        &mut self,
        dma_ch: &mut DmaChannel,
        data: &[u8],
        length: usize,
    ) -> Result<(), SGIError> {
        if length == 0 || length > data.len() || length % MAX_BLOCK_SIZE != 0 {
            return Err(SGIError::InvalidSize);
        }

        // Destination is SHAFIFO (32-bit). If the source buffer is not word-aligned,
        // we may need smaller SSIZE. Whether SSIZE<DSIZE is safe for SHAFIFO depends
        // on the DMA/peripheral semantics; warn loudly so callers can align buffers.
        let align_addr = data.as_ptr() as usize;
        if (align_addr % 4) != 0 {
            info!("Data buffer is not word-aligned, which means 4x AHB reads.");
        }

        let length_u32 = u32::try_from(length).map_err(|_| SGIError::InvalidSize)?;

        // Match NXP mcuxClDma_Utils_configureSgiSha2InputChannel style:
        // - Major loop count = 1
        // - NBYTES = total length
        // - Destination address fixed to SHAFIFO (DOFF = 0)

        let peri_address = (self.base_addr + SGI_SHA_FIFO_OFFSET) as *mut u32;

        let src_addr = data.as_ptr() as usize;
        let (ssize, soff) = if (src_addr % 4) == 0 && (length % 4) == 0 {
            (Size::THIRTYTWO_BIT, 4u16)
        } else if (src_addr % 2) == 0 && (length % 2) == 0 {
            (Size::SIXTEEN_BIT, 2u16)
        } else {
            (Size::EIGHT_BIT, 1u16)
        };

        // Needed for async completion (DMA IRQ -> waker).
        dma_ch.enable_interrupt();

        {
            let t = dma_ch.tcd();

            // Clear prior DONE/INT/ERR.
            t.ch_csr().write(|w| w.set_done(true));
            t.ch_es().write(|w| w.set_err(true));
            t.ch_int().write(|w| w.set_int(true));

            // Program the TCD for a multi-block transfer:

            t.tcd_saddr().write(|w| w.set_saddr(data.as_ptr() as u32));
            t.tcd_daddr().write(|w| w.set_daddr(peri_address as u32));

            t.tcd_soff().write(|w| w.set_soff(soff));
            t.tcd_doff().write(|w| w.set_doff(0u16));

            t.tcd_attr().write(|w| {
                w.set_ssize(ssize);
                w.set_dsize(Size::THIRTYTWO_BIT);
                w.set_smod(0);
                w.set_dmod(0);
            });

            t.tcd_nbytes_mloffno().write(|w| w.set_nbytes(length_u32));
            t.tcd_slast_sda().write(|w| w.set_slast_sda(0));
            t.tcd_dlast_sga().write(|w| w.set_dlast_sga(0));

            t.tcd_biter_elinkno().write(|w| w.set_biter(1));
            t.tcd_citer_elinkno().write(|w| w.set_citer(1));

            // Ensure all TCD writes are visible before starting.
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

            t.tcd_csr().write(|w| {
                w.set_intmajor(true);
                w.set_inthalf(false);
                w.set_dreq(Dreq::ERQ_FIELD_CLEAR);
                w.set_esg(Esg::NORMAL_FORMAT);
                w.set_majorelink(false);
                w.set_eeop(false);
                w.set_esda(false);
                w.set_bwc(Bwc::NO_STALL);
                w.set_start(Start::CHANNEL_STARTED);
            });
        }

        info!("DMA started for {} bytes (CITER=1, NBYTES={})", length, length_u32);
        Ok(())
    }

    pub fn sgi_operation_error(&self) -> Result<(), SGIError> {
        let status = self.status();
        if (status & SGI_STATUS_SHA_ERROR) != 0 {
            return Err(SGIError::ShaError);
        }
        if (status & SGI_STATUS_KEY_READ_ERR) != 0 {
            return Err(SGIError::KeyReadError);
        }
        if (status & SGI_STATUS_KEY_UNWRAP_ERR) != 0 {
            return Err(SGIError::KeyUnwrapError);
        }
        Ok(())
    }

    pub fn sgi_error(&self) -> Result<(), SGIError> {
        let status = self.status();
        let error_val = (status & SGI_STATUS_ERROR_MASK) >> SGI_STATUS_ERROR_SHIFT;
        if error_val != SGI_NO_ERROR {
            self.clear_errors(); // Clear errors after reading
            return Err(SGIError::HardwareError);
        }
        Ok(())
    }

    pub fn is_busy(&self) -> bool {
        (self.status() & SGI_STATUS_BUSY) != 0
    }

    pub fn is_sha2_busy(&self) -> bool {
        (self.status() & SGI_STATUS_SHA2_BUSY) != 0
    }

    pub fn wait_until_sha2_not_busy(&mut self) -> Result<(), SGIError> {
        let start_time = Instant::now();
        const MAX_WAIT_TIME_USEC: u64 = 1000; // 1 millisecond timeout for normal SHA2 operations.
        while self.is_sha2_busy() {
            if is_expired(start_time, MAX_WAIT_TIME_USEC) {
                // 1 millisecond timeout
                error!("Timeout waiting for SGI SHA2 operation to complete");
                return Err(SGIError::Timeout);
            }
            core::hint::spin_loop();
        }
        Ok(())
    }

    pub fn read_dataout_block(&self, output: &mut [u8], idx: usize) -> Result<(), SGIError> {
        let dataout_offset = SGI_DATOUT_START_OFFSET as usize;
        const TOTAL_DATAOUT_REGS: usize = 4; // DATOUTA, DATOUTB, DATOUTC, DATOUTD
        for i in 0..TOTAL_DATAOUT_REGS {
            let word = self.read_reg(dataout_offset as u32 + (i as u32 * 4));
            output[idx + (i * 4)..idx + (i * 4) + 4].copy_from_slice(&word.to_be_bytes());
        }
        Ok(())
    }

    pub fn interrupt_is_operation_done(&self) -> Result<bool, SGIError> {
        // Only report errors once the SHA2 engine is no longer busy.
        // This keeps error/timeout reporting consistent with the wait path.
        if self.is_sha2_busy() {
            return Ok(false);
        }

        self.sgi_operation_error()?;
        self.sgi_error()?;
        let interrupt_status = self.read_reg(SGI_INT_STATUS_OFFSET);
        Ok((interrupt_status & STATUS_SGI_INT_PDONE) != 0)
    }

    pub fn clear_operation_interrupt(&self) {
        self.write_reg(SGI_INT_STATUS_CLR_OFFSET, STATUS_SGI_INT_CLR);
    }

    pub fn enable_sgi_interrupt(&self) {
        self.write_reg(SGI_INT_ENABLE_OFFSET, STATUS_SGI_INT_EN);
    }

    pub fn clear_errors(&self) {
        self.write_reg(SGI_CTRL2_OFFSET, SGI_CTRL2_FLUSH);
    }

    pub fn sgi_stop_sha2_cmd(&self) {
        self.write_reg(SGI_SHA2_CTRL_OFFSET, SGI_SHA2_CTRL_STOP);
    }

    pub fn read_hash_output(&mut self, options: HashOptions, output: &mut [u8]) -> Result<(), SGIError> {
        if options.hash_mode == SGI_SHA2_CTRL_SIZE_SHA384 && output.len() < 48 {
            return Err(SGIError::BufferTooSmall);
        } else if options.hash_mode == SGI_SHA2_CTRL_SIZE_SHA512 && output.len() < 64 {
            return Err(SGIError::BufferTooSmall);
        }
        if options.op_mode == SGI_SHA2_CTRL_MODE_AUTO {
            // Automatic mode
            self.write_reg(SGI_SHA2_CTRL_OFFSET, SGI_SHA2_CTRL_STOP); // Stop AUTO hash.
            let mut loop_counter = 0;
            while self.is_busy() && loop_counter < 1000 {
                loop_counter += 1;
                core::hint::spin_loop();
                // The de-assertion of the busy bit in auto mode is a single pulse; a simple
                // count-based timeout is sufficient here.
            }
        }
        self.wait_until_sha2_not_busy()?;

        self.sgi_byte_order_big(true)?; // SHA output is always in big-endian order, so set byte order accordingly for reading the output hash.

        if self.sgi_operation_error().is_err() {
            self.clear_errors(); // Clear errors after reading
            return Err(SGIError::ShaError);
        }
        if self.sgi_error().is_err() {
            self.clear_errors(); // Clear errors after reading
            return Err(SGIError::HardwareError);
        }

        let max_sha2_dataout_pass = match options.hash_mode {
            SGI_SHA2_CTRL_SIZE_SHA384 => 2,
            SGI_SHA2_CTRL_SIZE_SHA512 => 3,
            _ => 0,
        }; // each pass reads 4 * 4 bytes = 16 bytes, so for SHA-384 we need 2 passes, for SHA-512 we need 3 passes; AFTER the first END_UP reading.
        let idx: usize = 0;

        self.read_dataout_block(output, idx)?;

        for i in 0..max_sha2_dataout_pass {
            self.write_reg(SGI_CTRL_OFFSET, SGI_CTRL_DATOUT_RES_TRIGGER_UP | SGI_CTRL_START);
            self.wait_until_sha2_not_busy()?;
            self.read_dataout_block(output, idx + (16 * (i + 1)))?; // Each block is 16 bytes
        }
        Ok(())
    }

    pub fn update_partial_output(&mut self, options: HashOptions, output: &mut [u8]) -> Result<(), SGIError> {
        if output.len() < 64 {
            return Err(SGIError::BufferTooSmall);
        }
        if options.op_mode == SGI_SHA2_CTRL_MODE_AUTO {
            // Automatic mode
            self.write_reg(SGI_SHA2_CTRL_OFFSET, SGI_SHA2_CTRL_STOP); // Stop AUTO hash.
            let mut loop_counter = 0;
            while self.is_busy() && loop_counter < 1000 {
                loop_counter += 1;
                core::hint::spin_loop();
                // The de-assertion of the busy bit in auto mode is a single pulse; a simple
                // count-based timeout is sufficient here.
            }
        }
        self.wait_until_sha2_not_busy()?;

        self.sgi_byte_order_big(true)?; // SHA output is always in big-endian order, so set byte order accordingly for reading the output hash.

        if self.sgi_operation_error().is_err() {
            self.clear_errors(); // Clear errors after reading
            return Err(SGIError::ShaError);
        }
        if self.sgi_error().is_err() {
            self.clear_errors(); // Clear errors after reading
            return Err(SGIError::HardwareError);
        }

        let max_sha2_dataout_pass = 3; // need all bytes read for update; each pass reads 4 * 4 bytes = 16 bytes, so need 3 more AFTER the first END_UP reading.
        let idx: usize = 0;

        self.read_dataout_block(output, idx)?;

        for i in 0..max_sha2_dataout_pass {
            self.write_reg(SGI_CTRL_OFFSET, SGI_CTRL_DATOUT_RES_TRIGGER_UP | SGI_CTRL_START);
            self.wait_until_sha2_not_busy()?;
            self.read_dataout_block(output, idx + (16 * (i + 1)))?; // Each block is 16 bytes
        }
        Ok(())
    }

    pub fn sgi_hash_reload(&mut self, mut options: HashOptions, prev_hash_result: &[u8]) -> Result<(), SGIError> {
        let curr_op_mode = options.op_mode;
        let curr_reload_setting = options.reload;
        let curr_init_setting = options.init;

        // Reload the internal hash state using the DATIN path (NORMAL mode) with HASH_RELOAD enabled.
        // This must NOT auto-init, otherwise the hardware IV would overwrite the provided state.
        options.op_mode = SGI_SHA2_CTRL_MODE_NORMAL;
        options.reload = SGI_SHA2_CTRL_HASH_RELOAD; // Set reload bit for hash reload operation
        options.init = SGI_SHA2_CTRL_NO_AUTO_INIT; // Clear auto init bit for hash reload operation, since we are providing the hash state to be reloaded and don't want SGI to overwrite it with IV.
        if prev_hash_result.len() != SGI_HASH_OUTPUT_SIZE as usize {
            return Err(SGIError::InvalidSize);
        }
        self.init_sgi_sha(options)?;
        self.start_sgi_hash(options, prev_hash_result)?;
        self.fill_sha2_fifo(options, prev_hash_result, prev_hash_result.len())?;
        // Load the hash state into the SGI using the appropriate input registers.
        self.wait_until_sha2_not_busy()?; // Wait until reload is complete and SGI is ready for next operation
        options.op_mode = curr_op_mode; // Restore original op mode
        options.reload = curr_reload_setting; // Restore original reload setting
        options.init = curr_init_setting; // Restore original init setting
        Ok(())
    }
}

// =============================================================================
