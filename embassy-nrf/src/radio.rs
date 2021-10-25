#![macro_use]

//! HAL interface to the RADIO peripheral.
//!
//! See product specification:
//!
//! - nRF52810: v1.3 Section 6.14
//!
//! TODO:
//! *) Check error handling
//!
//! Open Points:
//! *) rx and tx traits (what signature should be used)
//! *) "rx then tx" and "tx then rx" support (interesting for protocols)
//!
use crate::chip::{EASY_DMA_SIZE, FORCE_COPY_BUFFER_SIZE};
use crate::pac;
use crate::util::{slice_in_ram, slice_in_ram_or};
use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering::SeqCst};
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::Unborrow;
use embassy::waitqueue::AtomicWaker;
use embassy_hal_common::unborrow;
use futures::future::poll_fn;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    TxBufferTooLong,
    RxBufferTooLong,
    TxBufferZeroLength,
    RxBufferZeroLength,
    Transmit,
    Receive,
    CrcError,
    DMABufferNotInDataMemory,
    FrequencyTooLow,
    FrequencyTooHigh,
    TxAddressSelectionInvalid,
    ValueTooSmall,
    ValueTooLarge,
}

#[derive(Clone)]
pub struct Frequency(u16);

impl Frequency {
    pub fn new(frequency: u16) -> Result<Self, Error> {
        match frequency {
            0..=2399 => Err(Error::FrequencyTooLow),
            2400..=2500 => Ok(Frequency(frequency)),
            _ => Err(Error::FrequencyTooHigh),
        }
    }
}

impl From<Frequency> for u16 {
    fn from(val: Frequency) -> Self {
        val.0
    }
}

#[derive(Clone, Copy)]
pub struct TxAddressSelection(u8);

impl TxAddressSelection {
    pub fn new(selection: u8) -> Result<Self, Error> {
        match selection {
            0..=7 => Ok(TxAddressSelection(selection)),
            _ => Err(Error::TxAddressSelectionInvalid),
        }
    }
}

