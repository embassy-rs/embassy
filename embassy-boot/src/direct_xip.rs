use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

use crate::STATE_ERASE_VALUE;

/// An image slot. Link each image for the address of its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Slot {
    /// Slot A, used when the state partition is empty.
    A,
    /// Slot B.
    B,
}

impl Slot {
    /// Array index for this slot: A is 0, B is 1.
    pub const fn index(self) -> usize {
        self as usize
    }

    const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::A),
            1 => Some(Self::B),
            _ => None,
        }
    }

    pub(crate) const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

/// Errors from reading or changing boot state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DirectXipError {
    /// Flash access failed. The previous record is still available.
    Flash(NorFlashErrorKind),
    /// The request doesn't match the current trial state.
    BadState,
    /// Neither image is bootable. Stay in the bootloader for recovery.
    NoValidImage,
}

impl core::fmt::Display for DirectXipError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Flash(error) => write!(f, "Flash error: {error}"),
            Self::BadState => f.write_str("Request conflicts with the recorded boot state"),
            Self::NoValidImage => f.write_str("Neither image is valid"),
        }
    }
}

impl core::error::Error for DirectXipError {}

#[cfg(feature = "defmt")]
impl defmt::Format for DirectXipError {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::Flash(_) => defmt::write!(fmt, "DirectXipError::Flash(_)"),
            Self::BadState => defmt::write!(fmt, "DirectXipError::BadState"),
            Self::NoValidImage => defmt::write!(fmt, "DirectXipError::NoValidImage"),
        }
    }
}

impl<E: NorFlashError> From<E> for DirectXipError {
    fn from(error: E) -> Self {
        Self::Flash(error.kind())
    }
}

/// The state partition or its I/O buffer is too small.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DirectXipConfigError {
    /// The buffer is too short for a record, including flash alignment padding.
    BufferTooSmall,
    /// There isn't room for two erase pages.
    StateTooSmall,
}

impl core::fmt::Display for DirectXipConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::BufferTooSmall => "Buffer is too small for a boot record",
            Self::StateTooSmall => "State partition needs at least two erase pages",
        })
    }
}

impl core::error::Error for DirectXipConfigError {}

/// Each record has eight bytes of data followed by a copy with all bits flipped.
///
/// If power fails partway through changing a record, the two halves won't
/// match. The bootloader can still use the record on the other page.
///
/// | Byte | Content                     |
/// |------|-----------------------------|
/// | 0..4 | `generation`, little endian |
/// | 4    | `active` as a slot index    |
/// | 5    | `update`: 0, 1 or 2         |
/// | 6..8 | reserved, written as 0      |
#[derive(Clone, Copy, PartialEq, Eq)]
struct Record {
    generation: u32,
    active: Slot,
    update: Update,
}

/// Progress of an update in the slot opposite `active`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Update {
    None,
    /// Marked as updated, waiting for the next reset.
    Pending,
    /// Running on trial. The next reset rolls back unless it's marked as booted.
    Trial,
}

impl Record {
    const SIZE: usize = 16;

    const INITIAL: Self = Self {
        generation: 0,
        active: Slot::A,
        update: Update::None,
    };

    fn decode(bytes: &[u8]) -> Option<Self> {
        if (0..8).any(|i| bytes[i] != !bytes[i + 8]) {
            return None;
        }
        Some(Self {
            generation: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            active: Slot::from_index(bytes[4])?,
            update: match bytes[5] {
                0 => Update::None,
                1 => Update::Pending,
                2 => Update::Trial,
                _ => return None,
            },
        })
    }

    fn encode(self, bytes: &mut [u8]) {
        bytes.fill(STATE_ERASE_VALUE);
        bytes[..4].copy_from_slice(&self.generation.to_le_bytes());
        bytes[4..8].copy_from_slice(&[self.active as u8, self.update as u8, 0, 0]);
        for i in 0..8 {
            bytes[i + 8] = !bytes[i];
        }
    }

    fn newer_than(self, other: Self) -> bool {
        // Wrapping from u32::MAX to 0 still makes the record newer.
        self.generation.wrapping_sub(other.generation) as i32 > 0
    }
}

