//! Crypto Accelerator (CRYP)
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
use core::cmp::min;
use core::marker::PhantomData;

use aligned::{A4, Aligned};
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma::{ChannelAndRequest, TransferOptions};
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode};
use crate::{interrupt, pac, peripherals, rcc};

const DES_BLOCK_SIZE: usize = 8; // 64 bits
const AES_BLOCK_SIZE: usize = 16; // 128 bits

static CRYP_WAKER: AtomicWaker = AtomicWaker::new();

/// CRYP interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let bits = T::regs().misr().read();
        if bits.inmis() {
            T::regs().imscr().modify(|w| w.set_inim(false));
            CRYP_WAKER.wake();
        }
        if bits.outmis() {
            T::regs().imscr().modify(|w| w.set_outim(false));
            CRYP_WAKER.wake();
        }
    }
}

/// This trait encapsulates all cipher-specific behavior/
pub trait Cipher<'c> {
    /// Processing block size. Determined by the processor and the algorithm.
    const BLOCK_SIZE: usize;

    /// Indicates whether the cipher requires the application to provide padding.
    /// If `true`, no partial blocks will be accepted (a panic will occur).
    const REQUIRES_PADDING: bool = false;

    /// Returns the symmetric key.
    fn key(&self) -> &[u8];

    /// Returns the initialization vector.
    fn iv(&self) -> &[u8];

    /// Sets the processor algorithm mode according to the associated cipher.
    fn set_algomode(&self, p: pac::cryp::Cryp);

    /// Performs any key preparation within the processor, if necessary.
    fn prepare_key(&self, _p: pac::cryp::Cryp, _dir: Direction) {}

    /// Performs any cipher-specific initialization.
    fn init_phase_blocking<T: Instance, M: Mode>(&self, _p: pac::cryp::Cryp, _cryp: &Cryp<T, M>) {}

    /// Performs any cipher-specific initialization.
    async fn init_phase<T: Instance>(&self, _p: pac::cryp::Cryp, _cryp: &mut Cryp<'_, T, Async>) {}

    /// Called prior to processing the last data block for cipher-specific operations.
    fn pre_final(&self, _p: pac::cryp::Cryp, _dir: Direction, _padding_len: usize) -> [u32; 4] {
        return [0; 4];
    }

    /// Called after processing the last data block for cipher-specific operations.
    fn post_final_blocking<T: Instance, M: Mode>(
        &self,
        _p: pac::cryp::Cryp,
        _cryp: &Cryp<T, M>,
        _dir: Direction,
        _int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        _padding_mask: [u8; 16],
    ) {
    }

    /// Called after processing the last data block for cipher-specific operations.
    async fn post_final<T: Instance>(
        &self,
        _p: pac::cryp::Cryp,
        _cryp: &mut Cryp<'_, T, Async>,
        _dir: Direction,
        _int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        _padding_mask: [u8; 16],
    ) {
    }

    /// Returns the AAD header block as required by the cipher.
    fn get_header_block(&self) -> ([u8; 10], usize) {
        ([0; 10], 0)
    }

    /// CCM only: the counter block with the counter at zero, which the final
    /// phase feeds to the core instead of the GCM lengths block.
    fn ccm_ctr0(&self) -> Option<[u8; 16]> {
        None
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

/// TDES-ECB Cipher Mode
pub struct TdesEcb<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 0],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> TdesEcb<'c, KEY_SIZE> {
    /// Constructs a new AES-ECB cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE]) -> Self {
        return Self { key: key, iv: &[0; 0] };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for TdesEcb<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = DES_BLOCK_SIZE;
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(0));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(0));
            p.cr().modify(|w| w.set_algomode3(false));
        }
    }
}

impl<'c> CipherSized for TdesEcb<'c, { 112 / 8 }> {}
impl<'c> CipherSized for TdesEcb<'c, { 168 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for TdesEcb<'c, KEY_SIZE> {}

/// TDES-CBC Cipher Mode
pub struct TdesCbc<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 8],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> TdesCbc<'c, KEY_SIZE> {
    /// Constructs a new TDES-CBC cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 8]) -> Self {
        return Self { key: key, iv: iv };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for TdesCbc<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = DES_BLOCK_SIZE;
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(1));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(1));
            p.cr().modify(|w| w.set_algomode3(false));
        }
    }
}

impl<'c> CipherSized for TdesCbc<'c, { 112 / 8 }> {}
impl<'c> CipherSized for TdesCbc<'c, { 168 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for TdesCbc<'c, KEY_SIZE> {}

/// DES-ECB Cipher Mode
pub struct DesEcb<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 0],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> DesEcb<'c, KEY_SIZE> {
    /// Constructs a new AES-ECB cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE]) -> Self {
        return Self { key: key, iv: &[0; 0] };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for DesEcb<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = DES_BLOCK_SIZE;
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(2));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(2));
            p.cr().modify(|w| w.set_algomode3(false));
        }
    }
}

impl<'c> CipherSized for DesEcb<'c, { 56 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for DesEcb<'c, KEY_SIZE> {}

/// DES-CBC Cipher Mode
pub struct DesCbc<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 8],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> DesCbc<'c, KEY_SIZE> {
    /// Constructs a new AES-CBC cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 8]) -> Self {
        return Self { key: key, iv: iv };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for DesCbc<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = DES_BLOCK_SIZE;
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(3));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(3));
            p.cr().modify(|w| w.set_algomode3(false));
        }
    }
}

impl<'c> CipherSized for DesCbc<'c, { 56 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for DesCbc<'c, KEY_SIZE> {}

// The AES cipher types are shared with `aes`/`saes` and live in `crate::crypto`.
pub use crate::crypto::{AesCbc, AesCtr, AesEcb, Direction};
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
pub use crate::crypto::{AesCcm, AesGcm, AesGmac};

/// Counter block (CTR0/CTR1) derived from a CCM B0 block: the flags are reduced
/// to the length-field size and the counter field (the last `l` bytes, where
/// `l = (B0[0] & 0x07) + 1`) is set to zero, plus one for the payload phase.
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
fn ccm_b0_to_ctr(b0: &[u8; 16], ctr0: bool) -> [u8; 16] {
    let l = (b0[0] & 0x07) as usize + 1;
    let mut ctr = *b0;
    ctr[0] &= 0x07;
    ctr[16 - l..].fill(0);
    if !ctr0 {
        ctr[15] = 0x01;
    }
    ctr
}

/// `Cipher` hooks for the shared AES cipher types, derived from the
/// register-agnostic [`crate::crypto::Cipher`] description.
///
/// Each hook is a pure function of the cipher mode (`chmod_bits`), direction
/// and IV, so this single blanket implementation serves every AES cipher; the
/// DES/TDES ciphers keep their own implementations above.
impl<'c, C: crate::crypto::Cipher<'c>> Cipher<'c> for C {
    const BLOCK_SIZE: usize = <C as crate::crypto::Cipher<'c>>::BLOCK_SIZE;
    const REQUIRES_PADDING: bool = <C as crate::crypto::Cipher<'c>>::REQUIRES_PADDING;