impl Into<u8> for TxAddressSelection {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct Value4Bit(u8);

impl Value4Bit {
    pub fn new(val: u8) -> Result<Self, Error> {
        match val {
            0..=0x0f => Ok(Value4Bit(val)),
            _ => Err(Error::ValueTooLarge),
        }
    }
}

impl Into<u8> for Value4Bit {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct Value24Bit(u32);

impl Value24Bit {
    pub fn new(val: u32) -> Result<Self, Error> {
        match val {
            0..=0x0fff => Ok(Value24Bit(val)),
            _ => Err(Error::ValueTooLarge),
        }
    }
}

impl Into<u32> for Value24Bit {
    fn into(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct DataWhiteningIv(u8);

impl DataWhiteningIv {
    pub fn new(val: u8) -> Result<Self, Error> {
        match val {
            0..=0b0011_1111 => Err(Error::ValueTooSmall),
            0b0100_0000..=0b0111_1111 => Ok(DataWhiteningIv(val)),
            _ => Err(Error::ValueTooLarge),
        }
    }
}

impl Into<u32> for DataWhiteningIv {
    fn into(self) -> u32 {
        self.0.into()
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

pub enum CrcLength {
    CrcDisabled,
    Crc1byte,
    Crc2bytes,
    Crc3bytes,
}

#[non_exhaustive]
pub struct Config {
    /// Frequency
    pub frequency: Frequency,
    /// Mode (modulation and bitrate)
    pub mode: Mode,
    /// Length of the length field of the packet in bits
    pub length_length: Value4Bit,
    /// Length of S0 in bytes
    pub s0_length: S0Length,
    /// Length of S1 in bits
    pub s1_length: Value4Bit,
    /// Inclusion mode of S1
    pub s1_include: S1Include,
    /// Length of the preamble
    pub preamble_length: PreambleLength,
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
    /// Initial value for packet whitening
    pub whitening_iv: DataWhiteningIv,
    /// Base address 0
    pub base_address_0: u32,
    /// Base address 1
    pub base_address_1: u32,
    /// Prefix 0
    pub prefix_0: u8,
    /// Prefix 1
    pub prefix_1: u8,
    /// Prefix 2
    pub prefix_2: u8,
    /// Prefix 3
    pub prefix_3: u8,
    /// Prefix 4
    pub prefix_4: u8,
    /// Prefix 5
    pub prefix_5: u8,
    /// Prefix 6
    pub prefix_6: u8,
    /// Prefix 7
    pub prefix_7: u8,
    /// CRC length
    pub crc_length: CrcLength,
    /// CRC skip address
    pub crc_skip_address: bool,
    /// CRC polynomial
    pub crc_poly: Value24Bit,
    /// CRC initial value
    pub crc_init: Value24Bit,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            frequency: Frequency::new(2400).unwrap(),
            mode: Mode::Ble1Mbit,
            length_length: Value4Bit::new(8).unwrap(),
            s0_length: S0Length::S00bytes,
            s1_length: Value4Bit::new(0).unwrap(),
            s1_include: S1Include::Automatic,
            preamble_length: PreambleLength::P16bit,
            payload_max: 255,
            static_length: 0,
            base_address_length: BaseAddressLength::BAL4bytes,
            endianness: Endianness::Big,
            whitening: false,
            whitening_iv: DataWhiteningIv::new(0b0100_0000).unwrap(),
            base_address_0: 0,
            base_address_1: 0,
            prefix_0: 0,
            prefix_1: 0,
            prefix_2: 0,
            prefix_3: 0,
            prefix_4: 0,
            prefix_5: 0,
            prefix_6: 0,
            prefix_7: 0,
            crc_length: CrcLength::CrcDisabled,
            crc_skip_address: false,
            crc_poly: Value24Bit::new(0).unwrap(),
            crc_init: Value24Bit::new(0).unwrap(),
        }
    }
}

#[non_exhaustive]
pub struct TxConfig {
    /// Transmit power
    pub tx_power: TxPower,
    /// Transmission address
    pub tx_address: TxAddressSelection,
}

impl Default for TxConfig {
    fn default() -> Self {
        Self {
            tx_power: TxPower::ZerodBm,
            tx_address: TxAddressSelection::new(0).unwrap(),
        }
    }
}

#[non_exhaustive]
pub struct RxConfig {
    pub rx_address_0_active: bool,
    pub rx_address_1_active: bool,
    pub rx_address_2_active: bool,
    pub rx_address_3_active: bool,
    pub rx_address_4_active: bool,
    pub rx_address_5_active: bool,
    pub rx_address_6_active: bool,
    pub rx_address_7_active: bool,
}

impl Default for RxConfig {
    fn default() -> Self {
        Self {
            rx_address_0_active: false,
            rx_address_1_active: false,
            rx_address_2_active: false,
            rx_address_3_active: false,
            rx_address_4_active: false,
            rx_address_5_active: false,
            rx_address_6_active: false,
            rx_address_7_active: false,
        }
    }
}

/// Interface to a RADIO instance.
///
/// The following features are not supported:
/// * Bit counter compare
/// * Device address match
/// * Combined Tx-Rx and Rx-Tx (incl. inter frame spacing)
/// * Radio mode configuration (ramp up time and default TX value)
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
        r.pcnf0.write(|w| unsafe {
            w.lflen()
                .bits(config.length_length.into())
                .s0len()
                .bit(match config.s0_length {
                    S0Length::S00bytes => false,
                    S0Length::S01byte => true,
                })
                .s1len()
                .bits(config.s1_length.into())
                .s1incl()
                .bit(match config.s1_include {
                    S1Include::Automatic => false,
                    S1Include::Include => true,
                })
                .plen()
                .bit(match config.preamble_length {
                    PreambleLength::P8bit => false,
                    PreambleLength::P16bit => true,
                })
        });

        // PCNF1
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
        r.base0
            .write(|w| unsafe { w.base0().bits(config.base_address_0) });
        // BASE1
        r.base1
            .write(|w| unsafe { w.base1().bits(config.base_address_1) });

        // PREFIX0
        r.prefix0.write(|w| unsafe {
            w.ap0()
                .bits(config.prefix_0)
                .ap1()
                .bits(config.prefix_1)
                .ap2()
                .bits(config.prefix_2)
                .ap3()
                .bits(config.prefix_3)
        });
        // PREFIX1
        r.prefix1.write(|w| unsafe {
            w.ap4()
                .bits(config.prefix_4)
                .ap5()
                .bits(config.prefix_5)
                .ap6()
                .bits(config.prefix_6)
                .ap7()
                .bits(config.prefix_7)
        });

        // CRCCNF
        r.crccnf.write(|w| {
            w.len()
                .bits(match config.crc_length {
                    CrcLength::CrcDisabled => 0,
                    CrcLength::Crc1byte => 1,
                    CrcLength::Crc2bytes => 2,
                    CrcLength::Crc3bytes => 3,
                })
                .skipaddr()
                .bit(config.crc_skip_address)
        });

        // CRCPOLY
        r.crcpoly
            .write(|w| unsafe { w.bits(config.crc_poly.into()) });
        r.crcinit
            .write(|w| unsafe { w.bits(config.crc_init.into()) });

        // DATAWHITEIV
        r.datawhiteiv
            .write(|w| unsafe { w.bits(config.whitening_iv.into()) });

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

    fn wait_for_disabled_event(cx: &mut core::task::Context) -> Poll<()> {
        let r = T::regs();
        let s = T::state();

        s.end_waker.register(cx.waker());
        if r.events_disabled.read().bits() != 0 {
            r.events_disabled.reset();

            return Poll::Ready(());
        }

        // stop if an error occured TODO activate error interrupts
        if r.events_crcerror.read().bits() != 0 {
            r.events_crcerror.reset();
            r.tasks_stop.write(|w| unsafe { w.bits(1) });
        }

        Poll::Pending
    }

    // the first byte of packet needs to be the packet length
    pub fn transmit<'a>(
        &'a mut self,
        tx_config: &'a TxConfig,
        packet: &'a [u8],
    ) -> impl Future<Output = Result<(), Error>> + 'a {
        async move {
            slice_in_ram_or(packet, Error::DMABufferNotInDataMemory)?;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(SeqCst);

            let r = T::regs();

            // Set tx shorts
            r.shorts
                .write(|w| w.ready_start().bit(true).end_disable().bit(true));

            // TXADDRESS
            // TXADDRESS: 0 (default)
            r.txaddress
                .write(|w| unsafe { w.txaddress().bits(tx_config.tx_address.into()) });

            // TXPOWER
            match tx_config.tx_power {
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

            // enable "disabled" interrupt
            r.intenset.write(|w| w.disabled().bit(true));

            // set packet pointer
            r.packetptr
                .write(|w| unsafe { w.packetptr().bits(packet.as_ptr() as u32) });
            // start transmission task
            r.tasks_txen.write(|w| w.tasks_txen().bit(true));

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed.
            compiler_fence(SeqCst);

            // Wait for 'end' event.
            poll_fn(Self::wait_for_disabled_event).await;

            Ok(())
        }
    }

