// SGI (Secure Generic Interface) driver for MCXA
// Provides hashing, AES encryption/decryption, and HMAC primitives.
// Currently focused on hashing (SHA-384 and SHA-512) use cases, with AES and HMAC planned for future additions.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
use embassy_time::Instant;
use maitake_sync::WaitCell;
pub mod hash;
pub use hash::{
    BlockingHasher, ByteOrder, HashInit, HashMode, HashOptions as SgiHashOptions, HashReload, HashSize, MAX_BLOCK_SIZE,
    SGI_HASH_OUTPUT_SIZE, StreamingHasher,
};

use self::hash::HashOptions;
use crate::clocks::periph_helpers::Clk1MConfig;
use crate::clocks::{ClockError, WakeGuard, enable_and_reset};
use crate::dma::{DmaChannel, Transfer, TransferOptions};
use crate::interrupt::typelevel::{Binding, Handler, Interrupt as _};
use crate::pac::sgi as pac_sgi;
use crate::peripherals;

trait SealedInstance: crate::clocks::Gate<MrccPeriphConfig = Clk1MConfig> {
    fn info() -> &'static Info;
}

mod sealed {
    pub trait SealedMode {}
}

/// SGI peripheral instance.
///
/// This is a sealed trait to ensure only the HAL-defined SGI peripherals can be
/// used as instances.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + embassy_hal_internal::PeripheralType + 'static + Send {
    type Interrupt: crate::interrupt::typelevel::Interrupt;
}

pub(crate) struct Info {
    regs: pac_sgi::Sgi,
    wait_cell: WaitCell,
}

impl Info {
    #[inline(always)]
    fn regs(&self) -> pac_sgi::Sgi {
        self.regs
    }

    #[inline(always)]
    fn wait_cell(&self) -> &WaitCell {
        &self.wait_cell
    }
}

// SAFETY: pac_sgi::Sgi is a zero-sized register block accessor (pointer), and WaitCell is Sync.
unsafe impl Sync for Info {}

impl SealedInstance for peripherals::SGI0 {
    fn info() -> &'static Info {
        static INFO: Info = Info {
            regs: crate::pac::SGI0,
            wait_cell: WaitCell::new(),
        };
        &INFO
    }
}

impl Instance for peripherals::SGI0 {
    type Interrupt = crate::interrupt::typelevel::SGI;
}

/// SGI interrupt handler. Bind this to the SGI interrupt via bind_interrupts!, or use
/// Sgi::new() with SgiInterrupt::AsyncEnabled which enables the interrupt source manually.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::info().regs();
        regs.sgi_int_status_clr().write(|w| w.set_int_clr(true));
        T::info().wait_cell().wake();
    }
}

/// Blocking driver mode.
pub struct Blocking;
impl sealed::SealedMode for Blocking {}
impl Mode for Blocking {}

/// Async driver mode.
pub struct Async;
impl sealed::SealedMode for Async {}
impl Mode for Async {}

/// Trait implemented by Blocking and Async mode markers.
pub trait Mode: sealed::SealedMode {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum CryptoOp {
    // TODO: Implement other operations (e.g. AES) as needed. For now we only support SHA2, so this is the only variant.
    // Aes = 0,
    // Des = 1,
    // Tdes = 2,
    // Gfmul = 3,
    Sha2 = 4,
    // Cmac = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum DataOutRes {
    // SHA only needs TRIGGER_UP, other future operations will need more states.
    // EndUp = 0,
    // StartUp = 1,
    TriggerUp = 2,
    // NoUp = 3,
}

// FIFO limits used by SHA2 configuration.
const SHA2_FIFO_LOW_LIM: u8 = 0;
const SHA2_FIFO_HIGH_LIM_NORMAL_BLOCK: u8 = 7;

/// Errors exclusive to SGI peripheral initialization.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SetupError {
    /// Clock configuration error.
    ClockSetup(ClockError),
}

/// SGI Error types
#[derive(Debug, Clone, Copy)]
pub enum SgiError {
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

/// SGI (Secure Generic Interface) hardware abstraction.
pub struct Sgi<'d, M: Mode = Blocking> {
    info: &'static Info,
    _phantom: PhantomData<&'d mut M>,
    _wake_guard: Option<WakeGuard>,
}

