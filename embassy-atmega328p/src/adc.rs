//! Blocking 10-bit analog-to-digital converter.

use core::ptr::{read_volatile, write_volatile};

use crate::gpio::{PC0, PC1, PC2, PC3, PC4, PC5};

const ADCL: *mut u8 = 0x78 as *mut u8;
const ADCH: *mut u8 = 0x79 as *mut u8;
const ADCSRA: *mut u8 = 0x7a as *mut u8;
const ADMUX: *mut u8 = 0x7c as *mut u8;
const DIDR0: *mut u8 = 0x7e as *mut u8;

const ADEN: u8 = 1 << 7;
const ADSC: u8 = 1 << 6;

/// Ownership token for the ADC peripheral.
pub struct AdcPeripheral {
    _private: (),
}

impl AdcPeripheral {
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

/// ADC voltage reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reference {
    /// External voltage applied to AREF.
    Aref,
    /// AVCC, normally 5 V or 3.3 V depending on the board.
    Avcc,
    /// Internal nominal 1.1 V reference.
    Internal1V1,
}

/// A pin that can be connected to the ADC multiplexer.
pub trait AdcPin: sealed::AdcPin {}

mod sealed {
    pub trait AdcPin {
        const CHANNEL: u8;
    }
}

macro_rules! adc_pin {
    ($pin:ty, $channel:expr) => {
        impl sealed::AdcPin for $pin {
            const CHANNEL: u8 = $channel;
        }
        impl AdcPin for $pin {}
    };
}

adc_pin!(PC0, 0);
adc_pin!(PC1, 1);
adc_pin!(PC2, 2);
adc_pin!(PC3, 3);
adc_pin!(PC4, 4);
adc_pin!(PC5, 5);

/// Blocking ADC driver.
pub struct Adc {
    peripheral: AdcPeripheral,
    reference: Reference,
}

impl Adc {
    /// Enables the ADC with a conversion clock near 125 kHz.
    pub fn new(peripheral: AdcPeripheral, reference: Reference) -> Self {
        let prescaler = if crate::CPU_HZ == 16_000_000 {
            0b111 // /128
        } else {
            0b110 // /64
        };
        unsafe { write_volatile(ADCSRA, ADEN | prescaler) };
        Self { peripheral, reference }
    }

    /// Performs one blocking 10-bit conversion.
    pub fn read<P: AdcPin>(&mut self, _pin: &mut P) -> u16 {
        let reference = match self.reference {
            Reference::Aref => 0b00,
            Reference::Avcc => 0b01,
            Reference::Internal1V1 => 0b11,
        };

        unsafe {
            modify(DIDR0, 1 << P::CHANNEL, true);
            write_volatile(ADMUX, (reference << 6) | P::CHANNEL);
            modify(ADCSRA, ADSC, true);
            while read_volatile(ADCSRA) & ADSC != 0 {}

            // ADCL must be read before ADCH.
            let low = read_volatile(ADCL);
            let high = read_volatile(ADCH);
            u16::from_le_bytes([low, high])
        }
    }

    /// Disables the ADC and returns its ownership token.
    pub fn free(self) -> AdcPeripheral {
        unsafe { modify(ADCSRA, ADEN, false) };
        self.peripheral
    }
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
