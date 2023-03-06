#![feature(type_alias_impl_trait)]
#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
mod fmt;

use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};
use embedded_storage_async::nor_flash::NorFlash as AsyncNorFlash;

const BOOT_MAGIC: u8 = 0xD0;
const SWAP_MAGIC: u8 = 0xF0;

/// A region in flash used by the bootloader.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Partition {
    /// Start of the flash region.
    pub from: usize,
    /// End of the flash region.
    pub to: usize,
}

impl Partition {
    /// Create a new partition with the provided range
    pub const fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }

    /// Return the length of the partition
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.to - self.from
    }
}

/// The state of the bootloader after running prepare.
#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum State {
    /// Bootloader is ready to boot the active partition.
    Boot,
    /// Bootloader has swapped the active partition with the dfu partition and will attempt boot.
    Swap,
}

/// Errors returned by bootloader
#[derive(PartialEq, Eq, Debug)]
pub enum BootError {
    /// Error from flash.
    Flash(NorFlashErrorKind),
    /// Invalid bootloader magic
    BadMagic,
}

#[cfg(feature = "defmt")]
impl defmt::Format for BootError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            BootError::Flash(_) => defmt::write!(fmt, "BootError::Flash(_)"),
            BootError::BadMagic => defmt::write!(fmt, "BootError::BadMagic"),
        }
    }
}

impl<E> From<E> for BootError
where
    E: NorFlashError,
{
    fn from(error: E) -> Self {
        BootError::Flash(error.kind())
    }
}

/// Buffer aligned to 32 byte boundary, largest known alignment requirement for embassy-boot.
#[repr(align(32))]
pub struct AlignedBuffer<const N: usize>(pub [u8; N]);

impl<const N: usize> AsRef<[u8]> for AlignedBuffer<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsMut<[u8]> for AlignedBuffer<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Extension of the embedded-storage flash type information with block size and erase value.
pub trait Flash: NorFlash + ReadNorFlash {
    /// The block size that should be used when writing to flash. For most builtin flashes, this is the same as the erase
    /// size of the flash, but for external QSPI flash modules, this can be lower.
    const BLOCK_SIZE: usize;
    /// The erase value of the flash. Typically the default of 0xFF is used, but some flashes use a different value.
    const ERASE_VALUE: u8 = 0xFF;
}

/// Trait defining the flash handles used for active and DFU partition
pub trait FlashConfig {
    /// Flash type used for the state partition.
    type STATE: Flash;
    /// Flash type used for the active partition.
    type ACTIVE: Flash;
    /// Flash type used for the dfu partition.
    type DFU: Flash;

    /// Return flash instance used to write/read to/from active partition.
    fn active(&mut self) -> &mut Self::ACTIVE;
    /// Return flash instance used to write/read to/from dfu partition.
    fn dfu(&mut self) -> &mut Self::DFU;
    /// Return flash instance used to write/read to/from bootloader state.
    fn state(&mut self) -> &mut Self::STATE;
}

/// BootLoader works with any flash implementing embedded_storage and can also work with
/// different page sizes and flash write sizes.
pub struct BootLoader {
    // Page with current state of bootloader. The state partition has the following format:
    // | Range          | Description                                                                      |
    // | 0 - WRITE_SIZE | Magic indicating bootloader state. BOOT_MAGIC means boot, SWAP_MAGIC means swap. |
    // | WRITE_SIZE - N | Progress index used while swapping or reverting                                  |
    state: Partition,
    // Location of the partition which will be booted from
    active: Partition,
    // Location of the partition which will be swapped in when requested
    dfu: Partition,
}

impl BootLoader {
    /// Create a new instance of a bootloader with the given partitions.
    ///
    /// - All partitions must be aligned with the PAGE_SIZE const generic parameter.
    /// - The dfu partition must be at least PAGE_SIZE bigger than the active partition.
    pub fn new(active: Partition, dfu: Partition, state: Partition) -> Self {
        Self { active, dfu, state }
    }

    /// Return the boot address for the active partition.
    pub fn boot_address(&self) -> usize {
        self.active.from
    }

