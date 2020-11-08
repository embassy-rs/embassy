use anyfmt::{panic, *};
use core::cell::Cell;
use core::ptr;
use embassy::util::Signal;

use crate::hal::gpio::{Input, Level, Output, Pin, Port};
use crate::interrupt;
use crate::pac::generic::Reg;
use crate::pac::gpiote::_TASKS_OUT;
use crate::pac::{p0 as gpio, GPIOTE, P0, P1};

#[cfg(not(feature = "51"))]
use crate::pac::gpiote::{_TASKS_CLR, _TASKS_SET};

pub const CHANNEL_COUNT: usize = 8;

#[cfg(any(feature = "52833", feature = "52840"))]
pub const PIN_COUNT: usize = 48;
#[cfg(not(any(feature = "52833", feature = "52840")))]
pub const PIN_COUNT: usize = 32;

pub struct Gpiote {
    inner: GPIOTE,
    free_channels: Cell<u8>, // 0 = used, 1 = free. 8 bits for 8 channelself.
    channel_signals: [Signal<()>; CHANNEL_COUNT],
    port_signals: [Signal<()>; PIN_COUNT],
}

static mut INSTANCE: *const Gpiote = ptr::null_mut();

pub enum PortInputPolarity {
    High,
    Low,
}

pub enum InputChannelPolarity {
    None,
    HiToLo,
    LoToHi,
    Toggle,
}

/// Polarity of the `task out` operation.
pub enum OutputChannelPolarity {
    Set,
    Clear,
    Toggle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NewChannelError {
    NoFreeChannels,
}

impl Gpiote {
    pub fn new(gpiote: GPIOTE) -> Self {
        #[cfg(any(feature = "52833", feature = "52840"))]
        let ports = unsafe { &[&*P0::ptr(), &*P1::ptr()] };
        #[cfg(not(any(feature = "52833", feature = "52840")))]
        let ports = unsafe { &[&*P0::ptr()] };

        for &p in ports {
            // Enable latched detection
            p.detectmode.write(|w| w.detectmode().ldetect());
            // Clear latch
            p.latch.write(|w| unsafe { w.bits(0xFFFFFFFF) })
        }

        // Enable interrupts
        gpiote.events_port.write(|w| w);
        gpiote.intenset.write(|w| w.port().set());
        interrupt::unpend(interrupt::GPIOTE);
        interrupt::enable(interrupt::GPIOTE);

        Self {
            inner: gpiote,
            free_channels: Cell::new(0xFF), // all 8 channels free
            channel_signals: [
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
            ],
            // This is just horrible
            #[cfg(any(feature = "52833", feature = "52840"))]
            port_signals: [
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
            ],
            #[cfg(not(any(feature = "52833", feature = "52840")))]
            port_signals: [
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
                Signal::new(),
            ],
        }
    }

    fn allocate_channel(&self) -> Result<u8, NewChannelError> {
        interrupt::free(|_| {
            let chs = self.free_channels.get();
            let index = chs.trailing_zeros() as usize;
            if index == 8 {
                return Err(NewChannelError::NoFreeChannels);
            }
            self.free_channels.set(chs & !(1 << index));
            Ok(index as u8)
        })
    }

    fn free_channel(&self, index: u8) {
        interrupt::free(|_| {
            self.inner.config[index as usize].write(|w| w.mode().disabled());
            self.inner.intenclr.write(|w| unsafe { w.bits(1 << index) });

            self.free_channels
                .set(self.free_channels.get() | 1 << index);
            trace!("freed ch {:u8}", index);
        })
    }

    pub fn new_port_input<'a, T>(&'a self, pin: Pin<Input<T>>) -> PortInput<'a, T> {
        interrupt::free(|_| {
            unsafe { INSTANCE = self };
            PortInput { gpiote: self, pin }
        })
    }

    pub fn new_input_channel<'a, T>(
        &'a self,
        pin: Pin<Input<T>>,
        polarity: InputChannelPolarity,
    ) -> Result<InputChannel<'a, T>, NewChannelError> {
        interrupt::free(|_| {
            unsafe { INSTANCE = self };
            let index = self.allocate_channel()?;
            trace!("allocated in ch {:u8}", index as u8);

            self.inner.config[index as usize].write(|w| {
                match polarity {
                    InputChannelPolarity::HiToLo => w.mode().event().polarity().hi_to_lo(),
                    InputChannelPolarity::LoToHi => w.mode().event().polarity().lo_to_hi(),
                    InputChannelPolarity::None => w.mode().event().polarity().none(),
                    InputChannelPolarity::Toggle => w.mode().event().polarity().toggle(),
                };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(match pin.port() {
                    Port::Port0 => false,
                    Port::Port1 => true,
                });
                unsafe { w.psel().bits(pin.pin()) }
            });

            // Enable interrupt
            self.inner.intenset.write(|w| unsafe { w.bits(1 << index) });

            Ok(InputChannel {
                gpiote: self,
                index,
                pin,
            })
        })
    }

    pub fn new_output_channel<'a, T>(
        &'a self,
        pin: Pin<Output<T>>,
        level: Level,
        polarity: OutputChannelPolarity,
    ) -> Result<OutputChannel<'a>, NewChannelError> {
        interrupt::free(|_| {
            unsafe { INSTANCE = self };
            let index = self.allocate_channel()?;
            trace!("allocated out ch {:u8}", index);

            self.inner.config[index as usize].write(|w| {
                w.mode().task();
                match level {
                    Level::High => w.outinit().high(),
                    Level::Low => w.outinit().low(),
                };
                match polarity {
                    OutputChannelPolarity::Set => w.polarity().lo_to_hi(),
                    OutputChannelPolarity::Clear => w.polarity().hi_to_lo(),
                    OutputChannelPolarity::Toggle => w.polarity().toggle(),
                };
                #[cfg(any(feature = "52833", feature = "52840"))]
                w.port().bit(match pin.port() {
                    Port::Port0 => false,
                    Port::Port1 => true,
                });
                unsafe { w.psel().bits(pin.pin()) }
            });

            // Enable interrupt
            self.inner.intenset.write(|w| unsafe { w.bits(1 << index) });

            Ok(OutputChannel {
                gpiote: self,
                index,
            })
        })
    }
}

