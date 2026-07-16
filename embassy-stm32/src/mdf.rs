//! Multi-function Digital Filter (MDF)

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::dflt::{Acqmod, Bssel, Cckdir, Cicmod, Ckgmod, Datsrc, Rxfifo};
pub use crate::dflt::{ClockConfig, FilterConfig, SitfConfig, sample_from_dma_word, samples_from_dma_words};
use crate::dma::ringbuffer::Error as RingbufferError;
use crate::dma::{Channel, ReadableRingBuffer, TransferOptions};
use crate::gpio::{AfType, OutputType, Pull, Speed};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::mdf::Mdf as Regs;
use crate::{Peri, interrupt, rcc};

/// Marker types for MDF digital filters 0..5.
pub struct Flt0;
/// Marker type for filter 1.
pub struct Flt1;
/// Marker type for filter 2.
pub struct Flt2;
/// Marker type for filter 3.
pub struct Flt3;
/// Marker type for filter 4.
pub struct Flt4;
/// Marker type for filter 5.
pub struct Flt5;

/// Identifies one of the six MDF filter channels.
#[allow(private_bounds)]
pub trait Filter: SealedFilter {
    /// Filter index (0..5).
    const INDEX: u8;
    /// Interrupt for this filter.
    type Interrupt: Interrupt;
}

trait SealedFilter {}

macro_rules! impl_filter {
    ($flt:ident, $idx:literal, $int:ident) => {
        impl SealedFilter for $flt {}
        impl Filter for $flt {
            const INDEX: u8 = $idx;
            type Interrupt = crate::_generated::peripheral_interrupts::MDF1::$int;
        }
    };
}

impl_filter!(Flt0, 0, FLT0);
impl_filter!(Flt1, 1, FLT1);
impl_filter!(Flt2, 2, FLT2);
impl_filter!(Flt3, 3, FLT3);
impl_filter!(Flt4, 4, FLT4);
impl_filter!(Flt5, 5, FLT5);

/// MDF driver error.
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

/// Ring-buffered MDF driver for one digital filter.
pub struct Mdf<'d, T: Instance, F: Filter> {
    _peri: Peri<'d, T>,
    _filter: PhantomData<F>,
    ring_buffer: ReadableRingBuffer<'d, u32>,
}

/// Combined configuration for PDM capture on a filter/SITF pair.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Config {
    /// Clock generator settings.
    pub clock: ClockConfig,
    /// Serial interface settings.
    pub sitf: SitfConfig,
    /// Digital filter settings.
    pub filter: FilterConfig,
}

impl Config {
    /// Default configuration for the given filter / serial interface index.
    pub const fn for_filter(index: u8) -> Self {
        Self {
            clock: ClockConfig {
                procdiv: 0,
                cckdiv: crate::dflt::Cckdiv::Div1,
                cck0_output: true,
                cck1_output: false,
            },
            sitf: SitfConfig {
                mode: crate::dflt::Sitfmod::NormalSpi,
                clock_source: crate::dflt::Scksrc::Cck0,
                threshold: 4,
            },
            filter: FilterConfig {
                bitstream: bitstream_for_sitf(index),
                cic_decimation: 31,
                scale: 0b100000,
                acquisition_mode: Acqmod::AsynchronousContinuous,
                fifo_threshold: Rxfifo::HalfFull,
            },
        }
    }
}

impl<'d, T: Instance, F: Filter> Mdf<'d, T, F> {
    fn dma_opts() -> TransferOptions {
        TransferOptions {
            half_transfer_ir: true,
            ..Default::default()
        }
    }

    /// Create a new MDF instance for filter `F`.
    ///
    /// `cck` drives the PDM bit clock and `sdi` receives the PDM data stream for the
    /// serial interface with the same index as the filter.
    pub fn new<D>(
        peri: Peri<'d, T>,
        irq: impl interrupt::typelevel::Binding<F::Interrupt, FilterInterruptHandler<T, F>>
        + interrupt::typelevel::Binding<D::Interrupt, crate::dma::InterruptHandler<D>>
        + 'd,
        config: Config,
        cck: Peri<'d, impl CckPin<T>>,
        sdi: Peri<'d, impl SdiPin<T>>,
        dma: Peri<'d, D>,
        dma_buf: &'d mut [u32],
    ) -> Self
    where
        D: RxDma<T, F>,
    {
        set_as_af!(cck, AfType::output(OutputType::PushPull, Speed::VeryHigh));
        set_as_af!(sdi, AfType::input(Pull::None));

        rcc::enable_and_reset::<T>();
        F::Interrupt::unpend();
        unsafe { F::Interrupt::enable() };

        let regs = T::regs();
        let index = F::INDEX as usize;
        Self::apply_config(regs, index, &config);

        let request = dma.request();
        let ring_buffer = unsafe {
            ReadableRingBuffer::new(
                Channel::new(dma, irq),
                request,
                regs.dfltdr(index).as_ptr() as *mut u32,
                dma_buf,
                Self::dma_opts(),
            )
        };

        Self {
            _peri: peri,
            _filter: PhantomData,
            ring_buffer,
        }
    }

