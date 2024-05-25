//! DSI HOST

use core::marker::PhantomData;

use embassy_hal_internal::{into_ref, PeripheralRef};

//use crate::gpio::{AnyPin, SealedPin};
use crate::gpio::{AFType, AnyPin, Pull, Speed};
use crate::rcc::{self, RccPeripheral};
use crate::{peripherals, Peripheral};

/// Performs a busy-wait delay for a specified number of microseconds.
pub fn blocking_delay_ms(ms: u32) {
    #[cfg(feature = "time")]
    embassy_time::block_for(embassy_time::Duration::from_millis(ms as u64));
    #[cfg(not(feature = "time"))]
    cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.unwrap().0 / 1_000 * ms);
}

/// PacketTypes extracted from CubeMX
#[repr(u8)]
#[allow(dead_code)]
pub enum PacketType {
    /// DCS short write, no parameters
    DcsShortPktWriteP0,
    /// DCS short write, one parameter
    DcsShortPktWriteP1,
    /// Generic short write, no parameters
    GenShortPktWriteP0,
    /// Generic short write, one parameter
    GenShortPktWriteP1,
    /// Generic short write, two parameters
    GenShortPktWriteP2,
    /// DCS long write
    DcsLongPktWrite,
    /// Generic long write
    GenLongPktWrite,
    /// DCS short read
    DcsShortPktRead(u8),
    /// Generic short read, no parameters
    GenShortPktReadP0,
    /// Generic short read, one parameter
    GenShortPktReadP1(u8),
    /// Generic short read, two parameters
    GenShortPktReadP2(u8, u8),
    /// Used to set the maximum return packet size for reading data
    MaxReturnPktSize,
}

impl From<PacketType> for u8 {
    fn from(packet_type: PacketType) -> u8 {
        match packet_type {
            PacketType::DcsShortPktWriteP0 => 0x05,
            PacketType::DcsShortPktWriteP1 => 0x15,
            PacketType::GenShortPktWriteP0 => 0x03,
            PacketType::GenShortPktWriteP1 => 0x13,
            PacketType::GenShortPktWriteP2 => 0x23,
            PacketType::DcsLongPktWrite => 0x39,
            PacketType::GenLongPktWrite => 0x29,
            PacketType::DcsShortPktRead(_) => 0x06,
            PacketType::GenShortPktReadP0 => 0x04,
            PacketType::GenShortPktReadP1(_) => 0x14,
            PacketType::GenShortPktReadP2(_, _) => 0x24,
            PacketType::MaxReturnPktSize => 0x37,
        }
    }
}

