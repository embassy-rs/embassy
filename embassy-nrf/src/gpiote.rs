use core::cell::Cell;
use core::pin::Pin;
use core::ptr;
use defmt::trace;
use embassy::util::Signal;
use nrf52840_hal::gpio::{Floating, Input, Pin as GpioPin, Port};

use crate::interrupt;
use crate::pac::GPIOTE;

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

#[derive(defmt::Format)]
pub enum NewChannelError {
    NoFreeChannels,
}

impl Gpiote {
    pub fn new(gpiote: GPIOTE) -> Self {
        let signal: Signal<()> = Signal::new();

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

    pub fn new_input_channel<'a, T>(
        &'a self,
        pin: &'a GpioPin<Input<T>>,
        trigger_mode: EventPolarity,
    ) -> Result<Channel<'a>, NewChannelError> {
        interrupt::free(|_| {
            unsafe { INSTANCE = self };

            let chs = self.free_channels.get();
            let index = chs.trailing_zeros() as usize;
            if index == 8 {
                return Err(NewChannelError::NoFreeChannels);
            }
            self.free_channels.set(chs & !(1 << index));

            trace!("allocated ch {:u8}", index as u8);

            self.inner.config[index].write(|w| {
                match trigger_mode {
                    EventPolarity::HiToLo => w.mode().event().polarity().hi_to_lo(),
                    EventPolarity::LoToHi => w.mode().event().polarity().lo_to_hi(),
                    EventPolarity::None => w.mode().event().polarity().none(),
                    EventPolarity::Toggle => w.mode().event().polarity().toggle(),
                };
                w.port().bit(match pin.port() {
                    Port::Port0 => false,
                    Port::Port1 => true,
                });
                unsafe { w.psel().bits(pin.pin()) }
            });

            // Enable interrupt
            self.inner.intenset.write(|w| unsafe { w.bits(1 << index) });

            Ok(Channel {
                gpiote: self,
                index: index as u8,
            })
        })
    }
}

pub struct Channel<'a> {
    gpiote: &'a Gpiote,
    index: u8,
}

impl<'a> Drop for Channel<'a> {
    fn drop(&mut self) {
        let g = unsafe { Pin::new_unchecked(self.gpiote) };

        interrupt::free(|_| {
            self.gpiote.inner.config[self.index as usize].write(|w| w.mode().disabled());
            self.gpiote
                .inner
                .intenclr
                .write(|w| unsafe { w.bits(1 << self.index) });

            self.gpiote
                .free_channels
                .set(self.gpiote.free_channels.get() | 1 << self.index);
            trace!("freed ch {:u8}", self.index);
        })
    }
}

impl<'a> Channel<'a> {
    pub async fn wait(&self) -> () {
        self.gpiote.signals[self.index as usize].wait().await;
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
