//! Filter bank API.

use core::marker::PhantomData;

use super::{ExtendedId, Fifo, Id, StandardId};

const F32_RTR: u32 = 0b010; // set the RTR bit to match remote frames
const F32_IDE: u32 = 0b100; // set the IDE bit to match extended identifiers
const F16_RTR: u16 = 0b10000;
const F16_IDE: u16 = 0b01000;

/// A 16-bit filter list entry.
///
/// This can match data and remote frames using standard IDs.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ListEntry16(u16);

/// A 32-bit filter list entry.
///
/// This can match data and remote frames using extended or standard IDs.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ListEntry32(u32);

/// A 16-bit identifier mask.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mask16 {
    id: u16,
    mask: u16,
}

/// A 32-bit identifier mask.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Mask32 {
    id: u32,
    mask: u32,
}

impl ListEntry16 {
    /// Creates a filter list entry that accepts data frames with the given standard ID.
    ///
    /// This entry will *not* accept remote frames with the same ID.
    pub fn data_frames_with_id(id: StandardId) -> Self {
        Self(id.as_raw() << 5)
    }

    /// Creates a filter list entry that accepts remote frames with the given standard ID.
    pub fn remote_frames_with_id(id: StandardId) -> Self {
        Self(id.as_raw() << 5 | F16_RTR)
    }
}

impl ListEntry32 {
    /// Creates a filter list entry that accepts data frames with the given ID.
    ///
    /// This entry will *not* accept remote frames with the same ID.
    ///
    /// The filter will only accept *either* standard *or* extended frames, depending on `id`.
    pub fn data_frames_with_id(id: impl Into<Id>) -> Self {
        match id.into() {
            Id::Standard(id) => Self(u32::from(id.as_raw()) << 21),
            Id::Extended(id) => Self(id.as_raw() << 3 | F32_IDE),
        }
    }

    /// Creates a filter list entry that accepts remote frames with the given ID.
    pub fn remote_frames_with_id(id: impl Into<Id>) -> Self {
        match id.into() {
            Id::Standard(id) => Self(u32::from(id.as_raw()) << 21 | F32_RTR),
            Id::Extended(id) => Self(id.as_raw() << 3 | F32_IDE | F32_RTR),
        }
    }
}

impl Mask16 {
    /// Creates a 16-bit identifier mask that accepts all frames.
    ///
    /// This will accept both standard and extended data and remote frames with any ID.
    pub fn accept_all() -> Self {
        Self { id: 0, mask: 0 }
    }

    /// Creates a 16-bit identifier mask that accepts all frames with the given standard
    /// ID and mask combination.
    ///
    /// Filter logic: `frame_accepted = (incoming_id & mask) == (id & mask)`
    ///
    /// A mask of all all ones (`0x7FF`) matches an exact ID, a mask of 0 matches all IDs.
    ///
    /// Both data and remote frames with `id` will be accepted. Any extended frames will be
    /// rejected.
    pub fn frames_with_std_id(id: StandardId, mask: StandardId) -> Self {
        Self {
            id: id.as_raw() << 5,
            mask: mask.as_raw() << 5 | F16_IDE, // also require IDE = 0
        }
    }

    /// Make the filter accept data frames only.
    pub fn data_frames_only(&mut self) -> &mut Self {
        self.id &= !F16_RTR; // RTR = 0
        self.mask |= F16_RTR;
        self
    }

    /// Make the filter accept remote frames only.
    pub fn remote_frames_only(&mut self) -> &mut Self {
        self.id |= F16_RTR; // RTR = 1
        self.mask |= F16_RTR;
        self
    }
}

impl Mask32 {
    /// Creates a 32-bit identifier mask that accepts all frames.
    ///
    /// This will accept both standard and extended data and remote frames with any ID.
    pub fn accept_all() -> Self {
        Self { id: 0, mask: 0 }
    }

