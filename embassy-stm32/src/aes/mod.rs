//! AES HW Accelerator

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::aes::vals::Datatype;
use stm32_metapac::aes::vals::Gcmph;

use crate::dma::NoDma;
use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, Peripheral};

const AES_BLOCK_SIZE: usize = 16; // 128 bits
const BYTES_IN_WORD: usize = 4;

static AES_WAKER: AtomicWaker = AtomicWaker::new();
/// CRYP interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let ccf = T::regs().sr().read().ccf();
        if ccf {
            T::regs().cr().modify(|w| w.set_ccfie(false));
            AES_WAKER.wake();
        }
    }
}

/// AES-CCM Cipher Mode
pub struct AesCcm<
    'c,
    'd,
    const KEY_SIZE: usize,
    const TAG_SIZE: usize,
    const IV_SIZE: usize,
    T: Instance,
    DmaIn: 'static,
    DmaOut: 'static,
> {
    aes: &'c mut Aes<'d, T, DmaIn, DmaOut>,
    key: &'c [u8; KEY_SIZE],
    aad_header_buffer: [u8; 8],
    aad_header_len: usize,
    payload_len: usize,
    iv: [u8; 16],
    dir: Direction,
    aad_processed: bool,
}

impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut>
    AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, IV_SIZE, T, DmaIn, DmaOut>
{
    /// Constructs a new AES-CCM cipher for a cryptographic operation.
    pub fn new(
        aes: &'c mut Aes<'d, T, DmaIn, DmaOut>,
        key: &'c [u8; KEY_SIZE],
        nonce: &'c [u8; IV_SIZE],
        aad_len: usize,
        payload_len: usize,
        dir: Direction,
    ) -> Self {
        let mut aad_header_buffer: [u8; 8] = [0; 8];
        let aad_header_len = Self::setup_aad_header(&mut aad_header_buffer, aad_len);
        let mut iv: [u8; 16] = [0; 16];
        Self::setup_iv(&mut iv, nonce, aad_len, payload_len);

        return Self {
            aes,
            key,
            aad_header_buffer,
            aad_header_len,
            iv,
            payload_len,
            dir,
            aad_processed: false,
        };
    }

    /// Prepares header which will be sent at the beginning of first AAD block.
    ///
    /// Returns length of the header.
    fn setup_aad_header(aad_header_buffer: &mut [u8; 8], aad_len: usize) -> usize {
        if aad_len == 0 {
            return 0;
        }

        if aad_len < 65280 {
            aad_header_buffer[0] = (aad_len >> 8) as u8 & 0xFF;
            aad_header_buffer[1] = aad_len as u8 & 0xFF;
            return 2;
        } else {
            aad_header_buffer[0] = 0xFF;
            aad_header_buffer[1] = 0xFE;
            let aad_len_bytes: [u8; 4] = aad_len.to_be_bytes();
            aad_header_buffer[2] = aad_len_bytes[0];
            aad_header_buffer[3] = aad_len_bytes[1];
            aad_header_buffer[4] = aad_len_bytes[2];
            aad_header_buffer[5] = aad_len_bytes[3];
            return 6;
        }
    }
    /// Prepares payload that will land in IV register.
    /// For AES CCM, IV register is filled with payload called **block 0**.
    /// For details, refer to CCM specification in RFC 3610.
    pub fn setup_iv(block0: &mut [u8; 16], iv: &'c [u8; IV_SIZE], aad_len: usize, payload_len: usize) {
        if aad_len > 0 {
            block0[0] = 0x40;
        }
        block0[0] |= ((((TAG_SIZE as u8) - 2) >> 1) & 0x07) << 3;
        block0[0] |= ((15 - (iv.len() as u8)) - 1) & 0x07;

        block0[1..1 + iv.len()].copy_from_slice(iv);
        let payload_len_bytes: [u8; 4] = payload_len.to_be_bytes();
        if iv.len() <= 11 {
            block0[12] = payload_len_bytes[0];
        } else if payload_len_bytes[0] > 0 {
            panic!("Message is too large for given IV size.");
        }
        if iv.len() <= 12 {
            block0[13] = payload_len_bytes[1];
        } else if payload_len_bytes[1] > 0 {
            panic!("Message is too large for given IV size.");
        }
        block0[14] = payload_len_bytes[2];
        block0[15] = payload_len_bytes[3];
    }

    fn get_header_block(&self) -> &[u8] {
        return &self.aad_header_buffer[0..self.aad_header_len];
    }
    /// Starts AES CCM cipher operation.
    ///
    /// Operations done in scope of this are ordered exactly as described
    /// in RM0434 Rev 13, p. 618
    pub async fn start(&mut self)
    where
        Self: CipherSized + IVSized,
        DmaIn: crate::aes::DmaIn<T>,
        DmaOut: crate::aes::DmaOut<T>,
    {
        self.aes.disable();
        self.aes.set_ccm_chmod();
        self.aes.set_byte_datatype();
        self.aes.set_algorithm_phase(Gcmph::INITPHASE);
        self.aes.setup_direction(self.dir);
        self.aes.setup_key_register(self.key);
        self.aes.setup_iv_register(&self.iv);
        self.aes.enable();
        self.aes.wait_until_computation_complete_blocking();
        self.aes.clear_computation_complete_flag();

        #[cfg(feature = "defmt")]
        self.aes.log_aes_state();
    }

    /// Sets up authenticated associated data on the AES peripheral.
    ///
    /// Invariant: can be called only **once** per single decryption/encryption procedure
    pub async fn aad(&mut self, aad: &[u8])
    where
        Self: CipherSized + IVSized,
        DmaIn: crate::aes::DmaIn<T>,
        DmaOut: crate::aes::DmaOut<T>,
    {
        self.aad_processed = true;

        let mut aad_buffer: [u8; 16] = [0; 16];
        let mut aad_buffer_idx = 0;
        self.aes.set_algorithm_phase(Gcmph::HEADERPHASE);

        self.aes.enable();

        let header = self.get_header_block();
        aad_buffer[0..header.len()].copy_from_slice(header);
        aad_buffer_idx += header.len();

        let mut processed_aad_len = 0;

        while processed_aad_len < aad.len() {
            let remaining_aad_len = aad.len() - processed_aad_len;
            let remaining_buffer_len = AES_BLOCK_SIZE - aad_buffer_idx;

            // Fill buffer with as much data from aad as possible.
            let len_to_copy = core::cmp::min(remaining_aad_len, remaining_buffer_len);
            aad_buffer[aad_buffer_idx..aad_buffer_idx + len_to_copy]
                .copy_from_slice(&aad[processed_aad_len..processed_aad_len + len_to_copy]);
            aad_buffer_idx += len_to_copy;
            processed_aad_len += len_to_copy;

            // In case we didn't fill whole buffer ( i.e. whole aad has already been processed),
            // fill the remaining space with 0s.
            aad_buffer[aad_buffer_idx..].fill(0);

            Aes::<T, DmaIn, DmaOut>::write_bytes_dma(&mut self.aes.dma_in, &mut aad_buffer).await;

            // Reset the buffer idx for the next block processing
            aad_buffer_idx = 0;
        }
    }

    /// Performs encryption/decryption on provided payload.
    ///
    /// ## Contracts
    /// - Output buffer must be at least as long as the input buffer.
    /// - `aad` should be called beforehand, if `aad_len` has been set to nonzero value.
    ///
    /// **Panics** if either of them is not upheld.
    pub async fn payload(&mut self, input: &[u8], output: &mut [u8])
    where
        Self: CipherSized + IVSized,
        DmaIn: crate::aes::DmaIn<T>,
        DmaOut: crate::aes::DmaOut<T>,
    {
        if input.len() > output.len() {
            panic!("Output buffer length must match input length.");
        }
        if !self.aad_processed && self.aad_header_len > 0 {
            panic!("AES payload processing failed: AAD was supposed to be processed first")
        }

        self.aes.set_algorithm_phase(Gcmph::PAYLOADPHASE);

        // This enable use is only meaningful when there's no AAD phase beforehand.
        self.aes.enable();

        let input_len_remainder = self.payload_len % AES_BLOCK_SIZE;

        let mut idx: usize = 0;
        let full_blocks_len = self.payload_len - input_len_remainder;
        self.aes
            .write_read_bytes_dma(
                &input[idx..idx + full_blocks_len],
                &mut output[idx..idx + full_blocks_len],
            )
            .await;

        idx += full_blocks_len;

        if input_len_remainder > 0 {
            // Set up npblb so that AES knows to skip some trailing bytes
            if self.dir == Direction::Decrypt {
                let padding_len = AES_BLOCK_SIZE - input_len_remainder;
                T::regs().cr().modify(|w| w.set_npblb(padding_len as u8));
            }

            // Copy remaining message to the front, the rest SHOULD be 0s
            let mut in_buffer: [u8; 16] = [0; 16];
            in_buffer[..input_len_remainder].copy_from_slice(&input[idx..idx + input_len_remainder]);

            // Blocking read and write has a different endianness than DMA, hence we need to reverse the byte order
            // before and after the operation
            Self::reverse_bytes_in_words(&mut in_buffer);

            // We're falling back to polling data transfer,
            // so CCF needs to be manually reset after DMA usage
            self.aes.clear_computation_complete_flag();

            let mut out_buffer = [0; 16];
            self.aes.write_and_read_bytes_blocking(&in_buffer, &mut out_buffer);

            // Blocking read and write has a different endianness than DMA, hence we need to reverse the byte order
            // before and after the operation
            Self::reverse_bytes_in_words(&mut out_buffer);

            output[idx..idx + input_len_remainder].copy_from_slice(&out_buffer[..input_len_remainder]);
        }
    }

    /// Generates an authentication tag for authenticated ciphers including GCM, CCM, and GMAC.
    /// Called after the all data has been encrypted/decrypted by `payload`.
    pub async fn finish(&mut self) -> [u8; TAG_SIZE] {
        // We're falling back to polling data transfer,
        // so CCF needs to be manually reset after DMA usage
        self.aes.clear_computation_complete_flag();
        self.aes.set_algorithm_phase(Gcmph::FINALPHASE);

        let mut full_tag: [u8; 16] = [0; 16];

        self.aes.read_bytes_blocking(&mut full_tag);

        // Data swapping is **not** applied to the tag,
        // hence we do it ourselves
        Self::reverse_bytes_in_words(&mut full_tag);

        let mut tag: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag.copy_from_slice(&full_tag[0..TAG_SIZE]);

        self.aes.disable();

        tag
    }

    fn reverse_bytes_in_words<const SIZE: usize>(block: &mut [u8; SIZE]) {
        assert_eq!(SIZE % 4, 0);
        
        let words = block.array_chunks_mut::<4>();
        for word in words {
            word.reverse();
        }
    }
}

