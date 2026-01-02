use core::future::poll_fn;
use core::marker::PhantomData;
use core::ops::BitOr;
use core::task::Poll;

use embassy_hal_internal::Peri;

use super::acquisition_banks::*;
use super::config::*;
use super::errors::*;
use super::io_pin::*;
use super::pin_groups::*;
use super::types::*;
use super::{Instance, InterruptHandler, TSC_NUM_GROUPS};
use crate::interrupt::typelevel::Interrupt;
use crate::mode::{Async, Blocking, Mode as PeriMode};
use crate::{interrupt, rcc};

/// Internal structure holding masks for different types of TSC IOs.
///
/// These masks are used during the initial configuration of the TSC peripheral
/// and for validating pin types during operations like creating acquisition banks.
struct IOMasks {
    /// Mask representing all configured channel IOs
    channel_ios: u32,
    /// Mask representing all configured shield IOs
    shield_ios: u32,
    /// Mask representing all configured sampling IOs
    sampling_ios: u32,
}

/// TSC driver
pub struct Tsc<'d, T: Instance, K: PeriMode> {
    _peri: Peri<'d, T>,
    _pin_groups: PinGroups<'d, T>,
    state: State,
    config: Config,
    masks: IOMasks,
    _kind: PhantomData<K>,
}

impl<'d, T: Instance, K: PeriMode> Tsc<'d, T, K> {
    // Helper method to check if a pin is a channel pin
    fn is_channel_pin(&self, pin: IOPin) -> bool {
        (self.masks.channel_ios & pin) != 0
    }

    /// Get the status of all groups involved in a AcquisitionBank
    pub fn get_acquisition_bank_status(&self, bank: &AcquisitionBank) -> AcquisitionBankStatus {
        let mut bank_status = AcquisitionBankStatus::default();
        for pin in bank.pins_iterator() {
            let group = pin.group();
            let group_status = self.group_get_status(group);
            let index: usize = group.into();
            bank_status.groups[index] = Some(group_status);
        }
        bank_status
    }

    /// Get the values for all channels involved in a AcquisitionBank
    pub fn get_acquisition_bank_values(&self, bank: &AcquisitionBank) -> AcquisitionBankReadings {
        let mut bank_readings = AcquisitionBankReadings::default();
        for pin in bank.pins_iterator() {
            let group = pin.group();
            let value = self.group_get_value(group);
            let reading = ChannelReading {
                sensor_value: value,
                tsc_pin: pin,
            };
            let index: usize = group.into();
            bank_readings.groups[index] = Some(reading);
        }
        bank_readings
    }

    /// Creates a new TSC acquisition bank from the provided pin configuration.
    ///
    /// This method creates a `AcquisitionBank` that can be used for efficient,
    /// repeated TSC acquisitions. It automatically generates the appropriate mask
    /// for the provided pins.
    ///
    /// # Note on TSC Hardware Limitation
    ///
    /// The TSC hardware can only read one channel pin from each TSC group per acquisition.
    ///
    /// # Arguments
    /// * `acquisition_bank_pins` - The pin configuration for the acquisition bank.
    ///
    /// # Returns
    /// A new `AcquisitionBank` instance.
    ///
    /// # Example
    ///
    /// ```
    /// let tsc = // ... initialize TSC
    /// let tsc_sensor1: tsc::IOPinWithRole<G1, tsc_pin_roles::Channel> = ...;
    /// let tsc_sensor2: tsc::IOPinWithRole<G2, tsc_pin_roles::Channel> = ...;
    ///
    /// let bank = tsc.create_acquisition_bank(AcquisitionBankPins {
    ///     g1_pin: Some(tsc_sensor1),
    ///     g2_pin: Some(tsc_sensor2),
    ///     ..Default::default()
    /// });
    ///
    /// // Use the bank for acquisitions
    /// tsc.set_active_channels_bank(&bank);
    /// tsc.start();
    /// // ... perform acquisition ...
    /// ```
    pub fn create_acquisition_bank(&self, acquisition_bank_pins: AcquisitionBankPins) -> AcquisitionBank {
        let bank_mask = acquisition_bank_pins.iter().fold(0u32, BitOr::bitor);

        AcquisitionBank {
            pins: acquisition_bank_pins,
            mask: bank_mask,
        }
    }

    fn make_channels_mask<Itt>(&self, channels: Itt) -> Result<u32, AcquisitionBankError>
    where
        Itt: IntoIterator<Item = IOPin>,
    {
        let mut group_mask = 0u32;
        let mut channel_mask = 0u32;

        for channel in channels {
            if !self.is_channel_pin(channel) {
                return Err(AcquisitionBankError::InvalidChannelPin);
            }

            let group = channel.group();
            let group_bit: u32 = 1 << Into::<usize>::into(group);
            if group_mask & group_bit != 0 {
                return Err(AcquisitionBankError::MultipleChannelsPerGroup);
            }

            group_mask |= group_bit;
            channel_mask |= channel;
        }

        Ok(channel_mask)
    }