    /// Creates a 32-bit identifier mask that accepts all frames with the given extended
    /// ID and mask combination.
    ///
    /// Filter logic: `frame_accepted = (incoming_id & mask) == (id & mask)`
    ///
    /// A mask of all all ones (`0x1FFF_FFFF`) matches an exact ID, a mask of 0 matches all IDs.
    ///
    /// Both data and remote frames with `id` will be accepted. Standard frames will be rejected.
    pub fn frames_with_ext_id(id: ExtendedId, mask: ExtendedId) -> Self {
        Self {
            id: id.as_raw() << 3 | F32_IDE,
            mask: mask.as_raw() << 3 | F32_IDE, // also require IDE = 1
        }
    }

    /// Creates a 32-bit identifier mask that accepts all frames with the given standard
    /// ID and mask combination.
    ///
    /// Filter logic: `frame_accepted = (incoming_id & mask) == (id & mask)`
    ///
    /// A mask of all all ones (`0x7FF`) matches the exact ID, a mask of 0 matches all IDs.
    ///
    /// Both data and remote frames with `id` will be accepted. Extended frames will be rejected.
    pub fn frames_with_std_id(id: StandardId, mask: StandardId) -> Self {
        Self {
            id: u32::from(id.as_raw()) << 21,
            mask: u32::from(mask.as_raw()) << 21 | F32_IDE, // also require IDE = 0
        }
    }

    /// Make the filter accept data frames only.
    pub fn data_frames_only(&mut self) -> &mut Self {
        self.id &= !F32_RTR; // RTR = 0
        self.mask |= F32_RTR;
        self
    }

    /// Make the filter accept remote frames only.
    pub fn remote_frames_only(&mut self) -> &mut Self {
        self.id |= F32_RTR; // RTR = 1
        self.mask |= F32_RTR;
        self
    }
}

/// The configuration of a filter bank.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BankConfig {
    /// Specify up to 4 exact standard CAN ID's.
    List16([ListEntry16; 4]),
    /// Specify up to 2 exact standard or extended CAN ID's.
    List32([ListEntry32; 2]),
    /// Specify up to 2 standard ID's with masks.
    Mask16([Mask16; 2]),
    /// Specify a single extended ID with mask.
    Mask32(Mask32),
}

impl From<[ListEntry16; 4]> for BankConfig {
    #[inline]
    fn from(entries: [ListEntry16; 4]) -> Self {
        Self::List16(entries)
    }
}

impl From<[ListEntry32; 2]> for BankConfig {
    #[inline]
    fn from(entries: [ListEntry32; 2]) -> Self {
        Self::List32(entries)
    }
}

impl From<[Mask16; 2]> for BankConfig {
    #[inline]
    fn from(entries: [Mask16; 2]) -> Self {
        Self::Mask16(entries)
    }
}

impl From<Mask32> for BankConfig {
    #[inline]
    fn from(filter: Mask32) -> Self {
        Self::Mask32(filter)
    }
}

/// Interface to the filter banks of a CAN peripheral.
pub struct MasterFilters<'a> {
    /// Number of assigned filter banks.
    ///
    /// On chips with splittable filter banks, this value can be dynamic.
    bank_count: u8,
    _phantom: PhantomData<&'a ()>,
    info: &'static crate::can::Info,
}