    fn key(&self) -> &[u8] {
        <C as crate::crypto::Cipher<'c>>::key(self)
    }

    fn iv(&self) -> &[u8] {
        <C as crate::crypto::Cipher<'c>>::iv(self)
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        // The CRYP ALGOMODE numbering differs from the shared `chmod` one:
        // ECB 4, CBC 5, CTR 6, GCM/GMAC (0 + ALGOMODE3), CCM (1 + ALGOMODE3).
        let chmod = <C as crate::crypto::Cipher<'c>>::chmod_bits(self);
        #[cfg(cryp_v1)]
        match chmod {
            0 => p.cr().modify(|w| w.set_algomode(4)),
            1 => p.cr().modify(|w| w.set_algomode(5)),
            2 => p.cr().modify(|w| w.set_algomode(6)),
            _ => unreachable!("no authenticated modes on cryp_v1"),
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        match chmod {
            0 => p.cr().modify(|w| w.set_algomode0(4)),
            1 => p.cr().modify(|w| w.set_algomode0(5)),
            2 => p.cr().modify(|w| w.set_algomode0(6)),
            3 => {
                p.cr().modify(|w| w.set_algomode0(0));
                p.cr().modify(|w| w.set_algomode3(true));
            }
            _ => {
                p.cr().modify(|w| w.set_algomode0(1));
                p.cr().modify(|w| w.set_algomode3(true));
            }
        }
    }

    fn prepare_key(&self, p: pac::cryp::Cryp, dir: Direction) {
        // Only ECB and CBC decryption need the key prepared in the processor.
        if dir == Direction::Encrypt || <C as crate::crypto::Cipher<'c>>::chmod_bits(self) > 1 {
            return;
        }
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(7));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(7));
            p.cr().modify(|w| w.set_algomode3(false));
        }
        p.cr().modify(|w| w.set_crypen(true));
        while p.sr().read().busy() {}
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::cryp::Cryp, _cryp: &Cryp<T, M>) {
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        if <C as crate::crypto::Cipher<'c>>::uses_gcm_phases(self) {
            p.cr().modify(|w| w.set_gcm_ccmph(0));
            p.cr().modify(|w| w.set_crypen(true));
            if <C as crate::crypto::Cipher<'c>>::is_ccm_mode(self) {
                // The core consumes B0 and clears CRYPEN itself once done, so it is
                // enabled before B0 is written (as the ST HAL does); waiting on IFEM
                // here would never return.
                let b0: [u8; 16] = self.iv().try_into().unwrap();
                for word in Cryp::<T, M>::phase_block_words(&b0) {
                    p.din().write_value(word);
                }
            }
            while p.cr().read().crypen() {}
        }
        #[cfg(cryp_v1)]
        let _ = (p, _cryp);
    }

    async fn init_phase<T: Instance>(&self, p: pac::cryp::Cryp, cryp: &mut Cryp<'_, T, Async>) {
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        if <C as crate::crypto::Cipher<'c>>::uses_gcm_phases(self) {
            p.cr().modify(|w| w.set_gcm_ccmph(0));
            p.cr().modify(|w| w.set_crypen(true));
            if <C as crate::crypto::Cipher<'c>>::is_ccm_mode(self) {
                let b0: [u8; 16] = self.iv().try_into().unwrap();
                let words = Cryp::<T, Async>::phase_block_words(&b0);
                Cryp::<T, Async>::write_words(cryp.indma.as_mut().unwrap(), Self::BLOCK_SIZE, &words).await;
            }
            while p.cr().read().crypen() {}
        }
        #[cfg(cryp_v1)]
        let _ = (p, cryp);
    }

    fn pre_final(&self, p: pac::cryp::Cryp, dir: Direction, padding_len: usize) -> [u32; 4] {
        #[cfg(cryp_v1)]
        {
            let _ = (p, dir, padding_len);
            return [0; 4];
        }
        #[cfg(cryp_v2)]
        {
            if <C as crate::crypto::Cipher<'c>>::is_ccm_mode(self) {
                // Handle special CCM partial block process.
                if dir == Direction::Decrypt {
                    p.cr().modify(|w| w.set_crypen(false));
                    let iv1temp = p.init(1).ivrr().read();
                    let temp1 = [
                        p.csgcmccmr(0).read().swap_bytes(),
                        p.csgcmccmr(1).read().swap_bytes(),
                        p.csgcmccmr(2).read().swap_bytes(),
                        p.csgcmccmr(3).read().swap_bytes(),
                    ];
                    p.init(1).ivrr().write_value(iv1temp);
                    p.cr().modify(|w| w.set_algomode3(false));
                    p.cr().modify(|w| w.set_algomode0(6));
                    p.cr().modify(|w| w.set_crypen(true));
                    return temp1;
                }
            } else if <C as crate::crypto::Cipher<'c>>::uses_gcm_phases(self) && dir == Direction::Encrypt {
                // Handle special GCM partial block process.
                p.cr().modify(|w| w.set_crypen(false));
                p.cr().modify(|w| w.set_algomode3(false));
                p.cr().modify(|w| w.set_algomode0(6));
                let iv1r = p.csgcmccmr(7).read() - 1;
                p.init(1).ivrr().write_value(iv1r);
                p.cr().modify(|w| w.set_crypen(true));
            }
            return [0; 4];
        }
        #[cfg(any(cryp_v3, cryp_v4))]
        {
            let _ = dir;
            if <C as crate::crypto::Cipher<'c>>::uses_gcm_phases(self) {
                // Handle special GCM/CCM partial block process.
                p.cr().modify(|w| w.set_npblb(padding_len as u8));
            }
            [0; 4]
        }
    }

    fn post_final_blocking<T: Instance, M: Mode>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &Cryp<T, M>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        temp1: [u32; 4],
        padding_mask: [u8; AES_BLOCK_SIZE],
    ) {
        #[cfg(cryp_v2)]
        {
            if <C as crate::crypto::Cipher<'c>>::is_ccm_mode(self) {
                if dir == Direction::Decrypt {
                    // Handle special CCM partial block process.
                    let mut temp2 = [0; 4];
                    temp2[0] = p.csgcmccmr(0).read().swap_bytes();
                    temp2[1] = p.csgcmccmr(1).read().swap_bytes();
                    temp2[2] = p.csgcmccmr(2).read().swap_bytes();
                    temp2[3] = p.csgcmccmr(3).read().swap_bytes();
                    p.cr().modify(|w| w.set_algomode3(true));
                    p.cr().modify(|w| w.set_algomode0(1));
                    p.cr().modify(|w| w.set_gcm_ccmph(3));
                    // Header phase
                    p.cr().modify(|w| w.set_gcm_ccmph(1));
                    for i in 0..AES_BLOCK_SIZE {
                        int_data[i] = int_data[i] & padding_mask[i];
                    }
                    let mut in_data: [u32; 4] = [0; 4];
                    for i in 0..in_data.len() {
                        let mut int_bytes: [u8; 4] = [0; 4];
                        int_bytes.copy_from_slice(&int_data[(i * 4)..(i * 4) + 4]);
                        let int_word = u32::from_le_bytes(int_bytes);
                        in_data[i] = int_word;
                        in_data[i] = in_data[i] ^ temp1[i] ^ temp2[i];
                    }
                    cryp.write_words_blocking(Self::BLOCK_SIZE, &in_data);
                }
            } else if <C as crate::crypto::Cipher<'c>>::uses_gcm_phases(self) && dir == Direction::Encrypt {
                // Handle special GCM partial block process.
                p.cr().modify(|w| w.set_crypen(false));
                p.cr().modify(|w| w.set_algomode3(true));
                p.cr().modify(|w| w.set_algomode0(0));
                for i in 0..AES_BLOCK_SIZE {
                    int_data[i] = int_data[i] & padding_mask[i];
                }
                p.cr().modify(|w| w.set_crypen(true));
                p.cr().modify(|w| w.set_gcm_ccmph(3));

                cryp.write_bytes_blocking(Self::BLOCK_SIZE, int_data);
                cryp.read_bytes_blocking(Self::BLOCK_SIZE, int_data);
            }
        }
        #[cfg(not(cryp_v2))]
        let _ = (p, cryp, dir, int_data, temp1, padding_mask);
    }

    async fn post_final<T: Instance>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &mut Cryp<'_, T, Async>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        temp1: [u32; 4],
        padding_mask: [u8; AES_BLOCK_SIZE],
    ) {
        #[cfg(cryp_v2)]
        {
            if <C as crate::crypto::Cipher<'c>>::is_ccm_mode(self) {
                if dir == Direction::Decrypt {
                    // Handle special CCM partial block process.
                    let mut temp2 = [0; 4];
                    temp2[0] = p.csgcmccmr(0).read().swap_bytes();
                    temp2[1] = p.csgcmccmr(1).read().swap_bytes();
                    temp2[2] = p.csgcmccmr(2).read().swap_bytes();
                    temp2[3] = p.csgcmccmr(3).read().swap_bytes();
                    p.cr().modify(|w| w.set_algomode3(true));
                    p.cr().modify(|w| w.set_algomode0(1));
                    p.cr().modify(|w| w.set_gcm_ccmph(3));
                    // Header phase
                    p.cr().modify(|w| w.set_gcm_ccmph(1));
                    for i in 0..AES_BLOCK_SIZE {
                        int_data[i] = int_data[i] & padding_mask[i];
                    }
                    let mut in_data: [u32; 4] = [0; 4];
                    for i in 0..in_data.len() {
                        let mut int_bytes: [u8; 4] = [0; 4];
                        int_bytes.copy_from_slice(&int_data[(i * 4)..(i * 4) + 4]);
                        let int_word = u32::from_le_bytes(int_bytes);
                        in_data[i] = int_word;
                        in_data[i] = in_data[i] ^ temp1[i] ^ temp2[i];
                    }
                    Cryp::<T, Async>::write_words(cryp.indma.as_mut().unwrap(), Self::BLOCK_SIZE, &in_data).await;
                }
            } else if <C as crate::crypto::Cipher<'c>>::uses_gcm_phases(self) && dir == Direction::Encrypt {
                // Handle special GCM partial block process.
                p.cr().modify(|w| w.set_crypen(false));
                p.cr().modify(|w| w.set_algomode3(true));
                p.cr().modify(|w| w.set_algomode0(0));
                for i in 0..AES_BLOCK_SIZE {
                    int_data[i] = int_data[i] & padding_mask[i];
                }
                p.cr().modify(|w| w.set_crypen(true));
                p.cr().modify(|w| w.set_gcm_ccmph(3));

                let mut out_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];

                let read = Cryp::<T, Async>::read_bytes(cryp.outdma.as_mut().unwrap(), Self::BLOCK_SIZE, &mut out_data);
                let write = Cryp::<T, Async>::write_bytes(cryp.indma.as_mut().unwrap(), Self::BLOCK_SIZE, int_data);

                embassy_futures::join::join(read, write).await;

                int_data.copy_from_slice(&out_data);
            }
        }
        #[cfg(not(cryp_v2))]
        let _ = (p, cryp, dir, int_data, temp1, padding_mask);
    }

    fn get_header_block(&self) -> ([u8; 10], usize) {
        <C as crate::crypto::Cipher<'c>>::ccm_aad_header(self)
    }

    fn ccm_ctr0(&self) -> Option<[u8; 16]> {
        if !<C as crate::crypto::Cipher<'c>>::is_ccm_mode(self) {
            return None;
        }
        let b0: [u8; 16] = self.iv().try_into().unwrap();
        Some(ccm_b0_to_ctr(&b0, true))
    }
}