/// A/B boot state for the bootloader and application.
///
/// Write the update to [`Self::update_slot`], then call [`Self::mark_updated`].
/// On reset, the bootloader calls [`Self::prepare_boot`] to pick an image.
/// The running image calls [`Self::mark_booted`] after its startup checks pass.
/// If it resets before doing so, the bootloader goes back to the confirmed image.
///
/// This needs two erase pages of its own, separate from swap state. Each write
/// goes to the other page so a power failure leaves the last record intact.
///
/// A failed or cancelled call may still have written a record. Each state call
/// reloads flash, so use its result rather than assuming the write failed.
/// The flash driver must handle cancellation of its own operations.
///
/// Image validation and starting other cores are up to the caller. Use a
/// separate instance for each image pair.
pub struct DirectXipState<'d, STATE> {
    state: STATE,
    aligned: &'d mut [u8],
}

/// Blocking version of [`DirectXipState`], using the same flash layout.
pub struct BlockingDirectXipState<'d, STATE> {
    state: STATE,
    aligned: &'d mut [u8],
}

macro_rules! direct_xip_state {
    ($state:ident, $flash:path $(, $async:tt, $await:tt)?) => {
        impl<'d, STATE: $flash> $state<'d, STATE> {
            /// Scratch buffer size: 16 bytes rounded up to flash write and read sizes.
            pub const BUFFER_SIZE: usize = {
                let size = Record::SIZE
                    .next_multiple_of(STATE::WRITE_SIZE)
                    .next_multiple_of(STATE::READ_SIZE);
                core::assert!(size % STATE::WRITE_SIZE == 0 && size <= STATE::ERASE_SIZE);
                size
            };

            /// Open the state partition using `aligned` as scratch space.
            ///
            /// `aligned` must contain at least [`Self::BUFFER_SIZE`] bytes and meet
            /// the flash driver's alignment requirements.
            ///
            /// # Errors
            ///
            /// Returns [`DirectXipConfigError::BufferTooSmall`] for a short buffer,
            /// or [`DirectXipConfigError::StateTooSmall`] for fewer than two erase
            /// pages. Flash is left untouched.
            pub fn new(state: STATE, aligned: &'d mut [u8]) -> Result<Self, DirectXipConfigError> {
                if aligned.len() < Self::BUFFER_SIZE {
                    return Err(DirectXipConfigError::BufferTooSmall);
                }
                if state.capacity() / STATE::ERASE_SIZE < 2 {
                    return Err(DirectXipConfigError::StateTooSmall);
                }
                Ok(Self {
                    state,
                    aligned: &mut aligned[..Self::BUFFER_SIZE],
                })
            }

            /// The slot to write the next update to.
            ///
            /// Returns [`DirectXipError::BadState`] until the current trial is resolved.
            pub $( $async )? fn update_slot(&mut self) -> Result<Slot, DirectXipError> {
                let (record, _) = self.load() $(.$await)? ?;
                if record.update != Update::None {
                    return Err(DirectXipError::BadState);
                }
                Ok(record.active.other())
            }

            /// Try `slot` on the next reset.
            ///
            /// Write and validate the whole image first, including any required
            /// signature check. If `slot` is already marked, this does nothing.
            /// Returns [`DirectXipError::BadState`] if `slot` is the confirmed
            /// one or is being tried right now.
            pub $( $async )? fn mark_updated(&mut self, slot: Slot) -> Result<(), DirectXipError> {
                let (mut record, page) = self.load() $(.$await)? ?;
                if slot == record.active || record.update == Update::Trial {
                    return Err(DirectXipError::BadState);
                }
                if record.update == Update::Pending {
                    return Ok(());
                }
                record.update = Update::Pending;
                self.store(record, page) $(.$await)?
            }

            /// Whether `running` is marked as booted.
            ///
            /// `false` means the image is on trial. Run its startup checks, then
            /// call [`Self::mark_booted`].
            pub $( $async )? fn is_booted(&mut self, running: Slot) -> Result<bool, DirectXipError> {
                let (record, _) = self.load() $(.$await)? ?;
                Ok(running == record.active)
            }

            /// Keep the running image after its startup checks pass.
            ///
            /// Pass the slot this code is running from. If it's already confirmed,
            /// this does nothing. Otherwise, returns [`DirectXipError::BadState`]
            /// unless the bootloader has recorded a trial attempt.
            pub $( $async )? fn mark_booted(&mut self, running: Slot) -> Result<(), DirectXipError> {
                let (mut record, page) = self.load() $(.$await)? ?;
                if running == record.active {
                    return Ok(());
                }
                if record.update != Update::Trial {
                    return Err(DirectXipError::BadState);
                }
                record.active = running;
                record.update = Update::None;
                self.store(record, page) $(.$await)?
            }

            /// Pick an image to boot and save any changes to boot state.
            ///
            /// Check both images' vectors, bounds, and any required signatures,
            /// then pass the results in [`Slot::index`] order.
            ///
            /// A bad or expired trial falls back to the confirmed image. If that
            /// image is also bad, the other valid image becomes confirmed.
            /// Returns [`DirectXipError::NoValidImage`] when neither can boot.
            pub $( $async )? fn prepare_boot(&mut self, valid: [bool; 2]) -> Result<Slot, DirectXipError> {
                let (mut record, page) = self.load() $(.$await)? ?;
                let stored = record;
                let other = record.active.other();

                if record.update == Update::Pending && valid[other.index()] {
                    record.update = Update::Trial;
                    self.store(record, page) $(.$await)? ?;
                    return Ok(other);
                }
                record.update = Update::None;

                if !valid[record.active.index()] {
                    if !valid[other.index()] {
                        return Err(DirectXipError::NoValidImage);
                    }
                    record.active = other;
                }

                if record != stored {
                    self.store(record, page) $(.$await)? ?;
                }
                Ok(record.active)
            }

            // Return the latest record and the page we can overwrite next.
            $( $async )? fn load(&mut self) -> Result<(Record, usize), DirectXipError> {
                let first = self.read_page(0) $(.$await)? ?;
                let second = self.read_page(1) $(.$await)? ?;
                Ok(match (first, second) {
                    (Some(first), Some(second)) if second.newer_than(first) => (second, 0),
                    (Some(first), _) => (first, 1),
                    (None, Some(second)) => (second, 0),
                    (None, None) => (Record::INITIAL, 0),
                })
            }

            $( $async )? fn read_page(&mut self, page: usize) -> Result<Option<Record>, DirectXipError> {
                let start = (page * STATE::ERASE_SIZE) as u32;
                self.state.read(start, self.aligned) $(.$await)? ?;
                Ok(Record::decode(self.aligned))
            }

            $( $async )? fn store(&mut self, mut record: Record, page: usize) -> Result<(), DirectXipError> {
                let start = (page * STATE::ERASE_SIZE) as u32;
                record.generation = record.generation.wrapping_add(1);
                record.encode(self.aligned);
                self.state.erase(start, start + STATE::ERASE_SIZE as u32) $(.$await)? ?;
                self.state.write(start, self.aligned) $(.$await)? ?;
                Ok(())
            }
        }
    };
}

