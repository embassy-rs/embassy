//! Crypto Accelerator (CRYP)
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
use core::cmp::min;
use core::marker::PhantomData;
use core::ptr;

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
    _phantom: PhantomData<T>,
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
    fn prepare_key(&self, _p: pac::cryp::Cryp) {}

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

/// AES-ECB Cipher Mode
pub struct AesEcb<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 0],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> AesEcb<'c, KEY_SIZE> {
    /// Constructs a new AES-ECB cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE]) -> Self {
        return Self { key: key, iv: &[0; 0] };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesEcb<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn prepare_key(&self, p: pac::cryp::Cryp) {
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

impl<'c> CipherSized for AesEcb<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesEcb<'c, { 192 / 8 }> {}
impl<'c> CipherSized for AesEcb<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesEcb<'c, KEY_SIZE> {}

/// AES-CBC Cipher Mode
pub struct AesCbc<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 16],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> AesCbc<'c, KEY_SIZE> {
    /// Constructs a new AES-CBC cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 16]) -> Self {
        return Self { key: key, iv: iv };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesCbc<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;
    const REQUIRES_PADDING: bool = true;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn prepare_key(&self, p: pac::cryp::Cryp) {
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

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(5));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(5));
            p.cr().modify(|w| w.set_algomode3(false));
        }
    }
}

impl<'c> CipherSized for AesCbc<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesCbc<'c, { 192 / 8 }> {}
impl<'c> CipherSized for AesCbc<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesCbc<'c, KEY_SIZE> {}

/// AES-CTR Cipher Mode
pub struct AesCtr<'c, const KEY_SIZE: usize> {
    iv: &'c [u8; 16],
    key: &'c [u8; KEY_SIZE],
}

impl<'c, const KEY_SIZE: usize> AesCtr<'c, KEY_SIZE> {
    /// Constructs a new AES-CTR cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 16]) -> Self {
        return Self { key: key, iv: iv };
    }
}

impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesCtr<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &'c [u8] {
        self.iv
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        #[cfg(cryp_v1)]
        {
            p.cr().modify(|w| w.set_algomode(6));
        }
        #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
        {
            p.cr().modify(|w| w.set_algomode0(6));
            p.cr().modify(|w| w.set_algomode3(false));
        }
    }
}