impl<M: Mode> Drop for Sgi<'_, M> {
    fn drop(&mut self) {
        if self.is_sha2_busy() {
            self.sgi_stop_sha2_cmd();
        }

        // Don't leave DMA handshakes enabled if the driver is dropped mid operation.
        self.set_auto_dma_handshake(false, None);

        if self.sgi_operation_error().is_err() || self.sgi_error().is_err() {
            // If errors occurred, busy gets de-asserted and no interrupt is generated.
            self.clear_errors();
        }
    }
}

pub(crate) fn is_expired(start: Instant, timeout: u64) -> bool {
    Instant::now().duration_since(start).as_micros() > timeout
}

impl<'d, M: Mode> Sgi<'d, M> {
    fn new_inner<T: Instance>(_peri: Peri<'d, T>) -> Result<Self, SetupError> {
        let parts = unsafe { enable_and_reset::<T>(&Clk1MConfig).map_err(SetupError::ClockSetup)? };
        Ok(Self {
            info: T::info(),
            _phantom: PhantomData,
            _wake_guard: parts.wake_guard,
        })
    }
}

impl<'d> Sgi<'d, Async> {
    /// Create a new async SGI instance. Requires a bound interrupt handler via `bind_interrupts!`.
    pub fn new(
        peri: Peri<'d, peripherals::SGI0>,
        _irq: impl Binding<crate::interrupt::typelevel::SGI, InterruptHandler<peripherals::SGI0>> + 'd,
    ) -> Result<Self, SetupError> {
        let sgi = Self::new_inner(peri)?;
        crate::interrupt::typelevel::SGI::unpend();
        unsafe { crate::interrupt::typelevel::SGI::enable() };
        Ok(sgi)
    }

    /// Await SHA2 completion via the SGI operation-done interrupt.
    ///
    /// Hardware error paths may deassert busy without generating an interrupt;
    /// this function treats that as completion and returns the decoded error.
    pub(crate) async fn wait_sha2_complete_irq(&mut self) -> Result<(), SgiError> {
        self.info
            .wait_cell()
            .wait_for_value(|| {
                if self.is_sha2_busy() {
                    return None;
                }
                if let Err(e) = self.sgi_operation_error() {
                    return Some(Err(e));
                }
                if let Err(e) = self.sgi_error() {
                    return Some(Err(e));
                }
                Some(Ok(()))
            })
            .await
            .map_err(|_| SgiError::HardwareError)?
    }
}

impl<'d> Sgi<'d, Blocking> {
    /// Create a blocking-only SGI instance. No interrupt is enabled.
    pub fn new_blocking(peri: Peri<'d, peripherals::SGI0>) -> Result<Self, SetupError> {
        Self::new_inner(peri)
    }
}

impl<'d, M: Mode> Sgi<'d, M> {
    /// Reads SGI status register.
    #[inline(always)]
    fn status(&self) -> pac_sgi::regs::SgiStatus {
        self.info.regs().sgi_status().read()
    }

    /// Enables DMA handshake for datain/dataout registers.
    pub(crate) fn set_auto_dma_handshake(&mut self, input_fifo_enable: bool, output_fifo_enable: Option<bool>) {
        let output_enable = output_fifo_enable.unwrap_or(false); // Default to false if not provided

        // Reserved bits are written as 0.
        self.info.regs().sgi_auto_dma_ctrl().write(|w| {
            w.set_ife(input_fifo_enable);
            w.set_ofe(output_enable);
        });
    }