// The marker traits apply to every shared cipher type as well.
impl<C: crate::crypto::CipherSized> CipherSized for C {}
impl<C: crate::crypto::IVSized> IVSized for C {}
impl<const TAG_SIZE: usize, C: crate::crypto::CipherAuthenticated<TAG_SIZE>> CipherAuthenticated<TAG_SIZE> for C {}
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
    cr: u32,
    iv: [u32; 4],
    csgcmccm: [u32; 8],
    csgcm: [u32; 8],
    header_len: u64,
    payload_len: u64,
    // 4-byte aligned because the async DMA path transfers from this buffer as u32 words.
    aad_buffer: Aligned<A4, [u8; 16]>,
    aad_buffer_len: usize,
}

/// Crypto Accelerator Driver
pub struct Cryp<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _marker: PhantomData<M>,
    indma: Option<ChannelAndRequest<'d>>,
    outdma: Option<ChannelAndRequest<'d>>,
}

impl<'d, T: Instance> crate::suspend::SealedSuspendablePeripheral for Cryp<'d, T, Blocking> {
    type InternalState = Peri<'d, T>;

    fn resume(state: Self::InternalState) -> Self {
        critical_section::with(|cs| rcc::enable_and_reset_with_cs_no_refcount::<peripherals::CRYP>(cs));

        Self {
            _peripheral: state,
            _marker: PhantomData,
            indma: None,
            outdma: None,
        }
    }

