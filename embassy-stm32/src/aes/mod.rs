//! Cry.pto Accelerator (AES)
#[cfg(aes_v3b)]
use core::cmp::min;
use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::aes::vals::Datatype;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, Peripheral};

const AES_BLOCK_SIZE: usize = 16; // 128 bits

static AES_WAKER: AtomicWaker = AtomicWaker::new();

/// This trait encapsulates all cipher-specific behavior/
pub trait Cipher<'c> {
    /// Processing block size. Determined by the processor and the algorithm.
    const BLOCK_SIZE: usize;

    /// Indicates whether the cipher requires the application to provide padding.
    /// If `true`, no partial blocks will be accepted (a panic will occur).
    const REQUIRES_PADDING: bool = false;

    /// Returns the symmetric key.
    fn keyr(&self) -> &[u8];

    /// Returns the initialization vector.
    fn iv(&self) -> &[u8];

    /// Performs any key preparation within the processor, if necessary.
    fn prepare_keyr(&self, _p: &pac::aes::Aes) {}

    /// Performs any cipher-specific initialization.
    async fn init_phase<T: Instance>(&self, _p: &pac::aes::Aes, _aes: &mut Aes<'_, T>) {}

    /// Called prior to processing the last data block for cipher-specific operations.
    fn pre_final(&self, _p: &pac::aes::Aes, _dir: Direction, _padding_len: usize) -> [u32; 4] {
        return [0; 4];
    }

    /// Called after processing the last data block for cipher-specific operations.
    async fn post_final<T: Instance>(
        &self,
        _p: &pac::aes::Aes,
        _aes: &mut Aes<'_, T>,
        _dir: Direction,
        _int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        _padding_mask: [u8; 16],
    ) {
    }

    /// Returns the AAD header block as required by the cipher.
    fn get_header_block(&self) -> &[u8] {
        return [0; 0].as_slice();
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

/// AES-CCM Cipher Mode
pub struct AesCcm<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> {
    key: &'c [u8; KEY_SIZE],
    aad_header: [u8; 6],
    aad_header_len: usize,
    block0: [u8; 16],
    ctr: [u8; 16],
}

impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> AesCcm<'c, KEY_SIZE, TAG_SIZE, IV_SIZE> {
    /// Constructs a new AES-CCM cipher for a cry.ptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; IV_SIZE], aad_len: usize, payload_len: usize) -> Self {
        let mut aad_header: [u8; 6] = [0; 6];
        let mut aad_header_len = 0;
        let mut block0: [u8; 16] = [0; 16];
        if aad_len != 0 {
            if aad_len < 65280 {
                aad_header[0] = (aad_len >> 8) as u8 & 0xFF;
                aad_header[1] = aad_len as u8 & 0xFF;
                aad_header_len = 2;
            } else {
                aad_header[0] = 0xFF;
                aad_header[1] = 0xFE;
                let aad_len_bytes: [u8; 4] = aad_len.to_be_bytes();
                aad_header[2] = aad_len_bytes[0];
                aad_header[3] = aad_len_bytes[1];
                aad_header[4] = aad_len_bytes[2];
                aad_header[5] = aad_len_bytes[3];
                aad_header_len = 6;
            }
        }
        let total_aad_len = aad_header_len + aad_len;
        let mut aad_padding_len = 16 - (total_aad_len % 16);
        if aad_padding_len == 16 {
            aad_padding_len = 0;
        }
        aad_header_len += aad_padding_len;
        let total_aad_len_padded = aad_header_len + aad_len;
        if total_aad_len_padded > 0 {
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
        let mut ctr: [u8; 16] = [0; 16];
        ctr[0] = block0[0] & 0x07;
        ctr[1..1 + iv.len()].copy_from_slice(&block0[1..1 + iv.len()]);
        ctr[15] = 0x01;

        return Self {
            key: key,
            aad_header: aad_header,
            aad_header_len: aad_header_len,
            block0: block0,
            ctr: ctr,
        };
    }
}

impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> Cipher<'c>
    for AesCcm<'c, KEY_SIZE, TAG_SIZE, IV_SIZE>
{
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;

    fn keyr(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.ctr.as_slice()
    }

    async fn init_phase<T: Instance, DmaIn, DmaOut>(&self, p: &pac::aes::Aes, aes: &mut Aes<'_, T>) {}

    fn get_header_block(&self) -> &[u8] {
        return &self.aad_header[0..self.aad_header_len];
    }

    fn pre_final(&self, p: &pac::aes::Aes, _dir: Direction, padding_len: usize) -> [u32; 4] {
        //Handle special GCM partial block process.
        p.cr().modify(|w| w.set_npblb(padding_len as u8));
        [0; 4]
    }

    //     async fn post_final<T: Instance, DmaIn, DmaOut>(
    //         &self,
    //         p: &pac::aes::Aes,
    //         aes: &mut Aes<'_, T, DmaIn, DmaOut>,
    //         dir: Direction,
    //         int_data: &mut [u8; AES_BLOCK_SIZE],
    //         temp1: [u32; 4],
    //         padding_mask: [u8; 16],
    //     ) where
    //         DmaIn: crate::aes::DmaIn<T>,
    //         DmaOut: crate::aes::DmaOut<T>,
    //     {
    //         if dir == Direction::Decrypt {
    //             //Handle special CCM partial block process.
    //             let mut temp2 = [0; 4];
    //             temp2[0] = p.csgcmccmr(0).read().swap_bytes();
    //             temp2[1] = p.csgcmccmr(1).read().swap_bytes();
    //             temp2[2] = p.csgcmccmr(2).read().swap_bytes();
    //             temp2[3] = p.csgcmccmr(3).read().swap_bytes();
    //             p.cr().modify(|w| w.set_algomode3(true));
    //             p.cr().modify(|w| w.set_algomode0(1));
    //             p.cr().modify(|w| w.set_gcm_ccmph(3));
    //             // Header phase
    //             p.cr().modify(|w| w.set_gcm_ccmph(1));
    //             for i in 0..AES_BLOCK_SIZE {
    //                 int_data[i] = int_data[i] & padding_mask[i];
    //             }
    //             let mut in_data: [u32; 4] = [0; 4];
    //             for i in 0..in_data.len() {
    //                 let mut int_bytes: [u8; 4] = [0; 4];
    //                 int_bytes.copy_from_slice(&int_data[(i * 4)..(i * 4) + 4]);
    //                 let int_word = u32::from_le_bytes(int_bytes);
    //                 in_data[i] = int_word;
    //                 in_data[i] = in_data[i] ^ temp1[i] ^ temp2[i];
    //             }
    //             Aes::<T, DmaIn, DmaOut>::write_words(&mut aes.indma, Self::BLOCK_SIZE, &in_data).await;
    //         }
    //     }
}

impl<'c, const TAG_SIZE: usize, const IV_SIZE: usize> CipherSized for AesCcm<'c, { 128 / 8 }, TAG_SIZE, IV_SIZE> {}
impl<'c, const TAG_SIZE: usize, const IV_SIZE: usize> CipherSized for AesCcm<'c, { 192 / 8 }, TAG_SIZE, IV_SIZE> {}
impl<'c, const TAG_SIZE: usize, const IV_SIZE: usize> CipherSized for AesCcm<'c, { 256 / 8 }, TAG_SIZE, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<4> for AesCcm<'c, KEY_SIZE, 4, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<6> for AesCcm<'c, KEY_SIZE, 6, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<8> for AesCcm<'c, KEY_SIZE, 8, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<10> for AesCcm<'c, KEY_SIZE, 10, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<12> for AesCcm<'c, KEY_SIZE, 12, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<14> for AesCcm<'c, KEY_SIZE, 14, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<16> for AesCcm<'c, KEY_SIZE, 16, IV_SIZE> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 7> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 8> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 9> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 10> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 11> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 12> {}
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 13> {}

#[allow(dead_code)]
/// Holds the state information for a cipher operation.
/// Allows suspending/resuming of cipher operations.
pub struct Context<'c, C: Cipher<'c> + CipherSized> {
    phantom_data: PhantomData<&'c C>,
    cipher: &'c C,
    dir: Direction,
    last_block_processed: bool,
    header_processed: bool,
    aad_complete: bool,
    header_len: u64,
    payload_len: u64,
    aad_buffer: [u8; 16],
    aad_buffer_len: usize,
}

/// Selects whether the cry.pto processor operates in encryption or decryption mode.
#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    /// Encryption mode
    Encrypt,
    /// Decryption mode
    Decrypt,
}

/// Cry.pto Accelerator Driver
pub struct Aes<'d, T> {
    _peripheral: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Aes<'d, T> {
    /// Create a new AES driver.
    pub fn new(peri: impl Peripheral<P = T> + 'd) -> Self {
        T::enable_and_reset();
        let instance = Self { _peripheral: peri };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    /// Start a new encrypt or decrypt operation for the given cipher.
    pub async fn start<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &mut self,
        cipher: &'c C,
        dir: Direction,
    ) -> Context<'c, C> {
        let mut ctx: Context<'c, C> = Context {
            dir,
            last_block_processed: false,
            aad_complete: false,
            header_len: 0,
            payload_len: 0,
            cipher: cipher,
            phantom_data: PhantomData,
            header_processed: false,
            aad_buffer: [0; 16],
            aad_buffer_len: 0,
        };

        // 1 test-ready
        T::regs().cr().modify(|w| w.set_en(false));

        // 2 test-ready
        T::regs().cr().modify(|w| w.set_chmod10(00));
        T::regs().cr().modify(|w| w.set_chmod2(true));

        // Set data type to 8-bit. This will match software implementations.
        T::regs().cr().modify(|w| w.set_datatype(Datatype::BYTE));

        // 3 test-ready
        T::regs().cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::INITPHASE));

        // 4 test-ready
        if dir == Direction::Encrypt {
            T::regs().cr().modify(|w| w.set_mode(pac::aes::vals::Mode::MODE1));
        } else {
            T::regs().cr().modify(|w| w.set_mode(pac::aes::vals::Mode::MODE3));
        }

        // 5 test-ready
        let key = ctx.cipher.keyr();
        // Assumption: 128 bits
        T::regs().cr().modify(|w| w.set_keysize(false));
        //TODO: set up key
        self.load_keyr(key);

        //  test-ready - IV calculation is NOT verified.
        let iv = ctx.cipher.iv();
        let mut full_iv: [u8; 16] = [0; 16];
        full_iv[0..iv.len()].copy_from_slice(iv);
        let mut iv_idx = 0;
        let mut iv_word: [u8; 4] = [0; 4];
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().ivr(3).modify(|w| w.set_ivi(u32::from_be_bytes(iv_word)));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().ivr(2).modify(|w| w.set_ivi(u32::from_be_bytes(iv_word)));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().ivr(1).modify(|w| w.set_ivi(u32::from_be_bytes(iv_word)));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        T::regs().ivr(0).modify(|w| w.set_ivi(u32::from_be_bytes(iv_word)));

        // 6 test-ready
        T::regs().cr().modify(|w| w.set_en(true));

        // 7 test-ready
        while !T::regs().sr().read().ccf() {}

        // 8 test-ready
        T::regs().cr().write(|w| w.set_ccfc(true));

        ctx
    }

    /// Controls the header phase of cipher processing.
    /// This function is only valid for authenticated ciphers including GCM, CCM, and GMAC.
    /// All additional associated data (AAD) must be supplied to this function prior to starting the payload phase with `payload`.
    /// The AAD must be supplied in multiples of the block size (128-bits for AES, 64-bits for DES), except when supplying the last block.
    /// When supplying the last block of AAD, `last_aad_block` must be `true`.
    pub async fn aad<
        'c,
        const TAG_SIZE: usize,
        C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<TAG_SIZE>,
    >(
        &mut self,
        ctx: &mut Context<'c, C>,
        aad: &[u8],
        last_aad_block: bool,
    ) {
        // Perform checks for correctness.
        // TODO: ?
        if ctx.aad_complete {
            panic!("Cannot update AAD after starting payload!")
        }

        ctx.header_len += aad.len() as u64;

        // 1 test-ready
        T::regs()
            .cr()
            .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::HEADERPHASE));

        // 2 test-ready
        T::regs().cr().modify(|w| w.set_en(true));

        // 3 test-ready - make sure blocking vs async difference don't affect anything in testing
        // First write the header B1 block if not yet written.
        if !ctx.header_processed {
            ctx.header_processed = true;
            let header = ctx.cipher.get_header_block();
            ctx.aad_buffer[0..header.len()].copy_from_slice(header);
            ctx.aad_buffer_len += header.len();
        }

        // Fill the header block to make a full block.
        let len_to_copy = core::cmp::min(aad.len(), C::BLOCK_SIZE - ctx.aad_buffer_len);
        ctx.aad_buffer[ctx.aad_buffer_len..ctx.aad_buffer_len + len_to_copy].copy_from_slice(&aad[..len_to_copy]);
        ctx.aad_buffer_len += len_to_copy;
        ctx.aad_buffer[ctx.aad_buffer_len..].fill(0);
        let mut aad_len_remaining = aad.len() - len_to_copy;

        if ctx.aad_buffer_len < C::BLOCK_SIZE {
            // The buffer isn't full and this is the last buffer, so process it as is (already padded).
            if last_aad_block {
                self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer);
                while !T::regs().sr().read().ccf() {}
                T::regs().cr().write(|w| w.set_ccfc(true));
                // TODO: should not be needed?
                // assert_eq!(T::regs().sr().read().ifem(), true);

                // Switch to payload phase.
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_en(false));
                T::regs()
                    .cr()
                    .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::PAYLOADPHASE));
                // TODO: should not be needed?
                // T::regs().cr().modify(|w| w.fflush());
            } else {
                // Just return because we don't yet have a full block to process.
                return;
            }
        } else {
            // Load the full block from the buffer.
            self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer);
            while !T::regs().sr().read().ccf() {}
            T::regs().cr().write(|w| w.set_ccfc(true));
        }

        // Handle a partial block that is passed in.
        ctx.aad_buffer_len = 0;
        let leftovers = aad_len_remaining % C::BLOCK_SIZE;
        ctx.aad_buffer[..leftovers].copy_from_slice(&aad[aad.len() - leftovers..aad.len()]);
        ctx.aad_buffer_len += leftovers;
        ctx.aad_buffer[ctx.aad_buffer_len..].fill(0);
        aad_len_remaining -= leftovers;
        assert_eq!(aad_len_remaining % C::BLOCK_SIZE, 0);

        // Load full data blocks into core.
        let num_full_blocks = aad_len_remaining / C::BLOCK_SIZE;
        let start_index = len_to_copy;
        let end_index = start_index + (C::BLOCK_SIZE * num_full_blocks);
        self.write_bytes_blocking(C::BLOCK_SIZE, &aad[start_index..end_index]);
        while !T::regs().sr().read().ccf() {}
        T::regs().cr().write(|w| w.set_ccfc(true));

        if last_aad_block {
            if leftovers > 0 {
                self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer);
                while !T::regs().sr().read().ccf() {}
                T::regs().cr().write(|w| w.set_ccfc(true));
            }
            // Switch to payload phase.

            // todo: should not be needed
            // T::regs().cr().modify(|w| w.fflush());
        }

        self.store_context(ctx);
    }

    /// Performs encryption/decryption on the provided context.
    /// The context determines algorithm, mode, and state of the cry.pto accelerator.
    /// When the last piece of data is supplied, `last_block` should be `true`.
    /// This function panics under various mismatches of parameters.
    /// Output buffer must be at least as long as the input buffer.
    /// Data must be a multiple of block size (128-bits for AES, 64-bits for DES) for CBC and ECB modes.
    /// Padding or ciphertext stealing must be managed by the application for these modes.
    /// Data must also be a multiple of block size unless `last_block` is `true`.
    pub async fn payload<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &mut self,
        ctx: &mut Context<'c, C>,
        input: &[u8],
        output: &mut [u8],
        last_block: bool,
    ) {
        // self.load_context(ctx);

        let last_block_remainder = input.len() % C::BLOCK_SIZE;

        // Perform checks for correctness.
        if !ctx.aad_complete && ctx.header_len > 0 {
            panic!("Additional associated data must be processed first!");
        } else if !ctx.aad_complete {
            {
                ctx.aad_complete = true;
                T::regs()
                    .cr()
                    .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::PAYLOADPHASE));
            }
        }
        if ctx.last_block_processed {
            panic!("The last block has already been processed!");
        }
        if input.len() > output.len() {
            panic!("Output buffer length must match input length.");
        }
        if !last_block {
            if last_block_remainder != 0 {
                panic!("Input length must be a multiple of {} bytes.", C::BLOCK_SIZE);
            }
        }
        if C::REQUIRES_PADDING {
            if last_block_remainder != 0 {
                panic!("Input must be a multiple of {} bytes in ECB and CBC modes. Consider padding or ciphertext stealing.", C::BLOCK_SIZE);
            }
        }
        if last_block {
            ctx.last_block_processed = true;
        }

        // Load data into core, block by block.
        let num_full_blocks = input.len() / C::BLOCK_SIZE;
        for block in 0..num_full_blocks {
            let index = block * C::BLOCK_SIZE;
            // Read block out
            // Write block in
            self.write_bytes_blocking(C::BLOCK_SIZE, &input[index..index + C::BLOCK_SIZE]);
            // Read block out
            while !T::regs().sr().read().ccf() {}
            self.read_bytes_blocking(C::BLOCK_SIZE, &mut output[index..index + C::BLOCK_SIZE]);
            T::regs().cr().write(|w| w.set_ccfc(true));
        }

        // Handle the final block, which is incomplete.
        if last_block_remainder > 0 {
            let padding_len = C::BLOCK_SIZE - last_block_remainder;
            let temp1 = ctx.cipher.pre_final(&T::regs(), ctx.dir, padding_len);

            let mut intermediate_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            let mut last_block: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            last_block[..last_block_remainder].copy_from_slice(&input[input.len() - last_block_remainder..input.len()]);
            self.write_bytes_blocking(C::BLOCK_SIZE, &last_block);
            while !T::regs().sr().read().ccf() {}
            self.read_bytes_blocking(C::BLOCK_SIZE, &mut intermediate_data);
            T::regs().cr().write(|w| w.set_ccfc(true));
            // Handle the last block depending on mode.
            let output_len = output.len();
            output[output_len - last_block_remainder..output_len]
                .copy_from_slice(&intermediate_data[0..last_block_remainder]);

            let mut mask: [u8; 16] = [0; 16];
            mask[..last_block_remainder].fill(0xFF);
            ctx.cipher
                .post_final(&T::regs(), self, ctx.dir, &mut intermediate_data, temp1, mask)
                .await;
        }

        ctx.payload_len += input.len() as u64;

        self.store_context(ctx);
    }

    // Generates an authentication tag for authenticated ciphers including GCM, CCM, and GMAC.
    /// Called after the all data has been encrypted/decrypted by `payload`.
    pub async fn finish<
        'c,
        const TAG_SIZE: usize,
        C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<TAG_SIZE>,
    >(
        &mut self,
        mut ctx: Context<'c, C>,
    ) -> [u8; TAG_SIZE] {
        self.load_context(&mut ctx);

        T::regs()
            .cr()
            .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::FINALPHASE));

        let mut full_tag: [u8; 16] = [0; 16];
        while !T::regs().sr().read().ccf() {}
        self.read_bytes_blocking(C::BLOCK_SIZE, &mut full_tag);
        T::regs().cr().write(|w| w.set_ccfc(true));

        let mut tag: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag.copy_from_slice(&full_tag[0..TAG_SIZE]);

        T::regs().cr().modify(|w| w.set_en(false));

        tag
    }

    fn load_keyr(&self, key: &[u8]) {
        // Load the key into the registers.
        let mut keyidx = 0;
        let mut keyword: [u8; 4] = [0; 4];
        let keylen = key.len() * 8;

        if keylen > 64 {
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().keyr(3).modify(|w| w.set_key(u32::from_be_bytes(keyword)));
            keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
            keyidx += 4;
            T::regs().keyr(2).modify(|w| w.set_key(u32::from_be_bytes(keyword)));
        }
        keyword.copy_from_slice(&key[keyidx..keyidx + 4]);
        keyidx += 4;
        T::regs().keyr(1).modify(|w| w.set_key(u32::from_be_bytes(keyword)));
        keyword = [0; 4];
        keyword[0..key.len() - keyidx].copy_from_slice(&key[keyidx..key.len()]);
        T::regs().keyr(0).modify(|w| w.set_key(u32::from_be_bytes(keyword)));
    }

    // STATUS: should be good. assess endianness of IV
    fn store_context<'c, C: Cipher<'c> + CipherSized>(&self, ctx: &mut Context<'c, C>) {
        // Wait for data block processing to finish.
        while !T::regs().sr().read().ccf() {}
        T::regs().cr().write(|w| w.set_ccfc(true));

        // Save the peripheral state.
        //TODO assess endianness
        // ctx.cr = T::regs().cr().read().0;
        // ctx.iv[0] = T::regs().ivr(3).read().ivi();
        // ctx.iv[1] = T::regs().ivr(2).read().ivi();
        // ctx.iv[2] = T::regs().ivr(1).read().ivi();
        // ctx.iv[3] = T::regs().ivr(0).read().ivi();

        // TODO: replace this with suspend/resume logic
        // for i in 0..8 {
        //     ctx.csgcmccm[i] = T::regs().csgcmccmr(i).read();
        //     ctx.csgcm[i] = T::regs().csgcmr(i).read();
        // }
    }

    fn load_context<'c, C: Cipher<'c> + CipherSized>(&self, ctx: &Context<'c, C>) {
        // Reload state registers.
        // T::regs().cr().write(|w| w.0 = ctx.cr);
        // T::regs().ivr(3).write(|w| w.set_ivi(ctx.iv[0]));
        // T::regs().ivr(2).write(|w| w.set_ivi(ctx.iv[1]));
        // T::regs().ivr(1).write(|w| w.set_ivi(ctx.iv[2]));
        // T::regs().ivr(0).write(|w| w.set_ivi(ctx.iv[3]));

        // self.load_keyr(ctx.cipher.keyr());

        // T::regs().cr().write(|w| w.0 = ctx.cr);
    }

    fn write_bytes_blocking(&self, block_size: usize, blocks: &[u8]) {
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        let mut index = 0;
        let end_index = blocks.len();
        while index < end_index {
            let mut in_word: [u8; 4] = [0; 4];
            in_word.copy_from_slice(&blocks[index..index + 4]);
            T::regs().dinr().write(|w| w.set_din(u32::from_ne_bytes(in_word)));
            index += 4;
            if index % block_size == 0 {
                // Block until input FIFO is empty.
            }
        }
    }

    fn write_words_blocking(&self, block_size: usize, blocks: &[u32]) {
        assert_eq!((blocks.len() * 4) % block_size, 0);
        let mut byte_counter: usize = 0;
        for word in blocks {
            T::regs().dinr().write(|w| w.set_din(*word));
            byte_counter += 4;
            if byte_counter % block_size == 0 {
                // while !T::regs().sr().read().ccf() {}
                // T::regs().cr().write(|w| w.set_ccfc(true))
            }
        }
    }

    fn read_bytes_blocking(&self, block_size: usize, blocks: &mut [u8]) {
        // Block until there is output to read.
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        // Read block out
        let mut index = 0;
        let end_index = blocks.len();
        while index < end_index {
            let out_word: u32 = T::regs().doutr().read().dout();
            blocks[index..index + 4].copy_from_slice(u32::to_ne_bytes(out_word).as_slice());
            index += 4;
        }
    }
}

trait SealedInstance {
    fn regs() -> pac::aes::Aes;
}

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
