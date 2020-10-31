use core::cell::Cell;
use core::ptr;
use defmt::trace;
use embassy::util::Signal;

use crate::hal::gpio::{Input, Level, Output, Pin, Port};
use crate::interrupt;
use crate::pac::generic::Reg;
use crate::pac::gpiote::_TASKS_OUT;
use crate::pac::GPIOTE;

#[cfg(not(feature = "51"))]
use crate::pac::gpiote::{_TASKS_CLR, _TASKS_SET};

pub struct Gpiote {
    inner: GPIOTE,
    free_channels: Cell<u8>, // 0 = used, 1 = free. 8 bits for 8 channelself.
    signals: [Signal<()>; 8],
}

static mut INSTANCE: *const Gpiote = ptr::null_mut();

pub enum EventPolarity {
    None,
    HiToLo,
    LoToHi,
    Toggle,
}

/// Polarity of the `task out` operation.
pub enum TaskOutPolarity {
    Set,
    Clear,
    Toggle,
}

#[derive(defmt::Format)]
pub enum NewChannelError {
    NoFreeChannels,
}

impl Gpiote {
    pub fn new(gpiote: GPIOTE) -> Self {
        interrupt::unpend(interrupt::GPIOTE);
        interrupt::enable(interrupt::GPIOTE);

        Self {
            inner: gpiote,
            free_channels: Cell::new(0xFF), // all 8 channels free
            signals: [
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

    pub fn new_input_channel<'a, T>(
        &'a self,
        pin: Pin<Input<T>>,
        trigger_mode: EventPolarity,
    ) -> Result<InputChannel<'a, T>, NewChannelError> {
        interrupt::free(|_| {
            unsafe { INSTANCE = self };
            let index = self.allocate_channel()?;
            trace!("allocated in ch {:u8}", index as u8);

            self.inner.config[index as usize].write(|w| {
                match trigger_mode {
                    EventPolarity::HiToLo => w.mode().event().polarity().hi_to_lo(),
                    EventPolarity::LoToHi => w.mode().event().polarity().lo_to_hi(),
                    EventPolarity::None => w.mode().event().polarity().none(),
                    EventPolarity::Toggle => w.mode().event().polarity().toggle(),
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
        task_out_polarity: TaskOutPolarity,
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
                match task_out_polarity {
                    TaskOutPolarity::Set => w.polarity().lo_to_hi(),
                    TaskOutPolarity::Clear => w.polarity().hi_to_lo(),
                    TaskOutPolarity::Toggle => w.polarity().toggle(),
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
    pub async fn wait(&self) -> () {
        self.gpiote.signals[self.index as usize].wait().await;
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
            s.signals[i].signal(());
        }
    }
}