pub struct PortInput<'a, T> {
    gpiote: &'a Gpiote,
    pin: Pin<Input<T>>,
}

impl<'a, T> Drop for PortInput<'a, T> {
    fn drop(&mut self) {
        pin_conf(&self.pin).modify(|_, w| w.sense().disabled());
        self.gpiote.port_signals[pin_num(&self.pin)].reset();
    }
}

impl<'a, T> PortInput<'a, T> {
    pub async fn wait(&self, polarity: PortInputPolarity) {
        pin_conf(&self.pin).modify(|_, w| match polarity {
            PortInputPolarity::Low => w.sense().low(),
            PortInputPolarity::High => w.sense().high(),
        });
        self.gpiote.port_signals[pin_num(&self.pin)].wait().await;
    }

    pub fn pin(&self) -> &Pin<Input<T>> {
        &self.pin
    }
}

fn pin_num<T>(pin: &Pin<T>) -> usize {
    let port = match pin.port() {
        Port::Port0 => 0,
        #[cfg(any(feature = "52833", feature = "52840"))]
        Port::Port1 => 32,
    };

    port + pin.pin() as usize
}

fn pin_block<T>(pin: &Pin<T>) -> &gpio::RegisterBlock {
    let ptr = match pin.port() {
        Port::Port0 => P0::ptr(),
        #[cfg(any(feature = "52833", feature = "52840"))]
        Port::Port1 => P1::ptr(),
    };

    unsafe { &*ptr }
}

fn pin_conf<T>(pin: &Pin<T>) -> &gpio::PIN_CNF {
    &pin_block(pin).pin_cnf[pin.pin() as usize]
}

pub struct InputChannel<'a, T> {
    gpiote: &'a Gpiote,
    pin: Pin<Input<T>>,
    index: u8,
}

impl<'a, T> Drop for InputChannel<'a, T> {
    fn drop(&mut self) {
        self.gpiote.free_channel(self.index);
    }
}

impl<'a, T> InputChannel<'a, T> {
    pub async fn wait(&self) {
        self.gpiote.channel_signals[self.index as usize]
            .wait()
            .await;
    }

    pub fn pin(&self) -> &Pin<Input<T>> {
        &self.pin
    }
}

pub struct OutputChannel<'a> {
    gpiote: &'a Gpiote,
    index: u8,
}

impl<'a> Drop for OutputChannel<'a> {
    fn drop(&mut self) {
        self.gpiote.free_channel(self.index);
    }
}

impl<'a> OutputChannel<'a> {
    /// Triggers `task out` (as configured with task_out_polarity, defaults to Toggle).
    pub fn out(&self) {
        self.gpiote.inner.tasks_out[self.index as usize].write(|w| unsafe { w.bits(1) });
    }
    /// Triggers `task set` (set associated pin high).
    #[cfg(not(feature = "51"))]
    pub fn set(&self) {
        self.gpiote.inner.tasks_set[self.index as usize].write(|w| unsafe { w.bits(1) });
    }
    /// Triggers `task clear` (set associated pin low).
    #[cfg(not(feature = "51"))]
    pub fn clear(&self) {
        self.gpiote.inner.tasks_clr[self.index as usize].write(|w| unsafe { w.bits(1) });
    }

    /// Returns reference to task_out endpoint for PPI.
    pub fn task_out(&self) -> &Reg<u32, _TASKS_OUT> {
        &self.gpiote.inner.tasks_out[self.index as usize]
    }

    /// Returns reference to task_clr endpoint for PPI.
    #[cfg(not(feature = "51"))]
    pub fn task_clr(&self) -> &Reg<u32, _TASKS_CLR> {
        &self.gpiote.inner.tasks_clr[self.index as usize]
    }

    /// Returns reference to task_set endpoint for PPI.
    #[cfg(not(feature = "51"))]
    pub fn task_set(&self) -> &Reg<u32, _TASKS_SET> {
        &self.gpiote.inner.tasks_set[self.index as usize]
    }
}

#[interrupt]
unsafe fn GPIOTE() {
    let s = &(*INSTANCE);

    for i in 0..8 {
        if s.inner.events_in[i].read().bits() != 0 {
            s.inner.events_in[i].write(|w| w);
            s.channel_signals[i].signal(());
        }
    }

    if s.inner.events_port.read().bits() != 0 {
        s.inner.events_port.write(|w| w);

        #[cfg(any(feature = "52833", feature = "52840"))]
        let ports = &[&*P0::ptr(), &*P1::ptr()];
        #[cfg(not(any(feature = "52833", feature = "52840")))]
        let ports = &[&*P0::ptr()];

        let mut work = true;
        while work {
            work = false;
            for (port, &p) in ports.iter().enumerate() {
                for pin in BitIter(p.latch.read().bits()) {
                    work = true;
                    p.pin_cnf[pin as usize].modify(|_, w| w.sense().disabled());
                    p.latch.write(|w| w.bits(1 << pin));
                    s.port_signals[port * 32 + pin as usize].signal(());
                }
            }
        }
    }
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