    /// Perform necessary boot preparations like swapping images.
    ///
    /// The DFU partition is assumed to be 1 page bigger than the active partition for the swap
    /// algorithm to work correctly.
    ///
    /// SWAPPING
    ///
    /// Assume a flash size of 3 pages for the active partition, and 4 pages for the DFU partition.
    /// The swap index contains the copy progress, as to allow continuation of the copy process on
    /// power failure. The index counter is represented within 1 or more pages (depending on total
    /// flash size), where a page X is considered swapped if index at location (X + WRITE_SIZE)
    /// contains a zero value. This ensures that index updates can be performed atomically and
    /// avoid a situation where the wrong index value is set (page write size is "atomic").
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          0 |      1 |      2 |      3 |      - |
    /// |       DFU |          0 |      3 |      2 |      1 |      X |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// The algorithm starts by copying 'backwards', and after the first step, the layout is
    /// as follows:
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          1 |      1 |      2 |      1 |      - |
    /// |       DFU |          1 |      3 |      2 |      1 |      3 |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// The next iteration performs the same steps
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          2 |      1 |      2 |      1 |      - |
    /// |       DFU |          2 |      3 |      2 |      2 |      3 |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// And again until we're done
    ///
    /// +-----------+------------+--------+--------+--------+--------+
    /// | Partition | Swap Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+------------+--------+--------+--------+--------+
    /// |    Active |          3 |      3 |      2 |      1 |      - |
    /// |       DFU |          3 |      3 |      1 |      2 |      3 |
    /// +-----------+------------+--------+--------+--------+--------+
    ///
    /// REVERTING
    ///
    /// The reverting algorithm uses the swap index to discover that images were swapped, but that
    /// the application failed to mark the boot successful. In this case, the revert algorithm will
    /// run.
    ///
    /// The revert index is located separately from the swap index, to ensure that revert can continue
    /// on power failure.
    ///
    /// The revert algorithm works forwards, by starting copying into the 'unused' DFU page at the start.
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    //*/
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      1 |      2 |      1 |      - |
    /// |       DFU |            3 |      3 |      1 |      2 |      3 |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      1 |      2 |      1 |      - |
    /// |       DFU |            3 |      3 |      2 |      2 |      3 |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    /// +-----------+--------------+--------+--------+--------+--------+
    /// | Partition | Revert Index | Page 0 | Page 1 | Page 3 | Page 4 |
    /// +-----------+--------------+--------+--------+--------+--------+
    /// |    Active |            3 |      1 |      2 |      3 |      - |
    /// |       DFU |            3 |      3 |      2 |      1 |      3 |
    /// +-----------+--------------+--------+--------+--------+--------+
    ///
    pub fn prepare_boot<P: FlashConfig>(
        &mut self,
        p: &mut P,
        magic: &mut [u8],
        page: &mut [u8],
    ) -> Result<State, BootError> {
        // Ensure we have enough progress pages to store copy progress
        assert_partitions(self.active, self.dfu, self.state, page.len(), P::STATE::WRITE_SIZE);
        assert_eq!(magic.len(), P::STATE::WRITE_SIZE);

        // Copy contents from partition N to active
        let state = self.read_state(p, magic)?;
        if state == State::Swap {
            //
            // Check if we already swapped. If we're in the swap state, this means we should revert
            // since the app has failed to mark boot as successful
            //
            if !self.is_swapped(p, magic, page)? {
                trace!("Swapping");
                self.swap(p, magic, page)?;
                trace!("Swapping done");
            } else {
                trace!("Reverting");
                self.revert(p, magic, page)?;

                // Overwrite magic and reset progress
                let fstate = p.state();
                magic.fill(!P::STATE::ERASE_VALUE);
                fstate.write(self.state.from as u32, magic)?;
                fstate.erase(self.state.from as u32, self.state.to as u32)?;

                magic.fill(BOOT_MAGIC);
                fstate.write(self.state.from as u32, magic)?;
            }
        }
        Ok(state)
    }

    fn is_swapped<P: FlashConfig>(&mut self, p: &mut P, magic: &mut [u8], page: &mut [u8]) -> Result<bool, BootError> {
        let page_size = page.len();
        let page_count = self.active.len() / page_size;
        let progress = self.current_progress(p, magic)?;

        Ok(progress >= page_count * 2)
    }

    fn current_progress<P: FlashConfig>(&mut self, config: &mut P, aligned: &mut [u8]) -> Result<usize, BootError> {
        let write_size = aligned.len();
        let max_index = ((self.state.len() - write_size) / write_size) - 1;
        aligned.fill(!P::STATE::ERASE_VALUE);

        let flash = config.state();
        for i in 0..max_index {
            flash.read((self.state.from + write_size + i * write_size) as u32, aligned)?;

            if aligned.iter().any(|&b| b == P::STATE::ERASE_VALUE) {
                return Ok(i);
            }
        }
        Ok(max_index)
    }

    fn update_progress<P: FlashConfig>(&mut self, idx: usize, p: &mut P, magic: &mut [u8]) -> Result<(), BootError> {
        let flash = p.state();
        let write_size = magic.len();
        let w = self.state.from + write_size + idx * write_size;

        let aligned = magic;
        aligned.fill(!P::STATE::ERASE_VALUE);
        flash.write(w as u32, aligned)?;
        Ok(())
    }

    fn active_addr(&self, n: usize, page_size: usize) -> usize {
        self.active.from + n * page_size
    }

    fn dfu_addr(&self, n: usize, page_size: usize) -> usize {
        self.dfu.from + n * page_size
    }

    fn copy_page_once_to_active<P: FlashConfig>(
        &mut self,
        idx: usize,
        from_page: usize,
        to_page: usize,
        p: &mut P,
        magic: &mut [u8],
        page: &mut [u8],
    ) -> Result<(), BootError> {
        let buf = page;
        if self.current_progress(p, magic)? <= idx {
            let mut offset = from_page;
            for chunk in buf.chunks_mut(P::DFU::BLOCK_SIZE) {
                p.dfu().read(offset as u32, chunk)?;
                offset += chunk.len();
            }

            p.active().erase(to_page as u32, (to_page + buf.len()) as u32)?;

            let mut offset = to_page;
            for chunk in buf.chunks(P::ACTIVE::BLOCK_SIZE) {
                p.active().write(offset as u32, chunk)?;
                offset += chunk.len();
            }
            self.update_progress(idx, p, magic)?;
        }
        Ok(())
    }

    fn copy_page_once_to_dfu<P: FlashConfig>(
        &mut self,
        idx: usize,
        from_page: usize,
        to_page: usize,
        p: &mut P,
        magic: &mut [u8],
        page: &mut [u8],
    ) -> Result<(), BootError> {
        let buf = page;
        if self.current_progress(p, magic)? <= idx {
            let mut offset = from_page;
            for chunk in buf.chunks_mut(P::ACTIVE::BLOCK_SIZE) {
                p.active().read(offset as u32, chunk)?;
                offset += chunk.len();
            }

            p.dfu().erase(to_page as u32, (to_page + buf.len()) as u32)?;

            let mut offset = to_page;
            for chunk in buf.chunks(P::DFU::BLOCK_SIZE) {
                p.dfu().write(offset as u32, chunk)?;
                offset += chunk.len();
            }
            self.update_progress(idx, p, magic)?;
        }
        Ok(())
    }

