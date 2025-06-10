use core::marker::PhantomData;
use core::ops::BitOr;

use super::errors::GroupError;
use super::io_pin::*;
use super::Instance;
use crate::gpio::{AfType, AnyPin, OutputType, Speed};
use crate::Peri;

/// Pin type definition to control IO parameters
#[derive(PartialEq, Clone, Copy)]
pub enum PinType {
    /// Sensing channel pin connected to an electrode
    Channel,
    /// Sampling capacitor pin, one required for every pin group
    Sample,
    /// Shield pin connected to capacitive sensing shield
    Shield,
}

/// Pin struct that maintains usage
#[allow(missing_docs)]
pub struct Pin<'d, T, Group> {
    _pin: Peri<'d, AnyPin>,
    role: PinType,
    tsc_io_pin: IOPin,
    phantom: PhantomData<(T, Group)>,
}

impl<'d, T, Group> Pin<'d, T, Group> {
    /// Returns the role of this TSC pin.
    ///
    /// The role indicates whether this pin is configured as a channel,
    /// sampling capacitor, or shield in the TSC group.
    ///
    /// # Returns
    /// The `PinType` representing the role of this pin.
    pub fn role(&self) -> PinType {
        self.role
    }

    /// Returns the TSC IO pin associated with this pin.
    ///
    /// This method provides access to the specific TSC IO pin configuration,
    /// which includes information about the pin's group and position within that group.
    ///
    /// # Returns
    /// The `IOPin` representing this pin's TSC-specific configuration.
    pub fn tsc_io_pin(&self) -> IOPin {
        self.tsc_io_pin
    }
}

/// Represents a group of TSC (Touch Sensing Controller) pins.
///
/// In the TSC peripheral, pins are organized into groups of four IOs. Each group
/// must have exactly one sampling capacitor pin and can have multiple channel pins
/// or a single shield pin. This structure encapsulates these pin configurations
/// for a single TSC group.
///
/// # Pin Roles
/// - Sampling Capacitor: One required per group, used for charge transfer.
/// - Channel: Sensing pins connected to electrodes for touch detection.
/// - Shield: Optional, used for active shielding to improve sensitivity.
///
/// # Constraints
/// - Each group must have exactly one sampling capacitor pin.
/// - A group can have either channel pins or a shield pin, but not both.
/// - No more than one shield pin is allowed across all groups.
#[allow(missing_docs)]
pub struct PinGroup<'d, T, Group> {
    pin1: Option<Pin<'d, T, Group>>,
    pin2: Option<Pin<'d, T, Group>>,
    pin3: Option<Pin<'d, T, Group>>,
    pin4: Option<Pin<'d, T, Group>>,
}

impl<'d, T, G> Default for PinGroup<'d, T, G> {
    fn default() -> Self {
        Self {
            pin1: None,
            pin2: None,
            pin3: None,
            pin4: None,
        }
    }
}

/// Defines roles and traits for TSC (Touch Sensing Controller) pins.
///
/// This module contains marker types and traits that represent different roles
/// a TSC pin can have, such as channel, sample, or shield.
pub mod pin_roles {
    use super::{OutputType, PinType};

    /// Marker type for a TSC channel pin.
    #[derive(PartialEq, Clone, Copy, Debug)]
    pub struct Channel;

    /// Marker type for a TSC sampling pin.
    #[derive(PartialEq, Clone, Copy, Debug)]
    pub struct Sample;

    /// Marker type for a TSC shield pin.
    #[derive(PartialEq, Clone, Copy, Debug)]
    pub struct Shield;

    /// Trait for TSC pin roles.
    ///
    /// This trait defines the behavior and properties of different TSC pin roles.
    /// It is implemented by the marker types `Channel`, `Sample`, and `Shield`.
    pub trait Role {
        /// Returns the `PinType` associated with this role.
        fn pin_type() -> PinType;

        /// Returns the `OutputType` associated with this role.
        fn output_type() -> OutputType;
    }

    impl Role for Channel {
        fn pin_type() -> PinType {
            PinType::Channel
        }
        fn output_type() -> OutputType {
            OutputType::PushPull
        }
    }

    impl Role for Sample {
        fn pin_type() -> PinType {
            PinType::Sample
        }
        fn output_type() -> OutputType {
            OutputType::OpenDrain
        }
    }

    impl Role for Shield {
        fn pin_type() -> PinType {
            PinType::Shield
        }
        fn output_type() -> OutputType {
            OutputType::PushPull
        }
    }
}