/// This trait enables restriction of ciphers to specific key sizes.
pub trait CipherSized {}

/// This trait enables restriction of initialization vectors to sizes compatibile with a cipher mode.
pub trait IVSized {}

/// This trait enables restriction of a header phase to authenticated ciphers only.
pub trait CipherAuthenticated<const TAG_SIZE: usize> {
    /// Defines the authentication tag size.
    const TAG_SIZE: usize = TAG_SIZE;
}

impl<'c, 'd, const TAG_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherSized
    for AesCcm<'c, 'd, AES_BLOCK_SIZE, TAG_SIZE, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<4>
    for AesCcm<'c, 'd, KEY_SIZE, 4, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<6>
    for AesCcm<'c, 'd, KEY_SIZE, 6, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<8>
    for AesCcm<'c, 'd, KEY_SIZE, 8, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<10>
    for AesCcm<'c, 'd, KEY_SIZE, 10, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<12>
    for AesCcm<'c, 'd, KEY_SIZE, 12, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<14>
    for AesCcm<'c, 'd, KEY_SIZE, 14, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const IV_SIZE: usize, T: Instance, DmaIn, DmaOut> CipherAuthenticated<16>
    for AesCcm<'c, 'd, KEY_SIZE, 16, IV_SIZE, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 7, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 8, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 9, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 10, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 11, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 12, T, DmaIn, DmaOut>
{
}
impl<'c, 'd, const KEY_SIZE: usize, const TAG_SIZE: usize, T: Instance, DmaIn, DmaOut> IVSized
    for AesCcm<'c, 'd, KEY_SIZE, TAG_SIZE, 13, T, DmaIn, DmaOut>
{
}