    fn swap<P: FlashConfig>(&mut self, p: &mut P, magic: &mut [u8], page: &mut [u8]) -> Result<(), BootError> {
        let page_size = page.len();
        let page_count = self.active.len() / page_size;
        trace!("Page count: {}", page_count);
        for page_num in 0..page_count {
            trace!("COPY PAGE {}", page_num);
            // Copy active page to the 'next' DFU page.
            let active_page = self.active_addr(page_count - 1 - page_num, page_size);
            let dfu_page = self.dfu_addr(page_count - page_num, page_size);
            //trace!("Copy active {} to dfu {}", active_page, dfu_page);
            self.copy_page_once_to_dfu(page_num * 2, active_page, dfu_page, p, magic, page)?;

            // Copy DFU page to the active page
            let active_page = self.active_addr(page_count - 1 - page_num, page_size);
            let dfu_page = self.dfu_addr(page_count - 1 - page_num, page_size);
            //trace!("Copy dfy {} to active {}", dfu_page, active_page);
            self.copy_page_once_to_active(page_num * 2 + 1, dfu_page, active_page, p, magic, page)?;
        }

        Ok(())
    }

    fn revert<P: FlashConfig>(&mut self, p: &mut P, magic: &mut [u8], page: &mut [u8]) -> Result<(), BootError> {
        let page_size = page.len();
        let page_count = self.active.len() / page_size;
        for page_num in 0..page_count {
            // Copy the bad active page to the DFU page
            let active_page = self.active_addr(page_num, page_size);
            let dfu_page = self.dfu_addr(page_num, page_size);
            self.copy_page_once_to_dfu(page_count * 2 + page_num * 2, active_page, dfu_page, p, magic, page)?;

            // Copy the DFU page back to the active page
            let active_page = self.active_addr(page_num, page_size);
            let dfu_page = self.dfu_addr(page_num + 1, page_size);
            self.copy_page_once_to_active(page_count * 2 + page_num * 2 + 1, dfu_page, active_page, p, magic, page)?;
        }

        Ok(())
    }

    fn read_state<P: FlashConfig>(&mut self, config: &mut P, magic: &mut [u8]) -> Result<State, BootError> {
        let flash = config.state();
        flash.read(self.state.from as u32, magic)?;

        if !magic.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }
}

fn assert_partitions(active: Partition, dfu: Partition, state: Partition, page_size: usize, write_size: usize) {
    assert_eq!(active.len() % page_size, 0);
    assert_eq!(dfu.len() % page_size, 0);
    assert!(dfu.len() - active.len() >= page_size);
    assert!(2 * (active.len() / page_size) <= (state.len() - write_size) / write_size);
}

/// Convenience provider that uses a single flash for all partitions.
pub struct SingleFlashConfig<'a, F>
where
    F: Flash,
{
    flash: &'a mut F,
}

impl<'a, F> SingleFlashConfig<'a, F>
where
    F: Flash,
{
    /// Create a provider for a single flash.
    pub fn new(flash: &'a mut F) -> Self {
        Self { flash }
    }
}

impl<'a, F> FlashConfig for SingleFlashConfig<'a, F>
where
    F: Flash,
{
    type STATE = F;
    type ACTIVE = F;
    type DFU = F;

    fn active(&mut self) -> &mut Self::STATE {
        self.flash
    }
    fn dfu(&mut self) -> &mut Self::ACTIVE {
        self.flash
    }
    fn state(&mut self) -> &mut Self::DFU {
        self.flash
    }
}

/// A flash wrapper implementing the Flash and embedded_storage traits.
pub struct BootFlash<F, const BLOCK_SIZE: usize, const ERASE_VALUE: u8 = 0xFF>
where
    F: NorFlash + ReadNorFlash,
{
    flash: F,
}

impl<F, const BLOCK_SIZE: usize, const ERASE_VALUE: u8> BootFlash<F, BLOCK_SIZE, ERASE_VALUE>
where
    F: NorFlash + ReadNorFlash,
{
    /// Create a new instance of a bootable flash
    pub fn new(flash: F) -> Self {
        Self { flash }
    }
}

impl<F, const BLOCK_SIZE: usize, const ERASE_VALUE: u8> Flash for BootFlash<F, BLOCK_SIZE, ERASE_VALUE>
where
    F: NorFlash + ReadNorFlash,
{
    const BLOCK_SIZE: usize = BLOCK_SIZE;
    const ERASE_VALUE: u8 = ERASE_VALUE;
}

impl<F, const BLOCK_SIZE: usize, const ERASE_VALUE: u8> ErrorType for BootFlash<F, BLOCK_SIZE, ERASE_VALUE>
where
    F: ReadNorFlash + NorFlash,
{
    type Error = F::Error;
}

impl<F, const BLOCK_SIZE: usize, const ERASE_VALUE: u8> NorFlash for BootFlash<F, BLOCK_SIZE, ERASE_VALUE>
where
    F: ReadNorFlash + NorFlash,
{
    const WRITE_SIZE: usize = F::WRITE_SIZE;
    const ERASE_SIZE: usize = F::ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        F::erase(&mut self.flash, from, to)
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        F::write(&mut self.flash, offset, bytes)
    }
}

impl<F, const BLOCK_SIZE: usize, const ERASE_VALUE: u8> ReadNorFlash for BootFlash<F, BLOCK_SIZE, ERASE_VALUE>
where
    F: ReadNorFlash + NorFlash,
{
    const READ_SIZE: usize = F::READ_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        F::read(&mut self.flash, offset, bytes)
    }

    fn capacity(&self) -> usize {
        F::capacity(&self.flash)
    }
}

/// Convenience flash provider that uses separate flash instances for each partition.
pub struct MultiFlashConfig<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: Flash,
    STATE: Flash,
    DFU: Flash,
{
    active: &'a mut ACTIVE,
    state: &'a mut STATE,
    dfu: &'a mut DFU,
}

