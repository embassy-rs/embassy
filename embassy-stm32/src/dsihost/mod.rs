//! DSI HOST

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::dsihost::panel::DsiPanel;
use crate::gpio::{AfType, Flex};
use crate::interrupt::typelevel::Interrupt;
use crate::peripherals::{self};
use crate::rcc::dsi::DSI_CONFIG;
use crate::rcc::{self, RccPeripheral};
use crate::time::MaybeHertz;
use crate::{Peri, block_for_us, interrupt};

mod phy;
pub use phy::{DsiHostPhyConfig, DsiHostPhyLanes};

mod mode;
pub use mode::{
    DsiColor, DsiCommandConfig, DsiHostMode, DsiLtdcRefreshMode, DsiTearEventSource, DsiVideoConfig, DsiVideoMode,
};

pub mod panel;

static DSIHOST_WAKER: AtomicWaker = AtomicWaker::new();

/// DSIHOST interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
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
    _te: Flex<'d>,

    /// Lane byte clock frequency (62MHz max)
    /// Can be mux'd from PHY clock / 8, or PLL2Q using RCC D1CCIPR.DSISEL
    lane_byte_clock: MaybeHertz,

    /// TX escape clock frequency (20MHz max)
    /// Derived from pixel clock and TX Prescaler
    tx_escape_clock: MaybeHertz,
}

impl<'d, T: Instance> DsiHost<'d, T> {
    /// Note: Full-Duplex modes are not supported at this time
    pub fn new(_peri: Peri<'d, T>, te: Peri<'d, impl TePin<T>>) -> Self {
        rcc::enable_and_reset::<T>();

        set_as_af!(te, AfType::input(crate::gpio::Pull::Down));

        Self {
            _peri: PhantomData,
            _te: Flex::new(te),
            lane_byte_clock: None.into(),
            tx_escape_clock: None.into(),
        }
    }

    /// Enable the PLL and wait for the lock interrupt
    async fn enable_pll(&self) {
        let config = unsafe { DSI_CONFIG }.unwrap();

        // Set the PLL configuration
        T::regs().wrpcr().modify(|w| {
            w.set_ndiv(config.ndiv);

            #[cfg(dsihost_v1)]
            {
                w.set_idf(config.idf as u8);
                w.set_odf(config.odf as u8);
            }

            #[cfg(dsihost_u5)]
            {
                w.set_idf(config.idf);
                w.set_odf(config.odf);
            }
        });

        poll_fn(|cx| {
            let status = T::regs().wisr().read();

            if status.plllif() || status.pllls() {
                T::regs().wifcr().modify(|w| w.set_cplllif(true));
                Poll::Ready(())
            } else {
                DSIHOST_WAKER.register(cx.waker());

                T::regs().wifcr().modify(|w| w.set_cplllif(true));
                T::regs().wifcr().modify(|w| w.set_cplluif(true));
                T::regs().wier().modify(|w| w.set_plllie(true));
                Self::enable_interrupts(true);

                // Set the PLL enable bit and wait for lock
                T::regs().wrpcr().modify(|w| w.set_pllen(true));

                Poll::Pending
            }
        })
        .await;
    }