impl<'c> CipherSized for AesCtr<'c, { 128 / 8 }> {}
impl<'c> CipherSized for AesCtr<'c, { 192 / 8 }> {}
impl<'c> CipherSized for AesCtr<'c, { 256 / 8 }> {}
impl<'c, const KEY_SIZE: usize> IVSized for AesCtr<'c, KEY_SIZE> {}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
///AES-GCM Cipher Mode
pub struct AesGcm<'c, const KEY_SIZE: usize> {
    iv: [u8; 16],
    key: &'c [u8; KEY_SIZE],
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> AesGcm<'c, KEY_SIZE> {
    /// Constucts a new AES-GCM cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 12]) -> Self {
        let mut new_gcm = Self { key: key, iv: [0; 16] };
        new_gcm.iv[..12].copy_from_slice(iv);
        new_gcm.iv[15] = 2;
        new_gcm
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesGcm<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.iv.as_slice()
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        p.cr().modify(|w| w.set_algomode0(0));
        p.cr().modify(|w| w.set_algomode3(true));
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::cryp::Cryp, _cryp: &Cryp<T, M>) {
        p.cr().modify(|w| w.set_gcm_ccmph(0));
        p.cr().modify(|w| w.set_crypen(true));
        while p.cr().read().crypen() {}
    }

    async fn init_phase<T: Instance>(&self, p: pac::cryp::Cryp, _cryp: &mut Cryp<'_, T, Async>) {
        p.cr().modify(|w| w.set_gcm_ccmph(0));
        p.cr().modify(|w| w.set_crypen(true));
        while p.cr().read().crypen() {}
    }

    #[cfg(cryp_v2)]
    fn pre_final(&self, p: pac::cryp::Cryp, dir: Direction, _padding_len: usize) -> [u32; 4] {
        //Handle special GCM partial block process.
        if dir == Direction::Encrypt {
            p.cr().modify(|w| w.set_crypen(false));
            p.cr().modify(|w| w.set_algomode3(false));
            p.cr().modify(|w| w.set_algomode0(6));
            let iv1r = p.csgcmccmr(7).read() - 1;
            p.init(1).ivrr().write_value(iv1r);
            p.cr().modify(|w| w.set_crypen(true));
        }
        [0; 4]
    }

    #[cfg(any(cryp_v3, cryp_v4))]
    fn pre_final(&self, p: pac::cryp::Cryp, _dir: Direction, padding_len: usize) -> [u32; 4] {
        //Handle special GCM partial block process.
        p.cr().modify(|w| w.set_npblb(padding_len as u8));
        [0; 4]
    }

    #[cfg(cryp_v2)]
    fn post_final_blocking<T: Instance, M: Mode>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &Cryp<T, M>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        padding_mask: [u8; AES_BLOCK_SIZE],
    ) {
        if dir == Direction::Encrypt {
            //Handle special GCM partial block process.
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

    #[cfg(cryp_v2)]
    async fn post_final<T: Instance>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &mut Cryp<'_, T, Async>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        padding_mask: [u8; AES_BLOCK_SIZE],
    ) {
        if dir == Direction::Encrypt {
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
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c> CipherSized for AesGcm<'c, { 128 / 8 }> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c> CipherSized for AesGcm<'c, { 192 / 8 }> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c> CipherSized for AesGcm<'c, { 256 / 8 }> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> CipherAuthenticated<16> for AesGcm<'c, KEY_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> IVSized for AesGcm<'c, KEY_SIZE> {}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
/// AES-GMAC Cipher Mode
pub struct AesGmac<'c, const KEY_SIZE: usize> {
    iv: [u8; 16],
    key: &'c [u8; KEY_SIZE],
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> AesGmac<'c, KEY_SIZE> {
    /// Constructs a new AES-GMAC cipher for a cryptographic operation.
    pub fn new(key: &'c [u8; KEY_SIZE], iv: &'c [u8; 12]) -> Self {
        let mut new_gmac = Self { key: key, iv: [0; 16] };
        new_gmac.iv[..12].copy_from_slice(iv);
        new_gmac.iv[15] = 2;
        new_gmac
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> Cipher<'c> for AesGmac<'c, KEY_SIZE> {
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.iv.as_slice()
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        p.cr().modify(|w| w.set_algomode0(0));
        p.cr().modify(|w| w.set_algomode3(true));
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::cryp::Cryp, _cryp: &Cryp<T, M>) {
        p.cr().modify(|w| w.set_gcm_ccmph(0));
        p.cr().modify(|w| w.set_crypen(true));
        while p.cr().read().crypen() {}
    }

    async fn init_phase<T: Instance>(&self, p: pac::cryp::Cryp, _cryp: &mut Cryp<'_, T, Async>) {
        p.cr().modify(|w| w.set_gcm_ccmph(0));
        p.cr().modify(|w| w.set_crypen(true));
        while p.cr().read().crypen() {}
    }

    #[cfg(cryp_v2)]
    fn pre_final(&self, p: pac::cryp::Cryp, dir: Direction, _padding_len: usize) -> [u32; 4] {
        //Handle special GCM partial block process.
        if dir == Direction::Encrypt {
            p.cr().modify(|w| w.set_crypen(false));
            p.cr().modify(|w| w.set_algomode3(false));
            p.cr().modify(|w| w.set_algomode0(6));
            let iv1r = p.csgcmccmr(7).read() - 1;
            p.init(1).ivrr().write_value(iv1r);
            p.cr().modify(|w| w.set_crypen(true));
        }
        [0; 4]
    }

    #[cfg(any(cryp_v3, cryp_v4))]
    fn pre_final(&self, p: pac::cryp::Cryp, _dir: Direction, padding_len: usize) -> [u32; 4] {
        //Handle special GCM partial block process.
        p.cr().modify(|w| w.set_npblb(padding_len as u8));
        [0; 4]
    }

    #[cfg(cryp_v2)]
    fn post_final_blocking<T: Instance, M: Mode>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &Cryp<T, M>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        padding_mask: [u8; AES_BLOCK_SIZE],
    ) {
        if dir == Direction::Encrypt {
            //Handle special GCM partial block process.
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

    #[cfg(cryp_v2)]
    async fn post_final<T: Instance>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &mut Cryp<'_, T, Async>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        _temp1: [u32; 4],
        padding_mask: [u8; AES_BLOCK_SIZE],
    ) {
        if dir == Direction::Encrypt {
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
        }
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c> CipherSized for AesGmac<'c, { 128 / 8 }> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c> CipherSized for AesGmac<'c, { 192 / 8 }> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c> CipherSized for AesGmac<'c, { 256 / 8 }> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> CipherAuthenticated<16> for AesGmac<'c, KEY_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize> IVSized for AesGmac<'c, KEY_SIZE> {}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
/// AES-CCM Cipher Mode
pub struct AesCcm<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> {
    key: &'c [u8; KEY_SIZE],
    aad_header: [u8; 6],
    aad_header_len: usize,
    block0: [u8; 16],
    ctr: [u8; 16],
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> AesCcm<'c, KEY_SIZE, TAG_SIZE, IV_SIZE> {
    /// Constructs a new AES-CCM cipher for a cryptographic operation.
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
                let aad_len_bytes: [u8; 4] = (aad_len as u32).to_be_bytes();
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
        let payload_len_bytes: [u8; 4] = (payload_len as u32).to_be_bytes();
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

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize, const IV_SIZE: usize> Cipher<'c>
    for AesCcm<'c, KEY_SIZE, TAG_SIZE, IV_SIZE>
{
    const BLOCK_SIZE: usize = AES_BLOCK_SIZE;

    fn key(&self) -> &'c [u8] {
        self.key
    }

    fn iv(&self) -> &[u8] {
        self.ctr.as_slice()
    }

    fn set_algomode(&self, p: pac::cryp::Cryp) {
        p.cr().modify(|w| w.set_algomode0(1));
        p.cr().modify(|w| w.set_algomode3(true));
    }

    fn init_phase_blocking<T: Instance, M: Mode>(&self, p: pac::cryp::Cryp, cryp: &Cryp<T, M>) {
        p.cr().modify(|w| w.set_gcm_ccmph(0));

        cryp.write_bytes_blocking(Self::BLOCK_SIZE, &self.block0);

        p.cr().modify(|w| w.set_crypen(true));
        while p.cr().read().crypen() {}
    }

    async fn init_phase<T: Instance>(&self, p: pac::cryp::Cryp, cryp: &mut Cryp<'_, T, Async>) {
        p.cr().modify(|w| w.set_gcm_ccmph(0));

        Cryp::<T, Async>::write_bytes(cryp.indma.as_mut().unwrap(), Self::BLOCK_SIZE, &self.block0).await;

        p.cr().modify(|w| w.set_crypen(true));
        while p.cr().read().crypen() {}
    }

    fn get_header_block(&self) -> &[u8] {
        return &self.aad_header[0..self.aad_header_len];
    }

    #[cfg(cryp_v2)]
    fn pre_final(&self, p: pac::cryp::Cryp, dir: Direction, _padding_len: usize) -> [u32; 4] {
        //Handle special CCM partial block process.
        let mut temp1 = [0; 4];
        if dir == Direction::Decrypt {
            p.cr().modify(|w| w.set_crypen(false));
            let iv1temp = p.init(1).ivrr().read();
            temp1[0] = p.csgcmccmr(0).read().swap_bytes();
            temp1[1] = p.csgcmccmr(1).read().swap_bytes();
            temp1[2] = p.csgcmccmr(2).read().swap_bytes();
            temp1[3] = p.csgcmccmr(3).read().swap_bytes();
            p.init(1).ivrr().write_value(iv1temp);
            p.cr().modify(|w| w.set_algomode3(false));
            p.cr().modify(|w| w.set_algomode0(6));
            p.cr().modify(|w| w.set_crypen(true));
        }
        return temp1;
    }

    #[cfg(any(cryp_v3, cryp_v4))]
    fn pre_final(&self, p: pac::cryp::Cryp, _dir: Direction, padding_len: usize) -> [u32; 4] {
        //Handle special GCM partial block process.
        p.cr().modify(|w| w.set_npblb(padding_len as u8));
        [0; 4]
    }

    #[cfg(cryp_v2)]
    fn post_final_blocking<T: Instance, M: Mode>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &Cryp<T, M>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        temp1: [u32; 4],
        padding_mask: [u8; 16],
    ) {
        if dir == Direction::Decrypt {
            //Handle special CCM partial block process.
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
    }

    #[cfg(cryp_v2)]
    async fn post_final<T: Instance>(
        &self,
        p: pac::cryp::Cryp,
        cryp: &mut Cryp<'_, T, Async>,
        dir: Direction,
        int_data: &mut [u8; AES_BLOCK_SIZE],
        temp1: [u32; 4],
        padding_mask: [u8; 16],
    ) {
        if dir == Direction::Decrypt {
            //Handle special CCM partial block process.
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
    }
}

#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const TAG_SIZE: usize, const IV_SIZE: usize> CipherSized for AesCcm<'c, { 128 / 8 }, TAG_SIZE, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const TAG_SIZE: usize, const IV_SIZE: usize> CipherSized for AesCcm<'c, { 192 / 8 }, TAG_SIZE, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const TAG_SIZE: usize, const IV_SIZE: usize> CipherSized for AesCcm<'c, { 256 / 8 }, TAG_SIZE, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<4> for AesCcm<'c, KEY_SIZE, 4, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<6> for AesCcm<'c, KEY_SIZE, 6, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<8> for AesCcm<'c, KEY_SIZE, 8, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<10> for AesCcm<'c, KEY_SIZE, 10, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<12> for AesCcm<'c, KEY_SIZE, 12, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<14> for AesCcm<'c, KEY_SIZE, 14, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const IV_SIZE: usize> CipherAuthenticated<16> for AesCcm<'c, KEY_SIZE, 16, IV_SIZE> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 7> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 8> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 9> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 10> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 11> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
impl<'c, const KEY_SIZE: usize, const TAG_SIZE: usize> IVSized for AesCcm<'c, KEY_SIZE, TAG_SIZE, 12> {}
#[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
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
    cr: u32,
    iv: [u32; 4],
    csgcmccm: [u32; 8],
    csgcm: [u32; 8],
    header_len: u64,
    payload_len: u64,
    aad_buffer: [u8; 16],
    aad_buffer_len: usize,
}

/// Selects whether the crypto processor operates in encryption or decryption mode.
#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    /// Encryption mode
    Encrypt,
    /// Decryption mode
    Decrypt,
}

/// Crypto Accelerator Driver
pub struct Cryp<'d, T: Instance, M: Mode> {
    _peripheral: Peri<'d, T>,
    _phantom: PhantomData<M>,
    indma: Option<ChannelAndRequest<'d>>,
    outdma: Option<ChannelAndRequest<'d>>,
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
            _phantom: PhantomData,
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
            aad_buffer: [0; 16],
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

        ctx.cipher.prepare_key(T::regs());

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
        T::regs().cr().modify(|w| w.fflush());

        ctx.cipher.init_phase_blocking(T::regs(), self);

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
            let header = ctx.cipher.get_header_block();
            ctx.aad_buffer[0..header.len()].copy_from_slice(header);
            ctx.aad_buffer_len += header.len();
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
                self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer);
                // Block until input FIFO is empty.
                while !T::regs().sr().read().ifem() {}

                // Switch to payload phase.
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
                T::regs().cr().modify(|w| w.fflush());
            } else {
                // Just return because we don't yet have a full block to process.
                return;
            }
        } else {
            // Load the full block from the buffer.
            self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer);
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
                self.write_bytes_blocking(C::BLOCK_SIZE, &ctx.aad_buffer);
            }
            // Switch to payload phase.
            ctx.aad_complete = true;
            T::regs().cr().modify(|w| w.set_crypen(false));
            T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
            T::regs().cr().modify(|w| w.fflush());
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
                T::regs().cr().modify(|w| w.fflush());
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
    pub fn finish_blocking<
        'c,
        const TAG_SIZE: usize,
        C: Cipher<'c> + CipherSized + IVSized + CipherAuthenticated<TAG_SIZE>,
    >(
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

        self.write_words_blocking(C::BLOCK_SIZE, &footer);

        while !T::regs().sr().read().ofne() {}

        let mut full_tag: [u8; 16] = [0; 16];
        self.read_bytes_blocking(C::BLOCK_SIZE, &mut full_tag);
        let mut tag: [u8; TAG_SIZE] = [0; TAG_SIZE];
        tag.copy_from_slice(&full_tag[0..TAG_SIZE]);

        T::regs().cr().modify(|w| w.set_crypen(false));

        tag
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
        ctx.cipher.prepare_key(T::regs());
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

impl<'d, T: Instance> Cryp<'d, T, Async> {
    /// Create a new CRYP driver.
    pub fn new(
        peri: Peri<'d, T>,
        indma: Peri<'d, impl DmaIn<T>>,
        outdma: Peri<'d, impl DmaOut<T>>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        let instance = Self {
            _peripheral: peri,
            _phantom: PhantomData,
            indma: new_dma!(indma),
            outdma: new_dma!(outdma),
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
            aad_buffer: [0; 16],
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

        ctx.cipher.prepare_key(T::regs());

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
        T::regs().cr().modify(|w| w.fflush());

        ctx.cipher.init_phase(T::regs(), self).await;

        self.store_context(&mut ctx);

        ctx
    }

    #[cfg(any(cryp_v2, cryp_v3, cryp_v4))]
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
            let header = ctx.cipher.get_header_block();
            ctx.aad_buffer[0..header.len()].copy_from_slice(header);
            ctx.aad_buffer_len += header.len();
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
                Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &ctx.aad_buffer).await;
                assert_eq!(T::regs().sr().read().ifem(), true);

                // Switch to payload phase.
                ctx.aad_complete = true;
                T::regs().cr().modify(|w| w.set_crypen(false));
                T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
                T::regs().cr().modify(|w| w.fflush());
            } else {
                // Just return because we don't yet have a full block to process.
                return;
            }
        } else {
            // Load the full block from the buffer.
            Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &ctx.aad_buffer).await;
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
                Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &ctx.aad_buffer).await;
                assert_eq!(T::regs().sr().read().ifem(), true);
            }
            // Switch to payload phase.
            ctx.aad_complete = true;
            T::regs().cr().modify(|w| w.set_crypen(false));
            T::regs().cr().modify(|w| w.set_gcm_ccmph(2));
            T::regs().cr().modify(|w| w.fflush());
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
                T::regs().cr().modify(|w| w.fflush());
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
            let read = Self::read_bytes(
                self.outdma.as_mut().unwrap(),
                C::BLOCK_SIZE,
                &mut output[index..index + C::BLOCK_SIZE],
            );
            // Write block in
            let write = Self::write_bytes(
                self.indma.as_mut().unwrap(),
                C::BLOCK_SIZE,
                &input[index..index + C::BLOCK_SIZE],
            );
            embassy_futures::join::join(read, write).await;
        }

        // Handle the final block, which is incomplete.
        if last_block_remainder > 0 {
            let padding_len = C::BLOCK_SIZE - last_block_remainder;
            let temp1 = ctx.cipher.pre_final(T::regs(), ctx.dir, padding_len);

            let mut intermediate_data: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            let mut last_block: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
            last_block[..last_block_remainder].copy_from_slice(&input[input.len() - last_block_remainder..input.len()]);
            let read = Self::read_bytes(self.outdma.as_mut().unwrap(), C::BLOCK_SIZE, &mut intermediate_data);
            let write = Self::write_bytes(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &last_block);
            embassy_futures::join::join(read, write).await;

            // Handle the last block depending on mode.
            let output_len = output.len();
            output[output_len - last_block_remainder..output_len]
                .copy_from_slice(&intermediate_data[0..last_block_remainder]);

            let mut mask: [u8; 16] = [0; 16];
            mask[..last_block_remainder].fill(0xFF);
            ctx.cipher
                .post_final(T::regs(), self, ctx.dir, &mut intermediate_data, temp1, mask)
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

        let write = Self::write_words(self.indma.as_mut().unwrap(), C::BLOCK_SIZE, &footer);

        let mut full_tag: [u8; 16] = [0; 16];
        let read = Self::read_bytes(self.outdma.as_mut().unwrap(), C::BLOCK_SIZE, &mut full_tag);

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
        // Configure DMA to transfer input to crypto core.
        let dst_ptr: *mut u32 = T::regs().din().as_ptr();
        let num_words = blocks.len() / 4;
        let src_ptr: *const [u8] = ptr::slice_from_raw_parts(blocks.as_ptr().cast(), num_words);
        let options = TransferOptions {
            #[cfg(not(gpdma))]
            priority: crate::dma::Priority::High,
            ..Default::default()
        };
        let dma_transfer = unsafe { dma.write_raw(src_ptr, dst_ptr, options) };
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
        let num_words = blocks.len();
        let src_ptr: *const [u32] = ptr::slice_from_raw_parts(blocks.as_ptr().cast(), num_words);
        let options = TransferOptions {
            #[cfg(not(gpdma))]
            priority: crate::dma::Priority::High,
            ..Default::default()
        };
        let dma_transfer = unsafe { dma.write_raw(src_ptr, dst_ptr, options) };
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
        // Configure DMA to get output from crypto core.
        let src_ptr = T::regs().dout().as_ptr();
        let num_words = blocks.len() / 4;
        let dst_ptr = ptr::slice_from_raw_parts_mut(blocks.as_mut_ptr().cast(), num_words);
        let options = TransferOptions {
            #[cfg(not(gpdma))]
            priority: crate::dma::Priority::VeryHigh,
            ..Default::default()
        };
        let dma_transfer = unsafe { dma.read_raw(src_ptr, dst_ptr, options) };
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
