//! General Purpose Input / Output (GPIO)
use core::convert::Infallible;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Poll;

use apollo3_pac as pac;
use apollo3_pac::interrupt;
use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal::digital::{ErrorType, InputPin, OutputPin, StatefulOutputPin};

const NEW_WAKER: AtomicWaker = AtomicWaker::new();
static WAKERS: [AtomicWaker; 50] = [NEW_WAKER; 50];

const NEW_FIRED: AtomicBool = AtomicBool::new(false);
static FIRED: [AtomicBool; 50] = [NEW_FIRED; 50];

pub enum Pull {
    None,
    Up,
}

/// Sealed trait carrying a pad's identity.
pub(crate) trait SealedPin {
    /// Physical pad number (0-49).
    fn number(&self) -> u8;
}

#[allow(private_bounds)]
pub trait Pin: SealedPin + PeripheralType + Into<AnyPin> + 'static {
    /// Returns the physical pad number (0-49).
    #[inline(always)]
    fn pin_number(&self) -> u8 {
        self.number()
    }
}

/// Sealed trait that performs a pad's function-select (FNCSEL) configuration.
pub(crate) trait SealedConfigure {
    /// Write the 3-bit function-select field for this pad.
    fn set_fncsel_bits(&self, sel: u8);

    /// Configure the pad as an input with the given pull configuration.
    fn configure_as_input(&self, pull: Pull);

    /// Every Apollo3 pad exposes plain GPIO at function-select `0x03`.
    const GPIO_FNCSEL: u8 = 0x03;

    /// Route the pad to plain GPIO mode.
    #[inline(always)]
    fn set_as_gpio(&self) {
        self.set_fncsel_bits(Self::GPIO_FNCSEL);
    }
}

// Internal Helpers

#[inline(always)]
fn with_padkey<F: FnOnce()>(f: F) {
    let gpio = pac::GPIO;
    critical_section::with(|_| {
        gpio.padkey().write_value(pac::gpio::regs::Padkey(0x73));
        f();
        gpio.padkey().write_value(pac::gpio::regs::Padkey(0));
    });
}

#[inline(always)]
fn modify_cfg<F: FnOnce(u32) -> u32>(n: usize, f: F) {
    let gpio = pac::GPIO;
    match n / 8 {
        0 => gpio.cfga().modify(|w| w.0 = f(w.0)),
        1 => gpio.cfgb().modify(|w| w.0 = f(w.0)),
        2 => gpio.cfgc().modify(|w| w.0 = f(w.0)),
        3 => gpio.cfgd().modify(|w| w.0 = f(w.0)),
        4 => gpio.cfge().modify(|w| w.0 = f(w.0)),
        5 => gpio.cfgf().modify(|w| w.0 = f(w.0)),
        6 => gpio.cfgg().modify(|w| w.0 = f(w.0)),
        _ => unreachable!(),
    }
}

// Macro implementing all pin traits for a whole PADREG bank at once.
//
// Each pad supplies only its number; the struct name (`P{n}`), the typed setter
// (`set_pad{n}fncsel`) and the enum (`Pad{n}fncsel`) are all derived from it. The
// bank register (`padreg{a..m}`) is the one thing that can't be computed, so it is
// stated once per group. `From<P{n}> for AnyPin` is generated here too.

macro_rules! pins {
    ($($padreg:ident: [$($n:literal),* $(,)?]),* $(,)?) => {
        $($(
            paste::paste! {
                impl SealedPin for crate::peripherals::[<P $n>] {
                    #[inline(always)]
                    fn number(&self) -> u8 { $n }
                }

                impl Pin for crate::peripherals::[<P $n>] {}

                impl SealedConfigure for crate::peripherals::[<P $n>] {
                    #[inline(always)]
                    fn set_fncsel_bits(&self, sel: u8) {
                        let gpio = pac::GPIO;
                        with_padkey(|| {
                            gpio.$padreg().modify(|w| {
                                w.[<set_pad $n fncsel>](pac::gpio::vals::[<Pad $n fncsel>]::from_bits(sel))
                            });
                        });
                    }

                    #[inline(always)]
                    fn configure_as_input(&self, pull: Pull) {
                        let gpio = pac::GPIO;
                        with_padkey(|| {
                            gpio.$padreg().modify(|w| {
                                w.[<set_pad $n fncsel>](pac::gpio::vals::[<Pad $n fncsel>]::from_bits(Self::GPIO_FNCSEL));
                                w.[<set_pad $n inpen>](true);
                                w.[<set_pad $n pull>](match pull {
                                    Pull::Up => true,
                                    Pull::None => false,
                                });
                            });
                        });
                    }
                }

                impl From<crate::peripherals::[<P $n>]> for AnyPin {
                    #[inline(always)]
                    fn from(_val: crate::peripherals::[<P $n>]) -> Self {
                        Self { number: $n }
                    }
                }
            }
        )*)*
    };
}