    /// Configures the appropriate hash operation parameters (e.g. auto vs. normal mode, hash size, byte order) and enables the hash engine.
    pub(crate) fn init_sgi_sha(&mut self, options: HashOptions) -> Result<(), SgiError> {
        self.sgi_byte_order_big(options.byte_order)?; // Set byte order for input data.
        let mut fifo_low_lim = 0u8;
        let mut fifo_high_lim = 0u8;
        if options.op_mode == HashMode::Normal {
            fifo_low_lim = SHA2_FIFO_LOW_LIM;
            fifo_high_lim = SHA2_FIFO_HIGH_LIM_NORMAL_BLOCK;
        }

        let sha2_mode_auto = options.op_mode == HashMode::Auto;
        let sha2_size = match options.hash_size {
            HashSize::Sha384 => 2u8,
            HashSize::Sha512 => 3u8,
        };
        let hash_reload = options.reload == HashReload::Reload;
        let no_auto_init = options.init == HashInit::NoInit;

        // Enable hash engine with FIFO limits.
        self.info.regs().sgi_sha2_ctrl().write(|w| {
            w.set_sha2_en(true.into());
            w.set_sha2_mode((sha2_mode_auto as u8).into());
            w.set_sha2_size(sha2_size.into());
            w.set_sha2_low_lim(fifo_low_lim);
            w.set_sha2_high_lim(fifo_high_lim);
            w.set_sha2_count_en((true as u8).into());
            w.set_hash_reload(hash_reload.into());
            w.set_no_auto_init((no_auto_init as u8).into());
        });
        Ok(())
    }

    /// Configures byte order for SGI IO.
    pub(crate) fn sgi_byte_order_big(&self, byte_order: ByteOrder) -> Result<(), SgiError> {
        if self.is_busy() || self.is_sha2_busy() {
            return Err(SgiError::Busy);
        }
        let byte_order_big = byte_order == ByteOrder::BigEndian;

        // The hardware uses an inverted convention here in our existing driver:
        // - `byte_order_big = true` => clear BYTES_ORDER
        // - `byte_order_big = false` => set BYTES_ORDER
        self.info
            .regs()
            .sgi_ctrl2()
            .modify(|w| w.set_bytes_order(((!byte_order_big) as u8).into()));
        Ok(())
    }

    /// Give command to start SGI hash operation. Fills buffer if normal mode.
    pub(crate) fn start_sgi_hash(&mut self, options: HashOptions, data: &[u8]) -> Result<(), SgiError> {
        if self.is_busy() || self.is_sha2_busy() {
            return Err(SgiError::Busy);
        }

        if options.op_mode == HashMode::Normal {
            self.fill_sha2_fifo_normal(data)?;
        }
        // Keep as a single write to the CTRL register.
        self.info.regs().sgi_ctrl().write(|w| {
            w.set_crypto_op((CryptoOp::Sha2 as u8).into());
            w.set_start((true as u8).into());
        });
        Ok(())
    }

    /// Helper that maps normal mode datain/keyin registers writes to an index.
    #[inline(always)]
    fn write_fifo_word_normal(&self, word_index: usize, word: u32) {
        match word_index {
            0 => self.info.regs().sgi_datin0a().write(|w| w.set_datin0a(word)),
            1 => self.info.regs().sgi_datin0b().write(|w| w.set_datin0b(word)),
            2 => self.info.regs().sgi_datin0c().write(|w| w.set_datin0c(word)),
            3 => self.info.regs().sgi_datin0d().write(|w| w.set_datin0d(word)),
            4 => self.info.regs().sgi_datin1a().write(|w| w.set_datin1a(word)),
            5 => self.info.regs().sgi_datin1b().write(|w| w.set_datin1b(word)),
            6 => self.info.regs().sgi_datin1c().write(|w| w.set_datin1c(word)),
            7 => self.info.regs().sgi_datin1d().write(|w| w.set_datin1d(word)),
            8 => self.info.regs().sgi_datin2a().write(|w| w.set_datin2a(word)),
            9 => self.info.regs().sgi_datin2b().write(|w| w.set_datin2b(word)),
            10 => self.info.regs().sgi_datin2c().write(|w| w.set_datin2c(word)),
            11 => self.info.regs().sgi_datin2d().write(|w| w.set_datin2d(word)),
            12 => self.info.regs().sgi_datin3a().write(|w| w.set_datin3a(word)),
            13 => self.info.regs().sgi_datin3b().write(|w| w.set_datin3b(word)),
            14 => self.info.regs().sgi_datin3c().write(|w| w.set_datin3c(word)),
            15 => self.info.regs().sgi_datin3d().write(|w| w.set_datin3d(word)),
            16 => self.info.regs().sgi_key0a().write(|w| w.set_key0a(word)),
            17 => self.info.regs().sgi_key0b().write(|w| w.set_key0b(word)),
            18 => self.info.regs().sgi_key0c().write(|w| w.set_key0c(word)),
            19 => self.info.regs().sgi_key0d().write(|w| w.set_key0d(word)),
            20 => self.info.regs().sgi_key1a().write(|w| w.set_key1a(word)),
            21 => self.info.regs().sgi_key1b().write(|w| w.set_key1b(word)),
            22 => self.info.regs().sgi_key1c().write(|w| w.set_key1c(word)),
            23 => self.info.regs().sgi_key1d().write(|w| w.set_key1d(word)),
            24 => self.info.regs().sgi_key2a().write(|w| w.set_key2a(word)),
            25 => self.info.regs().sgi_key2b().write(|w| w.set_key2b(word)),
            26 => self.info.regs().sgi_key2c().write(|w| w.set_key2c(word)),
            27 => self.info.regs().sgi_key2d().write(|w| w.set_key2d(word)),
            28 => self.info.regs().sgi_key3a().write(|w| w.set_key3a(word)),
            29 => self.info.regs().sgi_key3b().write(|w| w.set_key3b(word)),
            30 => self.info.regs().sgi_key3c().write(|w| w.set_key3c(word)),
            31 => self.info.regs().sgi_key3d().write(|w| w.set_key3d(word)),
            _ => {
                // Caller bounds word_index.
                unsafe { core::hint::unreachable_unchecked() }
            }
        }
    }