    /// Start the DSI host
    ///
    /// LTDC should be initialized before starting DSIHOST
    ///
    /// * Enable regulator
    /// * Initialize PLL and wait for lock
    /// * Initialize PHY
    /// * Initialize Mode
    /// * Initialize Panel
    /// * Enable DSI peripheral and DSI Wrapper
    ///
    pub async fn start_panel<Panel: DsiPanel>(
        &mut self,
        phy_config: &DsiHostPhyConfig,
        mode: &DsiHostMode,
    ) -> Result<(), Error> {
        #[cfg(dsihost_v1)]
        self.enable_regulator().await;

        self.enable_pll().await;

        self.phy_init(phy_config);

        self.set_mode::<Panel>(mode)?;

        self.enable();

        self.enable_wrapper_dsi();

        let color = match mode {
            DsiHostMode::Video(config) => config.color,
            DsiHostMode::AdaptedCommand(config) => {
                // Enable tearing output
                self.write_cmd(0, 0x35, &[0x0])?;

                config.color
            }
        };

        self.init_panel::<Panel>(color).await?;

        Ok(())
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

    /// Enable the regulator
    #[cfg(dsihost_v1)]
    pub async fn enable_regulator(&mut self) {
        T::regs().wrpcr().modify(|w| w.set_regen(true));

        poll_fn(|cx| {
            if T::regs().wisr().read().rrif() {
                T::regs().wifcr().modify(|w| w.set_crrif(true));
                Poll::Ready(())
            } else {
                DSIHOST_WAKER.register(cx.waker());
                T::regs().wifcr().modify(|w| w.set_crrif(true));
                T::regs().wier().modify(|w| w.set_rrie(true));
                Self::enable_interrupts(true);
                Poll::Pending
            }
        })
        .await;
    }

    /// Wait for a tear interrupt event
    pub async fn wait_tear(&mut self) {
        poll_fn(|cx| {
            if T::regs().wisr().read().teif() {
                T::regs().wifcr().modify(|w| w.set_cteif(true));
                Self::enable_interrupts(true);
                Poll::Ready(())
            } else {
                DSIHOST_WAKER.register(cx.waker());
                T::regs().wier().modify(|w| w.set_teie(true));
                Self::enable_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    /// Wait for an end of refresh interrupt
    pub async fn wait_refresh(&mut self) {
        poll_fn(|cx| {
            if T::regs().wisr().read().erif() {
                T::regs().wifcr().modify(|w| w.set_cerif(true));
                Self::enable_interrupts(true);
                Poll::Ready(())
            } else {
                DSIHOST_WAKER.register(cx.waker());
                T::regs().wier().modify(|w| w.set_erie(true));
                Self::enable_interrupts(true);
                Poll::Pending
            }
        })
        .await
    }

    /// DCS or Generic short/long write command
    pub fn write_cmd(&mut self, channel_id: u8, address: u8, data: &[u8]) -> Result<(), Error> {
        match data.len() {
            0 => self.short_write(channel_id, PacketType::DcsShortPktWriteP0, address, 0),
            1 => self.short_write(channel_id, PacketType::DcsShortPktWriteP1, address, data[0]),
            _ => self.long_write(
                channel_id,
                PacketType::DcsLongPktWrite, // FIXME: This might be a generic long packet, as well...
                address,
                data,
            ),
        }
    }

    fn short_write(&mut self, channel_id: u8, packet_type: PacketType, param1: u8, param2: u8) -> Result<(), Error> {
        // Wait for Command FIFO empty
        self.wait_command_fifo_empty()?;

        // Configure the packet to send a short DCS command with 0 or 1 parameters
        // Update the DSI packet header with new information
        self.config_packet_header(channel_id, packet_type, param1, param2);

        self.wait_command_fifo_empty()?;

        let status = T::regs().isr1().read();
        if status.0 != 0 {
            error!("ISR1 after short_write(): {}", status);
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

        self.wait_command_fifo_empty()?;

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
            block_for_us(1_000);
        }
        Err(Error::FifoTimeout)
    }

    fn wait_command_fifo_not_full(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for Command FIFO not empty
            if !T::regs().gpsr().read().cmdff() {
                return Ok(());
            }
            block_for_us(1_000);
        }
        Err(Error::FifoTimeout)
    }

    fn wait_read_not_busy(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for read not busy
            if !self.read_busy() {
                return Ok(());
            }
            block_for_us(1_000);
        }
        Err(Error::ReadTimeout)
    }

    fn wait_payload_read_fifo_not_empty(&self) -> Result<(), Error> {
        for _ in 1..1000 {
            // Wait for payload read FIFO not empty
            if !T::regs().gpsr().read().prdfe() {
                return Ok(());
            }
            block_for_us(1_000);
        }
        Err(Error::FifoTimeout)
    }

    fn _packet_size_error(&self) -> bool {
        T::regs().isr1().read().pse()
    }

    fn read_busy(&self) -> bool {
        T::regs().gpsr().read().rcb()
    }

    /// Enable interrupts
    fn enable_interrupts(enable: bool) {
        T::Interrupt::unpend();
        if enable {
            unsafe { T::Interrupt::enable() };
        } else {
            T::Interrupt::disable()
        }
    }
}

/// Possible Error Types for DSI HOST
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    /// Color coding not supported by panel
    ColorNotSupported,
}

impl<'d, T: Instance> Drop for DsiHost<'d, T> {
    fn drop(&mut self) {}
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        cortex_m::asm::dsb();
        DsiHost::<T>::enable_interrupts(false);

        let isr0 = T::regs().isr0().read();
        let isr1 = T::regs().isr1().read();
        if isr0.0 != 0 {
            warn!("{}", isr0);
            unsafe { T::Interrupt::enable() };
        }

        if isr1.0 != 0 {
            warn!("{}", isr1);
            unsafe { T::Interrupt::enable() };
        }

        //let wisr = T::regs().wisr().read();
        //info!("{}", wisr);

        DSIHOST_WAKER.wake();
    }
}

trait SealedInstance: crate::rcc::SealedRccPeripheral {
    fn regs() -> crate::pac::dsihost::Dsihost;
}

/// DSI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + RccPeripheral + 'static + Send {
    /// Interrupt for this DSI instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

pin_trait!(TePin, Instance);

foreach_interrupt!(
    ($inst:ident, dsihost, DSIHOST, GLOBAL, $irq:ident) => {
        impl Instance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::dsihost::Dsihost {
                crate::pac::$inst
            }
        }
    };
);