    fn suspend(self) -> Self::InternalState {
        unsafe { self._peripheral.clone_unchecked() }
    }
}

impl<'d, T: Instance> Cryp<'d, T, Blocking> {
    /// Create a new CRYP driver in blocking mode.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let instance = Self {
            _peripheral: peri,
            _marker: PhantomData,
            indma: None,
            outdma: None,
        };

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        instance
    }
}

impl<'d, T: Instance, M: Mode> Cryp<'d, T, M> {
    /// Start a new encrypt or decrypt operation for the given cipher.
    pub fn start_blocking<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &self,
        cipher: &'c C,
        dir: Direction,
    ) -> Context<'c, C> {
        let mut ctx: Context<'c, C> = Context {
            dir,
            last_block_processed: false,
            cr: 0,
            iv: [0; 4],
            csgcmccm: [0; 8],
            csgcm: [0; 8],
            aad_complete: false,
            header_len: 0,
            payload_len: 0,
            cipher: cipher,
            phantom_data: PhantomData,
            header_processed: false,
            aad_buffer: Aligned([0; 16]),
            aad_buffer_len: 0,
        };

        T::regs().cr().modify(|w| w.set_crypen(false));

        let key = ctx.cipher.key();

        if key.len() == (128 / 8) {
            T::regs().cr().modify(|w| w.set_keysize(0));
        } else if key.len() == (192 / 8) {
            T::regs().cr().modify(|w| w.set_keysize(1));
        } else if key.len() == (256 / 8) {
            T::regs().cr().modify(|w| w.set_keysize(2));
        }

        self.load_key(key);

        // Set data type to 8-bit. This will match software implementations.
        T::regs().cr().modify(|w| w.set_datatype(2));

        ctx.cipher.prepare_key(T::regs(), dir);

        ctx.cipher.set_algomode(T::regs());

        // Set encrypt/decrypt
        if dir == Direction::Encrypt {
            T::regs().cr().modify(|w| w.set_algodir(false));
        } else {
            T::regs().cr().modify(|w| w.set_algodir(true));
        }

        // Load the IV into the registers.
        let iv = ctx.cipher.iv();
        let mut full_iv: [u8; 16] = [0; 16];
        full_iv[0..iv.len()].copy_from_slice(iv);
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        if ctx.cipher.ccm_ctr0().is_some() {
            // For CCM, `iv()` is the B0 block; the payload phase starts from
            // counter block CTR1.
            full_iv = ccm_b0_to_ctr(&full_iv, false);
        }
        let mut iv_idx = 0;
        let mut iv_word: [u8; 4] = [0; 4];
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().init(0).ivlr().write_value(u32::from_be_bytes(iv_word));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().init(0).ivrr().write_value(u32::from_be_bytes(iv_word));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().init(1).ivlr().write_value(u32::from_be_bytes(iv_word));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        T::regs().init(1).ivrr().write_value(u32::from_be_bytes(iv_word));

        // Flush in/out FIFOs
        T::regs().cr().modify(|w| w.set_fflush(true));

        ctx.cipher.init_phase_blocking(T::regs(), self);

        #[cfg(any(cryp_v3, cryp_v4))]
        T::regs().cr().modify(|w| w.set_npblb(0));

        self.store_context(&mut ctx);

        ctx
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
    /// Controls the header phase of cipher processing.
    /// This function is only valid for authenticated ciphers including GCM, CCM, and GMAC.
    /// All additional associated data (AAD) must be supplied to this function prior to starting the payload phase with `payload_blocking`.
    /// The AAD must be supplied in multiples of the block size (128-bits for AES, 64-bits for DES), except when supplying the last block.
    /// When supplying the last block of AAD, `last_aad_block` must be `true`.
    pub fn aad_blocking<
        'c,
        const TAG_SIZE: usize,
        C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<TAG_SIZE>,
    >(
        &self,
        ctx: &mut Context<'c, C>,
        aad: &[u8],
        last_aad_block: bool,
    ) {
        self.load_context(ctx);

        // Perform checks for correctness.
        if ctx.aad_complete {
            panic!("Cannot update AAD after starting payload!")
        }

        ctx.header_len += aad.len() as u64;

        // Header phase
        T::regs().cr().modify(|w| w.set_crypen(false));
        T::regs().cr().modify(|w| w.set_gcm_ccmph(1));
        T::regs().cr().modify(|w| w.set_crypen(true));

        // First write the header B1 block if not yet written.
        if !ctx.header_processed {
            ctx.header_processed = true;
            let (header, header_len) = ctx.cipher.get_header_block();
            ctx.aad_buffer[0..header_len].copy_from_slice(&header[..header_len]);
            ctx.aad_buffer_len += header_len;
        }

        // Fill the header block to make a full block.
        let len_to_copy = min(aad.len(), C::BLOCK_SIZE - ctx.aad_buffer_len);
        ctx.aad_buffer[ctx.aad_buffer_len..ctx.aad_buffer_len + len_to_copy].copy_from_slice(&aad[..len_to_copy]);
        ctx.aad_buffer_len += len_to_copy;
        ctx.aad_buffer[ctx.aad_buffer_len..].fill(0);
        let mut aad_len_remaining = aad.len() - len_to_copy;

        if ctx.aad_buffer_len < C::BLOCK_SIZE {
            // The buffer isn't full and this is the last buffer, so process it as is (already padded).
            if last_aad_block {
                self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer[..]);
                // Block until input FIFO is empty.
                while !T::regs().sr().read().ifem() {}

                // Switch to payload phase.
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
                T::regs().cr().modify(|w| w.set_fflush(true));
            } else {
                // Just return because we don't yet have a full block to process.
                return;
            }
        } else {
            // Load the full block from the buffer.
            self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer[..]);
            // Block until input FIFO is empty.
            while !T::regs().sr().read().ifem() {}
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

        if last_aad_block {
            if leftovers > 0 {
                self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer[..]);
            }
            // Switch to payload phase.
            ctx.aad_complete = true;
            T::regs().cr().modify(|w| w.set_crypen(false));
            T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
            T::regs().cr().modify(|w| w.set_fflush(true));
        }

        self.store_context(ctx);
    }

    /// Performs encryption/decryption on the provided context.
    /// The context determines algorithm, mode, and state of the crypto accelerator.
    /// When the last piece of data is supplied, `last_block` should be `true`.
    /// This function panics under various mismatches of parameters.
    /// Output buffer must be at least as long as the input buffer.
    /// Data must be a multiple of block size (128-bits for AES, 64-bits for DES) for CBC and ECB modes.
    /// Padding or ciphertext stealing must be managed by the application for these modes.
    /// Data must also be a multiple of block size unless `last_block` is `true`.
    pub fn payload_blocking<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &self,
        ctx: &mut Context<'c, C>,
        input: &[u8],
        output: &mut [u8],
        last_block: bool,
    ) {
        self.load_context(ctx);

        let last_block_remainder = input.len() % C::BLOCK_SIZE;

        // Perform checks for correctness.
        if !ctx.aad_complete && ctx.header_len > 0 {
            panic!("Additional associated data must be processed first!");
        } else if !ctx.aad_complete {
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            {
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
                T::regs().cr().modify(|w| w.set_fflush(true));
                T::regs().cr().modify(|w| w.set_crypen(true));
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
                panic!(
                    "Input must be a multiple of {} bytes in ECB and CBC modes. Consider padding or ciphertext stealing.",
                    C::BLOCK_SIZE
                );
            }
        }
        if last_block {
            ctx.last_block_processed = true;
        }

        // Load data into core, block by block.
        let num_full_blocks = input.len() / C::BLOCK_SIZE;
        for block in 0..num_full_blocks {
            let index = block * C::BLOCK_SIZE;
            // Write block in
            self.write_bytes_blocking(C::BLOCK_SIZE, &input[index..index + C::BLOCK_SIZE]);
            // Read block out
            self.read_bytes_blocking(C::BLOCK_SIZE, &mut output[index..index + C::BLOCK_SIZE]);
        }

        // Handle the final block, which is incomplete.
        if last_block_remainder > 0 {
            let padding_len = C::BLOCK_SIZE - last_block_remainder;
            let temp1 = ctx.cipher.pre_final(T::regs(), ctx.dir, padding_len);

            let mut intermediate_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            let mut last_block: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            last_block[..last_block_remainder].copy_from_slice(&input[input.len() - last_block_remainder..input.len()]);
            self.write_bytes_blocking(C::BLOCK_SIZE, &last_block);
            self.read_bytes_blocking(C::BLOCK_SIZE, &mut intermediate_data);

            // Handle the last block depending on mode.
            let output_len = output.len();
            output[output_len - last_block_remainder..output_len]
                .copy_from_slice(&intermediate_data[0..last_block_remainder]);

            let mut mask: [u8; 16] = [0; 16];
            mask[..last_block_remainder].fill(0xFF);
            ctx.cipher
                .post_final_blocking(T::regs(), self, ctx.dir, &mut intermediate_data, temp1, mask);
        }

        ctx.payload_len += input.len() as u64;

        self.store_context(ctx);
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
    /// Generates an authentication tag for authenticated ciphers including GCM, CCM, and GMAC.
    /// Called after the all data has been encrypted/decrypted by `payload`.
    pub fn finish_blocking<'c, const TAG_SIZE: usize, C: Cipher<'c> + CipherSized + IVSized>(
        &self,
        mut ctx: Context<'c, C>,
    ) -> [u8; TAG_SIZE] {
        self.load_context(&mut ctx);

        T::regs().cr().modify(|w| w.set_crypen(false));
        T::regs().cr().modify(|w| w.set_gcm_ccmph(3));
        T::regs().cr().modify(|w| w.set_crypen(true));

        let headerlen1: u32 = ((ctx.header_len * 8) >> 32) as u32;
        let headerlen2: u32 = (ctx.header_len * 8) as u32;
        let payloadlen1: u32 = ((ctx.payload_len * 8) >> 32) as u32;
        let payloadlen2: u32 = (ctx.payload_len * 8) as u32;

        #[cfg(cryp_v2)]
        let footer: [u32; 4] = [
            headerlen1.swap_bytes(),
            headerlen2.swap_bytes(),
            payloadlen1.swap_bytes(),
            payloadlen2.swap_bytes(),
        ];
        #[cfg(any(cryp_v3, cryp_v4))]
        let footer: [u32; 4] = [headerlen1, headerlen2, payloadlen1, payloadlen2];

        if let Some(ctr0) = ctx.cipher.ccm_ctr0() {
            self.write_words_blocking(C::BLOCK_SIZE, &Self::phase_block_words(&ctr0));
        } else {
            self.write_words_blocking(C::BLOCK_SIZE, &footer);
        }

        while !T::regs().sr().read().ofne() {}

        let mut full_tag: [u8; 16] = [0; 16];
        self.read_bytes_blocking(C::BLOCK_SIZE, &mut full_tag);
        let mut tag: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag.copy_from_slice(&full_tag[0..TAG_SIZE]);

        T::regs().cr().modify(|w| w.set_crypen(false));

        tag
    }

    /// Words of a 16-byte block fed in the CCM init or final phase (B0, CTR0).
    ///
    /// These blocks are not data: from cryp_v3 on they are taken as big-endian
    /// words regardless of DATATYPE, while cryp_v2 byte-swaps them like data
    /// (the ST HAL's rev.A path applies `__REV` for the 8-bit data type).
    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
    pub(crate) fn phase_block_words(block: &[u8; 16]) -> [u32; 4] {
        let mut words = [0u32; 4];
        for (word, chunk) in words.iter_mut().zip(block.chunks_exact(4)) {
            let chunk: [u8; 4] = chunk.try_into().unwrap();
            #[cfg(cryp_v2)]
            {
                *word = u32::from_ne_bytes(chunk);
            }
            #[cfg(any(cryp_v3, cryp_v4))]
            {
                *word = u32::from_be_bytes(chunk);
            }
        }
        words
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
        keyword = [0; 4];
        keyword[0..key.len() - keyidx].copy_from_slice(&key[keyidx..key.len()]);
        T::regs().key(3).krr().write_value(u32::from_be_bytes(keyword));
    }

    fn store_context<'c, C: Cipher<'c> + CipherSized>(&self, ctx: &mut Context<'c, C>) {
        // Wait for data block processing to finish. Once the core has disabled
        // itself (as it does at the end of the CCM init phase) nothing more
        // will happen; cryp_v4 then even reports a non-empty input FIFO, so
        // waiting would never return.
        if T::regs().cr().read().crypen() {
            while !T::regs().sr().read().ifem() {}
            while T::regs().sr().read().ofne() {}
            while T::regs().sr().read().busy() {}
        }

        // Disable crypto processor.
        T::regs().cr().modify(|w| w.set_crypen(false));

        // Save the peripheral state.
        ctx.cr = T::regs().cr().read().0;
        ctx.iv[0] = T::regs().init(0).ivlr().read();
        ctx.iv[1] = T::regs().init(0).ivrr().read();
        ctx.iv[2] = T::regs().init(1).ivlr().read();
        ctx.iv[3] = T::regs().init(1).ivrr().read();

        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        for i in 0..8 {
            ctx.csgcmccm[i] = T::regs().csgcmccmr(i).read();
            ctx.csgcm[i] = T::regs().csgcmr(i).read();
        }
    }

    fn load_context<'c, C: Cipher<'c> + CipherSized>(&self, ctx: &Context<'c, C>) {
        // Reload state registers.
        T::regs().cr().write(|w| w.0 = ctx.cr);
        T::regs().init(0).ivlr().write_value(ctx.iv[0]);
        T::regs().init(0).ivrr().write_value(ctx.iv[1]);
        T::regs().init(1).ivlr().write_value(ctx.iv[2]);
        T::regs().init(1).ivrr().write_value(ctx.iv[3]);

        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        for i in 0..8 {
            T::regs().csgcmccmr(i).write_value(ctx.csgcmccm[i]);
            T::regs().csgcmr(i).write_value(ctx.csgcm[i]);
        }
        self.load_key(ctx.cipher.key());

        // Prepare key if applicable.
        ctx.cipher.prepare_key(T::regs(), ctx.dir);
        T::regs().cr().write(|w| w.0 = ctx.cr);

        // Enable crypto processor.
        T::regs().cr().modify(|w| w.set_crypen(true));
    }

    fn write_bytes_blocking(&self, block_size: usize, blocks: &[u8]) {
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        let mut index = 0;
        let end_index = blocks.len();
        while index < end_index {
            let mut in_word: [u8; 4] = [0; 4];
            in_word.copy_from_slice(&blocks[index..index + 4]);
            T::regs().din().write_value(u32::from_ne_bytes(in_word));
            index += 4;
            if index % block_size == 0 {
                // Block until input FIFO is empty.
                while !T::regs().sr().read().ifem() {}
            }
        }
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
    fn write_words_blocking(&self, block_size: usize, blocks: &[u32]) {
        assert_eq!((blocks.len() * 4) % block_size, 0);
        let mut byte_counter: usize = 0;
        for word in blocks {
            T::regs().din().write_value(*word);
            byte_counter += 4;
            if byte_counter % block_size == 0 {
                // Block until input FIFO is empty.
                while !T::regs().sr().read().ifem() {}
            }
        }
    }

    fn read_bytes_blocking(&self, block_size: usize, blocks: &mut [u8]) {
        // Block until there is output to read.
        while !T::regs().sr().read().ofne() {}
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        // Read block out
        let mut index = 0;
        let end_index = blocks.len();
        while index < end_index {
            let out_word: u32 = T::regs().dout().read();
            blocks[index..index + 4].copy_from_slice(u32::to_ne_bytes(out_word).as_slice());
            index += 4;
        }
    }
}

impl<'d, 'c, T: Instance, C> crate::crypto::BlockingCipherOps<'c, C> for Cryp<'d, T, Blocking>
where
    C: crate::crypto::Cipher<'c> + crate::crypto::CipherSized + crate::crypto::IVSized + 'c,
{
    type Context = Context<'c, C>;

    fn start(&mut self, cipher: &'c C, dir: Direction) -> Result<Self::Context, crate::crypto::Error> {
        Ok(self.start_blocking(cipher, dir))
    }

    fn aad(&mut self, ctx: &mut Self::Context, aad: &[u8], last: bool) -> Result<(), crate::crypto::Error>
    where
        C: crate::crypto::CipherAuthenticated<16>,
    {
        // `cryp_v1` has no authenticated modes, so this never runs there.
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            self.aad_blocking::<16, C>(ctx, aad, last);
            Ok(())
        }
        #[cfg(cryp_v1)]
        {
            let _ = (ctx, aad, last);
            unreachable!()
        }
    }

    fn payload(
        &mut self,
        ctx: &mut Self::Context,
        input: &[u8],
        output: &mut [u8],
        last: bool,
    ) -> Result<(), crate::crypto::Error> {
        self.payload_blocking(ctx, input, output, last);
        Ok(())
    }

    fn finish(&mut self, ctx: Self::Context) -> Result<Option<[u8; 16]>, crate::crypto::Error> {
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        if <C as crate::crypto::Cipher>::uses_gcm_phases(ctx.cipher) {
            return Ok(Some(Cryp::<T, Blocking>::finish_blocking::<16, C>(self, ctx)));
        }
        let _ = ctx;
        Ok(None)
    }
}