    /// CPU driven FIFO filling for normal mode. Caller is responsible for starting the operation after filling the FIFO, and ensuring data size does not exceed FIFO capacity.
    fn fill_sha2_fifo_normal(&mut self, data: &[u8]) -> Result<(), SgiError> {
        const MAX_FIFO_NORM_WORD_SIZE: usize = 32;
        // Max size = (4 datain +  8 keyin) banks *  4 words per bank = 48 words = 192 bytes;
        // but we set it to 32 words (128 bytes) to fit one full SHA2 block, since the hardware processes data in blocks
        // Filling more than one block at a time may not be supported or may require additional handling.
        // but realistically we can only fill one SHA2 block of 1024 bits (128 bytes) at a time, which is 32 words.
        if data.len() > MAX_FIFO_NORM_WORD_SIZE * 4 {
            return Err(SgiError::InvalidSize);
        }
        for (i, chunk) in data.chunks_exact(4).enumerate() {
            let word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            self.write_fifo_word_normal(i, word);
        }
        Ok(())
    }

    /// Wrapper for normal/auto mode FIFO filling. Normal mode requires FIFO be filled prior to SHA2 operation start.
    pub(crate) fn fill_sha2_fifo(&mut self, options: HashOptions, data: &[u8], length: usize) -> Result<(), SgiError> {
        if options.op_mode == HashMode::Auto {
            self.fill_sha2_fifo_auto(data, length)?;
        }
        Ok(())
    }

    /// Helper to check if SHAFIFO is full and cannot accept more data.
    fn is_sha2_fifo_full(&self) -> bool {
        self.status().sha_fifo_full()
    }

    /// Fills SHAFIFO register using CPU writes in auto mode, with busy-waiting for FIFO availability.
    fn fill_sha2_fifo_auto(&mut self, data: &[u8], length: usize) -> Result<(), SgiError> {
        for chunk in data[..length].chunks_exact(4) {
            let word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            while self.is_sha2_fifo_full() {
                // Wait until there is space in the FIFO.
            }
            self.info.regs().sgi_sha_fifo().write(|w| w.set_fifo(word));
        }
        Ok(())
    }

    /// Fills SHAFIFO register using DMA in software start mode, handles misalignment and awaits completion in async manner.
    /// Returns immediately after starting the DMA transfer; caller should await completion and check for errors.
    pub(crate) fn fill_sha2_fifo_dma_start<'a, 'd_dma>(
        &mut self,
        dma_ch: &'a mut DmaChannel<'d_dma>,
        data: &[u8],
        length: usize,
    ) -> Result<Transfer<'a>, SgiError> {
        if length == 0 || length > data.len() || length % MAX_BLOCK_SIZE != 0 {
            return Err(SgiError::InvalidSize);
        }