direct_xip_state!(
    DirectXipState,
    embedded_storage_async::nor_flash::NorFlash,
    async,
    await
);
direct_xip_state!(BlockingDirectXipState, embedded_storage::nor_flash::NorFlash);

#[cfg(test)]
mod tests {
    use core::ops::Range;

    use embassy_futures::block_on;

    use super::*;
    use crate::mem_flash::MemFlash;

    type Flash = MemFlash<128, 64, 4>;

    const BOTH_VALID: [bool; 2] = [true, true];
    const BAD_STATE: Result<(), DirectXipError> = Err(DirectXipError::BadState);
    const FLASH_ERROR: DirectXipError = DirectXipError::Flash(NorFlashErrorKind::Other);

    #[derive(Default)]
    struct Device {
        flash: Flash,
        aligned: [u8; Record::SIZE],
    }

    impl Device {
        // Rebuild the handle each time, as the firmware would after a reset.
        fn state(&mut self) -> BlockingDirectXipState<'_, &mut Flash> {
            BlockingDirectXipState::new(&mut self.flash, &mut self.aligned).unwrap()
        }

        fn with_b_on_trial() -> Self {
            let mut device = Self::default();
            device.state().mark_updated(Slot::B).unwrap();
            device
        }

        fn with_b_confirmed() -> Self {
            let mut device = Self::with_b_on_trial();
            device.state().prepare_boot(BOTH_VALID).unwrap();
            device.state().mark_booted(Slot::B).unwrap();
            device
        }
    }

    #[test]
    fn constructors_reject_partitions_smaller_than_two_erase_pages() {
        let mut flash = MemFlash::<127, 64, 4>::default();
        let mut aligned = [0; Record::SIZE];
        assert_eq!(
            BlockingDirectXipState::new(&mut flash, &mut aligned).err(),
            Some(DirectXipConfigError::StateTooSmall)
        );
        assert_eq!(
            DirectXipState::new(&mut flash, &mut aligned).err(),
            Some(DirectXipConfigError::StateTooSmall)
        );
    }

    #[test]
    fn exact_buffer_size_supports_flash_access_units() {
        fn check<const WRITE: usize, const READ: usize>() {
            type Flash<const W: usize, const R: usize> = MemFlash<1024, 512, W, R>;
            let mut flash = Flash::<WRITE, READ>::default();
            let mut aligned = [0; 512];
            let size = BlockingDirectXipState::<Flash<WRITE, READ>>::BUFFER_SIZE;
            assert_eq!(
                BlockingDirectXipState::new(&mut flash, &mut aligned[..size - 1]).err(),
                Some(DirectXipConfigError::BufferTooSmall)
            );
            {
                let mut state = BlockingDirectXipState::new(&mut flash, &mut aligned[..size]).unwrap();
                state.mark_updated(Slot::B).unwrap();
                assert_eq!(state.prepare_boot(BOTH_VALID), Ok(Slot::B));
            }
            let size = DirectXipState::<Flash<WRITE, READ>>::BUFFER_SIZE;
            let mut state = DirectXipState::new(&mut flash, &mut aligned[..size]).unwrap();
            assert_eq!(block_on(state.prepare_boot(BOTH_VALID)), Ok(Slot::A));
        }

        check::<1, 1>();
        check::<4, 1>();
        check::<16, 1>();
        check::<32, 1>();
        check::<8, 32>();
        check::<256, 4>();
    }

    #[test]
    fn erased_state_boots_slot_a_and_updates_slot_b() {
        let mut device = Device::default();
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::A));
        assert_eq!(device.state().mark_booted(Slot::A), Ok(()));
        assert_eq!(device.state().update_slot(), Ok(Slot::B));
    }

    #[test]
    fn confirmed_update_is_kept_on_next_boot() {
        let mut device = Device::with_b_on_trial();
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::B));
        assert_eq!(device.state().is_booted(Slot::B), Ok(false));
        assert_eq!(device.state().mark_booted(Slot::B), Ok(()));
        assert_eq!(device.state().is_booted(Slot::B), Ok(true));
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::B));
        assert_eq!(device.state().update_slot(), Ok(Slot::A));
    }

    #[test]
    fn unconfirmed_update_rolls_back_on_next_boot() {
        let mut device = Device::with_b_on_trial();
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::B));
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::A));

        assert_eq!(device.state().mark_booted(Slot::B), BAD_STATE);
        assert_eq!(device.state().update_slot(), Ok(Slot::B));
    }

    #[test]
    fn requests_that_conflict_with_the_record_are_rejected() {
        let mut device = Device::default();
        assert_eq!(device.state().mark_updated(Slot::A), BAD_STATE);
        assert_eq!(device.state().mark_booted(Slot::B), BAD_STATE);

        let mut device = Device::with_b_on_trial();
        assert_eq!(device.state().update_slot(), Err(DirectXipError::BadState));
        assert_eq!(device.state().mark_updated(Slot::A), BAD_STATE);
        assert_eq!(device.state().mark_booted(Slot::B), BAD_STATE);
    }

    #[test]
    fn marking_the_pending_slot_again_writes_nothing() {
        let mut device = Device::with_b_on_trial();
        let before = device.flash.mem;
        assert_eq!(device.state().mark_updated(Slot::B), Ok(()));
        assert_eq!(device.flash.mem, before);
    }

    #[test]
    fn only_valid_slots_are_booted() {
        let cases = [
            (Device::default as fn() -> Device, [true, false], Ok(Slot::A)),
            (Device::default, [false, true], Ok(Slot::B)),
            (Device::default, [false, false], Err(DirectXipError::NoValidImage)),
            (Device::with_b_on_trial, [true, false], Ok(Slot::A)),
            (Device::with_b_on_trial, [false, true], Ok(Slot::B)),
            (
                Device::with_b_on_trial,
                [false, false],
                Err(DirectXipError::NoValidImage),
            ),
            (Device::with_b_confirmed, [true, false], Ok(Slot::A)),
            (Device::with_b_confirmed, [false, true], Ok(Slot::B)),
            (
                Device::with_b_confirmed,
                [false, false],
                Err(DirectXipError::NoValidImage),
            ),
        ];
        for (case, (device, valid, slot)) in cases.into_iter().enumerate() {
            assert_eq!(device().state().prepare_boot(valid), slot, "case {case}: {valid:?}");
        }
    }

    #[test]
    fn invalid_update_is_dropped_without_a_trial_boot() {
        let mut device = Device::with_b_on_trial();
        assert_eq!(device.state().prepare_boot([true, false]), Ok(Slot::A));
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::A));
        assert_eq!(device.state().update_slot(), Ok(Slot::B));
    }

    #[test]
    fn slot_replacing_an_invalid_confirmed_slot_is_confirmed() {
        let mut device = Device::default();
        assert_eq!(device.state().prepare_boot([false, true]), Ok(Slot::B));
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::B));
        assert_eq!(device.state().update_slot(), Ok(Slot::A));
    }

    #[test]
    fn async_and_blocking_states_share_records() {
        let mut flash = Flash::default();
        let mut aligned = [0; Record::SIZE];
        BlockingDirectXipState::new(&mut flash, &mut aligned)
            .unwrap()
            .mark_updated(Slot::B)
            .unwrap();

        let mut state = DirectXipState::new(&mut flash, &mut aligned).unwrap();
        assert_eq!(block_on(state.prepare_boot(BOTH_VALID)), Ok(Slot::B));
        assert_eq!(block_on(state.mark_booted(Slot::B)), Ok(()));

        let mut state = BlockingDirectXipState::new(&mut flash, &mut aligned).unwrap();
        assert_eq!(state.update_slot(), Ok(Slot::A));
    }

    #[test]
    fn record_after_a_generation_wrap_is_the_current_one() {
        let mut device = Device::default();
        let last = Record {
            generation: u32::MAX,
            ..Record::INITIAL
        };
        last.encode(&mut device.aligned);
        device.flash.program(0, &device.aligned).unwrap();

        device.state().mark_updated(Slot::B).unwrap();
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Ok(Slot::B));
    }

    #[test]
    fn flash_error_is_returned_instead_of_a_slot() {
        let mut device = Device::with_b_confirmed();
        device.flash.pending_read_successes = Some(0);
        assert_eq!(device.state().prepare_boot(BOTH_VALID), Err(FLASH_ERROR));
    }

    #[test]
    fn interrupted_write_keeps_the_previous_record() {
        // Confirming B leaves the latest record on page 0. The next write goes to page 1.
        const PAGE: Range<usize> = 64..128;
        let mut written = Device::with_b_confirmed();
        written.state().mark_updated(Slot::A).unwrap();

        for words in 0..Record::SIZE / 4 {
            let mut device = Device::with_b_confirmed();
            let torn = PAGE.start..PAGE.start + words * 4;
            device.flash.mem[PAGE].fill(0xFF);
            device.flash.mem[torn.clone()].copy_from_slice(&written.flash.mem[torn]);

            assert_eq!(device.state().update_slot(), Ok(Slot::A), "{words} words written");
            assert_eq!(
                device.state().prepare_boot(BOTH_VALID),
                Ok(Slot::B),
                "{words} words written"
            );
        }
    }

    #[test]
    fn interrupted_erase_keeps_the_previous_record() {
        // Try each set of words that could survive the interrupted erase.
        for kept_words in 0..1 << 16 {
            let mut device = Device::with_b_confirmed();
            device.flash.pending_erase_tear = Some(kept_words);

            assert_eq!(
                device.state().mark_updated(Slot::A),
                Err(FLASH_ERROR),
                "{kept_words:#06x}"
            );
            assert_eq!(device.state().update_slot(), Ok(Slot::A), "{kept_words:#06x}");
            assert_eq!(
                device.state().prepare_boot(BOTH_VALID),
                Ok(Slot::B),
                "{kept_words:#06x}"
            );
        }
    }
}
