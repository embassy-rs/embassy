use super::TSC_NUM_GROUPS;
use super::io_pin::*;
#[cfg(any(tsc_v2, tsc_v3))]
use super::pin_groups::G7;
#[cfg(tsc_v3)]
use super::pin_groups::G8;
use super::pin_groups::{G1, G2, G3, G4, G5, G6, pin_roles};
use super::types::{Group, GroupStatus};

/// Represents a collection of TSC (Touch Sensing Controller) pins for an acquisition bank.
///
/// This struct holds optional `tsc::IOPin` values for each TSC group, allowing for flexible
/// configuration of TSC acquisition banks. Each field corresponds to a specific TSC group
/// and can be set to `Some(tsc::IOPin)` if that group is to be included in the acquisition,
/// or `None` if it should be excluded.
#[allow(missing_docs)]
#[derive(Default)]
pub struct AcquisitionBankPins {
    pub g1_pin: Option<IOPinWithRole<G1, pin_roles::Channel>>,
    pub g2_pin: Option<IOPinWithRole<G2, pin_roles::Channel>>,
    pub g3_pin: Option<IOPinWithRole<G3, pin_roles::Channel>>,
    pub g4_pin: Option<IOPinWithRole<G4, pin_roles::Channel>>,
    pub g5_pin: Option<IOPinWithRole<G5, pin_roles::Channel>>,
    pub g6_pin: Option<IOPinWithRole<G6, pin_roles::Channel>>,
    #[cfg(any(tsc_v2, tsc_v3))]
    pub g7_pin: Option<IOPinWithRole<G7, pin_roles::Channel>>,
    #[cfg(tsc_v3)]
    pub g8_pin: Option<IOPinWithRole<G8, pin_roles::Channel>>,
}

impl AcquisitionBankPins {
    /// Returns an iterator over the pins in this acquisition bank.
    ///
    /// This method allows for easy traversal of all configured pins in the bank.
    pub fn iter(&self) -> AcquisitionBankPinsIterator<'_> {
        AcquisitionBankPinsIterator(AcquisitionBankIterator::new(self))
    }
}

/// Iterator for TSC acquisition banks.
///
/// This iterator allows traversing through the pins of a `AcquisitionBankPins` struct,
/// yielding each configured pin in order of the TSC groups.
pub struct AcquisitionBankIterator<'a> {
    pins: &'a AcquisitionBankPins,
    current_group: u8,
}

impl<'a> AcquisitionBankIterator<'a> {
    fn new(pins: &'a AcquisitionBankPins) -> Self {
        Self { pins, current_group: 0 }
    }

    fn next_pin(&mut self) -> Option<IOPin> {
        while self.current_group < TSC_NUM_GROUPS as u8 {
            let pin = match self.current_group {
                0 => self.pins.g1_pin.map(IOPinWithRole::get_pin),
                1 => self.pins.g2_pin.map(IOPinWithRole::get_pin),
                2 => self.pins.g3_pin.map(IOPinWithRole::get_pin),
                3 => self.pins.g4_pin.map(IOPinWithRole::get_pin),
                4 => self.pins.g5_pin.map(IOPinWithRole::get_pin),
                5 => self.pins.g6_pin.map(IOPinWithRole::get_pin),
                #[cfg(any(tsc_v2, tsc_v3))]
                6 => self.pins.g7_pin.map(IOPinWithRole::get_pin),
                #[cfg(tsc_v3)]
                7 => self.pins.g8_pin.map(IOPinWithRole::get_pin),
                _ => None,
            };
            self.current_group += 1;
            if pin.is_some() {
                return pin;
            }
        }
        None
    }
}

/// Iterator for TSC acquisition bank pins.
///
/// This iterator yields `tsc::IOPin` values for each configured pin in the acquisition bank.
pub struct AcquisitionBankPinsIterator<'a>(AcquisitionBankIterator<'a>);

impl<'a> Iterator for AcquisitionBankPinsIterator<'a> {
    type Item = IOPin;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_pin()
    }
}

impl AcquisitionBankPins {
    /// Returns an iterator over the available pins in the bank
    pub fn pins_iterator(&self) -> AcquisitionBankPinsIterator<'_> {
        AcquisitionBankPinsIterator(AcquisitionBankIterator::new(self))
    }
}