        // Destination is SHAFIFO (32-bit). If the source buffer is not word aligned, we need smaller transfers and the mux
        // will handle byte/halfword packing, but this is less efficient. Warn about it since it may indicate a usage issue.
        let align_addr = data.as_ptr() as usize;
        if (align_addr % 4) != 0 {
            #[cfg(feature = "defmt")]
            defmt::warn!("Data buffer is not word-aligned, which means 4x AHB reads.");
        }

        let peri_address = self.info.regs().sgi_sha_fifo().as_ptr() as *mut u32;

        // Use the current generic DMA helper in software-start mode.
        // This preserves the "legacy behavior" for mixed-width transfers while
        // using the shared DMA setup path from dma.rs.

        // NOTE: `length` is guaranteed to be a multiple of 128 bytes here, so it's always divisible by 2 and 4.

        let options = TransferOptions::COMPLETE_INTERRUPT;

        let src_addr = data.as_ptr() as usize;
        unsafe {
            let software_start = true; // need to trigger the hash operation start via software after setting up the DMA, so this is true regardless of auto vs. normal mode.
            if (src_addr % 4) == 0 {
                let words = core::slice::from_raw_parts(data.as_ptr() as *const u32, length / 4);
                dma_ch.setup_write_to_peripheral(words, peri_address, software_start, options)
            } else if (src_addr % 2) == 0 {
                let halfwords = core::slice::from_raw_parts(data.as_ptr() as *const u16, length / 2);
                dma_ch.setup_write_to_peripheral(halfwords, peri_address, software_start, options)
            } else {
                dma_ch.setup_write_to_peripheral(&data[..length], peri_address, software_start, options)
            }
        }
        .map_err(|_| SgiError::InvalidSize)?;

        let transfer = Transfer::new(dma_ch.reborrow());