impl<'a, ACTIVE, STATE, DFU> MultiFlashConfig<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: Flash,
    STATE: Flash,
    DFU: Flash,
{
    /// Create a new flash provider with separate configuration for all three partitions.
    pub fn new(active: &'a mut ACTIVE, state: &'a mut STATE, dfu: &'a mut DFU) -> Self {
        Self { active, state, dfu }
    }
}

impl<'a, ACTIVE, STATE, DFU> FlashConfig for MultiFlashConfig<'a, ACTIVE, STATE, DFU>
where
    ACTIVE: Flash,
    STATE: Flash,
    DFU: Flash,
{
    type STATE = STATE;
    type ACTIVE = ACTIVE;
    type DFU = DFU;

    fn active(&mut self) -> &mut Self::ACTIVE {
        self.active
    }
    fn dfu(&mut self) -> &mut Self::DFU {
        self.dfu
    }
    fn state(&mut self) -> &mut Self::STATE {
        self.state
    }
}
/// Errors returned by FirmwareUpdater
#[derive(Debug)]
pub enum FirmwareUpdaterError {
    /// Error from flash.
    Flash(NorFlashErrorKind),
    /// Signature errors.
    Signature(signature::Error),
}

#[cfg(feature = "defmt")]
impl defmt::Format for FirmwareUpdaterError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            FirmwareUpdaterError::Flash(_) => defmt::write!(fmt, "FirmwareUpdaterError::Flash(_)"),
            FirmwareUpdaterError::Signature(_) => defmt::write!(fmt, "FirmwareUpdaterError::Signature(_)"),
        }
    }
}

impl<E> From<E> for FirmwareUpdaterError
where
    E: NorFlashError,
{
    fn from(error: E) -> Self {
        FirmwareUpdaterError::Flash(error.kind())
    }
}

/// FirmwareUpdater is an application API for interacting with the BootLoader without the ability to
/// 'mess up' the internal bootloader state
pub struct FirmwareUpdater {
    state: Partition,
    dfu: Partition,
}

impl Default for FirmwareUpdater {
    fn default() -> Self {
        extern "C" {
            static __bootloader_state_start: u32;
            static __bootloader_state_end: u32;
            static __bootloader_dfu_start: u32;
            static __bootloader_dfu_end: u32;
        }

        let dfu = unsafe {
            Partition::new(
                &__bootloader_dfu_start as *const u32 as usize,
                &__bootloader_dfu_end as *const u32 as usize,
            )
        };
        let state = unsafe {
            Partition::new(
                &__bootloader_state_start as *const u32 as usize,
                &__bootloader_state_end as *const u32 as usize,
            )
        };

        trace!("DFU: 0x{:x} - 0x{:x}", dfu.from, dfu.to);
        trace!("STATE: 0x{:x} - 0x{:x}", state.from, state.to);
        FirmwareUpdater::new(dfu, state)
    }
}

impl FirmwareUpdater {
    /// Create a firmware updater instance with partition ranges for the update and state partitions.
    pub const fn new(dfu: Partition, state: Partition) -> Self {
        Self { dfu, state }
    }

    /// Return the length of the DFU area
    pub fn firmware_len(&self) -> usize {
        self.dfu.len()
    }

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    pub async fn get_state<F: AsyncNorFlash>(
        &mut self,
        flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<State, FirmwareUpdaterError> {
        flash.read(self.state.from as u32, aligned).await?;

        if !aligned.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }

    /// Verify the DFU given a public key. If there is an error then DO NOT
    /// proceed with updating the firmware as it must be signed with a
    /// corresponding private key (otherwise it could be malicious firmware).
    ///
    /// Mark to trigger firmware swap on next boot if verify suceeds.
    ///
    /// If the "ed25519-salty" feature is set (or another similar feature) then the signature is expected to have
    /// been generated from a SHA-512 digest of the firmware bytes.
    ///
    /// If no signature feature is set then this method will always return a
    /// signature error.
    ///
    /// # Safety
    ///
    /// The `_aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    #[cfg(feature = "_verify")]
    pub async fn verify_and_mark_updated<F: AsyncNorFlash>(
        &mut self,
        _flash: &mut F,
        _public_key: &[u8],
        _signature: &[u8],
        _update_len: usize,
        _aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        let _end = self.dfu.from + _update_len;
        let _read_size = _aligned.len();

        assert_eq!(_aligned.len(), F::WRITE_SIZE);
        assert!(_end <= self.dfu.to);

        #[cfg(feature = "ed25519-dalek")]
        {
            use ed25519_dalek::{Digest, PublicKey, Sha512, Signature, SignatureError, Verifier};

            let into_signature_error = |e: SignatureError| FirmwareUpdaterError::Signature(e.into());

            let public_key = PublicKey::from_bytes(_public_key).map_err(into_signature_error)?;
            let signature = Signature::from_bytes(_signature).map_err(into_signature_error)?;

            let mut digest = Sha512::new();

            let mut offset = self.dfu.from;
            let last_offset = _end / _read_size * _read_size;

            while offset < last_offset {
                _flash.read(offset as u32, _aligned).await?;
                digest.update(&_aligned);
                offset += _read_size;
            }

            let remaining = _end % _read_size;

            if remaining > 0 {
                _flash.read(last_offset as u32, _aligned).await?;
                digest.update(&_aligned[0..remaining]);
            }

            public_key
                .verify(&digest.finalize(), &signature)
                .map_err(into_signature_error)?
        }
        #[cfg(feature = "ed25519-salty")]
        {
            use salty::constants::{PUBLICKEY_SERIALIZED_LENGTH, SIGNATURE_SERIALIZED_LENGTH};
            use salty::{PublicKey, Sha512, Signature};

            fn into_signature_error<E>(_: E) -> FirmwareUpdaterError {
                FirmwareUpdaterError::Signature(signature::Error::default())
            }

            let public_key: [u8; PUBLICKEY_SERIALIZED_LENGTH] = _public_key.try_into().map_err(into_signature_error)?;
            let public_key = PublicKey::try_from(&public_key).map_err(into_signature_error)?;
            let signature: [u8; SIGNATURE_SERIALIZED_LENGTH] = _signature.try_into().map_err(into_signature_error)?;
            let signature = Signature::try_from(&signature).map_err(into_signature_error)?;

            let mut digest = Sha512::new();

            let mut offset = self.dfu.from;
            let last_offset = _end / _read_size * _read_size;

            while offset < last_offset {
                _flash.read(offset as u32, _aligned).await?;
                digest.update(&_aligned);
                offset += _read_size;
            }

            let remaining = _end % _read_size;

            if remaining > 0 {
                _flash.read(last_offset as u32, _aligned).await?;
                digest.update(&_aligned[0..remaining]);
            }

            let message = digest.finalize();
            let r = public_key.verify(&message, &signature);
            trace!(
                "Verifying with public key {}, signature {} and message {} yields ok: {}",
                public_key.to_bytes(),
                signature.to_bytes(),
                message,
                r.is_ok()
            );
            r.map_err(into_signature_error)?
        }

        self.set_magic(_aligned, SWAP_MAGIC, _flash).await
    }

    /// Mark to trigger firmware swap on next boot.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    #[cfg(not(feature = "_verify"))]
    pub async fn mark_updated<F: AsyncNorFlash>(
        &mut self,
        flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic(aligned, SWAP_MAGIC, flash).await
    }

    /// Mark firmware boot successful and stop rollback on reset.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    pub async fn mark_booted<F: AsyncNorFlash>(
        &mut self,
        flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic(aligned, BOOT_MAGIC, flash).await
    }