// NOTE: This type mutably borrows the CAN instance and has unique access to the registers while it
// exists.
impl MasterFilters<'_> {
    pub(crate) unsafe fn new(info: &'static crate::can::Info) -> Self {
        // Enable initialization mode.
        info.regs.0.fmr().modify(|reg| reg.set_finit(true));

        // Read the filter split value.
        let bank_count = info.regs.0.fmr().read().can2sb();

        // (Reset value of CAN2SB is 0x0E, 14, which, in devices with 14 filter banks, assigns all
        // of them to the master peripheral, and in devices with 28, assigns them 50/50 to
        // master/slave instances)

        Self {
            bank_count,
            _phantom: PhantomData,
            info,
        }
    }

    fn banks_imm(&self) -> FilterBanks {
        FilterBanks {
            start_idx: 0,
            bank_count: self.bank_count,
            info: self.info,
        }
    }

    /// Returns the number of filter banks currently assigned to this instance.
    ///
    /// Chips with splittable filter banks may start out with some banks assigned to the master
    /// instance and some assigned to the slave instance.
    pub fn num_banks(&self) -> u8 {
        self.bank_count
    }

    /// Disables all enabled filter banks.
    ///
    /// This causes all incoming frames to be disposed.
    pub fn clear(&mut self) -> &mut Self {
        self.banks_imm().clear();
        self
    }

    /// Disables a filter bank.
    ///
    /// If `index` is out of bounds, this will panic.
    pub fn disable_bank(&mut self, index: u8) -> &mut Self {
        self.banks_imm().disable(index);
        self
    }

    /// Configures a filter bank according to `config` and enables it.
    ///
    /// Each filter bank is associated with one of the two RX FIFOs, configured by the [`Fifo`]
    /// passed to this function. In the event that both FIFOs are configured to accept an incoming
    /// frame, the accepting filter bank with the lowest index wins. The FIFO state is ignored, so
    /// if the FIFO is full, it will overflow, even if the other FIFO is also configured to accept
    /// the frame.
    ///
    /// # Parameters
    ///
    /// - `index`: the filter index.
    /// - `fifo`: the receive FIFO the filter should pass accepted messages to.
    /// - `config`: the filter configuration.
    pub fn enable_bank(&mut self, index: u8, fifo: Fifo, config: impl Into<BankConfig>) -> &mut Self {
        self.banks_imm().enable(index, fifo, config.into());
        self
    }
}

impl MasterFilters<'_> {
    /// Sets the index at which the filter banks owned by the slave peripheral start.
    pub fn set_split(&mut self, split_index: u8) -> &mut Self {
        assert!(split_index <= self.info.num_filter_banks);
        self.info.regs.0.fmr().modify(|reg| reg.set_can2sb(split_index));
        self.bank_count = split_index;
        self
    }

    /// Accesses the filters assigned to the slave peripheral.
    pub fn slave_filters(&mut self) -> SlaveFilters<'_> {
        // NB: This mutably borrows `self`, so it has full access to the filter bank registers.
        SlaveFilters {
            start_idx: self.bank_count,
            bank_count: self.info.num_filter_banks - self.bank_count,
            _phantom: PhantomData,
            info: self.info,
        }
    }
}

impl Drop for MasterFilters<'_> {
    #[inline]
    fn drop(&mut self) {
        // Leave initialization mode.
        self.info.regs.0.fmr().modify(|regs| regs.set_finit(false));
    }
}

/// Interface to the filter banks assigned to a slave peripheral.
pub struct SlaveFilters<'a> {
    start_idx: u8,
    bank_count: u8,
    _phantom: PhantomData<&'a ()>,
    info: &'static crate::can::Info,
}

impl SlaveFilters<'_> {
    fn banks_imm(&self) -> FilterBanks {
        FilterBanks {
            start_idx: self.start_idx,
            bank_count: self.bank_count,
            info: self.info,
        }
    }

    /// Returns the number of filter banks currently assigned to this instance.
    ///
    /// Chips with splittable filter banks may start out with some banks assigned to the master
    /// instance and some assigned to the slave instance.
    pub fn num_banks(&self) -> u8 {
        self.bank_count
    }

    /// Disables all enabled filter banks.
    ///
    /// This causes all incoming frames to be disposed.
    pub fn clear(&mut self) -> &mut Self {
        self.banks_imm().clear();
        self
    }

    /// Disables a filter bank.
    ///
    /// If `index` is out of bounds, this will panic.
    pub fn disable_bank(&mut self, index: u8) -> &mut Self {
        self.banks_imm().disable(index);
        self
    }

    /// Configures a filter bank according to `config` and enables it.
    ///
    /// # Parameters
    ///
    /// - `index`: the filter index.
    /// - `fifo`: the receive FIFO the filter should pass accepted messages to.
    /// - `config`: the filter configuration.
    pub fn enable_bank(&mut self, index: u8, fifo: Fifo, config: impl Into<BankConfig>) -> &mut Self {
        self.banks_imm().enable(index, fifo, config.into());
        self
    }
}