/// Selects whether the AES processor operates in encryption or decryption mode.
#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    /// Encryption mode
    Encrypt,
    /// Decryption mode
    Decrypt,
}

/// AES Accelerator Driver
pub struct Aes<'d, T, DmaIn = NoDma, DmaOut = NoDma> {
    _peripheral: PeripheralRef<'d, T>,
    dma_in: PeripheralRef<'d, DmaIn>,
    dma_out: PeripheralRef<'d, DmaOut>,
}

impl<'d, T: Instance, DmaIn, DmaOut> Aes<'d, T, DmaIn, DmaOut> {
    /// Create a new AES driver.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        dma_in: impl Peripheral<P = DmaIn> + 'd,
        dma_out: impl Peripheral<P = DmaOut> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        T::enable_and_reset();
        into_ref!(peri, dma_in, dma_out);
        let instance = Self {
            _peripheral: peri,
            dma_in,
            dma_out,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Disables the AES peripheral.
    fn disable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(false));
    }

    /// Enable the AES peripheral.
    fn enable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(true));
    }
    fn set_ccm_chmod(&mut self) {
        T::regs().cr().modify(|w| w.set_chmod10(00));
        T::regs().cr().modify(|w| w.set_chmod2(true));
    }

    fn set_byte_datatype(&mut self) {
        T::regs().cr().modify(|w| w.set_datatype(Datatype::BYTE));
    }

    /// Sets the phase of the algorithm that the processor is in.
    /// Applicable for GCM and CCM encryption.
    fn set_algorithm_phase(&mut self, phase: pac::aes::vals::Gcmph) {
        T::regs().cr().modify(|w| w.set_gcmph(phase));
    }

    /// Configures operation direction on the processor.
    ///
    /// There are overall 4 modes, where more 2 and 4 is redundant for AES CCM.
    fn setup_direction(&mut self, dir: Direction) {
        if dir == Direction::Encrypt {
            T::regs().cr().modify(|w| w.set_mode(pac::aes::vals::Mode::MODE1));
        } else {
            T::regs().cr().modify(|w| w.set_mode(pac::aes::vals::Mode::MODE3));
        }
    }

    /// Blocks until the `calculation complete` flag turns 1.
    fn wait_until_computation_complete_blocking(&mut self) {
        while !T::regs().sr().read().ccf() {}
    }

    /// Resets the `computation complete flag` back to 0/.
    fn clear_computation_complete_flag(&mut self) {
        T::regs().cr().modify(|w| w.set_ccfc(true));
    }

    /// Fills key register with provided key.
    /// ## Contracts
    /// - Order of words in provided array: most significant word first, least significant word last.
    /// - Order of bytes in word: most significant byte first, least significant byte last.
    fn setup_key_register<const N: usize>(&mut self, key: &[u8; N]) {
        key // visualisation: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15]
            .array_chunks::<4>() // [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12], [13, 14, 15, 16]]
            .rev() // [[13, 14, 15, 16], [9, 10, 11, 12], [5, 6, 7, 8], [1, 2, 3, 4]]
            .enumerate() // [(0, [13, 14, 15, 16]),  (1, [9, 10, 11, 12]),  (2, [5, 6, 7, 8]),  (3, [1, 2, 3, 4])]
            .for_each(|(i, &word)| T::regs().keyr(i).modify(|w| w.set_key(u32::from_be_bytes(word))));
    }

    /// Fills key register with provided IV.
    /// ## Contracts
    /// - Order of words in provided array: most significant word first, least significant word last.
    /// - Order of bytes in word: most significant byte first, least significant byte last.
    fn setup_iv_register(&mut self, full_iv: &[u8; 16]) {
        //least significant word goes to IV register #0, most significant - IV register #3
        full_iv // visualisation: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15]
            .array_chunks::<4>() // [[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12], [13, 14, 15, 16]]
            .rev() // [[13, 14, 15, 16], [9, 10, 11, 12], [5, 6, 7, 8], [1, 2, 3, 4]]
            .enumerate() // [(0, [13, 14, 15, 16]),  (1, [9, 10, 11, 12]),  (2, [5, 6, 7, 8]),  (3, [1, 2, 3, 4])]
            .for_each(|(i, &word)| T::regs().ivr(i).modify(|w| w.set_ivi(u32::from_be_bytes(word))));
    }

    #[cfg(feature = "defmt")]
    fn log_aes_state(&mut self) {
        defmt::trace!("SR:{=u32:b}", T::regs().sr().read().0);
        defmt::trace!("CR: {=u32:b}", T::regs().cr().read().0);
        defmt::trace!("keyr0: {=u32:x}", T::regs().keyr(0).read().0);
        defmt::trace!("keyr1: {=u32:x}", T::regs().keyr(1).read().0);
        defmt::trace!("keyr2: {=u32:x}", T::regs().keyr(2).read().0);
        defmt::trace!("keyr3: {=u32:x}", T::regs().keyr(3).read().0);
        defmt::trace!("ivr0: {=u32:x}", T::regs().ivr(0).read().0);
        defmt::trace!("ivr1: {=u32:x}", T::regs().ivr(1).read().0);
        defmt::trace!("ivr2: {=u32:x}", T::regs().ivr(2).read().0);
        defmt::trace!("ivr3: {=u32:x}", T::regs().ivr(3).read().0);
    }

    /// Writes arbitrary number of blocks to the DINR.
    /// After every 4 word write, waits for computation complete flag.
    /// reads the output from DOUTR, and clears the flag.
    ///
    /// ## Contracts:
    /// - CCF must be cleared when invoking this method
    /// - both `blocks_in` and `blocks_out` length must be multiple of `AES_BLOCK_SIZE`
    /// - Length of `blocks_out` must be larger than length of `blocks_in`.
    fn write_and_read_bytes_blocking(&mut self, blocks_in: &[u8], blocks_out: &mut [u8]) {
        // Ensure input is a multiple of block size.
        assert_eq!(blocks_in.len() % AES_BLOCK_SIZE, 0);
        assert!(blocks_in.len() <= blocks_out.len());

        for (block_in, block_out) in blocks_in
            .array_chunks::<AES_BLOCK_SIZE>()
            .zip(blocks_out.array_chunks_mut::<AES_BLOCK_SIZE>())
        {
            // write words from input slice into DINR, one after another
            for &word_in in block_in.array_chunks::<BYTES_IN_WORD>() {
                T::regs().dinr().write(|w| w.set_din(u32::from_be_bytes(word_in)));
            }

            // wait until peripheral calculates the compuration result
            self.wait_until_computation_complete_blocking();

            // read the computation result words into the output slice, one after another
            for word_out in block_out.array_chunks_mut::<BYTES_IN_WORD>() {
                let read_word = T::regs().doutr().read().dout();
                word_out.copy_from_slice(&read_word.to_be_bytes());
            }

            self.clear_computation_complete_flag();
        }
    }

    /// Writes arbitrary number of blocks to the DINR.
    ///
    /// ## Contracts:
    /// - CCF must be cleared when invoking this method
    /// - `blocks_in` length must be multiple of `AES_BLOCK_SIZE`
    #[allow(unused)]
    fn write_bytes_blocking(&mut self, blocks_in: &[u8]) {
        const BYTES_IN_WORD: usize = 4;
        // Ensure input is a multiple of block size.
        assert_eq!(blocks_in.len() % AES_BLOCK_SIZE, 0);

        for block_in in blocks_in.array_chunks::<AES_BLOCK_SIZE>() {
            // write words from input slice into DINR, one after another
            for &word_in in block_in.array_chunks::<BYTES_IN_WORD>() {
                T::regs().dinr().write(|w| w.set_din(u32::from_be_bytes(word_in)));
            }

            // wait until peripheral calculates the compuration result
            self.wait_until_computation_complete_blocking();
            self.clear_computation_complete_flag();
        }
    }

    /// Writes arbitrary number of blocks to the DINR.
    ///
    /// Waits for computation complete flag.
    /// reads the output from DOUTR, and clears the flag.
    /// Contracts:
    /// - CCF must be cleared when invoking this method
    /// - `blocks_out` length must be multiple of `AES_BLOCK_SIZE`
    fn read_bytes_blocking(&mut self, blocks_out: &mut [u8]) {
        // Ensure input is a multiple of block size.
        assert_eq!(blocks_out.len() % AES_BLOCK_SIZE, 0);

        for block_out in blocks_out.array_chunks_mut::<AES_BLOCK_SIZE>() {
            // wait until peripheral calculates the compuration result
            self.wait_until_computation_complete_blocking();

            // read the computation result words into the output slice, one after another
            for word_out in block_out.array_chunks_mut::<BYTES_IN_WORD>() {
                let read_word = T::regs().doutr().read().dout();
                word_out.copy_from_slice(&read_word.to_be_bytes());
            }

            self.clear_computation_complete_flag();
        }
    }


    /// Performs writing to DINR, and awaits without blocking,
    /// due to using AES CCF interrupts.
    #[allow(unused)]
    async fn write_bytes_interrupt(&mut self, blocks_in: &[u8]) {
        assert_eq!(blocks_in.len() % AES_BLOCK_SIZE, 0);

        for block_in in blocks_in.array_chunks::<AES_BLOCK_SIZE>() {
            // write words from input slice into DINR, one after another
            for &word_in in block_in.array_chunks::<BYTES_IN_WORD>() {
                T::regs().dinr().write(|w| w.set_din(u32::from_be_bytes(word_in)));
            }

            poll_fn(|ctx| {
                if T::regs().sr().read().ccf() {
                    self.clear_computation_complete_flag();
                    return Poll::Ready(());
                }
                AES_WAKER.register(ctx.waker());
                T::regs().cr().modify(|w| w.set_ccfie(true));

                // Need to check condition **after** `register` to avoid a race
                // condition that would result in lost notifications.
                if T::regs().sr().read().ccf() {
                    self.clear_computation_complete_flag();
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
        }
    }

    /// Performs two DMA operations concurrently:
    /// - write `blocks_in` into `DINR`;
    /// - read `blocks_out` from `DOUTR`.
    ///
    /// ## Contracts
    /// - `blocks_in.len() <= blocks_out.len()`
    /// - Also, Refer to `write_bytes_dma` and `read_bytes_dma` methods.
    async fn write_read_bytes_dma(&mut self, blocks_in: &[u8], blocks_out: &mut [u8])
    where
        DmaOut: crate::aes::DmaOut<T>,
        DmaIn: crate::aes::DmaIn<T>,
    {
        assert!(blocks_in.len() <= blocks_out.len());

        let write_dma = Self::write_bytes_dma(&mut self.dma_in, blocks_in);
        let read_dma = Self::read_bytes_dma(&mut self.dma_out, blocks_out);

        embassy_futures::join::join(write_dma, read_dma).await;
    }

    /// Writes into `DINR` using DMA.
    ///
    /// ## Contracts
    /// - Length must be multiple of `AES_BLOCK_SIZE`.
    async fn write_bytes_dma(mut dma_in: &mut PeripheralRef<'d, DmaIn>, blocks: &[u8])
    where
        DmaIn: crate::aes::DmaIn<T>,
    {
        assert!(blocks.as_ptr().is_aligned_to(4));
        if blocks.len() == 0 {
            return;
        }
        
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % AES_BLOCK_SIZE, 0);
        // Configure DMA to transfer input to crypto core.
        let dma_request = dma_in.request();
        let dst_ptr =T::regs().dinr().as_ptr() as *mut u32;



        let num_words = blocks.len() / 4;
        let src_ptr = core::ptr::slice_from_raw_parts(blocks.as_ptr().cast(), num_words);

        let options = crate::dma::TransferOptions {
            #[cfg(not(gpdma))]
            priority: crate::dma::Priority::High,
            ..Default::default()
        };
        let dma_transfer =
            unsafe { crate::dma::Transfer::new_write_raw(&mut dma_in, dma_request, src_ptr, dst_ptr, options) };
        T::regs().cr().modify(|w| w.set_dmainen(true));
        // Wait for the transfer to complete.
        dma_transfer.await;
        T::regs().cr().modify(|w| w.set_dmaouten(false));
    }

    /// Writes into `DINR` using DMA.
    ///
    /// ## Contracts
    /// - Input slice **must** be aligned to u32. Otherwise the output data will be shifted.
    /// - Length must be multiple of `AES_BLOCK_SIZE`.
    async fn read_bytes_dma(dma_out: &mut PeripheralRef<'d, DmaOut>, blocks: &mut [u8])
    where
        DmaOut: crate::aes::DmaOut<T>,
    {
        if blocks.len() == 0 {
            return;
        }

        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % AES_BLOCK_SIZE, 0);
        // Configure DMA to get output from crypto core.
        let dma_request = dma_out.request();
        let src_ptr = T::regs().doutr().as_ptr() as *mut u32;
        let num_words = blocks.len() / 4;
        let dst_ptr = core::ptr::slice_from_raw_parts_mut(blocks.as_mut_ptr().cast(), num_words);

        let options = crate::dma::TransferOptions {
            #[cfg(not(gpdma))]
            priority: crate::dma::Priority::VeryHigh,
            ..Default::default()
        };
        let dma_transfer =
            unsafe { crate::dma::Transfer::new_read_raw(dma_out, dma_request, src_ptr, dst_ptr, options) };
        T::regs().cr().modify(|w| w.set_dmaouten(true));

        // Wait for the transfer to complete.
        dma_transfer.await;
        T::regs().cr().modify(|w| w.set_dmaouten(false));
    }
}

/// AES SealedInstance trait.
///
/// Allows access to PAC exclusively for `aes` module.
trait SealedInstance {
    fn regs() -> pac::aes::Aes;
}

/// AES instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + Peripheral<P = Self> + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this AES instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, aes, AES, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::aes::Aes {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
