use embassy_hal_internal::{Peri, PeripheralType};
use nxp_pac::dac::{BufSpdCtrl, Dacrfs, Fifoen, Fiforst, Swrst, Trgsel};
use nxp_pac::port::Mux;

use super::clocks::PoweredClock;
use crate::clkout::Div4;
use crate::clocks::periph_helpers::DacConfig;
use crate::clocks::{ClockError, Gate, WakeGuard, enable_and_reset};
use crate::gpio::GpioPin;

pub struct Dac {
    regs: crate::pac::dac::Dac,
    _power: DacPower,
    _wg: Option<WakeGuard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum InitError {
    ClockInit(ClockError),
}

impl From<ClockError> for InitError {
    fn from(value: ClockError) -> Self {
        InitError::ClockInit(value)
    }
}

/// Power guard. Sets the given bit in SPC soc_cntrl and resets it on drop
pub(crate) struct DacPower {
    bit: u8,
}

impl DacPower {
    pub fn power_on(bit: u8) -> Self {
        let spc = crate::pac::SPC0;
        spc.active_cfg1()
            .modify(|w| w.set_soc_cntrl(w.soc_cntrl() | (1 << bit)));
        spc.lp_cfg1().modify(|w| w.set_soc_cntrl(w.soc_cntrl() | (1 << bit)));
        DacPower { bit }
    }
}

impl Drop for DacPower {
    fn drop(&mut self) {
        let spc = crate::pac::SPC0;
        spc.active_cfg1()
            .modify(|w| w.set_soc_cntrl(w.soc_cntrl() & !(1 << self.bit)));
        spc.lp_cfg1()
            .modify(|w| w.set_soc_cntrl(w.soc_cntrl() & !(1 << self.bit)));
    }
}

pub(crate) mod sealed {
    use crate::clocks::periph_helpers::DacInstance;

    pub trait SealedPin {}

    pub trait SealedInstance {
        const SOC_CNTRL_BIT: u8;
        const INSTANCE: DacInstance;
        fn regs() -> crate::pac::dac::Dac;
    }
}

pub trait Instance: sealed::SealedInstance + PeripheralType + Gate<MrccPeriphConfig = DacConfig> {}
pub trait DacPin<Instance>: sealed::SealedPin + GpioPin {}

impl Dac {
    /// Create a dac instance.
    pub fn new<P: Instance>(
        _instance: Peri<'static, P>,
        pin: Peri<'static, impl DacPin<P>>,
    ) -> Result<Self, InitError> {
        let clock = unsafe {
            enable_and_reset::<P>(&DacConfig {
                div: Div4::no_div(),
                power: PoweredClock::AlwaysEnabled,
                instance: P::INSTANCE,
            })?
        };
        let power = DacPower::power_on(<P as sealed::SealedInstance>::SOC_CNTRL_BIT);

        pin.set_pull(crate::gpio::Pull::Disabled);
        pin.set_slew_rate(crate::gpio::SlewRate::Fast.into());
        pin.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
        pin.set_function(Mux::Mux0);

        let dac = <P as sealed::SealedInstance>::regs();

        dac.rcr().modify(|w| {
            w.set_swrst(Swrst::SoftwareReset);
            w.set_fiforst(Fiforst::FifoReset);
        });
        dac.rcr().modify(|w| {
            w.set_swrst(Swrst::NoEffect);
            w.set_fiforst(Fiforst::NoEffect);
        });

        dac.gcr().write(|w| {
            w.set_dacrfs(Dacrfs::Vrefh0);
            w.set_fifoen(Fifoen::BufferMode);
            w.set_buf_en(true);
            w.set_buf_spd_ctrl(BufSpdCtrl::LlpMode);
            w.set_trgsel(Trgsel::Hardware);
            w.set_iref_ptat_ext_sel(true);
            w.set_latch_cyc(1);
            w.set_dacen(true);
        });
        Ok(Self {
            _wg: clock.wake_guard,
            _power: power,
            regs: dac,
        })
    }

    /// Write to the dac.
    /// This will immediately trigger conversion.
    /// The output value can be between 0 and 4095.
    /// The voltage produced will be value/4095*Vref.
    pub fn write(&self, value: u16) {
        self.regs.data().write(|w| w.set_data(value));
    }
}

impl Drop for Dac {
    fn drop(&mut self) {
        self.regs.gcr().write(|w| w.set_dacen(false));
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_dac_instance {
    ($n:literal) => {
        paste::paste! {
            impl crate::dac::sealed::SealedInstance for crate::peripherals::[<DAC $n>] {
                const SOC_CNTRL_BIT: u8 = 4 + $n;
                const INSTANCE: crate::clocks::periph_helpers::DacInstance = crate::clocks::periph_helpers::DacInstance::[<Dac $n>];

                fn regs() -> crate::pac::dac::Dac {
                    crate::pac::[<DAC $n>]
                }
            }

            impl crate::dac::Instance for crate::peripherals::[<DAC $n>] {}
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_dac_pin {
    ($pin:ident, $peri:ident) => {
        impl crate::dac::sealed::SealedPin for crate::peripherals::$pin {}
        impl crate::dac::DacPin<crate::peripherals::$peri> for crate::peripherals::$pin {}
    };
}