/// DSIHOST driver.
pub struct DsiHost<'d, T: Instance> {
    _peri: PhantomData<&'d mut T>,
    _te: PeripheralRef<'d, AnyPin>,
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Note: Full-Duplex modes are not supported at this time
    pub fn new(_peri: impl Peripheral<P = T> + 'd, te: impl Peripheral<P = impl TePin<T>> + 'd) -> Self {
        into_ref!(te);

        rcc::enable_and_reset::<T>();

        // Set Tearing Enable pin according to CubeMx example
        te.set_as_af_pull(te.af_num(), AFType::OutputPushPull, Pull::None);
        te.set_speed(Speed::Low);
        /*
                T::regs().wcr().modify(|w| {
                    w.set_dsien(true);
                });
        */
        Self {
            _peri: PhantomData,
            _te: te.map_into(),
        }
    }

    /// Get the DSIHOST hardware version. Found in the reference manual for comparison.
    pub fn get_version(&self) -> u32 {
        T::regs().vr().read().version()
    }

    /// Set the enable bit in the control register and assert that it has been enabled
    pub fn enable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(true));
        assert!(T::regs().cr().read().en())
    }

    /// Unset the enable bit in the control register and assert that it has been disabled
    pub fn disable(&mut self) {
        T::regs().cr().modify(|w| w.set_en(false));
        assert!(!T::regs().cr().read().en())
    }

    /// Set the DSI enable bit in the wrapper control register and assert that it has been enabled
    pub fn enable_wrapper_dsi(&mut self) {
        T::regs().wcr().modify(|w| w.set_dsien(true));
        assert!(T::regs().wcr().read().dsien())
    }

    /// Unset the DSI enable bit in the wrapper control register and assert that it has been disabled
    pub fn disable_wrapper_dsi(&mut self) {
        T::regs().wcr().modify(|w| w.set_dsien(false));
        assert!(!T::regs().wcr().read().dsien())
    }

    /// DCS or Generic short/long write command
    pub fn write_cmd(&mut self, channel_id: u8, address: u8, data: &[u8]) -> Result<(), Error> {
        assert!(data.len() > 0);

        if data.len() == 1 {
            self.short_write(channel_id, PacketType::DcsShortPktWriteP1, address, data[0])
        } else {
            self.long_write(
                channel_id,
                PacketType::DcsLongPktWrite, // FIXME: This might be a generic long packet, as well...
                address,
                data,
            )
        }
    }

    fn short_write(&mut self, channel_id: u8, packet_type: PacketType, param1: u8, param2: u8) -> Result<(), Error> {
        #[cfg(feature = "defmt")]
        defmt::debug!("short_write: BEGIN wait for command fifo empty");

        // Wait for Command FIFO empty
        self.wait_command_fifo_empty()?;
        #[cfg(feature = "defmt")]
        defmt::debug!("short_write: END wait for command fifo empty");

        // Configure the packet to send a short DCS command with 0 or 1 parameters
        // Update the DSI packet header with new information
        self.config_packet_header(channel_id, packet_type, param1, param2);

        self.wait_command_fifo_empty()?;

        let status = T::regs().isr1().read().0;
        if status != 0 {
            error!("ISR1 after short_write(): {:b}", status);
        }
        Ok(())
    }

    fn config_packet_header(&mut self, channel_id: u8, packet_type: PacketType, param1: u8, param2: u8) {
        T::regs().ghcr().write(|w| {
            w.set_dt(packet_type.into());
            w.set_vcid(channel_id);
            w.set_wclsb(param1);
            w.set_wcmsb(param2);
        });
    }

    /// Write long DCS or long Generic command.
    ///
    /// `params` is expected to contain at least 2 elements. Use [`short_write`] for a single element.
    fn long_write(&mut self, channel_id: u8, packet_type: PacketType, address: u8, data: &[u8]) -> Result<(), Error> {
        // Must be a long packet if we do the long write, obviously.
        assert!(matches!(
            packet_type,
            PacketType::DcsLongPktWrite | PacketType::GenLongPktWrite
        ));

        // params needs to have at least 2 elements, otherwise short_write should be used
        assert!(data.len() >= 2);

        #[cfg(feature = "defmt")]
        defmt::debug!("long_write: BEGIN wait for command fifo empty");

        self.wait_command_fifo_empty()?;

        #[cfg(feature = "defmt")]
        defmt::debug!("long_write: DONE wait for command fifo empty");

        // Note: CubeMX example "NbParams" is always one LESS than params.len()
        // DCS code (last element of params) must be on payload byte 1 and if we have only 2 more params,
        // then they must go into data2 and data3
        T::regs().gpdr().write(|w| {
            // data[2] may or may not exist.
            if let Some(x) = data.get(2) {
                w.set_data4(*x);
            }
            // data[0] and [1] have to exist if long_write is called.
            w.set_data3(data[1]);
            w.set_data2(data[0]);

            // DCS Code
            w.set_data1(address);
        });

        self.wait_command_fifo_empty()?;

        // These steps are only necessary if more than 1x 4 bytes need to go into the FIFO
        if data.len() >= 4 {
            // Generate an iterator that iterates over chunks of exactly 4 bytes
            let iter = data[3..data.len()].chunks_exact(4);
            // Obtain remainder before consuming iter
            let remainder = iter.remainder();

            // Keep filling the buffer with remaining data
            for param in iter {
                self.wait_command_fifo_not_full()?;
                T::regs().gpdr().write(|w| {
                    w.set_data4(param[3]);
                    w.set_data3(param[2]);
                    w.set_data2(param[1]);
                    w.set_data1(param[0]);
                });

                self.wait_command_fifo_empty().unwrap();
            }

            // If the remaining data was not devisible by 4 we get a remainder
            if remainder.len() >= 1 {
                self.wait_command_fifo_not_full()?;
                T::regs().gpdr().write(|w| {
                    if let Some(x) = remainder.get(2) {
                        w.set_data3(*x);
                    }
                    if let Some(x) = remainder.get(1) {
                        w.set_data2(*x);
                    }
                    w.set_data1(remainder[0]);
                });
                self.wait_command_fifo_empty().unwrap();
            }
        }
        // Configure the packet to send a long DCS command
        self.config_packet_header(
            channel_id,
            packet_type,
            ((data.len() + 1) & 0x00FF) as u8,        // +1 to account for address byte
            (((data.len() + 1) & 0xFF00) >> 8) as u8, // +1 to account for address byte
        );

        self.wait_command_fifo_empty()?;

        let status = T::regs().isr1().read().0;
        if status != 0 {
            error!("ISR1 after long_write(): {:b}", status);
        }
        Ok(())
    }

    /// Read DSI Register
    pub fn read(
        &mut self,
        channel_id: u8,
        packet_type: PacketType,
        read_size: u16,
        data: &mut [u8],
    ) -> Result<(), Error> {
        if data.len() != read_size as usize {
            return Err(Error::InvalidReadSize);
        }

        // Set the maximum return packet size
        self.short_write(
            channel_id,
            PacketType::MaxReturnPktSize,
            (read_size & 0xFF) as u8,
            ((read_size & 0xFF00) >> 8) as u8,
        )?;

        // Set the packet header according to the packet_type
        use PacketType::*;
        match packet_type {
            DcsShortPktRead(cmd) => self.config_packet_header(channel_id, packet_type, cmd, 0),
            GenShortPktReadP0 => self.config_packet_header(channel_id, packet_type, 0, 0),
            GenShortPktReadP1(param1) => self.config_packet_header(channel_id, packet_type, param1, 0),
            GenShortPktReadP2(param1, param2) => self.config_packet_header(channel_id, packet_type, param1, param2),
            _ => return Err(Error::InvalidPacketType),
        }

        self.wait_read_not_busy()?;

        // Obtain chunks of 32-bit so the entire FIFO data register can be read
        for bytes in data.chunks_exact_mut(4) {
            self.wait_payload_read_fifo_not_empty()?;

            // Only perform a single read on the entire register to avoid unintended side-effects
            let gpdr = T::regs().gpdr().read();
            bytes[0] = gpdr.data1();
            bytes[1] = gpdr.data2();
            bytes[2] = gpdr.data3();
            bytes[3] = gpdr.data4();
        }

        // Collect the remaining chunks and read the corresponding number of bytes from the FIFO
        let remainder = data.chunks_exact_mut(4).into_remainder();
        if !remainder.is_empty() {
            self.wait_payload_read_fifo_not_empty()?;
            // Only perform a single read on the entire register to avoid unintended side-effects
            let gpdr = T::regs().gpdr().read();
            if let Some(x) = remainder.get_mut(0) {
                *x = gpdr.data1()
            }
            if let Some(x) = remainder.get_mut(1) {
                *x = gpdr.data2()
            }
            if let Some(x) = remainder.get_mut(2) {
                *x = gpdr.data3()
            }
        }

        /*
        // Used this to check whether there are read errors. Does not seem like it.
        if !self.read_busy() {
            defmt::debug!("Read not busy!");
            if self.packet_size_error() {
                return Err(Error::ReadError);
            }
        }
        */
        Ok(())
    }

    fn wait_command_fifo_empty(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for Command FIFO empty
            if T::regs().gpsr().read().cmdfe() {
                return Ok(());
            }
            blocking_delay_ms(1);
        }
        Err(Error::FifoTimeout)
    }

    fn wait_command_fifo_not_full(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for Command FIFO not empty
            if !T::regs().gpsr().read().cmdff() {
                return Ok(());
            }
            blocking_delay_ms(1);
        }
        Err(Error::FifoTimeout)
    }

    fn wait_read_not_busy(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for read not busy
            if !self.read_busy() {
                return Ok(());
            }
            blocking_delay_ms(1);
        }
        Err(Error::ReadTimeout)
    }

    fn wait_payload_read_fifo_not_empty(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for payload read FIFO not empty
            if !T::regs().gpsr().read().prdfe() {
                return Ok(());
            }
            blocking_delay_ms(1);
        }
        Err(Error::FifoTimeout)
    }

    fn _packet_size_error(&self) -> bool {
        T::regs().isr1().read().pse()
    }

    fn read_busy(&self) -> bool {
        T::regs().gpsr().read().rcb()
    }
}

/// Possible Error Types for DSI HOST
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// Waiting for FIFO empty flag timed out
    FifoTimeout,
    /// The specified `PacketType` is invalid for the selected operation
    InvalidPacketType,
    /// Provided read size does not match the read buffer length
    InvalidReadSize,
    /// Error during read
    ReadError,
    /// Read operation timed out
    ReadTimeout,
}

impl<'d, T: Instance> Drop for DsiHost<'d, T> {
    fn drop(&mut self) {}
}

trait SealedInstance: crate::rcc::SealedRccPeripheral {
    fn regs() -> crate::pac::dsihost::Dsihost;
}

/// DSI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {}

pin_trait!(TePin, Instance);

foreach_peripheral!(
    (dsihost, $inst:ident) => {
        impl crate::dsihost::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::dsihost::Dsihost {
                crate::pac::$inst
            }
        }

        impl crate::dsihost::Instance for peripherals::$inst {}
    };
);