    fn apply_config(regs: Regs, index: usize, config: &Config) {
        use crate::pac::mdf::vals as v;

        regs.ckgcr().modify(|w| {
            w.set_ckgmod(v::Ckgmod::from_bits(Ckgmod::Immediate.to_bits()));
            w.set_procdiv(config.clock.procdiv);
            w.set_cckdiv(v::Cckdiv::from_bits(config.clock.cckdiv.to_bits()));
            w.set_cck0dir(v::Cckdir::from_bits(if config.clock.cck0_output {
                Cckdir::Output.to_bits()
            } else {
                Cckdir::Input.to_bits()
            }));
            w.set_cck1dir(v::Cckdir::from_bits(if config.clock.cck1_output {
                Cckdir::Output.to_bits()
            } else {
                Cckdir::Input.to_bits()
            }));
            w.set_cck0en(config.clock.cck0_output);
            w.set_cck1en(config.clock.cck1_output);
            w.set_ckgden(true);
        });

        regs.sitfcr(index).modify(|w| {
            w.set_sitfmod(v::Sitfmod::from_bits(config.sitf.mode.to_bits()));
            w.set_scksrc(v::Scksrc::from_bits(config.sitf.clock_source.to_bits()));
            w.set_sth(config.sitf.threshold);
            w.set_sitfen(true);
        });

        regs.bsmxcr(index).modify(|w| {
            w.set_bssel(v::Bssel::from_bits(config.filter.bitstream.to_bits()));
        });
        regs.dfltcicr(index).modify(|w| {
            w.set_datsrc(v::Datsrc::from_bits(Datsrc::Bsmx.to_bits()));
            w.set_cicmod(v::Cicmod::from_bits(Cicmod::Sinc5.to_bits()));
            w.set_mcicd(config.filter.cic_decimation);
            w.set_scale(config.filter.scale);
        });
        regs.dfltrsfr(index).modify(|w| {
            w.set_rsfltbyp(true);
            w.set_hpfbyp(true);
        });
        regs.dfltcr(index).modify(|w| {
            w.set_acqmod(v::Acqmod::from_bits(config.filter.acquisition_mode.to_bits()));
            w.set_fth(v::Rxfifo::from_bits(config.filter.fifo_threshold.to_bits()));
            w.set_dmaen(true);
        });
    }

    /// Start filter acquisition and DMA.
    pub fn start(&mut self) {
        self.ring_buffer.start();
        let index = F::INDEX as usize;
        T::regs().dfltcr(index).modify(|w| {
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

impl<'d, T: Instance, F: Filter> Drop for Mdf<'d, T, F> {
    fn drop(&mut self) {
        let index = F::INDEX as usize;
        let regs = T::regs();
        regs.dfltcr(index).modify(|w| {
            w.set_dfltrun(false);
            w.set_dflten(false);
            w.set_dmaen(false);
        });
        regs.sitfcr(index).modify(|w| w.set_sitfen(false));
        if index == 0 {
            regs.ckgcr().modify(|w| w.set_ckgden(false));
        }
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Regs;
}

/// MDF instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {}

/// MDF filter interrupt handler.
pub struct FilterInterruptHandler<T: Instance, F: Filter> {
    _marker: PhantomData<(T, F)>,
}

impl<T: Instance, F: Filter> interrupt::typelevel::Handler<F::Interrupt> for FilterInterruptHandler<T, F> {
    unsafe fn on_interrupt() {
        let index = F::INDEX as usize;
        let isr = T::regs().dfltisr(index).read();
        if isr.dovrf() || isr.rfovrf() {
            T::regs().dfltisr(index).modify(|w| {
                w.set_dovrf(true);
                w.set_rfovrf(true);
            });
        }
    }
}

pin_trait!(CckPin, Instance);
pin_trait!(CkiPin, Instance);
pin_trait!(SdiPin, Instance);

dma_trait!(RxDma, Instance, Filter);

foreach_peripheral!(
    (mdf, $inst:ident) => {
        impl crate::mdf::SealedInstance for crate::peripherals::$inst {
            fn regs() -> crate::pac::mdf::Mdf {
                crate::pac::$inst
            }
        }
        impl crate::mdf::Instance for crate::peripherals::$inst {}
    };
);

/// Return the BSMX bitstream selection for a given serial interface index.
pub const fn bitstream_for_sitf(index: u8) -> Bssel {
    match index {
        0 => Bssel::Bs0R,
        1 => Bssel::Bs1R,
        2 => Bssel::Bs2R,
        3 => Bssel::Bs3R,
        4 => Bssel::Bs4R,
        5 => Bssel::Bs5R,
        _ => Bssel::Bs0R,
    }
}