/// Represents a collection of TSC pins to be acquired simultaneously.
///
/// This struct contains a set of pins to be used in a TSC acquisition with a pre-computed and
/// verified mask for efficiently setting up the TSC peripheral before performing an acquisition.
/// It ensures that only one channel pin per TSC group is included, adhering to hardware limitations.
pub struct AcquisitionBank {
    pub(super) pins: AcquisitionBankPins,
    pub(super) mask: u32,
}

impl AcquisitionBank {
    /// Returns an iterator over the available pins in the bank.
    pub fn pins_iterator(&self) -> AcquisitionBankPinsIterator<'_> {
        self.pins.pins_iterator()
    }

    /// Returns the mask for this bank.
    pub fn mask(&self) -> u32 {
        self.mask
    }

    /// Retrieves the TSC I/O pin for a given group in this acquisition bank.
    ///
    /// # Arguments
    /// * `group` - The TSC group to retrieve the pin for.
    ///
    /// # Returns
    /// An `Option<tsc::IOPin>` containing the pin if it exists for the given group, or `None` if not.
    pub fn get_pin(&self, group: Group) -> Option<IOPin> {
        match group {
            Group::One => self.pins.g1_pin.map(|p| p.pin),
            Group::Two => self.pins.g2_pin.map(|p| p.pin),
            Group::Three => self.pins.g3_pin.map(|p| p.pin),
            Group::Four => self.pins.g4_pin.map(|p| p.pin),
            Group::Five => self.pins.g5_pin.map(|p| p.pin),
            Group::Six => self.pins.g6_pin.map(|p| p.pin),
            #[cfg(any(tsc_v2, tsc_v3))]
            Group::Seven => self.pins.g7_pin.map(|p| p.pin),
            #[cfg(tsc_v3)]
            Group::Eight => self.pins.g8_pin.map(|p| p.pin),
        }
    }
}

/// Represents the status of all TSC groups in an acquisition bank
#[derive(Default)]
pub struct AcquisitionBankStatus {
    pub(super) groups: [Option<GroupStatus>; TSC_NUM_GROUPS],
}

impl AcquisitionBankStatus {
    /// Check if all groups in the bank are complete
    pub fn all_complete(&self) -> bool {
        self.groups
            .iter()
            .all(|&status| status.map_or(true, |s| s == GroupStatus::Complete))
    }

    /// Check if any group in the bank is ongoing
    pub fn any_ongoing(&self) -> bool {
        self.groups.iter().any(|&status| status == Some(GroupStatus::Ongoing))
    }

    /// Get the status of a specific group, if the group is present in the bank
    pub fn get_group_status(&self, group: Group) -> Option<GroupStatus> {
        let index: usize = group.into();
        self.groups[index]
    }

    /// Iterator for groups present in the bank
    pub fn iter(&self) -> impl Iterator<Item = (Group, GroupStatus)> + '_ {
        self.groups.iter().enumerate().filter_map(|(group_num, status)| {
            status.and_then(|s| Group::try_from(group_num).ok().map(|group| (group, s)))
        })
    }
}

/// Represents the result of a Touch Sensing Controller (TSC) acquisition for a specific pin.
///
/// This struct contains a reference to the `tsc::IOPin` from which a value was read,
/// along with the actual sensor reading for that pin. It provides a convenient way
/// to associate TSC readings with their corresponding pins after an acquisition.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug)]
pub struct ChannelReading {
    /// The sensor reading value obtained from the TSC acquisition.
    /// Lower values typically indicate a detected touch, while higher values indicate no touch.
    pub sensor_value: u16,

    /// The `tsc::IOPin` associated with this reading.
    /// This allows for easy identification of which pin the reading corresponds to.
    pub tsc_pin: IOPin,
}

/// Represents the readings from all TSC groups
#[derive(Default)]
pub struct AcquisitionBankReadings {
    pub(super) groups: [Option<ChannelReading>; TSC_NUM_GROUPS],
}

impl AcquisitionBankReadings {
    /// Get the reading for a specific group, if the group is present in the bank
    pub fn get_group_reading(&self, group: Group) -> Option<ChannelReading> {
        let index: usize = group.into();
        self.groups[index]
    }

    /// Iterator for readings for groups present in the bank
    pub fn iter(&self) -> impl Iterator<Item = ChannelReading> + '_ {
        self.groups.iter().filter_map(|&x| x)
    }
}