        #[cfg(feature = "defmt")]
        defmt::trace!("DMA started for {=usize} bytes (software-start, DSIZE=32)", length);
        Ok(transfer)
    }

    /// Checks for SGI peripheral errors (SHA2, key read, key unwrap). This should be called after checking that the operation is done,
    /// since some errors only get latched/reported once busy is de-asserted. Clears any errors after reading.
    pub(crate) fn sgi_operation_error(&self) -> Result<(), SgiError> {
        let status = self.status();
        if status.sha_error() {
            return Err(SgiError::ShaError);
        }
        if status.key_read_err() {
            return Err(SgiError::KeyReadError);
        }
        if status.key_unwrap_err() {
            return Err(SgiError::KeyUnwrapError);
        }
        Ok(())
    }

    /// SGI instance error (encompasses internal (e.g. PRNG) errors and usage errors like invalid commands or data).
    pub(crate) fn sgi_error(&self) -> Result<(), SgiError> {
        let status = self.status();
        if status.error() != pac_sgi::vals::Error::NO_ERROR {
            self.clear_errors(); // Clear errors after reading
            return Err(SgiError::HardwareError);
        }
        Ok(())
    }

    /// SGI status busy bit indicates the peripheral is currently processing data and cannot accept new commands or data.
    /// This is independent from the SHA2 busy bit, which specifically indicates the hash engine is active.
    fn is_busy(&self) -> bool {
        self.status().busy()
    }

    /// Checks if the SHA2 engine is busy processing data. This is independent from the BUSY bit, which may be de-asserted in error conditions.
    fn is_sha2_busy(&self) -> bool {
        self.status().sha2_busy()
    }

    /// busy-waits with deterministic timeout for SHA2 completion, and returns a timeout error if it takes too long.
    pub(crate) fn wait_until_sha2_not_busy(&mut self) -> Result<(), SgiError> {
        let start_time = Instant::now();
        const MAX_WAIT_TIME_USEC: u64 = 1000; // 1 millisecond timeout for normal SHA2 operations.
        while self.is_sha2_busy() {
            if is_expired(start_time, MAX_WAIT_TIME_USEC) {
                // 1 millisecond timeout
                #[cfg(feature = "defmt")]
                defmt::error!("Timeout waiting for SGI SHA2 operation to complete");
                return Err(SgiError::Timeout);
            }
        }
        Ok(())
    }

    /// Read a block of output data from the SGI dataout registers into the provided buffer at the specified index.
    ///Note: This must be used by the update output functions after TRIGGER_UP to read the whole output.
    fn read_dataout_block(&self, output: &mut [u8], idx: usize) -> Result<(), SgiError> {
        const TOTAL_DATAOUT_REGS: usize = 4; // DATOUTA, DATOUTB, DATOUTC, DATOUTD
        let words = [
            self.info.regs().sgi_datouta().read().datouta(),
            self.info.regs().sgi_datoutb().read().datoutb(),
            self.info.regs().sgi_datoutc().read().datoutc(),
            self.info.regs().sgi_datoutd().read().datoutd(),
        ];

        for (i, word) in words.into_iter().take(TOTAL_DATAOUT_REGS).enumerate() {
            output[idx + (i * 4)..idx + (i * 4) + 4].copy_from_slice(&word.to_be_bytes());
        }
        Ok(())
    }

    /// Clear the SGI operation-done interrupt flag.
    fn clear_operation_interrupt(&self) {
        self.info.regs().sgi_int_status_clr().write(|w| w.set_int_clr(true));
    }

    ///Enables the SGI NVIC interrupt source.
    fn enable_sgi_interrupt(&self) {
        self.info.regs().sgi_int_enable().write(|w| w.set_int_en(true));
    }

    /// Enable the SGI operation-done interrupt.
    ///
    /// This clears any stale pending state and enables the SGI peripheral interrupt source.
    ///
    /// Call this *before* starting the SHA2 operation you intend to await.
    pub(crate) fn enable_operation_done_interrupt(&self) {
        self.clear_operation_interrupt();
        self.enable_sgi_interrupt();
    }

    /// Clears SGI errors by flushing the registers.
    fn clear_errors(&self) {
        self.info.regs().sgi_ctrl2().write(|w| w.set_flush((true as u8).into()));
    }

    /// Issue stop command to halt the SHA2 operation.
    pub(crate) fn sgi_stop_sha2_cmd(&self) {
        self.info
            .regs()
            .sgi_sha2_ctrl()
            .modify(|w| w.set_sha2_stop((true as u8).into()));
    }

    /// Reads the hash output from the SGI dataout registers into the provided buffer. This should be called after confirming operation completion,
    /// and it checks for errors before reading. It handles both normal and auto mode completion, including triggering additional reads for the larger SHA-512 outputs.
    pub(crate) fn read_hash_output(&mut self, options: HashOptions, output: &mut [u8]) -> Result<(), SgiError> {
        if options.hash_size == HashSize::Sha384 && output.len() < 48 {
            return Err(SgiError::BufferTooSmall);
        } else if options.hash_size == HashSize::Sha512 && output.len() < 64 {
            return Err(SgiError::BufferTooSmall);
        }
        if options.op_mode == HashMode::Auto {
            // Automatic mode
            self.info
                .regs()
                .sgi_sha2_ctrl()
                .modify(|w| w.set_sha2_stop((true as u8).into())); // Stop AUTO hash.
            let mut loop_counter = 0;
            while self.is_busy() && loop_counter < 1000 {
                loop_counter += 1;
                // The de-assertion of the busy bit in auto mode is a single pulse; a simple
                // count-based timeout is sufficient here.
            }
        }
        self.wait_until_sha2_not_busy()?;

        self.sgi_byte_order_big(ByteOrder::BigEndian)?; // SHA output is always in big-endian order, so set byte order accordingly for reading the output hash.

        if self.sgi_operation_error().is_err() {
            self.clear_errors(); // Clear errors after reading
            return Err(SgiError::ShaError);
        }
        if self.sgi_error().is_err() {
            self.clear_errors(); // Clear errors after reading
            return Err(SgiError::HardwareError);
        }

        let max_sha2_dataout_pass = match options.hash_size {
            HashSize::Sha384 => 2,
            HashSize::Sha512 => 3,
        }; // each pass reads 4 * 4 bytes = 16 bytes, so for SHA-384 we need 2 passes, for SHA-512 we need 3 passes; AFTER the first END_UP reading.
        let idx: usize = 0;

        self.read_dataout_block(output, idx)?;

        for i in 0..max_sha2_dataout_pass {
            self.info.regs().sgi_ctrl().write(|w| {
                w.set_datout_res((DataOutRes::TriggerUp as u8).into());
                w.set_start((true as u8).into());
            });
            self.wait_until_sha2_not_busy()?;
            self.read_dataout_block(output, idx + (16 * (i + 1)))?; // Each block is 16 bytes
        }
        Ok(())
    }

    /// To be used with partial modes. This function reads the current hash output (64 bytes for SHA2) and it can be stored in Ctx and reloaded later.
    pub(crate) fn update_partial_output(&mut self, options: HashOptions, output: &mut [u8]) -> Result<(), SgiError> {
        if output.len() < 64 {
            return Err(SgiError::BufferTooSmall);
        }
        if options.op_mode == HashMode::Auto {
            // Automatic mode
            self.info
                .regs()
                .sgi_sha2_ctrl()
                .modify(|w| w.set_sha2_stop((true as u8).into())); // Stop AUTO hash.
            let mut loop_counter = 0;
            while self.is_busy() && loop_counter < 1000 {
                loop_counter += 1;
                // The de-assertion of the busy bit in auto mode is a single pulse; a simple
                // count-based timeout is sufficient here.
            }
        }
        self.wait_until_sha2_not_busy()?;

        self.sgi_byte_order_big(ByteOrder::BigEndian)?; // SHA output is always in big-endian order, so set byte order accordingly for reading the output hash.

        if self.sgi_operation_error().is_err() {
            self.clear_errors();
            return Err(SgiError::ShaError);
        }
        if self.sgi_error().is_err() {
            self.clear_errors();
            return Err(SgiError::HardwareError);
        }

        let max_sha2_dataout_pass = 3; // need all bytes read for update; each pass reads 4 * 4 bytes = 16 bytes, so need 3 more AFTER the first END_UP reading.
        let idx: usize = 0;

        self.read_dataout_block(output, idx)?;

        for i in 0..max_sha2_dataout_pass {
            self.info.regs().sgi_ctrl().write(|w| {
                w.set_datout_res((DataOutRes::TriggerUp as u8).into());
                w.set_start((true as u8).into());
            });
            self.wait_until_sha2_not_busy()?;
            self.read_dataout_block(output, idx + (16 * (i + 1)))?; // Each block is 16 bytes
        }
        Ok(())
    }

    /// Reloads a previously computed partial hash result back into SGI internal hash registers.
    pub(crate) fn sgi_hash_reload(
        &mut self,
        mut options: HashOptions,
        prev_hash_result: &[u8],
    ) -> Result<(), SgiError> {
        // Reload the internal hash state using the DATIN path (NORMAL mode) with HASH_RELOAD enabled.
        // This must NOT auto-init, otherwise the hardware IV would overwrite the provided state.
        options.op_mode = HashMode::Normal; // Use normal mode for hash reload to allow loading the hash state through the datain registers.
        options.reload = HashReload::Reload; // Set reload bit for hash reload operation
        options.init = HashInit::NoInit; // Clear auto init bit for hash reload operation, since we are providing the hash state to be reloaded and don't want SGI to overwrite it with IV.

        if prev_hash_result.len() != SGI_HASH_OUTPUT_SIZE as usize {
            return Err(SgiError::InvalidSize);
        }

        self.init_sgi_sha(options)?;
        self.start_sgi_hash(options, prev_hash_result)?;
        self.fill_sha2_fifo(options, prev_hash_result, prev_hash_result.len())?;

        // Load the hash state into the SGI using the appropriate input registers.
        self.wait_until_sha2_not_busy()?; // Wait until reload is complete and SGI is ready for next operation
        Ok(())
    }
}
