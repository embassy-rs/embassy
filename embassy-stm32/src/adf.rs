//! Audio Digital Filter (ADF)

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::dflt::{Acqmod, Cckdir, Ccken, Cicmod, Ckgmod, Datsrc, Rxfifo};
pub use crate::dflt::{ClockConfig, FilterConfig, SitfConfig, sample_from_dma_word, samples_from_dma_words};
use crate::dma::ringbuffer::Error as RingbufferError;
use crate::dma::{Channel, ReadableRingBuffer, TransferOptions};
use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::adf::Adf as Regs;
use crate::{Peri, interrupt, rcc};

/// Marker type for digital filter 0 (the only filter on ADF).
pub struct Flt0;

trait SealedFilter {}

impl SealedFilter for Flt0 {}

/// Identifies the ADF digital filter (filter 0 only).
#[allow(private_bounds)]
pub trait Filter: SealedFilter {}

impl Filter for Flt0 {}

/// ADF driver error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// DMA ring buffer error.
    Ringbuffer(RingbufferError),
}

impl From<RingbufferError> for Error {
    fn from(err: RingbufferError) -> Self {
        Self::Ringbuffer(err)
    }
}

/// Ring-buffered ADF driver.
pub struct Adf<'d, T: Instance> {
    _peri: Peri<'d, T>,
    ring_buffer: ReadableRingBuffer<'d, u32>,
}

/// Combined configuration for a typical PDM microphone capture on filter 0.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Clock generator settings.
    pub clock: ClockConfig,
    /// Serial interface 0 settings.
    pub sitf: SitfConfig,
    /// Digital filter 0 settings.
    pub filter: FilterConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            clock: ClockConfig::default(),
            sitf: SitfConfig::default(),
            filter: pdm_filter_config(),
        }
    }
}

impl<'d, T: Instance> Adf<'d, T> {
    fn dma_opts() -> TransferOptions {
        TransferOptions {
            half_transfer_ir: true,
            ..Default::default()
        }
    }

    /// Create a new ADF instance configured for PDM capture on filter 0.
    ///
    /// `cck` drives the PDM bit clock and `sdi` receives the PDM data stream.
    pub fn new<D>(
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
        config: Config,
        cck: Peri<'d, impl CckPin<T>>,
        sdi: Peri<'d, impl SdiPin<T>>,
        dma: Peri<'d, D>,
        dma_buf: &'d mut [u32],
    ) -> Self
    where
        D: RxDma<T, Flt0>,
    {
        set_as_af!(cck, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        set_as_af!(sdi, AfType::input(Pull::None));

        rcc::enable_and_reset::<T>();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let regs = T::regs();
        Self::apply_config(regs, &config);

        let request = dma.request();
        let ring_buffer = unsafe {
            ReadableRingBuffer::new(
                Channel::new(dma, irq),
                request,
                regs.dfltdr().as_ptr() as *mut u32,
                dma_buf,
                Self::dma_opts(),
            )
        };

        Self {
            _peri: peri,
            ring_buffer,
        }
    }

    fn apply_config(regs: Regs, config: &Config) {
        regs.ckgcr().modify(|w| {
            w.set_ckgmod(Ckgmod::Immediate);
            w.set_procdiv(config.clock.procdiv);
            w.set_cckdiv(config.clock.cckdiv);
            w.set_cck0dir(if config.clock.cck0_output {
                Cckdir::Output
            } else {
                Cckdir::Input
            });
            w.set_cck1dir(if config.clock.cck1_output {
                Cckdir::Output
            } else {
                Cckdir::Input
            });
            w.set_cck0en(if config.clock.cck0_output {
                Ccken::Generated
            } else {
                Ccken::NotGenerated
            });
            w.set_cck1en(if config.clock.cck1_output {
                Ccken::Generated
            } else {
                Ccken::NotGenerated
            });
            w.set_ckgden(true);
        });

        regs.sitfcr().modify(|w| {
            w.set_sitfmod(config.sitf.mode);
            w.set_scksrc(config.sitf.clock_source);
            w.set_sth(config.sitf.threshold);
            w.set_sitfen(true);
        });

        regs.bsmxcr().modify(|w| w.set_bssel(config.filter.bitstream));
        regs.dfltcicr().modify(|w| {
            w.set_datsrc(Datsrc::Bsmx);
            w.set_cicmod(Cicmod::Sinc5);
            w.set_mcicd(config.filter.cic_decimation);
            w.set_scale(config.filter.scale);
        });
        regs.dfltrsfr().modify(|w| {
            w.set_rsfltbyp(true);
            w.set_hpfbyp(true);
        });
        regs.dfltcr().modify(|w| {
            w.set_acqmod(config.filter.acquisition_mode);
            w.set_fth(config.filter.fifo_threshold);
            w.set_dmaen(true);
        });
    }

    /// Start filter acquisition and DMA.
    pub fn start(&mut self) {
        self.ring_buffer.start();
        T::regs().dfltcr().modify(|w| {
            w.set_dflten(true);
            w.set_dfltrun(true);
        });
    }

    /// Read raw 32-bit DMA words from the filter output ring buffer.
    pub async fn read_raw(&mut self, words: &mut [u32]) -> Result<(), Error> {
        self.ring_buffer.read_exact(words).await?;
        Ok(())
    }

    /// Return the kernel clock frequency in Hz.
    pub fn kernel_clock_hz(&self) -> u32 {
        <T as crate::rcc::SealedRccPeripheral>::frequency().0
    }
}

impl<'d, T: Instance> Drop for Adf<'d, T> {
    fn drop(&mut self) {
        let regs = T::regs();
        regs.dfltcr().modify(|w| {
            w.set_dfltrun(false);
            w.set_dflten(false);
            w.set_dmaen(false);
        });
        regs.sitfcr().modify(|w| w.set_sitfen(false));
        regs.ckgcr().modify(|w| w.set_ckgden(false));
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Regs;
}

/// ADF instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// Filter 0 interrupt.
    type Interrupt: interrupt::typelevel::Interrupt;
}

/// ADF interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let isr = T::regs().dfltisr().read();
        if isr.dovrf() || isr.rfovrf() {
            T::regs().dfltisr().modify(|w| {
                w.set_dovrf(true);
                w.set_rfovrf(true);
            });
        }
    }
}

pin_trait!(CckPin, Instance);
pin_trait!(SdiPin, Instance);

dma_trait!(RxDma, Instance, Filter);

foreach_peripheral!(
    (adf, $inst:ident) => {
        impl crate::adf::SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::adf::Adf {
                crate::pac::$inst
            }
        }
        impl crate::adf::Instance for crate::peripherals::$inst {
            type Interrupt = crate::_generated::peripheral_interrupts::$inst::FLT0;
        }
    };
);

/// Default continuous-capture filter configuration for PDM use on filter 0 / SITF 0.
pub const fn pdm_filter_config() -> FilterConfig {
    FilterConfig {
        bitstream: crate::dflt::Bssel::Bs0R,
        cic_decimation: 31,
        scale: 0b100000,
        acquisition_mode: Acqmod::AsynchronousContinuous,
        fifo_threshold: Rxfifo::HalfFull,
    }
}