// Pin Definitions — grouped by PADREG bank (pad numbers only)

pins! {
    padrega: [0, 1, 2, 3],
    padregb: [4, 5, 6, 7],
    padregc: [8, 9, 10, 11],
    padregd: [12, 13, 14, 15],
    padrege: [16, 17, 18, 19],
    padregf: [20, 21, 22, 23],
    padregg: [24, 25, 26, 27],
    padregh: [28, 29, 30, 31],
    padregi: [32, 33, 34, 35],
    padregj: [36, 37, 38, 39],
    padregk: [40, 41, 42, 43],
    padregl: [44, 45, 46, 47],
    padregm: [48, 49],
}

/// Type Erasure and Flex Wrapper

/// Type-erased GPIO pin
pub struct AnyPin {
    number: u8,
}

impl_peripheral!(AnyPin);

impl SealedPin for AnyPin {
    #[inline(always)]
    fn number(&self) -> u8 {
        self.number
    }
}

impl Pin for AnyPin {}

// `From<P{n}> for AnyPin` is generated per pad by the `pins!` macro above.

/// GPIO flexible pin, handling raw register writes for state manipulation.
pub struct Flex<'d> {
    pub(crate) pin: Peri<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    #[inline(always)]
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        Self { pin: pin.into() }
    }

    #[inline(always)]
    pub fn set_as_output(&mut self) {
        let gpio = pac::GPIO;
        let n = self.pin.pin_number() as usize;

        if n < 32 {
            gpio.enca().write_value(1 << n);
        } else {
            gpio.encb().write_value(pac::gpio::regs::Encb(1 << (n - 32)));
        }
    }

    #[inline(always)]
    pub fn set_high(&mut self) {
        let n = self.pin.pin_number() as usize;
        let gpio = pac::GPIO;
        if n < 32 {
            gpio.wtsa().write_value(1 << n);
        } else {
            gpio.wtsb().write_value(pac::gpio::regs::Wtsb(1 << (n - 32)));
        }
    }

    #[inline(always)]
    pub fn set_low(&mut self) {
        let n = self.pin.pin_number() as usize;
        let gpio = pac::GPIO;
        if n < 32 {
            gpio.wtca().write_value(1 << n);
        } else {
            gpio.wtcb().write_value(pac::gpio::regs::Wtcb(1 << (n - 32)));
        }
    }

    /// Read the pin state (true = high, false = low)
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        let n = self.pin.pin_number() as usize;
        let gpio = pac::GPIO;
        if n < 32 {
            (gpio.rda().read() & (1 << n)) != 0
        } else {
            (gpio.rdb().read().0 & (1 << (n - 32))) != 0
        }
    }

    /// Internal method to wait for an edge
    async fn wait_for_edge(&mut self, incfg: u32, intd: u32) {
        let n = self.pin.pin_number() as usize;
        let gpio = pac::GPIO;
        let k = n % 8;
        let shift = k * 4;

        with_padkey(|| {
            modify_cfg(n, |val| {
                let mut v = val;
                // clear INCFG (bit 0) and INTD (bit 3)
                v &= !(0x09 << shift);
                // set INCFG and INTD
                v |= (incfg << shift) | ((intd << 3) << shift);
                v
            });

            // Clear stale interrupts and enable BEFORE entering the poll_fn
            // (INT0EN requires PADKEY)
            if n < 32 {
                gpio.int0clr().write(|w| {
                    w.0 = 1 << n;
                });
                gpio.int0en().modify(|w| {
                    w.0 |= 1 << n;
                });
            } else {
                gpio.int1clr().write(|w| {
                    w.0 = 1 << (n - 32);
                });
                gpio.int1en().modify(|w| {
                    w.0 |= 1 << (n - 32);
                });
            }
        });

        // Reset the fired flag
        FIRED[n].store(false, Ordering::Relaxed);

        core::future::poll_fn(move |cx| {
            // Register waker
            WAKERS[n].register(cx.waker());

            // Check if the ISR has flagged that the interrupt fired
            if FIRED[n].load(Ordering::Relaxed) {
                // We leave the interrupt enabled. It will fire again next time.
                // The task might wait_for_edge again.
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

/// Output Driver

/// Digital output pin driver
pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> Output<'d> {
    #[inline(always)]
    #[allow(private_bounds)]
    pub fn new(pin: Peri<'d, impl Pin + SealedConfigure>) -> Self {
        // Route the pad to plain GPIO (fncsel = 0x03) through the PAC.
        pin.set_as_gpio();

        let n = pin.pin_number() as usize;
        let k = n % 8;
        let shift = k * 4 + 1; // gpioNoutcfg is at bit 1 and 2

        with_padkey(|| {
            modify_cfg(n, |val| {
                let mut v = val;
                v &= !(0x03 << shift);
                v |= 0x01 << shift; // 1 = Pushpull
                v
            });
        });

        let mut flex = Flex { pin: pin.into() };
        flex.set_as_output();
        flex.set_low(); // Default to low

        Self { pin: flex }
    }

    #[inline(always)]
    pub fn set_high(&mut self) {
        self.pin.set_high();
    }

    #[inline(always)]
    pub fn set_low(&mut self) {
        self.pin.set_low();
    }
}

impl<'d> ErrorType for Output<'d> {
    type Error = Infallible;
}

impl<'d> OutputPin for Output<'d> {
    #[inline(always)]
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    #[inline(always)]
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }
}

impl<'d> StatefulOutputPin for Output<'d> {
    #[inline(always)]
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_high())
    }

    #[inline(always)]
    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.pin.is_high())
    }
}

