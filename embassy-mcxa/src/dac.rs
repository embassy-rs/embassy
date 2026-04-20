use embassy_hal_internal::{Peri, PeripheralType};
use nxp_pac::dac::vals::{BufEn, BufSpdCtrl, Dacrfs, Fifoen, Fiforst, Swrst, Trgsel};
use nxp_pac::port::vals::Mux;

use super::clocks::PoweredClock;
use crate::clkout::Div4;
use crate::clocks::periph_helpers::DacConfig;
use crate::clocks::{ClockError, Gate, WakeGuard, enable_and_reset};
use crate::pac::DAC0;
use crate::peripherals::DAC0;

pub struct Dac {
    dac: crate::pac::dac::Dac,
    _power: sealed::DacPower,
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

mod sealed {
    pub struct DacPower {
        offset: u8,
    }

    impl DacPower {
        pub fn power_on(offset: u8) -> Self {
            let spc = crate::pac::SPC0;
            spc.active_cfg1()
                .modify(|w| w.set_soc_cntrl((1 << offset) | w.soc_cntrl()));
            spc.lp_cfg1().modify(|w| w.set_soc_cntrl((1 << offset) | w.soc_cntrl()));
            DacPower { offset }
        }
    }

    impl Drop for DacPower {
        fn drop(&mut self) {
            let spc = crate::pac::SPC0;
            spc.active_cfg1()
                .modify(|w| w.set_soc_cntrl((0 << self.offset) | w.soc_cntrl()));
            spc.lp_cfg1()
                .modify(|w| w.set_soc_cntrl((0 << self.offset) | w.soc_cntrl()));
        }
    }

    pub trait SealedPin {}
    pub trait SealedInstance {
        fn power_on() -> DacPower;
        fn dac() -> crate::pac::dac::Dac;
    }
}

pub trait Instance: sealed::SealedInstance + PeripheralType + Gate<MrccPeriphConfig = DacConfig> {}
pub trait DacPin: sealed::SealedPin + PeripheralType {
    type Instance: Instance;
}

impl Instance for crate::peripherals::DAC0 {}
impl sealed::SealedInstance for crate::peripherals::DAC0 {
    fn power_on() -> sealed::DacPower {
        sealed::DacPower::power_on(4)
    }
    fn dac() -> crate::pac::dac::Dac {
        crate::pac::DAC0
    }
}

impl DacPin for crate::peripherals::P2_2 {
    type Instance = DAC0;
}
impl sealed::SealedPin for crate::peripherals::P2_2 {}

impl Dac {
    /// Create a dac instance.
    pub fn new<P: DacPin + crate::gpio::GpioPin>(
        _instance: Peri<'static, P::Instance>,
        pin: Peri<'static, P>,
    ) -> Result<Self, InitError> {
        let clock = unsafe {
            enable_and_reset::<P::Instance>(&DacConfig {
                div: Div4::no_div(),
                power: PoweredClock::AlwaysEnabled,
            })?
        };
        let power = <P::Instance as sealed::SealedInstance>::power_on();

        pin.set_pull(crate::gpio::Pull::Disabled);
        pin.set_slew_rate(crate::gpio::SlewRate::Fast.into());
        pin.set_drive_strength(crate::gpio::DriveStrength::Normal.into());
        pin.set_function(Mux::MUX0);

        let dac = <P::Instance as sealed::SealedInstance>::dac();

        dac.rcr().modify(|w| {
            w.set_swrst(Swrst::SOFTWARE_RESET);
            w.set_fiforst(Fiforst::FIFO_RESET);
        });
        dac.rcr().modify(|w| {
            w.set_swrst(Swrst::NO_EFFECT);
            w.set_fiforst(Fiforst::NO_EFFECT);
        });

        dac.gcr().write(|w| {
            w.set_dacrfs(Dacrfs::VREFH0);
            w.set_fifoen(Fifoen::BUFFER_MODE);
            w.set_buf_en(BufEn::NO_USE_BUF);
            w.set_buf_spd_ctrl(BufSpdCtrl::LLP_MODE);
            w.set_trgsel(Trgsel::HARDWARE);
            w.set_iref_ptat_ext_sel(true);
            w.set_latch_cyc(1);
            w.set_dacen(true);
        });
        Ok(Self {
            _wg: clock.wake_guard,
            _power: power,
            dac,
        })
    }

    /// Write to the dac.
    /// This will immediately trigger conversion.
    /// The output value can be between 0 and 4095.
    /// The voltage produced will be value/4095*Vref.
    pub fn write(&self, value: u16) {
        let dac0 = DAC0;
        dac0.data().write(|w| w.set_data(value));
    }
}

impl Drop for Dac {
    fn drop(&mut self) {
        self.dac.gcr().write(|w| w.set_dacen(false));
    }
}
