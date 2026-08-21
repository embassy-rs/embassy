//! Blocking Dallas/Maxim 1-Wire bus using any GPIO pin.

use core::ptr::{read_volatile, write_volatile};

use crate::gpio::Pin;

/// Bit-banged 1-Wire master.
///
/// The bus requires an external pull-up resistor, normally about 4.7 kOhm.
/// Timing-critical slots temporarily disable interrupts.
pub struct OneWire<P: Pin> {
    pin: P,
}

impl<P: Pin> OneWire<P> {
    /// Creates a 1-Wire bus and releases the line to the external pull-up.
    pub fn new(pin: P) -> Self {
        avr_device::interrupt::free(|_| release::<P>());
        Self { pin }
    }

    /// Sends a reset pulse and returns whether a presence pulse was detected.
    pub fn reset(&mut self) -> bool {
        avr_device::interrupt::free(|_| {
            drive_low::<P>();
            delay_us(480);
            release::<P>();
            delay_us(70);
            let present = !read_level::<P>();
            delay_us(410);
            present
        })
    }

    /// Writes one least-significant-bit-first bus bit.
    pub fn write_bit(&mut self, bit: bool) {
        avr_device::interrupt::free(|_| {
            drive_low::<P>();
            if bit {
                delay_us(6);
                release::<P>();
                delay_us(64);
            } else {
                delay_us(60);
                release::<P>();
                delay_us(10);
            }
        });
    }

    /// Reads one bus bit.
    pub fn read_bit(&mut self) -> bool {
        avr_device::interrupt::free(|_| {
            drive_low::<P>();
            delay_us(6);
            release::<P>();
            delay_us(9);
            let bit = read_level::<P>();
            delay_us(55);
            bit
        })
    }

    /// Writes one byte, least-significant bit first.
    pub fn write_byte(&mut self, byte: u8) {
        for bit in 0..8 {
            self.write_bit(byte & (1 << bit) != 0);
        }
    }

    /// Reads one byte, least-significant bit first.
    pub fn read_byte(&mut self) -> u8 {
        let mut byte = 0;
        for bit in 0..8 {
            if self.read_bit() {
                byte |= 1 << bit;
            }
        }
        byte
    }

    /// Sends the Skip ROM command (`0xCC`).
    pub fn skip_rom(&mut self) {
        self.write_byte(0xcc);
    }

    /// Selects one device by its eight-byte ROM code.
    pub fn match_rom(&mut self, rom: &[u8; 8]) {
        self.write_byte(0x55);
        for &byte in rom {
            self.write_byte(byte);
        }
    }

    /// Releases and returns the owned GPIO pin.
    pub fn free(self) -> P {
        avr_device::interrupt::free(|_| release::<P>());
        self.pin
    }
}

/// Computes the Dallas/Maxim CRC-8 used by 1-Wire devices.
pub fn crc8(bytes: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &byte in bytes {
        let mut value = byte;
        for _ in 0..8 {
            let mix = (crc ^ value) & 1;
            crc >>= 1;
            if mix != 0 {
                crc ^= 0x8c;
            }
            value >>= 1;
        }
    }
    crc
}

fn drive_low<P: Pin>() {
    unsafe {
        modify(P::PORT, P::MASK, false);
        modify(P::DDR, P::MASK, true);
    }
}

fn release<P: Pin>() {
    unsafe {
        modify(P::DDR, P::MASK, false);
        modify(P::PORT, P::MASK, false);
    }
}

fn read_level<P: Pin>() -> bool {
    unsafe { read_volatile(P::PIN) & P::MASK != 0 }
}

fn delay_us(us: u32) {
    avr_device::asm::delay_cycles(us.saturating_mul(crate::CPU_HZ / 1_000_000));
}

unsafe fn modify(register: *mut u8, mask: u8, set: bool) {
    let current = unsafe { read_volatile(register) };
    let value = if set { current | mask } else { current & !mask };
    unsafe { write_volatile(register, value) };
}