    async fn set_magic<F: AsyncNorFlash>(
        &mut self,
        aligned: &mut [u8],
        magic: u8,
        flash: &mut F,
    ) -> Result<(), FirmwareUpdaterError> {
        flash.read(self.state.from as u32, aligned).await?;

        if aligned.iter().any(|&b| b != magic) {
            aligned.fill(0);

            flash.write(self.state.from as u32, aligned).await?;
            flash.erase(self.state.from as u32, self.state.to as u32).await?;

            aligned.fill(magic);
            flash.write(self.state.from as u32, aligned).await?;
        }
        Ok(())
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub async fn write_firmware<F: AsyncNorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), FirmwareUpdaterError> {
        assert!(data.len() >= F::ERASE_SIZE);

        flash
            .erase(
                (self.dfu.from + offset) as u32,
                (self.dfu.from + offset + data.len()) as u32,
            )
            .await?;

        trace!(
            "Erased from {} to {}",
            self.dfu.from + offset,
            self.dfu.from + offset + data.len()
        );

        FirmwareWriter(self.dfu)
            .write_block(offset, data, flash, block_size)
            .await?;

        Ok(())
    }

    /// Prepare for an incoming DFU update by erasing the entire DFU area and
    /// returning a `FirmwareWriter`.
    ///
    /// Using this instead of `write_firmware` allows for an optimized API in
    /// exchange for added complexity.
    pub async fn prepare_update<F: AsyncNorFlash>(
        &mut self,
        flash: &mut F,
    ) -> Result<FirmwareWriter, FirmwareUpdaterError> {
        flash.erase((self.dfu.from) as u32, (self.dfu.to) as u32).await?;

        trace!("Erased from {} to {}", self.dfu.from, self.dfu.to);

        Ok(FirmwareWriter(self.dfu))
    }

    //
    // Blocking API
    //