impl<'d, T: Instance> Cryp<'d, T, Async> {
    /// Create a new CRYP driver.
    pub fn new<D1: DmaIn<T>, D2: DmaOut<T>>(
        peri: Peri<'d, T>,
        indma: Peri<'d, D1>,
        outdma: Peri<'d, D2>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D1::Interrupt, crate::dma::InterruptHandler<D1>>
        + interrupt::typelevel::Binding<D2::Interrupt, crate::dma::InterruptHandler<D2>>
        + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let instance = Self {
            _peripheral: peri,
            _marker: PhantomData,
            indma: new_dma!(indma, _irq),
            outdma: new_dma!(outdma, _irq),
        };

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
            cr: 0,
            iv: [0; 4],
            csgcmccm: [0; 8],
            csgcm: [0; 8],
            aad_complete: false,
            header_len: 0,
            payload_len: 0,
            cipher: cipher,
            phantom_data: PhantomData,
            header_processed: false,
            aad_buffer: Aligned([0; 16]),
            aad_buffer_len: 0,
        };

        T::regs().cr().modify(|w| w.set_crypen(false));

        let key = ctx.cipher.key();

        if key.len() == (128 / 8) {
            T::regs().cr().modify(|w| w.set_keysize(0));
        } else if key.len() == (192 / 8) {
            T::regs().cr().modify(|w| w.set_keysize(1));
        } else if key.len() == (256 / 8) {
            T::regs().cr().modify(|w| w.set_keysize(2));
        }