/// Input Driver

/// Digital input pin driver
pub struct Input<'d> {
    pin: Flex<'d>,
}

impl<'d> Input<'d> {
    #[inline(always)]
    #[allow(private_bounds)]
    pub fn new(pin: Peri<'d, impl Pin + SealedConfigure>, pull: Pull) -> Self {
        pin.configure_as_input(pull);
        Self {
            pin: Flex { pin: pin.into() },
        }
    }

    pub async fn wait_for_rising_edge(&mut self) {
        // L2H: INCFG=0, INTD=0
        self.pin.wait_for_edge(0, 0).await;
    }

    pub async fn wait_for_falling_edge(&mut self) {
        // H2L: INCFG=0, INTD=1
        self.pin.wait_for_edge(0, 1).await;
    }

    pub async fn wait_for_any_edge(&mut self) {
        // BOTH: INCFG=1, INTD=1
        self.pin.wait_for_edge(1, 1).await;
    }
}

impl<'d> ErrorType for Input<'d> {
    type Error = Infallible;
}

impl<'d> InputPin for Input<'d> {
    #[inline(always)]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_high())
    }

    #[inline(always)]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.pin.is_high())
    }
}

/// Interrupt Handler

#[interrupt]
fn GPIO() {
    let gpio = pac::GPIO;
    let stat0 = gpio.int0stat().read().0;
    let stat1 = gpio.int1stat().read().0;

    // Clear the interrupts
    gpio.int0clr().write(|w| {
        w.0 = stat0;
    });
    gpio.int1clr().write(|w| {
        w.0 = stat1;
    });

    // Wake tasks and set fired flag
    // We DO NOT disable int0en here to avoid needing padkey in the ISR
    for i in 0..32 {
        if (stat0 & (1 << i)) != 0 {
            FIRED[i].store(true, Ordering::Relaxed);
            WAKERS[i].wake();
        }
    }

    for i in 0..18 {
        if (stat1 & (1 << i)) != 0 {
            FIRED[32 + i].store(true, Ordering::Relaxed);
            WAKERS[32 + i].wake();
        }
    }
}

// Force the linker to keep the GPIO interrupt handler from being stripped.
// The `#[interrupt]` macro exports the handler under the symbol name `GPIO`
// (via `#[no_mangle]`), but does not leave a Rust-visible value by that name,
// so we reference the exported symbol through an `extern "C"` declaration.
#[used]
static KEEP_GPIO: unsafe extern "C" fn() = {
    unsafe extern "C" {
        fn GPIO();
    }
    GPIO
};