/// Represents a group of TSC pins with their associated roles.
///
/// This struct allows for type-safe configuration of TSC pin groups,
/// ensuring that pins are assigned appropriate roles within their group.
/// This type is essentially just a wrapper type around a `PinGroup` value.
///
/// # Type Parameters
/// - `'d`: Lifetime of the pin group.
/// - `T`: The TSC instance type.
/// - `G`: The group identifier.
/// - `R1`, `R2`, `R3`, `R4`: Role types for each pin in the group, defaulting to `Channel`.
pub struct PinGroupWithRoles<
    'd,
    T: Instance,
    G,
    R1 = pin_roles::Channel,
    R2 = pin_roles::Channel,
    R3 = pin_roles::Channel,
    R4 = pin_roles::Channel,
> {
    /// The underlying pin group without role information.
    pub pin_group: PinGroup<'d, T, G>,
    _phantom: PhantomData<(R1, R2, R3, R4)>,
}

impl<'d, T: Instance, G, R1, R2, R3, R4> Default for PinGroupWithRoles<'d, T, G, R1, R2, R3, R4> {
    fn default() -> Self {
        Self {
            pin_group: PinGroup::default(),
            _phantom: PhantomData,
        }
    }
}

impl<'d, T: Instance, G> PinGroup<'d, T, G> {
    fn contains_exactly_one_shield_pin(&self) -> bool {
        let shield_count = self.shield_pins().count();
        shield_count == 1
    }

    fn check_group(&self) -> Result<(), GroupError> {
        let mut channel_count = 0;
        let mut shield_count = 0;
        let mut sample_count = 0;
        for pin in self.pins().into_iter().flatten() {
            match pin.role {
                PinType::Channel => {
                    channel_count += 1;
                }
                PinType::Shield => {
                    shield_count += 1;
                }
                PinType::Sample => {
                    sample_count += 1;
                }
            }
        }

        // Every group requires exactly one sampling capacitor
        if sample_count != 1 {
            return Err(GroupError::NoSamplingCapacitor);
        }

        // Each group must have at least one shield or channel IO
        if shield_count == 0 && channel_count == 0 {
            return Err(GroupError::NoChannelOrShield);
        }

        // Any group can either contain channel ios or a shield IO.
        // (An active shield requires its own sampling capacitor)
        if shield_count != 0 && channel_count != 0 {
            return Err(GroupError::MixedChannelAndShield);
        }

        // No more than one shield IO is allow per group and amongst all groups
        if shield_count > 1 {
            return Err(GroupError::MultipleShields);
        }

        Ok(())
    }

    /// Returns a reference to the first pin in the group, if configured.
    pub fn pin1(&self) -> Option<&Pin<'d, T, G>> {
        self.pin1.as_ref()
    }

    /// Returns a reference to the second pin in the group, if configured.
    pub fn pin2(&self) -> Option<&Pin<'d, T, G>> {
        self.pin2.as_ref()
    }

    /// Returns a reference to the third pin in the group, if configured.
    pub fn pin3(&self) -> Option<&Pin<'d, T, G>> {
        self.pin3.as_ref()
    }

    /// Returns a reference to the fourth pin in the group, if configured.
    pub fn pin4(&self) -> Option<&Pin<'d, T, G>> {
        self.pin4.as_ref()
    }

    fn sample_pins(&self) -> impl Iterator<Item = IOPin> + '_ {
        self.pins_filtered(PinType::Sample)
    }

    fn shield_pins(&self) -> impl Iterator<Item = IOPin> + '_ {
        self.pins_filtered(PinType::Shield)
    }

    fn channel_pins(&self) -> impl Iterator<Item = IOPin> + '_ {
        self.pins_filtered(PinType::Channel)
    }

    fn pins_filtered(&self, pin_type: PinType) -> impl Iterator<Item = IOPin> + '_ {
        self.pins().into_iter().filter_map(move |pin| {
            pin.as_ref()
                .and_then(|p| if p.role == pin_type { Some(p.tsc_io_pin) } else { None })
        })
    }

    fn make_channel_ios_mask(&self) -> u32 {
        self.channel_pins().fold(0, u32::bitor)
    }

    fn make_shield_ios_mask(&self) -> u32 {
        self.shield_pins().fold(0, u32::bitor)
    }

    fn make_sample_ios_mask(&self) -> u32 {
        self.sample_pins().fold(0, u32::bitor)
    }

    fn pins(&self) -> [&Option<Pin<'d, T, G>>; 4] {
        [&self.pin1, &self.pin2, &self.pin3, &self.pin4]
    }

    fn pins_mut(&mut self) -> [&mut Option<Pin<'d, T, G>>; 4] {
        [&mut self.pin1, &mut self.pin2, &mut self.pin3, &mut self.pin4]
    }
}