        self.load_key(key);

        // Set data type to 8-bit. This will match software implementations.
        T::regs().cr().modify(|w| w.set_datatype(2));

        ctx.cipher.prepare_key(T::regs(), dir);

        ctx.cipher.set_algomode(T::regs());

        // Set encrypt/decrypt
        if dir == Direction::Encrypt {
            T::regs().cr().modify(|w| w.set_algodir(false));
        } else {
            T::regs().cr().modify(|w| w.set_algodir(true));
        }

        // Load the IV into the registers.
        let iv = ctx.cipher.iv();
        let mut full_iv: [u8; 16] = [0; 16];
        full_iv[0..iv.len()].copy_from_slice(iv);
        let mut iv_idx = 0;
        let mut iv_word: [u8; 4] = [0; 4];
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().init(0).ivlr().write_value(u32::from_be_bytes(iv_word));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().init(0).ivrr().write_value(u32::from_be_bytes(iv_word));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        iv_idx += 4;
        T::regs().init(1).ivlr().write_value(u32::from_be_bytes(iv_word));
        iv_word.copy_from_slice(&full_iv[iv_idx..iv_idx + 4]);
        T::regs().init(1).ivrr().write_value(u32::from_be_bytes(iv_word));

        // Flush in/out FIFOs
        T::regs().cr().modify(|w| w.set_fflush(true));

