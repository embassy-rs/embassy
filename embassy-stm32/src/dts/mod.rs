//! Digital Temperature Sensor (DTS)

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::InterruptExt;
use crate::peripherals::DTS;
use crate::time::Hertz;
use crate::{interrupt, pac, rcc};

mod tsel;
pub use tsel::TriggerSel;

#[allow(missing_docs)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SampleTime {
    ClockCycles1 = 1,
    ClockCycles2 = 2,
    ClockCycles3 = 3,
    ClockCycles4 = 4,
    ClockCycles5 = 5,
    ClockCycles6 = 6,
    ClockCycles7 = 7,
    ClockCycles8 = 8,
    ClockCycles9 = 9,
    ClockCycles10 = 10,
    ClockCycles11 = 11,
    ClockCycles12 = 12,
    ClockCycles13 = 13,
    ClockCycles14 = 14,
    ClockCycles15 = 15,
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Config
pub struct Config {
    /// Sample time
    pub sample_time: SampleTime,
    /// Trigger selection
    pub trigger: TriggerSel,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sample_time: SampleTime::ClockCycles1,
            trigger: TriggerSel::Software,
        }
    }
}

/// The read-only factory calibration values used for converting a
/// measurement to a temperature.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FactoryCalibration {
    /// The calibration temperature in degrees Celsius.
    pub t0: u8,
    /// The frequency at the calibration temperature.
    pub fmt0: Hertz,
    /// The ramp coefficient in Hertz per degree Celsius.
    pub ramp_coeff: u16,
}

const MAX_DTS_CLK_FREQ: Hertz = Hertz::mhz(1);

/// Digital temperature sensor driver.
pub struct Dts<'d> {
    _peri: Peri<'d, DTS>,
}

static WAKER: AtomicWaker = AtomicWaker::new();

impl<'d> Dts<'d> {
    /// Create a new temperature sensor driver.
    pub fn new(
        _peri: Peri<'d, DTS>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::DTS, InterruptHandler> + 'd,
        config: Config,
    ) -> Self {
        rcc::enable_and_reset::<DTS>();

        let prescaler = rcc::frequency::<DTS>() / MAX_DTS_CLK_FREQ;

        if prescaler > 127 {
            panic!("DTS PCLK frequency must be less than 127 MHz.");
        }

        Self::regs().cfgr1().modify(|w| {
            w.set_refclk_sel(false);
            w.set_hsref_clk_div(prescaler as u8);
            w.set_q_meas_opt(false);
            // Software trigger
            w.set_intrig_sel(0);
            w.set_smp_time(config.sample_time as u8);
            w.set_intrig_sel(config.trigger as u8);
            w.set_start(true);
            w.set_en(true);
        });

        interrupt::DTS.unpend();
        unsafe { interrupt::DTS.enable() };

        Self { _peri }
    }

    /// Reconfigure the driver.
    pub fn set_config(&mut self, config: &Config) {
        Self::regs().cfgr1().modify(|w| {
            w.set_smp_time(config.sample_time as u8);
            w.set_intrig_sel(config.trigger as u8);
        });
    }

    /// Get the read-only factory calibration values used for converting a
    /// measurement to a temperature.
    pub fn factory_calibration() -> FactoryCalibration {
        let t0valr1 = Self::regs().t0valr1().read();
        let t0 = match t0valr1.t0() {
            0 => 30,
            1 => 130,
            _ => unimplemented!(),
        };
        let fmt0 = Hertz::hz(t0valr1.fmt0() as u32 * 100);

        let ramp_coeff = Self::regs().rampvalr().read().ramp_coeff();

        FactoryCalibration { t0, fmt0, ramp_coeff }
    }

    /// Perform an asynchronous temperature measurement. The returned future can
    /// be awaited to obtain the measurement.
    ///
    /// The future returned waits for the next measurement to complete.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use embassy_stm32::{bind_interrupts, dts};
    /// use embassy_stm32::dts::Dts;
    ///
    /// bind_interrupts!(struct Irqs {
    ///     DTS => temp::InterruptHandler;
    /// });
    ///
    /// # async {
    /// # let p: embassy_stm32::Peripherals = todo!();
    /// let mut dts = Dts::new(p.DTS, Irqs, Default::default());
    /// let v: u16 = dts.read().await;
    /// # };
    /// ```
    pub async fn read(&mut self) -> u16 {
        let r = Self::regs();

        r.itenr().modify(|w| w.set_iteen(true));

        poll_fn(|cx| {
            WAKER.register(cx.waker());
            if r.itenr().read().iteen() {
                Poll::Pending
            } else {
                Poll::Ready(r.dr().read().mfreq())
            }
        })
        .await
    }

    /// Returns the last measurement made, if any.
    ///
    /// There is no guarantee that the measurement is recent or that a
    /// measurement has ever completed.
    pub fn read_immediate(&mut self) -> u16 {
        Self::regs().dr().read().mfreq()
    }

    fn regs() -> pac::dts::Dts {
        pac::DTS
    }
}

impl<'d> Drop for Dts<'d> {
    fn drop(&mut self) {
        Self::regs().cfgr1().modify(|w| w.set_en(false));
        rcc::disable::<DTS>();
    }
}

/// Interrupt handler.
pub struct InterruptHandler {
    _private: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::DTS> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = pac::DTS;
        let (sr, itenr) = (r.sr().read(), r.itenr().read());

        if (itenr.iteen() && sr.itef()) || (itenr.aiteen() && sr.aitef()) {
            r.itenr().modify(|w| {
                w.set_iteen(false);
                w.set_aiteen(false);
            });
            r.icifr().modify(|w| {
                w.set_citef(true);
                w.set_caitef(true);
            });
        } else if (itenr.itlen() && sr.itlf()) || (itenr.aitlen() && sr.aitlf()) {
            r.itenr().modify(|w| {
                w.set_itlen(false);
                w.set_aitlen(false);
            });
            r.icifr().modify(|w| {
                w.set_citlf(true);
                w.set_caitlf(true);
            });
        } else if (itenr.ithen() && sr.ithf()) || (itenr.aithen() && sr.aithf()) {
            r.itenr().modify(|w| {
                w.set_ithen(false);
                w.set_aithen(false);
            });
            r.icifr().modify(|w| {
                w.set_cithf(true);
                w.set_caithf(true);
            });
        } else {
            return;
        }

        compiler_fence(Ordering::SeqCst);
        WAKER.wake();
    }
}
