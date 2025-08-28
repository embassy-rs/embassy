#![no_std]
// Doc feature labels can be tested locally by running RUSTDOCFLAGS="--cfg=docsrs" cargo +nightly doc
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg_hide), doc(cfg_hide(doc, docsrs)))]
#![cfg_attr(
    docsrs,
    doc = "<div style='padding:30px;background:#810;color:#fff;text-align:center;'><p>You might want to <a href='https://docs.embassy.dev/embassy-mspm0'>browse the `embassy-mspm0` documentation on the Embassy website</a> instead.</p><p>The documentation here on `docs.rs` is built for a single chip only, while on the Embassy website you can pick your exact chip from the top menu. Available peripherals and their APIs change depending on the chip.</p></div>\n\n"
)]
#![doc = include_str!("../README.md")]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

// This must be declared early as well for
mod macros;

pub mod dma;
pub mod gpio;
pub mod i2c;
pub mod timer;
pub mod uart;
pub mod wwdt;

/// Operating modes for peripherals.
pub mod mode {
    trait SealedMode {}

    /// Operating mode for a peripheral.
    #[allow(private_bounds)]
    pub trait Mode: SealedMode {}

    /// Blocking mode.
    pub struct Blocking;
    impl SealedMode for Blocking {}
    impl Mode for Blocking {}

    /// Async mode.
    pub struct Async;
    impl SealedMode for Async {}
    impl Mode for Async {}
}

#[cfg(feature = "_time-driver")]
mod time_driver;

pub(crate) mod _generated {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]

    include!(concat!(env!("OUT_DIR"), "/_generated.rs"));
}

// Reexports
pub(crate) use _generated::gpio_pincm;
pub use _generated::{peripherals, Peripherals};
pub use embassy_hal_internal::Peri;
#[cfg(feature = "unstable-pac")]
pub use mspm0_metapac as pac;
#[cfg(not(feature = "unstable-pac"))]
pub(crate) use mspm0_metapac as pac;

pub use crate::_generated::interrupt;

/// Macro to bind interrupts to handlers.
///
/// This defines the right interrupt handlers, and creates a unit struct (like `struct Irqs;`)
/// and implements the right [`Binding`]s for it. You can pass this struct to drivers to
/// prove at compile-time that the right interrupts have been bound.
///
/// Example of how to bind one interrupt:
///
/// ```rust,ignore
/// use embassy_nrf::{bind_interrupts, spim, peripherals};
///
/// bind_interrupts!(
///     /// Binds the SPIM3 interrupt.
///     struct Irqs {
///         SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
///     }
/// );
/// ```
///
/// Example of how to bind multiple interrupts in a single macro invocation:
///
/// ```rust,ignore
/// use embassy_nrf::{bind_interrupts, spim, twim, peripherals};
///
/// bind_interrupts!(struct Irqs {
///     SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
///     TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
/// });
/// ```

// developer note: this macro can't be in `embassy-hal-internal` due to the use of `$crate`.
#[macro_export]
macro_rules! bind_interrupts {
    ($(#[$attr:meta])* $vis:vis struct $name:ident {
        $(
            $(#[cfg($cond_irq:meta)])?
            $irq:ident => $(
                $(#[cfg($cond_handler:meta)])?
                $handler:ty
            ),*;
        )*
    }) => {
        #[derive(Copy, Clone)]
        $(#[$attr])*
        $vis struct $name;

        $(
            #[allow(non_snake_case)]
            #[no_mangle]
            $(#[cfg($cond_irq)])?
            unsafe extern "C" fn $irq() {
                unsafe {
                    $(
                        $(#[cfg($cond_handler)])?
                        <$handler as $crate::interrupt::typelevel::Handler<$crate::interrupt::typelevel::$irq>>::on_interrupt();

                    )*
                }
            }

            $(#[cfg($cond_irq)])?
            $crate::bind_interrupts!(@inner
                $(
                    $(#[cfg($cond_handler)])?
                    unsafe impl $crate::interrupt::typelevel::Binding<$crate::interrupt::typelevel::$irq, $handler> for $name {}
                )*
            );
        )*
    };
    (@inner $($t:tt)*) => {
        $($t)*
    }
}

/// `embassy-mspm0` global configuration.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    // TODO: OSC configuration.
    /// The size of DMA block transfer burst.
    ///
    /// If this is set to a value
    pub dma_burst_size: dma::BurstSize,

    /// Whether the DMA channels are used in a fixed priority or a round robin fashion.
    ///
    /// If [`false`], the DMA priorities are fixed.
    ///
    /// If [`true`], after a channel finishes a transfer it becomes the lowest priority.
    pub dma_round_robin: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dma_burst_size: dma::BurstSize::Complete,
            dma_round_robin: false,
        }
    }
}

pub fn init(config: Config) -> Peripherals {
    critical_section::with(|cs| {
        let peripherals = Peripherals::take_with_cs(cs);

        // TODO: Further clock configuration

        pac::SYSCTL.mclkcfg().modify(|w| {
            // Enable MFCLK
            w.set_usemftick(true);
            // MDIV must be disabled if MFCLK is enabled.
            w.set_mdiv(0);
        });

        // Enable MFCLK for peripheral use
        //
        // TODO: Optional?
        pac::SYSCTL.genclken().modify(|w| {
            w.set_mfpclken(true);
        });

        pac::SYSCTL.borthreshold().modify(|w| {
            w.set_level(0);
        });

        gpio::init(pac::GPIOA);
        #[cfg(gpio_pb)]
        gpio::init(pac::GPIOB);
        #[cfg(gpio_pc)]
        gpio::init(pac::GPIOC);

        _generated::enable_group_interrupts(cs);

        #[cfg(any(mspm0c110x, mspm0l110x))]
        unsafe {
            use crate::_generated::interrupt::typelevel::Interrupt;
            crate::interrupt::typelevel::GPIOA::enable();
        }

        // SAFETY: Peripherals::take_with_cs will only be run once or panic.
        unsafe { dma::init(cs, config.dma_burst_size, config.dma_round_robin) };

        #[cfg(feature = "_time-driver")]
        time_driver::init(cs);

        peripherals
    })
}

pub(crate) mod sealed {
    #[allow(dead_code)]
    pub trait Sealed {}
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}