        ctx.cipher.init_phase(T::regs(), self).await;

        #[cfg(any(cryp_v3, cryp_v4))]
        T::regs().cr().modify(|w| w.set_npblb(0));

        self.store_context(&mut ctx);

        ctx
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
    /// Controls the header phase of cipher processing.
    /// This function is only valid for authenticated ciphers including GCM, CCM, and GMAC.
    /// All additional associated data (AAD) must be supplied to this function prior to starting the payload phase with `payload`.
    /// The AAD must be supplied in multiples of the block size (128-bits for AES, 64-bits for DES), except when supplying the last block.
    /// When supplying the last block of AAD, `last_aad_block` must be `true`.
    ///
    /// # Buffer alignment
    ///
    /// Best performance is achieved when `aad` is 4-byte aligned: the DMA
    /// transfers as 32-bit words and skips its FIFO packing path.
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
        self.load_context(ctx);

        // Perform checks for correctness.
        if ctx.aad_complete {
            panic!("Cannot update AAD after starting payload!")
        }

        ctx.header_len += aad.len() as u64;

        // Header phase
        T::regs().cr().modify(|w| w.set_crypen(false));
        T::regs().cr().modify(|w| w.set_gcm_ccmph(1));
        T::regs().cr().modify(|w| w.set_crypen(true));

        // First write the header B1 block if not yet written.
        if !ctx.header_processed {
            ctx.header_processed = true;
            let (header, header_len) = ctx.cipher.get_header_block();
            ctx.aad_buffer[0..header_len].copy_from_slice(&header[..header_len]);
            ctx.aad_buffer_len += header_len;
        }

        // Fill the header block to make a full block.
        let len_to_copy = min(aad.len(), C::BLOCK_SIZE - ctx.aad_buffer_len);
        ctx.aad_buffer[ctx.aad_buffer_len..ctx.aad_buffer_len + len_to_copy].copy_from_slice(&aad[..len_to_copy]);
        ctx.aad_buffer_len += len_to_copy;
        ctx.aad_buffer[ctx.aad_buffer_len..].fill(0);
        let mut aad_len_remaining = aad.len() - len_to_copy;