    /// Sets the active channels for the next TSC acquisition.
    ///
    /// This is a low-level method that directly sets the channel mask. For most use cases,
    /// consider using `set_active_channels_bank` with a `AcquisitionBank` instead, which
    /// provides a higher-level interface and additional safety checks.
    ///
    /// This method configures which sensor channels will be read during the next
    /// touch sensing acquisition cycle. It should be called before starting a new
    /// acquisition with the start() method.
    ///
    /// # Arguments
    /// * `mask` - A 32-bit mask where each bit represents a channel. Set bits indicate
    ///            active channels.
    ///
    /// # Note
    /// Only one pin from each TSC group can be read for each acquisition. This method
    /// does not perform checks to ensure this limitation is met. Incorrect masks may
    /// lead to unexpected behavior.
    ///
    /// # Safety
    /// This method doesn't perform extensive checks on the provided mask. Ensure that
    /// the mask is valid and adheres to hardware limitations to avoid undefined behavior.
    pub fn set_active_channels_mask(&mut self, mask: u32) {
        T::regs().ioccr().write(|w| w.0 = mask | self.masks.shield_ios);
    }

    /// Convenience method for setting active channels directly from a slice of tsc::IOPin.
    /// This method performs safety checks but is less efficient for repeated use.
    pub fn set_active_channels(&mut self, channels: &[IOPin]) -> Result<(), AcquisitionBankError> {
        let mask = self.make_channels_mask(channels.iter().cloned())?;
        self.set_active_channels_mask(mask);
        Ok(())
    }

    /// Sets the active channels for the next TSC acquisition using a pre-configured acquisition bank.
    ///
    /// This method efficiently configures the TSC peripheral to read the channels specified
    /// in the provided `AcquisitionBank`. It's the recommended way to set up
    /// channel configurations for acquisition, especially when using the same set of channels repeatedly.
    ///
    /// # Arguments
    ///
    /// * `bank` - A reference to a `AcquisitionBank` containing the pre-configured
    ///            TSC channel mask.
    ///
    /// # Example
    ///
    /// ```
    /// let tsc_sensor1: tsc::IOPinWithRole<G1, Channel> = ...;
    /// let tsc_sensor2: tsc::IOPinWithRole<G5, Channel> = ...;
    /// let mut touch_controller: Tsc<'_, TSC, Async> = ...;
    /// let bank = touch_controller.create_acquisition_bank(AcquisitionBankPins {
    ///     g1_pin: Some(tsc_sensor1),
    ///     g2_pin: Some(tsc_sensor2),
    ///     ..Default::default()
    /// });
    ///
    /// touch_controller.set_active_channels_bank(&bank);
    /// touch_controller.start();
    /// // ... perform acquisition ...
    /// ```
    ///
    /// This method should be called before starting a new acquisition with the `start()` method.
    pub fn set_active_channels_bank(&mut self, bank: &AcquisitionBank) {
        self.set_active_channels_mask(bank.mask)
    }

    fn extract_groups(io_mask: u32) -> u32 {
        let mut groups: u32 = 0;
        for idx in 0..TSC_NUM_GROUPS {
            if io_mask & (0x0F << (idx * 4)) != 0 {
                groups |= 1 << idx
            }
        }
        groups
    }

    fn new_inner(peri: Peri<'d, T>, pin_groups: PinGroups<'d, T>, config: Config) -> Result<Self, GroupError> {
        pin_groups.check()?;

        let masks = IOMasks {
            channel_ios: pin_groups.make_channel_ios_mask(),
            shield_ios: pin_groups.make_shield_ios_mask(),
            sampling_ios: pin_groups.make_sample_ios_mask(),
        };

        rcc::enable_and_reset::<T>();

        T::regs().cr().modify(|w| {
            w.set_tsce(true);
            w.set_ctph(config.ct_pulse_high_length.into());
            w.set_ctpl(config.ct_pulse_low_length.into());
            w.set_sse(config.spread_spectrum);
            // Prevent invalid configuration for pulse generator prescaler
            if config.ct_pulse_low_length == ChargeTransferPulseCycle::_1
                && (config.pulse_generator_prescaler == PGPrescalerDivider::_1
                    || config.pulse_generator_prescaler == PGPrescalerDivider::_2)
            {
                w.set_pgpsc(PGPrescalerDivider::_4.into());
            } else if config.ct_pulse_low_length == ChargeTransferPulseCycle::_2
                && config.pulse_generator_prescaler == PGPrescalerDivider::_1
            {
                w.set_pgpsc(PGPrescalerDivider::_2.into());
            } else {
                w.set_pgpsc(config.pulse_generator_prescaler.into());
            }
            w.set_ssd(config.spread_spectrum_deviation.into());
            w.set_sspsc(config.spread_spectrum_prescaler);

            w.set_mcv(config.max_count_value.into());
            w.set_syncpol(config.synchro_pin_polarity);
            w.set_am(config.acquisition_mode);
        });

        // Set IO configuration
        // Disable Schmitt trigger hysteresis on all used TSC IOs
        T::regs()
            .iohcr()
            .write(|w| w.0 = !(masks.channel_ios | masks.shield_ios | masks.sampling_ios));

        // Set channel and shield IOs
        T::regs().ioccr().write(|w| w.0 = masks.channel_ios | masks.shield_ios);

        // Set sampling IOs
        T::regs().ioscr().write(|w| w.0 = masks.sampling_ios);

        // Set the groups to be acquired
        // Lower bits of `iogcsr` are for enabling groups, while the higher bits are for reading
        // status of acquisiton for a group, see method `Tsc::group_get_status`.
        T::regs()
            .iogcsr()
            .write(|w| w.0 = Self::extract_groups(masks.channel_ios));

        // Disable interrupts
        T::regs().ier().modify(|w| {
            w.set_eoaie(false);
            w.set_mceie(false);
        });

        // Clear flags
        T::regs().icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        unsafe {
            T::Interrupt::enable();
        }

        Ok(Self {
            _peri: peri,
            _pin_groups: pin_groups,
            state: State::Ready,
            config,
            masks,
            _kind: PhantomData,
        })
    }