    // the first byte of packet needs to be the packet length
    pub fn receive<'a>(
        &'a mut self,
        rx_config: &'a RxConfig,
        packet: &'a mut [u8],
    ) -> impl Future<Output = Result<i8, Error>> + 'a {
        async move {
            slice_in_ram_or(packet, Error::DMABufferNotInDataMemory)?;

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // before any DMA action has started.
            compiler_fence(SeqCst);

            let r = T::regs();

            // Set rx shorts
            r.shorts.write(|w| {
                w.ready_start()
                    .bit(true)
                    .end_disable()
                    .bit(true)
                    .address_rssistart()
                    .bit(true)
                    .disabled_rssistop()
                    .bit(true)
            });

            // RXADDRESSES
            r.rxaddresses.write(|w| {
                w.addr0()
                    .bit(rx_config.rx_address_0_active)
                    .addr1()
                    .bit(rx_config.rx_address_1_active)
                    .addr2()
                    .bit(rx_config.rx_address_2_active)
                    .addr3()
                    .bit(rx_config.rx_address_3_active)
                    .addr4()
                    .bit(rx_config.rx_address_4_active)
                    .addr5()
                    .bit(rx_config.rx_address_5_active)
                    .addr6()
                    .bit(rx_config.rx_address_6_active)
                    .addr7()
                    .bit(rx_config.rx_address_7_active)
            });

            // enable "disabled" interrupt
            r.intenset.write(|w| w.disabled().bit(true));

            // set packet pointer
            r.packetptr
                .write(|w| unsafe { w.packetptr().bits(packet.as_ptr() as u32) });
            // start transmission task
            r.tasks_rxen.write(|w| w.tasks_rxen().bit(true));

            // Conservative compiler fence to prevent optimizations that do not
            // take in to account actions by DMA. The fence has been placed here,
            // after all possible DMA actions have completed.
            compiler_fence(SeqCst);

            // Wait for 'end' event.
            poll_fn(Self::wait_for_disabled_event).await;

            if r.crcstatus.read().crcstatus().is_crcerror() {
                return Err(Error::CrcError);
            }

            Ok(-(r.rssisample.read().rssisample().bits() as i8))
        }
    }
}

impl<'a, T: Instance> Drop for Radio<'a, T> {
    fn drop(&mut self) {
        info!("radio drop");

        // TODO when implementing async here, check for abort

        // disable
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