        if ctx.aad_buffer_len < C::BLOCK_SIZE {
            // The buffer isn't full and this is the last buffer, so process it as is (already padded).
            if last_aad_block {
                Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &ctx.aad_buffer[..]).await;
                assert_eq!(T::regs().sr().read().ifem(), true);

                // Switch to payload phase.
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
                T::regs().cr().modify(|w| w.set_fflush(true));
            } else {
                // Just return because we don't yet have a full block to process.
                return;
            }
        } else {
            // Load the full block from the buffer.
            Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &ctx.aad_buffer[..]).await;
            assert_eq!(T::regs().sr().read().ifem(), true);
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
        Self::write_bytes(
            self.indma.as_mut().unwrap(),
            C::BLOCK_SIZE,
            &aad[start_index..end_index],
        )
        .await;

        if last_aad_block {
            if leftovers > 0 {
                Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &ctx.aad_buffer[..]).await;
                assert_eq!(T::regs().sr().read().ifem(), true);
            }
            // Switch to payload phase.
            ctx.aad_complete = true;
            T::regs().cr().modify(|w| w.set_crypen(false));
            T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
            T::regs().cr().modify(|w| w.set_fflush(true));
        }

        self.store_context(ctx);
    }

    /// Performs encryption/decryption on the provided context.
    /// The context determines algorithm, mode, and state of the crypto accelerator.
    /// When the last piece of data is supplied, `last_block` should be `true`.
    /// This function panics under various mismatches of parameters.
    /// Output buffer must be at least as long as the input buffer.
    /// Data must be a multiple of block size (128-bits for AES, 64-bits for DES) for CBC and ECB modes.
    /// Padding or ciphertext stealing must be managed by the application for these modes.
    /// Data must also be a multiple of block size unless `last_block` is `true`.
    ///
    /// # Buffer alignment
    ///
    /// Best performance is achieved when `input` and `output` are 4-byte
    /// aligned: the DMA transfers as 32-bit words and skips its FIFO packing
    /// path. `[u8; N]` arrays are not aligned by default; wrap them in a
    /// `#[repr(align(4))]` newtype (or `aligned::Aligned<aligned::A4, _>`) to
    /// guarantee alignment.
    pub async fn payload<'c, C: Cipher<'c> + CipherSized + IVSized>(
        &mut self,
        ctx: &mut Context<'c, C>,
        input: &[u8],
        output: &mut [u8],
        last_block: bool,
    ) {
        self.load_context(ctx);

        let last_block_remainder = input.len() % C::BLOCK_SIZE;

        // Perform checks for correctness.
        if !ctx.aad_complete && ctx.header_len > 0 {
            panic!("Additional associated data must be processed first!");
        } else if !ctx.aad_complete {
            #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
            {
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
                T::regs().cr().modify(|w| w.set_fflush(true));
                T::regs().cr().modify(|w| w.set_crypen(true));
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
                panic!(
                    "Input must be a multiple of {} bytes in ECB and CBC modes. Consider padding or ciphertext stealing.",
                    C::BLOCK_SIZE
                );
            }
        }
        if last_block {
            ctx.last_block_processed = true;
        }

        // Feed all full blocks to the core in one DMA round-trip per chunk.
        // With burst-level handshake and 4-beat (16-byte) bursts, the channel
        // fires one burst per CRYP request, so the FIFO never overflows.
        // BNDT is 16 bits so cap each chunk at the largest multiple of the
        // cipher block size that fits.
        let num_full_blocks = input.len() / C::BLOCK_SIZE;
        let bulk_len = num_full_blocks * C::BLOCK_SIZE;
        let max_chunk = (0xFFFF / C::BLOCK_SIZE) * C::BLOCK_SIZE;
        let mut offset = 0;
        while offset < bulk_len {
            let chunk = (bulk_len - offset).min(max_chunk);
            let read = Self::read_bytes(
                self.outdma.as_mut().unwrap(),
                C::BLOCK_SIZE,
                &mut output[offset..offset + chunk],
            );
            let write = Self::write_bytes(
                self.indma.as_mut().unwrap(),
                C::BLOCK_SIZE,
                &input[offset..offset + chunk],
            );
            embassy_futures::join::join(read, write).await;
            offset += chunk;
        }

        // Handle the final block, which is incomplete.
        if last_block_remainder > 0 {
            let padding_len = C::BLOCK_SIZE - last_block_remainder;
            let temp1 = ctx.cipher.pre_final(T::regs(), ctx.dir, padding_len);

            // Stack locals are 4-byte aligned for the CRYP DMA (word-width).
            let mut intermediate_data: Aligned<A4, [u8; AES_BLOCK_SIZE]> = Aligned([0; AES_BLOCK_SIZE]);
            let mut last_block: Aligned<A4, [u8; AES_BLOCK_SIZE]> = Aligned([0; AES_BLOCK_SIZE]);
            last_block[..last_block_remainder].copy_from_slice(&input[input.len() - last_block_remainder..input.len()]);
            let read = Self::read_bytes(self.outdma.as_mut().unwrap(), C::BLOCK_SIZE, &mut *intermediate_data);
            let write = Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &*last_block);
            embassy_futures::join::join(read, write).await;

            // Handle the last block depending on mode.
            let output_len = output.len();
            output[output_len - last_block_remainder..output_len]
                .copy_from_slice(&intermediate_data[0..last_block_remainder]);

            let mut mask: [u8; 16] = [0; 16];
            mask[..last_block_remainder].fill(0xFF);
            ctx.cipher
                .post_final(T::regs(), self, ctx.dir, &mut *intermediate_data, temp1, mask)
                .await;
        }

        ctx.payload_len += input.len() as u64;

        self.store_context(ctx);
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
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

        T::regs().cr().modify(|w| w.set_crypen(false));
        T::regs().cr().modify(|w| w.set_gcm_ccmph(3));
        T::regs().cr().modify(|w| w.set_crypen(true));

        let headerlen1: u32 = ((ctx.header_len * 8) >> 32) as u32;
        let headerlen2: u32 = (ctx.header_len * 8) as u32;
        let payloadlen1: u32 = ((ctx.payload_len * 8) >> 32) as u32;
        let payloadlen2: u32 = (ctx.payload_len * 8) as u32;

        #[cfg(cryp_v2)]
        let footer: [u32; 4] = [
            headerlen1.swap_bytes(),
            headerlen2.swap_bytes(),
            payloadlen1.swap_bytes(),
            payloadlen2.swap_bytes(),
        ];
        #[cfg(any(cryp_v3, cryp_v4))]
        let footer: [u32; 4] = [headerlen1, headerlen2, payloadlen1, payloadlen2];

        let ccm_ctr0 = ctx.cipher.ccm_ctr0().map(|ctr0| Self::phase_block_words(&ctr0));
        let block = ccm_ctr0.as_ref().unwrap_or(&footer);
        let write = Self::write_words(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, block);

        let mut full_tag: Aligned<A4, [u8; 16]> = Aligned([0; 16]);
        let read = Self::read_bytes(self.outdma.as_mut().unwrap(), C::BLOCK_SIZE, &mut *full_tag);

        embassy_futures::join::join(read, write).await;

        let mut tag: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag.copy_from_slice(&full_tag[0..TAG_SIZE]);

        T::regs().cr().modify(|w| w.set_crypen(false));

        tag
    }

    async fn write_bytes(dma: &mut ChannelAndRequest<'d>, block_size: usize, blocks: &[u8]) {
        if blocks.len() == 0 {
            return;
        }
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        let dst_ptr: *mut u32 = T::regs().din().as_ptr();
        let options = TransferOptions {
            priority: crate::dma::Priority::High,
            // GPDMA only: 4-beat bursts (16 bytes = one AES block) per
            // peripheral request, matching Linux's stm32-cryp
            // `dst_maxburst = CRYP_DMA_BURST_REG = 4`.
            #[cfg(gpdma)]
            burst_length: crate::dma::Burst::_4Beats,
            ..Default::default()
        };
        // Fast path: 4-byte aligned source becomes a u32 transfer (SDW=DDW=
        // Word), so the channel doesn't have to pack/unpack across widths.
        // Otherwise let the channel handle the mismatch via its FIFO.
        let dma_transfer = unsafe {
            if blocks.as_ptr() as usize % 4 == 0 {
                let num_words = blocks.len() / 4;
                let src: *const [u32] = core::ptr::slice_from_raw_parts(blocks.as_ptr() as *const u32, num_words);
                dma.write_raw(src, dst_ptr, options)
            } else {
                dma.write_raw(blocks, dst_ptr, options)
            }
        };
        T::regs().dmacr().modify(|w| w.set_dien(true));
        // Wait for the transfer to complete.
        dma_transfer.await;
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
    async fn write_words(dma: &mut ChannelAndRequest<'d>, block_size: usize, blocks: &[u32]) {
        if blocks.len() == 0 {
            return;
        }
        // Ensure input is a multiple of block size.
        assert_eq!((blocks.len() * 4) % block_size, 0);
        // Configure DMA to transfer input to crypto core.
        let dst_ptr: *mut u32 = T::regs().din().as_ptr();
        let options = TransferOptions {
            priority: crate::dma::Priority::High,
            #[cfg(gpdma)]
            burst_length: crate::dma::Burst::_4Beats,
            ..Default::default()
        };
        let dma_transfer = unsafe { dma.write_raw(blocks, dst_ptr, options) };
        T::regs().dmacr().modify(|w| w.set_dien(true));
        // Wait for the transfer to complete.
        dma_transfer.await;
    }

    async fn read_bytes(dma: &mut ChannelAndRequest<'d>, block_size: usize, blocks: &mut [u8]) {
        if blocks.len() == 0 {
            return;
        }
        // Ensure input is a multiple of block size.
        assert_eq!(blocks.len() % block_size, 0);
        let src_ptr = T::regs().dout().as_ptr();
        let options = TransferOptions {
            priority: crate::dma::Priority::VeryHigh,
            #[cfg(gpdma)]
            burst_length: crate::dma::Burst::_4Beats,
            ..Default::default()
        };
        // See write_bytes above for the alignment fast/slow path rationale.
        let dma_transfer = unsafe {
            if blocks.as_ptr() as usize % 4 == 0 {
                let num_words = blocks.len() / 4;
                let dst: *mut [u32] = core::ptr::slice_from_raw_parts_mut(blocks.as_mut_ptr() as *mut u32, num_words);
                dma.read_raw(src_ptr, dst, options)
            } else {
                dma.read_raw(src_ptr, blocks, options)
            }
        };
        T::regs().dmacr().modify(|w| w.set_doen(true));
        // Wait for the transfer to complete.
        dma_transfer.await;
    }
}

trait SealedInstance {
    fn regs() -> pac::cryp::Cryp;
}

/// CRYP instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + crate::rcc::RccPeripheral + 'static + Send {
    /// Interrupt for this CRYP instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

foreach_interrupt!(
    ($inst:ident, cryp, CRYP, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::cryp::Cryp {
                crate::pac::$inst
            }
        }
    };
);

dma_trait!(DmaIn, Instance);
dma_trait!(DmaOut, Instance);