    /// Start charge transfer acquisition
    pub fn start(&mut self) {
        self.state = State::Busy;

        // Disable interrupts
        T::regs().ier().modify(|w| {
            w.set_eoaie(false);
            w.set_mceie(false);
        });

        // Clear flags
        T::regs().icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        // Set the touch sensing IOs not acquired to the default mode
        T::regs().cr().modify(|w| {
            w.set_iodef(self.config.io_default_mode);
        });

        // Start the acquisition
        T::regs().cr().modify(|w| {
            w.set_start(true);
        });
    }

    /// Stop charge transfer acquisition
    pub fn stop(&mut self) {
        T::regs().cr().modify(|w| {
            w.set_start(false);
        });

        // Set the touch sensing IOs in low power mode
        T::regs().cr().modify(|w| {
            w.set_iodef(false);
        });

        // Clear flags
        T::regs().icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        self.state = State::Ready;
    }

    /// Get current state of acquisition
    pub fn get_state(&mut self) -> State {
        if self.state == State::Busy && T::regs().isr().read().eoaf() {
            if T::regs().isr().read().mcef() {
                self.state = State::Error
            } else {
                self.state = State::Ready
            }
        }
        self.state
    }

    /// Get the individual group status to check acquisition complete
    pub fn group_get_status(&self, index: Group) -> GroupStatus {
        // Status bits are set by hardware when the acquisition on the corresponding
        // enabled analog IO group is complete, cleared when new acquisition is started
        let status = match index {
            Group::One => T::regs().iogcsr().read().g1s(),
            Group::Two => T::regs().iogcsr().read().g2s(),
            Group::Three => T::regs().iogcsr().read().g3s(),
            Group::Four => T::regs().iogcsr().read().g4s(),
            Group::Five => T::regs().iogcsr().read().g5s(),
            Group::Six => T::regs().iogcsr().read().g6s(),
            #[cfg(any(tsc_v2, tsc_v3))]
            Group::Seven => T::regs().iogcsr().read().g7s(),
            #[cfg(tsc_v3)]
            Group::Eight => T::regs().iogcsr().read().g8s(),
        };
        match status {
            true => GroupStatus::Complete,
            false => GroupStatus::Ongoing,
        }
    }

    /// Get the count for the acquisiton, valid once group status is set
    pub fn group_get_value(&self, index: Group) -> u16 {
        T::regs().iogcr(index.into()).read().cnt()
    }

    /// Discharge the IOs for subsequent acquisition
    pub fn discharge_io(&mut self, status: bool) {
        // Set the touch sensing IOs in low power mode
        T::regs().cr().modify(|w| {
            w.set_iodef(!status);
        });
    }
}

impl<'d, T: Instance, K: PeriMode> Drop for Tsc<'d, T, K> {
    fn drop(&mut self) {
        rcc::disable::<T>();
    }
}

impl<'d, T: Instance> Tsc<'d, T, Async> {
    /// Create a Tsc instance that can be awaited for completion
    pub fn new_async(
        peri: Peri<'d, T>,
        pin_groups: PinGroups<'d, T>,
        config: Config,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Result<Self, GroupError> {
        Self::new_inner(peri, pin_groups, config)
    }

    /// Asyncronously wait for the end of an acquisition
    pub async fn pend_for_acquisition(&mut self) {
        poll_fn(|cx| match self.get_state() {
            State::Busy => {
                T::waker().register(cx.waker());
                T::regs().ier().write(|w| w.set_eoaie(true));
                if self.get_state() != State::Busy {
                    T::regs().ier().write(|w| w.set_eoaie(false));
                    return Poll::Ready(());
                }
                Poll::Pending
            }
            _ => {
                T::regs().ier().write(|w| w.set_eoaie(false));
                Poll::Ready(())
            }
        })
        .await;
    }
}

impl<'d, T: Instance> Tsc<'d, T, Blocking> {
    /// Create a Tsc instance that must be polled for completion
    pub fn new_blocking(peri: Peri<'d, T>, pin_groups: PinGroups<'d, T>, config: Config) -> Result<Self, GroupError> {
        Self::new_inner(peri, pin_groups, config)
    }

    /// Wait for end of acquisition
    pub fn poll_for_acquisition(&mut self) {
        while self.get_state() == State::Busy {}
    }
}
