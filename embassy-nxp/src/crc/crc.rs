#![macro_use]
use crate::pac::syscon::vals::CrcgenRst;
use embassy_hal_internal::{Peri, PeripheralType};

use crate::{pac, peripherals};
// regs.rs -> nxp-pac
// 10 = CRC-32
// 01 = CRC-16
// 00 = CRC-CCITT
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Polynomial {
    // x^16 + x^12 + x^5 + 1
    Ccitt = 0b00,
    // x^16 + x^15 + x^2 + 1
    Crc16 = 0b01,
    // standard CRC-32
    Crc32 = 0b10,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub polynomial: Polynomial,
    //If active every byte written in WR_DATA is reversed before processed
    pub reverse_in: bool,
    //If active every byte written in WR_DATA is complemented ('~byte') before processed
    pub complement_in: bool,
    //If active the result is reversed bit by bit 
    pub reverse_out: bool,
    //If active the result is complemented ('~sum')
    pub complement_out: bool,
    //final value
    pub seed: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polynomial: Polynomial::Ccitt,
            reverse_in: false,
            complement_in: false,
            reverse_out: false,
            complement_out: false,
            seed: 0xFFFF,
        }
    }
}

impl Config {
    //CRC-32 processes reversed bits and compements the final result
    pub fn crc32() -> Self {
        Self {
            polynomial: Polynomial::Crc32,
            reverse_in: true,
            complement_in: false,
            reverse_out: true,
            complement_out: true,
            seed: 0xFFFF_FFFF,
        }
    }
}
//exclusive acces to hardware peripherals through 'Peri<'d, T>'
// guarantees only one active instance at a time for the same peripheral
pub struct Crc<'d, T: Instance> {
    _p: Peri<'d, T>,
}

impl<'d, T: Instance> Crc<'d, T> {
    //Initialize CRC peripherals
    pub fn new(peri: Peri<'d, T>, config: Config) -> Self {
        //clock enable and reset hardware
        T::enable_and_reset();

        let r = T::regs();
        //configure mode register
        r.mode().write(|w| {
            w.set_crc_poly(config.polynomial as u8);
            w.set_bit_rvs_wr(config.reverse_in);
            w.set_cmpl_wr(config.complement_in);
            w.set_bit_rvs_sum(config.reverse_out);
            w.set_cmpl_sum(config.complement_out);
        });
        //load the inital seed value
        r.seed().write_value(pac::crc_engine::regs::Seed(config.seed));

        Self { _p: peri }
    }
    //reloads only the seed register
    pub fn reset(&mut self, seed: u32) {
        T::regs().seed().write_value(pac::crc_engine::regs::Seed(seed));
    }
    //sends data buffer to CRC engine
    pub fn feed(&mut self, data: &[u8]) {
        //WR_DATA register address : offset 0x08 in CRC block
        let base = T::regs().as_ptr() as *mut u8;
        let wr_data_addr = unsafe { base.add(0x08) };
        let mut data = data;
        // write bytes until buffer alligned 4 bytes
        while !data.is_empty() && (data.as_ptr() as usize) & 3 != 0 {
            unsafe { core::ptr::write_volatile(wr_data_addr, data[0]) };
            data = &data[1..];
        }
        //write in 32 bytes blocks
        let wr_data_addr32 = wr_data_addr as *mut u32;
        let chunks = data.chunks_exact(4);
        let remainder = chunks.remainder();
        for chunk in chunks {
            let word = u32::from_ne_bytes(chunk.try_into().unwrap());
            unsafe { core::ptr::write_volatile(wr_data_addr32, word) };
        }
        //write remainder bytes
        for &byte in remainder {
            unsafe { core::ptr::write_volatile(wr_data_addr, byte) };
        }
    }
    //return 32 byte result
    pub fn sum32(&self) -> u32 {
        T::regs().sum().read().0
    }
    //returns 16 byte result
    pub fn sum16(&self) -> u16 {
        T::regs().sum().read().0 as u16
    }
    //reads internal state of CRC engine
    // used by seed when data is recieved in more than one set
    pub fn checkpoint(&mut self) -> u32 {
        let r = T::regs();
        let mode = r.mode().read();

        r.mode().modify(|w| {
            w.set_bit_rvs_sum(false);
            w.set_cmpl_sum(false);
        });

        let raw = r.sum().read().0;
        //reset to original config
        r.mode().write_value(mode);

        raw
    }
}

impl<'d, T: Instance> Drop for Crc<'d, T> {
    fn drop(&mut self) {
        //disable clock when driver is destroyed
        T::disable();
    }
}
//gives acces to peripherals registers
pub(crate) trait SealedInstance {
    fn regs() -> pac::crc_engine::CrcEngine;
}
// trait implemented for each existing CRC hardware instance on chip
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    fn enable_and_reset();
    fn disable();
}

impl SealedInstance for peripherals::CRC_ENGINE {
    fn regs() -> pac::crc_engine::CrcEngine {
        pac::CRC_ENGINE
    }
}

impl Instance for peripherals::CRC_ENGINE {
    fn enable_and_reset() {
        //activate CRC clock
        pac::SYSCON.ahbclkctrl0().modify(|w| w.set_crcgen(true));
        //reset hardware
        pac::SYSCON.presetctrl0().modify(|w| w.set_crcgen_rst(CrcgenRst::ASSERTED));
        pac::SYSCON.presetctrl0().modify(|w| w.set_crcgen_rst(CrcgenRst::RELEASED)); 
    }
    fn disable() {
        pac::SYSCON.ahbclkctrl0().modify(|w| w.set_crcgen(false)); 
    }
}
