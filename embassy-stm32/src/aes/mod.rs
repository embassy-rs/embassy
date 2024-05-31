//! Cry.pto Accelerator (AES)
#[cfg(aes_v3b)]
use core::cmp::min;
use core::marker::PhantomData;
use core::ops::DerefMut;

use embassy_hal_internal::{into_ref, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::aes::vals::Datatype;
use rand_core::block;

use crate::interrupt::typelevel::Interrupt;
use crate::{interrupt, pac, peripherals, Peripheral};

const AES_BLOCK_SIZE: usize = 16; // 128 bits

static AES_WAKER: AtomicWaker = AtomicWaker::new();

/// This trait encapsulates all cipher-specific behavior/
pub trait Cipher<'c> {
    /// Processing block size. Determined by the processor and the algorithm.
    const BLOCK_SIZE: usize;

    /// Returns the symmetric key.
    fn keyr(&self) -> &[u8];

    /// Returns the initialization vector.
    fn iv(&self) -> [u8; 16];


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
    aad_header: [u8; 32],
    aad_header_len: usize,
    block0: [u8; 16],
    ctr: [u8; 16],
}

impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> AesCcm<'c, KEY_SIZE, TAG_SIZE, IV_SIZE> {
    /// Constructs a new AES-CCM cipher for a cry.ptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; IV_SIZE], aad_len: usize, payload_len: usize) -> Self {
        let mut aad_header: [u8; 32] = [0; 32];
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
        let total_aad_len_padded = aad_header_len + aad_len + aad_padding_len;
        if total_aad_len_padded > 0 {
            block0[0] = 0x40;
        }
        block0[0] |= ((((TAG_SIZE as u8) - 2) >> 1) & 0x07) << 3;
        block0[0] |= ((15 - (iv.len() as u8)) - 1) & 0x07;
        defmt::info!("Block 0: {=u8:x}", block0[0]);
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

    fn iv(&self) -> [u8; 16] {
        self.block0.clone()
    }


    fn get_header_block(&self) -> &[u8] {
        return &self.aad_header[0..self.aad_header_len];
    }




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
        into_ref!(peri);
        let instance = Self { _peripheral: peri };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }

    fn disable_aes(&self) {
        T::regs().cr().modify(|w| w.set_en(false));
    }

    fn enable_aes(&self) {
        T::regs().cr().modify(|w| w.set_en(true));
    }
    fn set_ccm_chmod(&self) {
        T::regs().cr().modify(|w| w.set_chmod10(00));
        T::regs().cr().modify(|w| w.set_chmod2(true));
    }

    fn set_byte_datatype(&self){
        T::regs().cr().modify(|w| w.set_datatype(Datatype::NONE));
    }

    fn setup_direction(&self, dir: Direction) {
        if dir == Direction::Encrypt {
            T::regs().cr().modify(|w| w.set_mode(pac::aes::vals::Mode::MODE1));
        } else {
            T::regs().cr().modify(|w| w.set_mode(pac::aes::vals::Mode::MODE3));
        }
    }

    fn wait_until_calculation_complete(&self) {
        while !T::regs().sr().read().ccf() {}
    }


    fn clear_calculation_complete_flag(&self) {
        T::regs().cr().modify(|w| w.set_ccfc(true));
    }

    fn setup_iv(&self, full_iv: &[u8;16]) {
        #[cfg(feature = "defmt")]
        defmt::info!("FULL IV:{=[u8]:x}", &full_iv[..]);

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
        #[cfg(feature = "defmt")]
        defmt::info!("LAST IV WORD:{=u32:x}", u32::from_le_bytes(iv_word));
        T::regs().ivr(0).modify(|w| w.set_ivi(u32::from_be_bytes(iv_word)));
    }
    #[cfg(feature = "defmt")]
    fn log_aes_state(&self) {
        defmt::info!(" start SR:{=u32:b}", T::regs().sr().read().0);
        defmt::info!("CR: {=u32:b}", T::regs().cr().read().0);
        defmt::info!("keyr0: {=u32:x}", T::regs().keyr(0).read().0);
        defmt::info!("keyr1: {=u32:x}", T::regs().keyr(1).read().0);
        defmt::info!("keyr2: {=u32:x}", T::regs().keyr(2).read().0);
        defmt::info!("keyr3: {=u32:x}", T::regs().keyr(3).read().0);
        defmt::info!("ivr0: {=u32:x}", T::regs().ivr(0).read().0);
        defmt::info!("ivr1: {=u32:x}", T::regs().ivr(1).read().0);
        defmt::info!("ivr2: {=u32:x}", T::regs().ivr(2).read().0);
        defmt::info!("ivr3: {=u32:x}", T::regs().ivr(3).read().0);

    }
    /// Start a new encrypt or decrypt operation for the given cipher.
    pub async fn start<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &mut self,
        cipher: &'c C,
        dir: Direction,
    ) -> Context<'c, C> {
        let ctx: Context<'c, C> = Context {
            dir,
            cipher: cipher,
            phantom_data: PhantomData,
        };
        self.disable_aes();

        self.set_ccm_chmod();

        self.set_byte_datatype();

        T::regs().cr().modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::INITPHASE));

        self.setup_direction(dir);

        let key = ctx.cipher.keyr();
        self.load_keyr(key);

        let iv = ctx.cipher.iv();
        self.setup_iv(&iv);

        // 6 test-ready
        self.enable_aes();

        // 7 test-ready
        self.wait_until_calculation_complete();

        // 8 test-ready
        self.clear_calculation_complete_flag();

        #[cfg(feature = "defmt")]
        self.log_aes_state();
        ctx
    }

    /// Start a new encrypt or decrypt operation for the given cipher.
    pub async fn start_ecb<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &mut self,
        cipher: &'c C,
        dir: Direction,
    ) -> Context<'c, C> {
        let ctx: Context<'c, C> = Context {
            dir,
            cipher: cipher,
            phantom_data: PhantomData,
        };
        self.disable_aes();

        T::regs().cr().modify(|w| w.set_chmod10(00));
        T::regs().cr().modify(|w| w.set_chmod2(false));

        self.set_byte_datatype();


        self.setup_direction(dir);

        let key = ctx.cipher.keyr();
        self.load_keyr(key);


        // 6 test-ready
        self.enable_aes();

        // // 7 test-ready
        // self.wait_until_calculation_complete();

        // // 8 test-ready
        // self.clear_calculation_complete_flag();

        // #[cfg(feature = "defmt")]
        // self.log_aes_state();
        ctx
    }


    /// Sets up authenticated associated data on the AES peripheral.
    /// 
    /// Invariant: can be called only **once** per single decryption/encryption procedure
    pub async fn aad<
        'c,
        const TAG_SIZE: usize,
        C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<TAG_SIZE>,
    >(
        &mut self,
        ctx: &mut Context<'c, C>,
        aad: &[u8],
    ) {
        let mut aad_buffer: [u8;16] = [0;16];
        let mut aad_buffer_idx = 0;
        T::regs()
            .cr()
            .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::HEADERPHASE));

        // 2 test-ready
        self.enable_aes();

        // 3 test-ready - make sure blocking vs async difference don't affect anything in testing
        // First write the header B1 block if not yet written.
        let header = ctx.cipher.get_header_block();
        aad_buffer[0..header.len()].copy_from_slice(header);
        aad_buffer_idx += header.len();

        let mut processed_aad_len = 0;

        while processed_aad_len < aad.len() {
            let remaining_aad_len = aad.len()- processed_aad_len;
            let remaining_buffer_len = C::BLOCK_SIZE - aad_buffer_idx;

            // Fill buffer with as much data from aad as possible.
            let len_to_copy = core::cmp::min(remaining_aad_len, remaining_buffer_len);
            aad_buffer[aad_buffer_idx..aad_buffer_idx + len_to_copy].copy_from_slice(&aad[processed_aad_len..processed_aad_len+len_to_copy]);
            aad_buffer_idx += len_to_copy;
            processed_aad_len += len_to_copy;

            // In case we didn't fill whole buffer ( i.e. whole aad has already been processed),
            // fill the remaining space with 0s.
            aad_buffer[aad_buffer_idx..].fill(0);

            self.write_bytes_blocking_no_read(C::BLOCK_SIZE, &aad_buffer);



            // Reset the buffer idx for the next block processing
            aad_buffer_idx = 0;
        }
        
        
    
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
    ) {
        if input.len() > output.len() {
            panic!("Output buffer length must match input length.");
        }
        // TODO there was a check that made sure that AAD was processed if its len was configured as more than 0. Bring it back
    
        T::regs()
        .cr()
        .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::PAYLOADPHASE));

        // Only useful when there's no AAD.
        self.enable_aes();
        

        let full_blocks = input.len() / C::BLOCK_SIZE;
        let mut idx: usize = 0;
        for _ in 0..full_blocks {
            self.write_then_read_bytes(
                C::BLOCK_SIZE,
                &input[idx..idx + C::BLOCK_SIZE],
                &mut output[idx..idx + C::BLOCK_SIZE],
            );

            idx+=C::BLOCK_SIZE;
        }
        let remaining_input_len = input.len() % C::BLOCK_SIZE;
        if remaining_input_len > 0 {

            // Set up npblb so that AES knows to skip some trailing bytes
            let padding_len = C::BLOCK_SIZE - remaining_input_len;
            if ctx.dir == Direction::Decrypt{
                T::regs().cr().modify(|w| w.set_npblb(padding_len as u8));
            }

            // Copy remaining message to the front, the rest SHOULD be 0s
            let mut in_buffer: [u8;16] = [0;16];
            in_buffer[..remaining_input_len].copy_from_slice(&input[idx..idx+remaining_input_len]);

            let mut out_buffer = [0;16];

            self.write_then_read_bytes(
                C::BLOCK_SIZE,
                &in_buffer,
                &mut out_buffer,
            );

            output[idx..idx+remaining_input_len].copy_from_slice(&out_buffer[..remaining_input_len]);


        }


    }

    // Generates an authentication tag for authenticated ciphers including GCM, CCM, and GMAC.
    /// Called after the all data has been encrypted/decrypted by `payload`.
    pub async fn finish<
        'c,
        const TAG_SIZE: usize,
        C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<TAG_SIZE>,
    >(
        &mut self,
        _ctx: Context<'c, C>,
    ) -> [u8; TAG_SIZE] {

        T::regs()
            .cr()
            .modify(|w| w.set_gcmph(pac::aes::vals::Gcmph::FINALPHASE));

        let mut full_tag: [u8; 16] = [0; 16];

        while !T::regs().sr().read().ccf() {}
        self.read_bytes_blocking(C::BLOCK_SIZE, &mut full_tag);
        T::regs().cr().modify(|w| w.set_ccfc(true));

        let mut tag: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag.copy_from_slice(&full_tag[0..TAG_SIZE]);

        self.disable_aes();

        tag
    }
    /// Works ONLY for AES128
    fn load_keyr(&self, key: &[u8]) {
        T::regs().cr().modify(|w| w.set_keysize(false));
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

    fn write_then_read_bytes(&self, block_size: usize, blocks_in: &[u8], blocks_out: &mut [u8]) {
        const BYTES_IN_WORD: usize = 4;
        // Ensure input is a multiple of block size.
        assert_eq!(blocks_in.len() % block_size, 0);
        assert_eq!(blocks_in.len(), blocks_out.len());
        let mut index: usize = 0;
        let end_index = blocks_in.len();
        while index < end_index {
            let mut in_word: [u8; 4] = [0; BYTES_IN_WORD];
            in_word.copy_from_slice(&blocks_in[index..index + BYTES_IN_WORD]);
            #[cfg(feature = "defmt")]
            defmt::info!("WRITE :{=u32:x}",u32::from_be_bytes(in_word));
            T::regs().dinr().write(|w| w.set_din(u32::from_be_bytes(in_word)));
            index += BYTES_IN_WORD;
            if index % block_size == 0 {
                // wait until coputation clear flag appears
                while !T::regs().sr().read().ccf() {}
                #[cfg(feature = "defmt")]
                defmt::info!("after flag");
                let num_reads = block_size / BYTES_IN_WORD;
                for k in 0..num_reads {
                    let out_word = T::regs().doutr().read().dout();
                    #[cfg(feature = "defmt")]
                    defmt::info!("READ:{=u32:x}",out_word);         
                    blocks_out[k * BYTES_IN_WORD..(k + 1) * BYTES_IN_WORD].copy_from_slice(&out_word.to_be_bytes())
                }
                // clear computation complete flag
                T::regs().cr().modify(|w| w.set_ccfc(true));

                // Block until input FIFO is empty.
            }
        }
    }

    fn write_bytes_blocking_no_read(&self, block_size: usize, blocks: &[u8]) {
        if blocks.len() == 0 {
            return;
        }
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        let mut index: usize = 0;
        let end_index = blocks.len();
        while index < end_index {
            let mut in_word: [u8; 4] = [0; 4];
            in_word.copy_from_slice(&blocks[index..index + 4]);
            #[cfg(feature = "defmt")]
            defmt::info!("WRITE NO READ:{=u32:x}",u32::from_be_bytes(in_word));
            T::regs().dinr().write(|w| w.set_din(u32::from_be_bytes(in_word)));
            index += 4;
            if index % block_size == 0 {
                self.wait_until_calculation_complete();
                self.clear_calculation_complete_flag();
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
        defmt::info!("{=u32:b}", T::regs().sr().read().0);
            blocks[index..index + 4].copy_from_slice(u32::to_be_bytes(out_word).as_slice());
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

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