#[cfg(any(tsc_v2, tsc_v3))]
macro_rules! TSC_V2_V3_GUARD {
    ($e:expr) => {{
        #[cfg(any(tsc_v2, tsc_v3))]
        {
            $e
        }
        #[cfg(not(any(tsc_v2, tsc_v3)))]
        {
            compile_error!("Group 7 is not supported in this TSC version")
        }
    }};
}

#[cfg(tsc_v3)]
macro_rules! TSC_V3_GUARD {
    ($e:expr) => {{
        #[cfg(tsc_v3)]
        {
            $e
        }
        #[cfg(not(tsc_v3))]
        {
            compile_error!("Group 8 is not supported in this TSC version")
        }
    }};
}

macro_rules! trait_to_io_pin {
    (G1IO1Pin) => {
        IOPin::Group1Io1
    };
    (G1IO2Pin) => {
        IOPin::Group1Io2
    };
    (G1IO3Pin) => {
        IOPin::Group1Io3
    };
    (G1IO4Pin) => {
        IOPin::Group1Io4
    };

    (G2IO1Pin) => {
        IOPin::Group2Io1
    };
    (G2IO2Pin) => {
        IOPin::Group2Io2
    };
    (G2IO3Pin) => {
        IOPin::Group2Io3
    };
    (G2IO4Pin) => {
        IOPin::Group2Io4
    };

    (G3IO1Pin) => {
        IOPin::Group3Io1
    };
    (G3IO2Pin) => {
        IOPin::Group3Io2
    };
    (G3IO3Pin) => {
        IOPin::Group3Io3
    };
    (G3IO4Pin) => {
        IOPin::Group3Io4
    };

    (G4IO1Pin) => {
        IOPin::Group4Io1
    };
    (G4IO2Pin) => {
        IOPin::Group4Io2
    };
    (G4IO3Pin) => {
        IOPin::Group4Io3
    };
    (G4IO4Pin) => {
        IOPin::Group4Io4
    };

    (G5IO1Pin) => {
        IOPin::Group5Io1
    };
    (G5IO2Pin) => {
        IOPin::Group5Io2
    };
    (G5IO3Pin) => {
        IOPin::Group5Io3
    };
    (G5IO4Pin) => {
        IOPin::Group5Io4
    };

    (G6IO1Pin) => {
        IOPin::Group6Io1
    };
    (G6IO2Pin) => {
        IOPin::Group6Io2
    };
    (G6IO3Pin) => {
        IOPin::Group6Io3
    };
    (G6IO4Pin) => {
        IOPin::Group6Io4
    };

    (G7IO1Pin) => {
        TSC_V2_V3_GUARD!(IOPin::Group7Io1)
    };
    (G7IO2Pin) => {
        TSC_V2_V3_GUARD!(IOPin::Group7Io2)
    };
    (G7IO3Pin) => {
        TSC_V2_V3_GUARD!(IOPin::Group7Io3)
    };
    (G7IO4Pin) => {
        TSC_V2_V3_GUARD!(IOPin::Group7Io4)
    };

    (G8IO1Pin) => {
        TSC_V3_GUARD!(IOPin::Group8Io1)
    };
    (G8IO2Pin) => {
        TSC_V3_GUARD!(IOPin::Group8Io2)
    };
    (G8IO3Pin) => {
        TSC_V3_GUARD!(IOPin::Group8Io3)
    };
    (G8IO4Pin) => {
        TSC_V3_GUARD!(IOPin::Group8Io4)
    };
}

macro_rules! impl_set_io {
    ($method:ident, $group:ident, $trait:ident, $index:expr) => {
        #[doc = concat!("Create a new pin1 for ", stringify!($group), " TSC group instance.")]
        pub fn $method<Role: pin_roles::Role>(&mut self, pin: Peri<'d, impl $trait<T>>) -> IOPinWithRole<$group, Role> {
            critical_section::with(|_| {
                pin.set_low();
                pin.set_as_af(pin.af_num(), AfType::output(Role::output_type(), Speed::VeryHigh));
                let tsc_io_pin = trait_to_io_pin!($trait);
                let new_pin = Pin {
                    _pin: pin.into(),
                    role: Role::pin_type(),
                    tsc_io_pin,
                    phantom: PhantomData,
                };
                *self.pin_group.pins_mut()[$index] = Some(new_pin);
                IOPinWithRole {
                    pin: tsc_io_pin,
                    phantom: PhantomData,
                }
            })
        }
    };
}