    /// Obtain the current state.
    ///
    /// This is useful to check if the bootloader has just done a swap, in order
    /// to do verifications and self-tests of the new image before calling
    /// `mark_booted`.
    pub fn get_state_blocking<F: NorFlash>(
        &mut self,
        flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<State, FirmwareUpdaterError> {
        flash.read(self.state.from as u32, aligned)?;

        if !aligned.iter().any(|&b| b != SWAP_MAGIC) {
            Ok(State::Swap)
        } else {
            Ok(State::Boot)
        }
    }

    /// Verify the DFU given a public key. If there is an error then DO NOT
    /// proceed with updating the firmware as it must be signed with a
    /// corresponding private key (otherwise it could be malicious firmware).
    ///
    /// Mark to trigger firmware swap on next boot if verify suceeds.
    ///
    /// If the "ed25519-salty" feature is set (or another similar feature) then the signature is expected to have
    /// been generated from a SHA-512 digest of the firmware bytes.
    ///
    /// If no signature feature is set then this method will always return a
    /// signature error.
    ///
    /// # Safety
    ///
    /// The `_aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being read from
    /// and written to.
    #[cfg(feature = "_verify")]
    pub fn verify_and_mark_updated_blocking<F: NorFlash>(
        &mut self,
        _flash: &mut F,
        _public_key: &[u8],
        _signature: &[u8],
        _update_len: usize,
        _aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        let _end = self.dfu.from + _update_len;
        let _read_size = _aligned.len();

        assert_eq!(_aligned.len(), F::WRITE_SIZE);
        assert!(_end <= self.dfu.to);

        #[cfg(feature = "ed25519-dalek")]
        {
            use ed25519_dalek::{Digest, PublicKey, Sha512, Signature, SignatureError, Verifier};

            let into_signature_error = |e: SignatureError| FirmwareUpdaterError::Signature(e.into());

            let public_key = PublicKey::from_bytes(_public_key).map_err(into_signature_error)?;
            let signature = Signature::from_bytes(_signature).map_err(into_signature_error)?;

            let mut digest = Sha512::new();

            let mut offset = self.dfu.from;
            let last_offset = _end / _read_size * _read_size;

            while offset < last_offset {
                _flash.read(offset as u32, _aligned)?;
                digest.update(&_aligned);
                offset += _read_size;
            }

            let remaining = _end % _read_size;

            if remaining > 0 {
                _flash.read(last_offset as u32, _aligned)?;
                digest.update(&_aligned[0..remaining]);
            }

            public_key
                .verify(&digest.finalize(), &signature)
                .map_err(into_signature_error)?
        }
        #[cfg(feature = "ed25519-salty")]
        {
            use salty::constants::{PUBLICKEY_SERIALIZED_LENGTH, SIGNATURE_SERIALIZED_LENGTH};
            use salty::{PublicKey, Sha512, Signature};

            fn into_signature_error<E>(_: E) -> FirmwareUpdaterError {
                FirmwareUpdaterError::Signature(signature::Error::default())
            }

            let public_key: [u8; PUBLICKEY_SERIALIZED_LENGTH] = _public_key.try_into().map_err(into_signature_error)?;
            let public_key = PublicKey::try_from(&public_key).map_err(into_signature_error)?;
            let signature: [u8; SIGNATURE_SERIALIZED_LENGTH] = _signature.try_into().map_err(into_signature_error)?;
            let signature = Signature::try_from(&signature).map_err(into_signature_error)?;

            let mut digest = Sha512::new();

            let mut offset = self.dfu.from;
            let last_offset = _end / _read_size * _read_size;

            while offset < last_offset {
                _flash.read(offset as u32, _aligned)?;
                digest.update(&_aligned);
                offset += _read_size;
            }

            let remaining = _end % _read_size;

            if remaining > 0 {
                _flash.read(last_offset as u32, _aligned)?;
                digest.update(&_aligned[0..remaining]);
            }

            let message = digest.finalize();
            let r = public_key.verify(&message, &signature);
            trace!(
                "Verifying with public key {}, signature {} and message {} yields ok: {}",
                public_key.to_bytes(),
                signature.to_bytes(),
                message,
                r.is_ok()
            );
            r.map_err(into_signature_error)?
        }

        self.set_magic_blocking(_aligned, SWAP_MAGIC, _flash)
    }

    /// Mark to trigger firmware swap on next boot.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    #[cfg(not(feature = "_verify"))]
    pub fn mark_updated_blocking<F: NorFlash>(
        &mut self,
        flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic_blocking(aligned, SWAP_MAGIC, flash)
    }

    /// Mark firmware boot successful and stop rollback on reset.
    ///
    /// # Safety
    ///
    /// The `aligned` buffer must have a size of F::WRITE_SIZE, and follow the alignment rules for the flash being written to.
    pub fn mark_booted_blocking<F: NorFlash>(
        &mut self,
        flash: &mut F,
        aligned: &mut [u8],
    ) -> Result<(), FirmwareUpdaterError> {
        assert_eq!(aligned.len(), F::WRITE_SIZE);
        self.set_magic_blocking(aligned, BOOT_MAGIC, flash)
    }

    fn set_magic_blocking<F: NorFlash>(
        &mut self,
        aligned: &mut [u8],
        magic: u8,
        flash: &mut F,
    ) -> Result<(), FirmwareUpdaterError> {
        flash.read(self.state.from as u32, aligned)?;

        if aligned.iter().any(|&b| b != magic) {
            aligned.fill(0);

            flash.write(self.state.from as u32, aligned)?;
            flash.erase(self.state.from as u32, self.state.to as u32)?;

            aligned.fill(magic);
            flash.write(self.state.from as u32, aligned)?;
        }
        Ok(())
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub fn write_firmware_blocking<F: NorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), FirmwareUpdaterError> {
        assert!(data.len() >= F::ERASE_SIZE);

        flash.erase(
            (self.dfu.from + offset) as u32,
            (self.dfu.from + offset + data.len()) as u32,
        )?;

        trace!(
            "Erased from {} to {}",
            self.dfu.from + offset,
            self.dfu.from + offset + data.len()
        );

        FirmwareWriter(self.dfu).write_block_blocking(offset, data, flash, block_size)?;

        Ok(())
    }

    /// Prepare for an incoming DFU update by erasing the entire DFU area and
    /// returning a `FirmwareWriter`.
    ///
    /// Using this instead of `write_firmware_blocking` allows for an optimized
    /// API in exchange for added complexity.
    pub fn prepare_update_blocking<F: NorFlash>(
        &mut self,
        flash: &mut F,
    ) -> Result<FirmwareWriter, FirmwareUpdaterError> {
        flash.erase((self.dfu.from) as u32, (self.dfu.to) as u32)?;

        trace!("Erased from {} to {}", self.dfu.from, self.dfu.to);

        Ok(FirmwareWriter(self.dfu))
    }
}

/// FirmwareWriter allows writing blocks to an already erased flash.
pub struct FirmwareWriter(Partition);

impl FirmwareWriter {
    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub async fn write_block<F: AsyncNorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), F::Error> {
        trace!(
            "Writing firmware at offset 0x{:x} len {}",
            self.0.from + offset,
            data.len()
        );

        let mut write_offset = self.0.from + offset;
        for chunk in data.chunks(block_size) {
            trace!("Wrote chunk at {}: {:?}", write_offset, chunk);
            flash.write(write_offset as u32, chunk).await?;
            write_offset += chunk.len();
        }
        /*
        trace!("Wrote data, reading back for verification");

        let mut buf: [u8; 4096] = [0; 4096];
        let mut data_offset = 0;
        let mut read_offset = self.dfu.from + offset;
        for chunk in buf.chunks_mut(block_size) {
            flash.read(read_offset as u32, chunk).await?;
            trace!("Read chunk at {}: {:?}", read_offset, chunk);
            assert_eq!(&data[data_offset..data_offset + block_size], chunk);
            read_offset += chunk.len();
            data_offset += chunk.len();
        }
        */

        Ok(())
    }

