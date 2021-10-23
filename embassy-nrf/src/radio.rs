#![macro_use]

use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::pac;
use crate::util::{slice_in_ram, slice_in_ram_or};
use core::convert;
use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::traits;
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

#[derive(Clone)]
pub struct Frequency {
    val: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    TxBufferZeroLength,
    RxBufferZeroLength,
    Transmit,
    Receive,
    DMABufferNotInDataMemory,
    AddressNack,
    DataNack,
    Overrun,
}

impl Frequency {
    pub fn new(frequency: u16) -> Self {
        if frequency < 2400 || frequency > 2500 {
            // do proper error handling
            Frequency { val: 2400 }
        } else {
            Frequency { val: frequency }
        }
    }
}

impl From<u16> for Frequency {
    fn from(frequency: u16) -> Self {
        if frequency < 2400 || frequency > 2500 {
            // do proper error handling
            Frequency { val: 2400 }
        } else {
            Frequency { val: frequency }
        }
    }
}

impl From<Frequency> for u16 {
    fn from(frequency: Frequency) -> Self {
        frequency.val
    }
}

pub enum Mode {
    Ble1Mbit,
    Ble2Mbit,
    Nrf1Mbit,
    Nrf2Mbit,
}

pub enum TxPower {
    Pos4dBm,
    Pos3dBm,
    ZerodBm,
    Neg4dBm,
    Neg8dBm,
    Neg12dBm,
    Neg16dBm,
    Neg20dBm,
    Neg30dBm,
    Neg40dBm,
}

pub enum PreambleLength {
    P8bit,
    P16bit,
}

pub enum S0Length {
    S00bytes,
    S01byte,
}

pub enum S1Include {
    Automatic,
    Include,
}

pub enum BaseAddressLength {
    BAL2bytes,
    BAL3bytes,
    BAL4bytes,
}

pub enum Endianness {
    Little,
    Big,
}

#[non_exhaustive]
pub struct Config {
    /// Frequency
    pub frequency: Frequency,
    /// Mode (modulation and bitrate)
    pub mode: Mode,
    /// Transmit power
    pub tx_power: TxPower,
    /// Length of the length field of the packet in bits; TODO: implement 4 bit value
    pub length_length: u8,
    /// Length of S0 in bytes
    pub length_s0: S0Length,
    /// Length of S1 in bits; TODO: implement 4bit value
    pub length_s1: u8,
    /// Inclusion mode of S1
    pub s1_include: S1Include,
    /// Length of the preamble
    pub length_preamble: PreambleLength,
    /// Maximum length of the packet payload
    pub payload_max: u8,
    /// Static length in bytes
    pub static_length: u8,
    /// Base address length in bytes
    pub base_address_length: BaseAddressLength,
    /// Endianness of the packet
    pub endianness: Endianness,
    /// Use packet whitening
    pub whitening: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::new(2400),
            mode: Mode::Ble1Mbit,
            tx_power: TxPower::ZerodBm,
            length_length: 8,
            length_s0: S0Length::S00bytes,
            length_s1: 0,
            s1_include: S1Include::Automatic,
            length_preamble: PreambleLength::P16bit,
            payload_max: 255,
            static_length: 0,
            base_address_length: BaseAddressLength::BAL4bytes,
            endianness: Endianness::Big,
            whitening: false,
        }
    }
}