macro_rules! group_impl {
    ($group:ident, $trait1:ident, $trait2:ident, $trait3:ident, $trait4:ident) => {
        impl<'d, T: Instance, R1: pin_roles::Role, R2: pin_roles::Role, R3: pin_roles::Role, R4: pin_roles::Role>
            PinGroupWithRoles<'d, T, $group, R1, R2, R3, R4>
        {
            impl_set_io!(set_io1, $group, $trait1, 0);
            impl_set_io!(set_io2, $group, $trait2, 1);
            impl_set_io!(set_io3, $group, $trait3, 2);
            impl_set_io!(set_io4, $group, $trait4, 3);
        }
    };
}

group_impl!(G1, G1IO1Pin, G1IO2Pin, G1IO3Pin, G1IO4Pin);
group_impl!(G2, G2IO1Pin, G2IO2Pin, G2IO3Pin, G2IO4Pin);
group_impl!(G3, G3IO1Pin, G3IO2Pin, G3IO3Pin, G3IO4Pin);
group_impl!(G4, G4IO1Pin, G4IO2Pin, G4IO3Pin, G4IO4Pin);
group_impl!(G5, G5IO1Pin, G5IO2Pin, G5IO3Pin, G5IO4Pin);
group_impl!(G6, G6IO1Pin, G6IO2Pin, G6IO3Pin, G6IO4Pin);
#[cfg(any(tsc_v2, tsc_v3))]
group_impl!(G7, G7IO1Pin, G7IO2Pin, G7IO3Pin, G7IO4Pin);
#[cfg(tsc_v3)]
group_impl!(G8, G8IO1Pin, G8IO2Pin, G8IO3Pin, G8IO4Pin);

/// Group 1 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G1 {}
/// Group 2 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G2 {}
/// Group 3 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G3 {}
/// Group 4 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G4 {}
/// Group 5 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G5 {}
/// Group 6 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G6 {}
/// Group 7 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G7 {}
/// Group 8 marker type.
#[derive(Clone, Copy, Debug)]
pub enum G8 {}

/// Represents the collection of pin groups for the Touch Sensing Controller (TSC).
///
/// Each field corresponds to a specific group of TSC pins:
#[allow(missing_docs)]
pub struct PinGroups<'d, T: Instance> {
    pub g1: Option<PinGroup<'d, T, G1>>,
    pub g2: Option<PinGroup<'d, T, G2>>,
    pub g3: Option<PinGroup<'d, T, G3>>,
    pub g4: Option<PinGroup<'d, T, G4>>,
    pub g5: Option<PinGroup<'d, T, G5>>,
    pub g6: Option<PinGroup<'d, T, G6>>,
    #[cfg(any(tsc_v2, tsc_v3))]
    pub g7: Option<PinGroup<'d, T, G7>>,
    #[cfg(tsc_v3)]
    pub g8: Option<PinGroup<'d, T, G8>>,
}

impl<'d, T: Instance> PinGroups<'d, T> {
    pub(super) fn check(&self) -> Result<(), GroupError> {
        let mut shield_count = 0;

        // Helper function to check a single group
        fn check_group<C, T: Instance>(
            group: &Option<PinGroup<'_, T, C>>,
            shield_count: &mut u32,
        ) -> Result<(), GroupError> {
            if let Some(group) = group {
                group.check_group()?;
                if group.contains_exactly_one_shield_pin() {
                    *shield_count += 1;
                    if *shield_count > 1 {
                        return Err(GroupError::MultipleShields);
                    }
                }
            }
            Ok(())
        }

        // Check each group
        check_group(&self.g1, &mut shield_count)?;
        check_group(&self.g2, &mut shield_count)?;
        check_group(&self.g3, &mut shield_count)?;
        check_group(&self.g4, &mut shield_count)?;
        check_group(&self.g5, &mut shield_count)?;
        check_group(&self.g6, &mut shield_count)?;
        #[cfg(any(tsc_v2, tsc_v3))]
        check_group(&self.g7, &mut shield_count)?;
        #[cfg(tsc_v3)]
        check_group(&self.g8, &mut shield_count)?;

        Ok(())
    }