struct FilterBanks {
    start_idx: u8,
    bank_count: u8,
    info: &'static crate::can::Info,
}

impl FilterBanks {
    fn clear(&mut self) {
        let mask = filter_bitmask(self.start_idx, self.bank_count);

        self.info.regs.0.fa1r().modify(|reg| {
            for i in 0..28usize {
                if (0x01u32 << i) & mask != 0 {
                    reg.set_fact(i, false);
                }
            }
        });
    }

    fn assert_bank_index(&self, index: u8) {
        assert!((self.start_idx..self.start_idx + self.bank_count).contains(&index));
    }

    fn disable(&mut self, index: u8) {
        self.assert_bank_index(index);
        self.info
            .regs
            .0
            .fa1r()
            .modify(|reg| reg.set_fact(index as usize, false))
    }

    fn enable(&mut self, index: u8, fifo: Fifo, config: BankConfig) {
        self.assert_bank_index(index);

        // Configure mode.
        let mode = matches!(config, BankConfig::List16(_) | BankConfig::List32(_));
        self.info.regs.0.fm1r().modify(|reg| reg.set_fbm(index as usize, mode));

        // Configure scale.
        let scale = matches!(config, BankConfig::List32(_) | BankConfig::Mask32(_));
        self.info.regs.0.fs1r().modify(|reg| reg.set_fsc(index as usize, scale));

        // Configure filter register.
        let (fxr1, fxr2);
        match config {
            BankConfig::List16([a, b, c, d]) => {
                fxr1 = (u32::from(b.0) << 16) | u32::from(a.0);
                fxr2 = (u32::from(d.0) << 16) | u32::from(c.0);
            }
            BankConfig::List32([a, b]) => {
                fxr1 = a.0;
                fxr2 = b.0;
            }
            BankConfig::Mask16([a, b]) => {
                fxr1 = (u32::from(a.mask) << 16) | u32::from(a.id);
                fxr2 = (u32::from(b.mask) << 16) | u32::from(b.id);
            }
            BankConfig::Mask32(a) => {
                fxr1 = a.id;
                fxr2 = a.mask;
            }
        };
        let bank = self.info.regs.0.fb(index as usize);
        bank.fr1().write(|w| w.0 = fxr1);
        bank.fr2().write(|w| w.0 = fxr2);

        // Assign to the right FIFO
        self.info.regs.0.ffa1r().modify(|reg| {
            reg.set_ffa(
                index as usize,
                match fifo {
                    Fifo::Fifo0 => false,
                    Fifo::Fifo1 => true,
                },
            )
        });

        // Set active.
        self.info.regs.0.fa1r().modify(|reg| reg.set_fact(index as usize, true))
    }
}

/// Computes a bitmask for per-filter-bank registers that only includes filters in the given range.
fn filter_bitmask(start_idx: u8, bank_count: u8) -> u32 {
    let count_mask = (1 << bank_count) - 1; // `bank_count` 1-bits
    count_mask << start_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_bitmask() {
        assert_eq!(filter_bitmask(0, 1), 0x1);
        assert_eq!(filter_bitmask(1, 1), 0b10);
        assert_eq!(filter_bitmask(0, 4), 0xf);
        assert_eq!(filter_bitmask(1, 3), 0xe);
        assert_eq!(filter_bitmask(8, 1), 0x100);
        assert_eq!(filter_bitmask(8, 4), 0xf00);
    }
}