/// Interface to a RADIO instance.
pub struct Radio<'d, T: Instance> {
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Radio<'d, T> {
    pub fn new(
        _radio: impl Unborrow<Target = T> + 'd,
        irq: impl Unborrow<Target = T::Interrupt> + 'd,
        config: Config,
    ) -> Self {
        unborrow!(irq);

        let r = T::regs();

        // Enable RADIO instance.
        r.power.write(|w| w.power().enabled());

        // MODE: data rate and modulation
        match config.mode {
            Mode::Ble1Mbit => r.mode.write(|w| w.mode().ble_1mbit()),
            Mode::Ble2Mbit => r.mode.write(|w| w.mode().ble_2mbit()),
            Mode::Nrf1Mbit => r.mode.write(|w| w.mode().nrf_1mbit()),
            Mode::Nrf2Mbit => r.mode.write(|w| w.mode().nrf_2mbit()),
        }

        // FREQUENCY
        // FREQUENCY: [0..100] freq = 2400 MHz + freq
        r.frequency.write(|w| unsafe {
            w.frequency()
                .bits((u16::from(config.frequency.clone()) - 2400) as u8)
        });

        // PCNF0
        // LFLEN: length field length in bits => 8
        // S0LEN: S0 length in bytes => 0 (default)
        // S1LEN: S1 length in bits => 0 (default)
        // S1INCL: 0 (default)
        // PLEN: 0 (default)
        r.pcnf0.write(|w| unsafe {
            w.lflen()
                .bits(config.length_length)
                .s0len()
                .bit(match config.length_s0 {
                    S0Length::S00bytes => false,
                    S0Length::S01byte => true,
                })
                .s1len()
                .bits(config.length_s1)
                .s1incl()
                .bit(match config.s1_include {
                    S1Include::Automatic => false,
                    S1Include::Include => true,
                })
                .plen()
                .bit(match config.length_preamble {
                    PreambleLength::P8bit => false,
                    PreambleLength::P16bit => true,
                })
        });

        // PCNF1
        // MAXLEN: max length of payload packet => 255
        // STATLEN: 0 (default)
        // BALEN: base address length => 4
        // ENDIAN: 0 (default) => 1
        // WHITEEN: 0 (default)
        r.pcnf1.write(|w| unsafe {
            w.maxlen()
                .bits(config.payload_max)
                .statlen()
                .bits(config.static_length)
                .balen()
                .bits(match config.base_address_length {
                    BaseAddressLength::BAL2bytes => 2,
                    BaseAddressLength::BAL3bytes => 3,
                    BaseAddressLength::BAL4bytes => 4,
                })
                .endian()
                .bit(match config.endianness {
                    Endianness::Little => false,
                    Endianness::Big => true,
                })
                .whiteen()
                .bit(config.whitening)
        });

        // BASE0
        // BASE0: 0xABCDABCD
        r.base0.write(|w| unsafe { w.base0().bits(0xABCDABCD) });

        // PREFIX0
        // AP0: 0xDA
        r.prefix0.write(|w| unsafe { w.ap0().bits(0xEF) });

        // TXADDRESS
        // TXADDRESS: 0 (default)
        r.txaddress.write(|w| unsafe { w.txaddress().bits(0) });

        // TXPOWER
        match config.tx_power {
            TxPower::Pos4dBm => r.txpower.write(|w| w.txpower().pos4d_bm()),
            TxPower::Pos3dBm => r.txpower.write(|w| w.txpower().pos3d_bm()),
            TxPower::ZerodBm => r.txpower.write(|w| w.txpower()._0d_bm()),
            TxPower::Neg4dBm => r.txpower.write(|w| w.txpower().neg4d_bm()),
            TxPower::Neg8dBm => r.txpower.write(|w| w.txpower().neg8d_bm()),
            TxPower::Neg12dBm => r.txpower.write(|w| w.txpower().neg12d_bm()),
            TxPower::Neg16dBm => r.txpower.write(|w| w.txpower().neg16d_bm()),
            TxPower::Neg20dBm => r.txpower.write(|w| w.txpower().neg20d_bm()),
            TxPower::Neg30dBm => r.txpower.write(|w| w.txpower().neg30d_bm()),
            TxPower::Neg40dBm => r.txpower.write(|w| w.txpower().neg40d_bm()),
        }

        // CRCCNF
        // LEN: length => 3
        // SKIPADDR: 0 (default)
        r.crccnf.write(|w| w.len().bits(3));

        // CRCPOLY
        // x24 + x10 + x9 + x6 + x4 + x3 + x + 1
        // CRCPOLY: 00000000_00000110_01011011
        r.crcpoly
            .write(|w| unsafe { w.bits(0b00000000_00000110_01011011) });

        // Shortcuts
        // READY - START
        // END - DISABLE
        r.shorts
            .write(|w| w.ready_start().bit(true).end_disable().bit(true));

        // Disable all events interrupts
        r.intenclr.write(|w| unsafe { w.bits(0xFFFF_FFFF) });

        irq.set_handler(Self::on_interrupt);
        irq.unpend();
        irq.enable();

        Self {
            phantom: PhantomData,
        }
    }

    fn on_interrupt(_: *mut ()) {
        let r = T::regs();
        let s = T::state();

        if r.events_disabled.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.disabled().clear());
        }
        if r.events_crcerror.read().bits() != 0 {
            s.end_waker.wake();
            r.intenclr.write(|w| w.crcerror().clear());
        }
    }

    // /// Set TX buffer, checking that it is in RAM and has suitable length.
    // unsafe fn set_tx_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
    //     slice_in_ram_or(buffer, Error::DMABufferNotInDataMemory)?;

    //     if buffer.len() == 0 {
    //         return Err(Error::TxBufferZeroLength);
    //     }
    //     if buffer.len() > EASY_DMA_SIZE {
    //         return Err(Error::TxBufferTooLong);
    //     }

    //     let r = T::regs();

    //     r.txd.ptr.write(|w|
    //         // We're giving the register a pointer to the stack. Since we're
    //         // waiting for the I2C transaction to end before this stack pointer
    //         // becomes invalid, there's nothing wrong here.
    //         //
    //         // The PTR field is a full 32 bits wide and accepts the full range
    //         // of values.
    //         w.ptr().bits(buffer.as_ptr() as u32));
    //     r.txd.maxcnt.write(|w|
    //         // We're giving it the length of the buffer, so no danger of
    //         // accessing invalid memory. We have verified that the length of the
    //         // buffer fits in an `u8`, so the cast to `u8` is also fine.
    //         //
    //         // The MAXCNT field is 8 bits wide and accepts the full range of
    //         // values.
    //         w.maxcnt().bits(buffer.len() as _));

    //     Ok(())
    // }

    // /// Set RX buffer, checking that it has suitable length.
    // unsafe fn set_rx_buffer(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
    //     // NOTE: RAM slice check is not necessary, as a mutable
    //     // slice can only be built from data located in RAM.

    //     if buffer.len() == 0 {
    //         return Err(Error::RxBufferZeroLength);
    //     }
    //     if buffer.len() > EASY_DMA_SIZE {
    //         return Err(Error::RxBufferTooLong);
    //     }

    //     let r = T::regs();

    //     r.rxd.ptr.write(|w|
    //         // We're giving the register a pointer to the stack. Since we're
    //         // waiting for the I2C transaction to end before this stack pointer
    //         // becomes invalid, there's nothing wrong here.
    //         //
    //         // The PTR field is a full 32 bits wide and accepts the full range
    //         // of values.
    //         w.ptr().bits(buffer.as_mut_ptr() as u32));
    //     r.rxd.maxcnt.write(|w|
    //         // We're giving it the length of the buffer, so no danger of
    //         // accessing invalid memory. We have verified that the length of the
    //         // buffer fits in an `u8`, so the cast to the type of maxcnt
    //         // is also fine.
    //         //
    //         // Note that that nrf52840 maxcnt is a wider
    //         // type than a u8, so we use a `_` cast rather than a `u8` cast.
    //         // The MAXCNT field is thus at least 8 bits wide and accepts the
    //         // full range of values that fit in a `u8`.
    //         w.maxcnt().bits(buffer.len() as _));

    //     Ok(())
    // }

    // fn clear_errorsrc(&mut self) {
    //     let r = T::regs();
    //     r.errorsrc
    //         .write(|w| w.anack().bit(true).dnack().bit(true).overrun().bit(true));
    // }

    // /// Get Error instance, if any occurred.
    // fn read_errorsrc(&self) -> Result<(), Error> {
    //     let r = T::regs();

    //     let err = r.errorsrc.read();
    //     if err.anack().is_received() {
    //         return Err(Error::AddressNack);
    //     }
    //     if err.dnack().is_received() {
    //         return Err(Error::DataNack);
    //     }
    //     if err.overrun().is_received() {
    //         return Err(Error::DataNack);
    //     }
    //     Ok(())
    // }

    /// Wait for stop or error
    // fn wait(&mut self) {
    //     let r = T::regs();
    //     loop {
    //         if r.events_end.read().bits() != 0 {
    //             r.events_end.reset();
    //             break;
    //         }
    //         if r.events_crcerror.read().bits() != 0 {
    //             r.events_crcerror.reset();
    //             r.tasks_stop.write(|w| unsafe { w.bits(1) });
    //         }
    //     }
    // }

    fn wait_for_end_event(cx: &mut core::task::Context) -> Poll<()> {
        let r = T::regs();
        let s = T::state();

        s.end_waker.register(cx.waker());
        if r.events_disabled.read().bits() != 0 {
            r.events_disabled.reset();

            return Poll::Ready(());
        }

        // stop if an error occured
        if r.events_crcerror.read().bits() != 0 {
            r.events_crcerror.reset();
            r.tasks_stop.write(|w| unsafe { w.bits(1) });
        }

        Poll::Pending
    }

    // the first byte of packet needs to be the packet length
    pub fn write<'a>(
        &'a mut self,
        packet: &'a [u8],
    ) -> impl Future<Output = Result<(), Error>> + 'a {
        async move {
            slice_in_ram_or(packet, Error::DMABufferNotInDataMemory)?;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(SeqCst);

            let r = T::regs();

            // // copy data into buffer
            // let mut len = 0;

            // for byte in bytes {
            //     self.packet[len + 1] = *byte;
            //     len += 1;
            // }

            // self.packet[0] = (len + 1) as u8;

            // enable "disabled" interrupt
            r.intenset.write(|w| w.disabled().bit(true));

            // set packet pointer
            r.packetptr
                .write(|w| unsafe { w.packetptr().bits(packet.as_ptr() as u32) });
            // start transmission task
            r.tasks_txen.write(|w| w.tasks_txen().bit(true));

            // Set up DMA write.
            // unsafe {
            //     self.set_tx_buffer(bytes)?;
            // }

            // // Reset events
            // r.events_stopped.reset();
            // r.events_error.reset();
            // r.events_lasttx.reset();
            // self.clear_errorsrc();

            // // Enable events
            // r.intenset.write(|w| w.stopped().set().error().set());

            // // Start write operation.
            // r.shorts.write(|w| w.lasttx_stop().enabled());
            // r.tasks_starttx.write(|w|
            // // `1` is a valid value to write to task registers.
            // unsafe { w.bits(1) });

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed.
            compiler_fence(SeqCst);

            // Wait for 'end' event.
            poll_fn(Self::wait_for_end_event).await;

            // self.read_errorsrc()?;

            // if r.txd.amount.read().bits() != bytes.len() as u32 {
            //     return Err(Error::Transmit);
            // }

            Ok(())
        }
    }
}

impl<'a, T: Instance> Drop for Radio<'a, T> {
    fn drop(&mut self) {
        info!("radio drop");

        // TODO when implementing async here, check for abort

        // disable!
        let r = T::regs();
        r.power.write(|w| w.power().disabled());

        info!("radio drop: done");
    }
}

pub(crate) mod sealed {
    use super::*;

    pub struct State {
        pub end_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                end_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        fn regs() -> &'static pac::radio::RegisterBlock;
        fn state() -> &'static State;
    }
}

pub trait Instance: Unborrow<Target = Self> + sealed::Instance + 'static {
    type Interrupt: Interrupt;
}

macro_rules! impl_radio {
    ($type:ident, $pac_type:ident, $irq:ident) => {
        impl crate::radio::sealed::Instance for peripherals::$type {
            fn regs() -> &'static pac::radio::RegisterBlock {
                unsafe { &*pac::$pac_type::ptr() }
            }
            fn state() -> &'static crate::radio::sealed::State {
                static STATE: crate::radio::sealed::State = crate::radio::sealed::State::new();
                &STATE
            }
        }
        impl crate::radio::Instance for peripherals::$type {
            type Interrupt = crate::interrupt::$irq;
        }
    };
}