    pub(super) fn make_channel_ios_mask(&self) -> u32 {
        #[allow(unused_mut)]
        let mut mask = self.g1.as_ref().map_or(0, |g| g.make_channel_ios_mask())
            | self.g2.as_ref().map_or(0, |g| g.make_channel_ios_mask())
            | self.g3.as_ref().map_or(0, |g| g.make_channel_ios_mask())
            | self.g4.as_ref().map_or(0, |g| g.make_channel_ios_mask())
            | self.g5.as_ref().map_or(0, |g| g.make_channel_ios_mask())
            | self.g6.as_ref().map_or(0, |g| g.make_channel_ios_mask());
        #[cfg(any(tsc_v2, tsc_v3))]
        {
            mask |= self.g7.as_ref().map_or(0, |g| g.make_channel_ios_mask());
        }
        #[cfg(tsc_v3)]
        {
            mask |= self.g8.as_ref().map_or(0, |g| g.make_channel_ios_mask());
        }
        mask
    }

    pub(super) fn make_shield_ios_mask(&self) -> u32 {
        #[allow(unused_mut)]
        let mut mask = self.g1.as_ref().map_or(0, |g| g.make_shield_ios_mask())
            | self.g2.as_ref().map_or(0, |g| g.make_shield_ios_mask())
            | self.g3.as_ref().map_or(0, |g| g.make_shield_ios_mask())
            | self.g4.as_ref().map_or(0, |g| g.make_shield_ios_mask())
            | self.g5.as_ref().map_or(0, |g| g.make_shield_ios_mask())
            | self.g6.as_ref().map_or(0, |g| g.make_shield_ios_mask());
        #[cfg(any(tsc_v2, tsc_v3))]
        {
            mask |= self.g7.as_ref().map_or(0, |g| g.make_shield_ios_mask());
        }
        #[cfg(tsc_v3)]
        {
            mask |= self.g8.as_ref().map_or(0, |g| g.make_shield_ios_mask());
        }
        mask
    }

    pub(super) fn make_sample_ios_mask(&self) -> u32 {
        #[allow(unused_mut)]
        let mut mask = self.g1.as_ref().map_or(0, |g| g.make_sample_ios_mask())
            | self.g2.as_ref().map_or(0, |g| g.make_sample_ios_mask())
            | self.g3.as_ref().map_or(0, |g| g.make_sample_ios_mask())
            | self.g4.as_ref().map_or(0, |g| g.make_sample_ios_mask())
            | self.g5.as_ref().map_or(0, |g| g.make_sample_ios_mask())
            | self.g6.as_ref().map_or(0, |g| g.make_sample_ios_mask());
        #[cfg(any(tsc_v2, tsc_v3))]
        {
            mask |= self.g7.as_ref().map_or(0, |g| g.make_sample_ios_mask());
        }
        #[cfg(tsc_v3)]
        {
            mask |= self.g8.as_ref().map_or(0, |g| g.make_sample_ios_mask());
        }
        mask
    }
}

impl<'d, T: Instance> Default for PinGroups<'d, T> {
    fn default() -> Self {
        Self {
            g1: None,
            g2: None,
            g3: None,
            g4: None,
            g5: None,
            g6: None,
            #[cfg(any(tsc_v2, tsc_v3))]
            g7: None,
            #[cfg(tsc_v3)]
            g8: None,
        }
    }
}

pin_trait!(G1IO1Pin, Instance);
pin_trait!(G1IO2Pin, Instance);
pin_trait!(G1IO3Pin, Instance);
pin_trait!(G1IO4Pin, Instance);

pin_trait!(G2IO1Pin, Instance);
pin_trait!(G2IO2Pin, Instance);
pin_trait!(G2IO3Pin, Instance);
pin_trait!(G2IO4Pin, Instance);

pin_trait!(G3IO1Pin, Instance);
pin_trait!(G3IO2Pin, Instance);
pin_trait!(G3IO3Pin, Instance);
pin_trait!(G3IO4Pin, Instance);

pin_trait!(G4IO1Pin, Instance);
pin_trait!(G4IO2Pin, Instance);
pin_trait!(G4IO3Pin, Instance);
pin_trait!(G4IO4Pin, Instance);

pin_trait!(G5IO1Pin, Instance);
pin_trait!(G5IO2Pin, Instance);
pin_trait!(G5IO3Pin, Instance);
pin_trait!(G5IO4Pin, Instance);

pin_trait!(G6IO1Pin, Instance);
pin_trait!(G6IO2Pin, Instance);
pin_trait!(G6IO3Pin, Instance);
pin_trait!(G6IO4Pin, Instance);

pin_trait!(G7IO1Pin, Instance);
pin_trait!(G7IO2Pin, Instance);
pin_trait!(G7IO3Pin, Instance);
pin_trait!(G7IO4Pin, Instance);

pin_trait!(G8IO1Pin, Instance);
pin_trait!(G8IO2Pin, Instance);
pin_trait!(G8IO3Pin, Instance);
pin_trait!(G8IO4Pin, Instance);