    /// Write data to a flash page.
    ///
    /// The buffer must follow alignment requirements of the target flash and a multiple of page size big.
    ///
    /// # Safety
    ///
    /// Failing to meet alignment and size requirements may result in a panic.
    pub fn write_block_blocking<F: NorFlash>(
        &mut self,
        offset: usize,
        data: &[u8],
        flash: &mut F,
        block_size: usize,
    ) -> Result<(), F::Error> {
        trace!(
            "Writing firmware at offset 0x{:x} len {}",
            self.0.from + offset,
            data.len()
        );

        let mut write_offset = self.0.from + offset;
        for chunk in data.chunks(block_size) {
            trace!("Wrote chunk at {}: {:?}", write_offset, chunk);
            flash.write(write_offset as u32, chunk)?;
            write_offset += chunk.len();
        }
        /*
        trace!("Wrote data, reading back for verification");

        let mut buf: [u8; 4096] = [0; 4096];
        let mut data_offset = 0;
        let mut read_offset = self.dfu.from + offset;
        for chunk in buf.chunks_mut(block_size) {
            flash.read(read_offset as u32, chunk).await?;
            trace!("Read chunk at {}: {:?}", read_offset, chunk);
            assert_eq!(&data[data_offset..data_offset + block_size], chunk);
            read_offset += chunk.len();
            data_offset += chunk.len();
        }
        */

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;
    use core::future::Future;

    use embedded_storage::nor_flash::ErrorType;
    use embedded_storage_async::nor_flash::ReadNorFlash as AsyncReadNorFlash;
    use futures::executor::block_on;

    use super::*;

    /*
    #[test]
    fn test_bad_magic() {
        let mut flash = MemFlash([0xff; 131072]);
        let mut flash = SingleFlashConfig::new(&mut flash);

        let mut bootloader = BootLoader::<4096>::new(ACTIVE, DFU, STATE);

        assert_eq!(
            bootloader.prepare_boot(&mut flash),
            Err(BootError::BadMagic)
        );
    }
    */

    #[test]
    fn test_boot_state() {
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);

        let mut flash = MemFlash::<131072, 4096, 4>([0xff; 131072]);
        flash.0[0..4].copy_from_slice(&[BOOT_MAGIC; 4]);
        let mut flash = SingleFlashConfig::new(&mut flash);

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);

        let mut magic = [0; 4];
        let mut page = [0; 4096];
        assert_eq!(
            State::Boot,
            bootloader.prepare_boot(&mut flash, &mut magic, &mut page).unwrap()
        );
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_swap_state() {
        const STATE: Partition = Partition::new(0, 4096);
        const ACTIVE: Partition = Partition::new(4096, 61440);
        const DFU: Partition = Partition::new(61440, 122880);
        let mut flash = MemFlash::<131072, 4096, 4>([0xff; 131072]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];
        let mut aligned = [0; 4];

        for i in ACTIVE.from..ACTIVE.to {
            flash.0[i] = original[i - ACTIVE.from];
        }

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut updater = FirmwareUpdater::new(DFU, STATE);
        let mut offset = 0;
        for chunk in update.chunks(4096) {
            block_on(updater.write_firmware(offset, chunk, &mut flash, 4096)).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut flash, &mut aligned)).unwrap();

        let mut magic = [0; 4];
        let mut page = [0; 4096];
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::new(&mut flash), &mut magic, &mut page)
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(flash.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }

        // Running again should cause a revert
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::new(&mut flash), &mut magic, &mut page)
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(flash.0[i], original[i - ACTIVE.from], "Index {}", i);
        }

        // Last page is untouched
        for i in DFU.from..DFU.to - 4096 {
            assert_eq!(flash.0[i], update[i - DFU.from], "Index {}", i);
        }

        // Mark as booted
        block_on(updater.mark_booted(&mut flash, &mut aligned)).unwrap();
        assert_eq!(
            State::Boot,
            bootloader
                .prepare_boot(&mut SingleFlashConfig::new(&mut flash), &mut magic, &mut page)
                .unwrap()
        );
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash_active_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut active = MemFlash::<16384, 4096, 8>([0xff; 16384]);
        let mut dfu = MemFlash::<16384, 2048, 8>([0xff; 16384]);
        let mut state = MemFlash::<4096, 128, 4>([0xff; 4096]);
        let mut aligned = [0; 4];

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            active.0[i] = original[i - ACTIVE.from];
        }

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(2048) {
            block_on(updater.write_firmware(offset, chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut magic = [0; 4];
        let mut page = [0; 4096];

        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::new(&mut active, &mut state, &mut dfu),
                    &mut magic,
                    &mut page
                )
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(dfu.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }
    }

    #[test]
    #[cfg(not(feature = "_verify"))]
    fn test_separate_flash_dfu_page_biggest() {
        const STATE: Partition = Partition::new(2048, 4096);
        const ACTIVE: Partition = Partition::new(4096, 16384);
        const DFU: Partition = Partition::new(0, 16384);

        let mut aligned = [0; 4];
        let mut active = MemFlash::<16384, 2048, 4>([0xff; 16384]);
        let mut dfu = MemFlash::<16384, 4096, 8>([0xff; 16384]);
        let mut state = MemFlash::<4096, 128, 4>([0xff; 4096]);

        let original: [u8; ACTIVE.len()] = [rand::random::<u8>(); ACTIVE.len()];
        let update: [u8; DFU.len()] = [rand::random::<u8>(); DFU.len()];

        for i in ACTIVE.from..ACTIVE.to {
            active.0[i] = original[i - ACTIVE.from];
        }

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut offset = 0;
        for chunk in update.chunks(4096) {
            block_on(updater.write_firmware(offset, chunk, &mut dfu, chunk.len())).unwrap();
            offset += chunk.len();
        }
        block_on(updater.mark_updated(&mut state, &mut aligned)).unwrap();

        let mut bootloader: BootLoader = BootLoader::new(ACTIVE, DFU, STATE);
        let mut magic = [0; 4];
        let mut page = [0; 4096];
        assert_eq!(
            State::Swap,
            bootloader
                .prepare_boot(
                    &mut MultiFlashConfig::new(&mut active, &mut state, &mut dfu,),
                    &mut magic,
                    &mut page
                )
                .unwrap()
        );

        for i in ACTIVE.from..ACTIVE.to {
            assert_eq!(active.0[i], update[i - ACTIVE.from], "Index {}", i);
        }

        // First DFU page is untouched
        for i in DFU.from + 4096..DFU.to {
            assert_eq!(dfu.0[i], original[i - DFU.from - 4096], "Index {}", i);
        }
    }

    #[test]
    #[should_panic]
    fn test_range_asserts() {
        const ACTIVE: Partition = Partition::new(4096, 4194304);
        const DFU: Partition = Partition::new(4194304, 2 * 4194304);
        const STATE: Partition = Partition::new(0, 4096);
        assert_partitions(ACTIVE, DFU, STATE, 4096, 4);
    }

    #[test]
    #[cfg(feature = "_verify")]
    fn test_verify() {
        // The following key setup is based on:
        // https://docs.rs/ed25519-dalek/latest/ed25519_dalek/#example

        use ed25519_dalek::Keypair;
        use rand::rngs::OsRng;

        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        use ed25519_dalek::{Digest, Sha512, Signature, Signer};
        let firmware: &[u8] = b"This are bytes that would otherwise be firmware bytes for DFU.";
        let mut digest = Sha512::new();
        digest.update(&firmware);
        let message = digest.finalize();
        let signature: Signature = keypair.sign(&message);

        use ed25519_dalek::PublicKey;
        let public_key: PublicKey = keypair.public;

        // Setup flash

        const STATE: Partition = Partition::new(0, 4096);
        const DFU: Partition = Partition::new(4096, 8192);
        let mut flash = MemFlash::<8192, 4096, 4>([0xff; 8192]);

        let firmware_len = firmware.len();

        let mut write_buf = [0; 4096];
        write_buf[0..firmware_len].copy_from_slice(firmware);
        NorFlash::write(&mut flash, DFU.from as u32, &write_buf).unwrap();

        // On with the test

        let mut updater = FirmwareUpdater::new(DFU, STATE);

        let mut aligned = [0; 4];

        assert!(block_on(updater.verify_and_mark_updated(
            &mut flash,
            &public_key.to_bytes(),
            &signature.to_bytes(),
            firmware_len,
            &mut aligned,
        ))
        .is_ok());
    }
    struct MemFlash<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize>([u8; SIZE]);

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> NorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const WRITE_SIZE: usize = WRITE_SIZE;
        const ERASE_SIZE: usize = ERASE_SIZE;
        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            let from = from as usize;
            let to = to as usize;
            assert!(from % ERASE_SIZE == 0);
            assert!(to % ERASE_SIZE == 0, "To: {}, erase size: {}", to, ERASE_SIZE);
            for i in from..to {
                self.0[i] = 0xFF;
            }
            Ok(())
        }

        fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), Self::Error> {
            assert!(data.len() % WRITE_SIZE == 0);
            assert!(offset as usize % WRITE_SIZE == 0);
            assert!(offset as usize + data.len() <= SIZE);

            self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

            Ok(())
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ErrorType
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        type Error = Infallible;
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> ReadNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const READ_SIZE: usize = 1;

        fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            let len = buf.len();
            buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
            Ok(())
        }

        fn capacity(&self) -> usize {
            SIZE
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> super::Flash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const BLOCK_SIZE: usize = ERASE_SIZE;
        const ERASE_VALUE: u8 = 0xFF;
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncReadNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const READ_SIZE: usize = 1;

        type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn read<'a>(&'a mut self, offset: u32, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            async move {
                let len = buf.len();
                buf[..].copy_from_slice(&self.0[offset as usize..offset as usize + len]);
                Ok(())
            }
        }

        fn capacity(&self) -> usize {
            SIZE
        }
    }

    impl<const SIZE: usize, const ERASE_SIZE: usize, const WRITE_SIZE: usize> AsyncNorFlash
        for MemFlash<SIZE, ERASE_SIZE, WRITE_SIZE>
    {
        const WRITE_SIZE: usize = WRITE_SIZE;
        const ERASE_SIZE: usize = ERASE_SIZE;

        type EraseFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn erase(&mut self, from: u32, to: u32) -> Self::EraseFuture<'_> {
            async move {
                let from = from as usize;
                let to = to as usize;
                assert!(from % ERASE_SIZE == 0);
                assert!(to % ERASE_SIZE == 0);
                for i in from..to {
                    self.0[i] = 0xFF;
                }
                Ok(())
            }
        }

        type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>> + 'a;
        fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
            info!("Writing {} bytes to 0x{:x}", data.len(), offset);
            async move {
                assert!(data.len() % WRITE_SIZE == 0);
                assert!(offset as usize % WRITE_SIZE == 0);
                assert!(
                    offset as usize + data.len() <= SIZE,
                    "OFFSET: {}, LEN: {}, FLASH SIZE: {}",
                    offset,
                    data.len(),
                    SIZE
                );

                self.0[offset as usize..offset as usize + data.len()].copy_from_slice(data);

                Ok(())
            }
        }
    }
}
