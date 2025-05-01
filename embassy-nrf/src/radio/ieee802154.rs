//! IEEE 802.15.4 radio driver

use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;

use super::{Error, Instance, InterruptHandler, TxPower};
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::{self};
use crate::pac::radio::vals;
pub use crate::pac::radio::vals::State as RadioState;
use crate::Peri;

/// Default (IEEE compliant) Start of Frame Delimiter
pub const DEFAULT_SFD: u8 = 0xA7;

// TODO expose the other variants in `pac::CCAMODE_A`
/// Clear Channel Assessment method
pub enum Cca {
    /// Carrier sense
    CarrierSense,
    /// Energy Detection / Energy Above Threshold
    EnergyDetection {
        /// Energy measurements above this value mean that the channel is assumed to be busy.
        /// Note the measurement range is 0..0xFF - where 0 means that the received power was
        /// less than 10 dB above the selected receiver sensitivity. This value is not given in dBm,
        /// but can be converted. See the nrf52840 Product Specification Section 6.20.12.4
        /// for details.
        ed_threshold: u8,
    },
}

/// IEEE 802.15.4 radio driver.
pub struct Radio<'d, T: Instance> {
    _p: Peri<'d, T>,
    needs_enable: bool,
}

impl<'d, T: Instance> Radio<'d, T> {
    /// Create a new IEEE 802.15.4 radio driver.
    pub fn new(
        radio: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        let r = T::regs();

        // Disable and enable to reset peripheral
        r.power().write(|w| w.set_power(false));
        r.power().write(|w| w.set_power(true));

        // Enable 802.15.4 mode
        r.mode().write(|w| w.set_mode(vals::Mode::IEEE802154_250KBIT));
        // Configure CRC skip address
        r.crccnf().write(|w| {
            w.set_len(vals::Len::TWO);
            w.set_skipaddr(vals::Skipaddr::IEEE802154);
        });
        // Configure CRC polynomial and init
        r.crcpoly().write(|w| w.set_crcpoly(0x0001_1021));
        r.crcinit().write(|w| w.set_crcinit(0));
        r.pcnf0().write(|w| {
            // 8-bit on air length
            w.set_lflen(8);
            // Zero bytes S0 field length
            w.set_s0len(false);
            // Zero bytes S1 field length
            w.set_s1len(0);
            // Do not include S1 field in RAM if S1 length > 0
            w.set_s1incl(vals::S1incl::AUTOMATIC);
            // Zero code Indicator length
            w.set_cilen(0);
            // 32-bit zero preamble
            w.set_plen(vals::Plen::_32BIT_ZERO);
            // Include CRC in length
            w.set_crcinc(vals::Crcinc::INCLUDE);
        });
        r.pcnf1().write(|w| {
            // Maximum packet length
            w.set_maxlen(Packet::MAX_PSDU_LEN);
            // Zero static length
            w.set_statlen(0);
            // Zero base address length
            w.set_balen(0);
            // Little-endian
            w.set_endian(vals::Endian::LITTLE);
            // Disable packet whitening
            w.set_whiteen(false);
        });

        // Enable NVIC interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let mut radio = Self {
            _p: radio,
            needs_enable: false,
        };

        radio.set_sfd(DEFAULT_SFD);
        radio.set_transmission_power(0);
        radio.set_channel(11);
        radio.set_cca(Cca::CarrierSense);

        radio
    }

    /// Changes the radio channel
    pub fn set_channel(&mut self, channel: u8) {
        let r = T::regs();
        if channel < 11 || channel > 26 {
            panic!("Bad 802.15.4 channel");
        }
        let frequency_offset = (channel - 10) * 5;
        self.needs_enable = true;
        r.frequency().write(|w| {
            w.set_frequency(frequency_offset);
            w.set_map(vals::Map::DEFAULT);
        });
    }

    /// Changes the Clear Channel Assessment method
    pub fn set_cca(&mut self, cca: Cca) {
        let r = T::regs();
        self.needs_enable = true;
        match cca {
            Cca::CarrierSense => r.ccactrl().write(|w| w.set_ccamode(vals::Ccamode::CARRIER_MODE)),
            Cca::EnergyDetection { ed_threshold } => {
                // "[ED] is enabled by first configuring the field CCAMODE=EdMode in CCACTRL
                // and writing the CCAEDTHRES field to a chosen value."
                r.ccactrl().write(|w| {
                    w.set_ccamode(vals::Ccamode::ED_MODE);
                    w.set_ccaedthres(ed_threshold);
                });
            }
        }
    }

    /// Changes the Start of Frame Delimiter (SFD)
    pub fn set_sfd(&mut self, sfd: u8) {
        let r = T::regs();
        r.sfd().write(|w| w.set_sfd(sfd));
    }

    /// Clear interrupts
    pub fn clear_all_interrupts(&mut self) {
        let r = T::regs();
        r.intenclr().write(|w| w.0 = 0xffff_ffff);
    }

    /// Changes the radio transmission power
    pub fn set_transmission_power(&mut self, power: i8) {
        let r = T::regs();
        self.needs_enable = true;

        let tx_power: TxPower = match power {
            #[cfg(not(any(feature = "nrf52811", feature = "_nrf5340-net")))]
            8 => TxPower::POS8_DBM,
            #[cfg(not(any(feature = "nrf52811", feature = "_nrf5340-net")))]
            7 => TxPower::POS7_DBM,
            #[cfg(not(any(feature = "nrf52811", feature = "_nrf5340-net")))]
            6 => TxPower::POS6_DBM,
            #[cfg(not(any(feature = "nrf52811", feature = "_nrf5340-net")))]
            5 => TxPower::POS5_DBM,
            #[cfg(not(feature = "_nrf5340-net"))]
            4 => TxPower::POS4_DBM,
            #[cfg(not(feature = "_nrf5340-net"))]
            3 => TxPower::POS3_DBM,
            #[cfg(not(any(feature = "nrf52811", feature = "_nrf5340-net")))]
            2 => TxPower::POS2_DBM,
            0 => TxPower::_0_DBM,
            #[cfg(feature = "_nrf5340-net")]
            -1 => TxPower::NEG1_DBM,
            #[cfg(feature = "_nrf5340-net")]
            -2 => TxPower::NEG2_DBM,
            #[cfg(feature = "_nrf5340-net")]
            -3 => TxPower::NEG3_DBM,
            -4 => TxPower::NEG4_DBM,
            #[cfg(feature = "_nrf5340-net")]
            -5 => TxPower::NEG5_DBM,
            #[cfg(feature = "_nrf5340-net")]
            -6 => TxPower::NEG6_DBM,
            #[cfg(feature = "_nrf5340-net")]
            -7 => TxPower::NEG7_DBM,
            -8 => TxPower::NEG8_DBM,
            -12 => TxPower::NEG12_DBM,
            -16 => TxPower::NEG16_DBM,
            -20 => TxPower::NEG20_DBM,
            -30 => TxPower::NEG30_DBM,
            -40 => TxPower::NEG40_DBM,
            _ => panic!("Invalid transmission power value"),
        };

        r.txpower().write(|w| w.set_txpower(tx_power));
    }

    /// Waits until the radio state matches the given `state`
    fn wait_for_radio_state(&self, state: RadioState) {
        while self.state() != state {}
    }

    /// Get the current radio state
    fn state(&self) -> RadioState {
        T::regs().state().read().state()
    }

    /// Moves the radio from any state to the DISABLED state
    fn disable(&mut self) {
        let r = T::regs();
        // See figure 110 in nRF52840-PS
        loop {
            match self.state() {
                RadioState::DISABLED => return,
                // idle or ramping up
                RadioState::RX_RU | RadioState::RX_IDLE | RadioState::TX_RU | RadioState::TX_IDLE => {
                    r.tasks_disable().write_value(1);
                    self.wait_for_radio_state(RadioState::DISABLED);
                    return;
                }
                // ramping down
                RadioState::RX_DISABLE | RadioState::TX_DISABLE => {
                    self.wait_for_radio_state(RadioState::DISABLED);
                    return;
                }
                // cancel ongoing transfer or ongoing CCA
                RadioState::RX => {
                    r.tasks_ccastop().write_value(1);
                    r.tasks_stop().write_value(1);
                    self.wait_for_radio_state(RadioState::RX_IDLE);
                }
                RadioState::TX => {
                    r.tasks_stop().write_value(1);
                    self.wait_for_radio_state(RadioState::TX_IDLE);
                }
                _ => unreachable!(),
            }
        }
    }

    fn set_buffer(&mut self, buffer: &[u8]) {
        let r = T::regs();
        r.packetptr().write_value(buffer.as_ptr() as u32);
    }

    /// Moves the radio to the RXIDLE state
    fn receive_prepare(&mut self) {
        // clear related events
        T::regs().events_ccabusy().write_value(0);
        T::regs().events_phyend().write_value(0);
        // NOTE to avoid errata 204 (see rev1 v1.4) we do TX_IDLE -> DISABLED -> RXIDLE
        let disable = match self.state() {
            RadioState::DISABLED => false,
            RadioState::RX_IDLE => self.needs_enable,
            _ => true,
        };
        if disable {
            self.disable();
        }
        self.needs_enable = false;
    }

    /// Prepare radio for receiving a packet
    fn receive_start(&mut self, packet: &mut Packet) {
        // NOTE we do NOT check the address of `packet` because the mutable reference ensures it's
        // allocated in RAM
        let r = T::regs();

        self.receive_prepare();

        // Configure shortcuts
        //
        // The radio goes through following states when receiving a 802.15.4 packet
        //
        // enable RX → ramp up RX → RX idle → Receive → end (PHYEND)
        r.shorts().write(|w| w.set_rxready_start(true));

        // set up RX buffer
        self.set_buffer(packet.buffer.as_mut());

        // start transfer
        dma_start_fence();

        match self.state() {
            // Re-start receiver
            RadioState::RX_IDLE => r.tasks_start().write_value(1),
            // Enable receiver
            _ => r.tasks_rxen().write_value(1),
        }
    }

    /// Cancel receiving packet
    fn receive_cancel() {
        let r = T::regs();
        r.shorts().write(|_| {});
        r.tasks_stop().write_value(1);
        loop {
            match r.state().read().state() {
                RadioState::DISABLED | RadioState::RX_IDLE => break,
                _ => (),
            }
        }
        // DMA transfer may have been in progress so synchronize with its memory operations
        dma_end_fence();
    }

    /// Receives one radio packet and copies its contents into the given `packet` buffer
    ///
    /// This methods returns the `Ok` variant if the CRC included the packet was successfully
    /// validated by the hardware; otherwise it returns the `Err` variant. In either case, `packet`
    /// will be updated with the received packet's data
    pub async fn receive(&mut self, packet: &mut Packet) -> Result<(), Error> {
        let s = T::state();
        let r = T::regs();

        // Start the read
        self.receive_start(packet);

        let dropper = OnDrop::new(|| Self::receive_cancel());

        self.clear_all_interrupts();
        // wait until we have received something
        core::future::poll_fn(|cx| {
            s.event_waker.register(cx.waker());

            if r.events_phyend().read() != 0 {
                r.events_phyend().write_value(0);
                trace!("RX done poll");
                return Poll::Ready(());
            } else {
                r.intenset().write(|w| w.set_phyend(true));
            };

            Poll::Pending
        })
        .await;

        dma_end_fence();
        dropper.defuse();

        let crc = r.rxcrc().read().rxcrc() as u16;
        if r.crcstatus().read().crcstatus() == vals::Crcstatus::CRCOK {
            Ok(())
        } else {
            Err(Error::CrcFailed(crc))
        }
    }

    /// Tries to send the given `packet`
    ///
    /// This method performs Clear Channel Assessment (CCA) first and sends the `packet` only if the
    /// channel is observed to be *clear* (no transmission is currently ongoing), otherwise no
    /// packet is transmitted and the `Err` variant is returned
    ///
    /// NOTE this method will *not* modify the `packet` argument. The mutable reference is used to
    /// ensure the `packet` buffer is allocated in RAM, which is required by the RADIO peripheral
    // NOTE we do NOT check the address of `packet` because the mutable reference ensures it's
    // allocated in RAM
    pub async fn try_send(&mut self, packet: &mut Packet) -> Result<(), Error> {
        let s = T::state();
        let r = T::regs();

        // enable radio to perform cca
        self.receive_prepare();

        /// transmit result
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub enum TransmitResult {
            /// Success
            Success,
            /// Clear channel assessment reported channel in use
            ChannelInUse,
        }

        // Configure shortcuts
        //
        // The radio goes through following states when sending a 802.15.4 packet
        //
        // enable RX → ramp up RX → clear channel assessment (CCA) → CCA result
        // CCA idle → enable TX → start TX → TX → end (PHYEND) → disabled
        //
        // CCA might end up in the event CCABUSY in which there will be no transmission
        r.shorts().write(|w| {
            w.set_rxready_ccastart(true);
            w.set_ccaidle_txen(true);
            w.set_txready_start(true);
            w.set_ccabusy_disable(true);
            w.set_phyend_disable(true);
        });

        // Set transmission buffer
        self.set_buffer(packet.buffer.as_mut());

        // the DMA transfer will start at some point after the following write operation so
        // we place the compiler fence here
        dma_start_fence();
        // start CCA. In case the channel is clear, the data at packetptr will be sent automatically

        match self.state() {
            // Re-start receiver
            RadioState::RX_IDLE => r.tasks_ccastart().write_value(1),
            // Enable receiver
            _ => r.tasks_rxen().write_value(1),
        }

        self.clear_all_interrupts();
        let result = core::future::poll_fn(|cx| {
            s.event_waker.register(cx.waker());

            if r.events_phyend().read() != 0 {
                r.events_phyend().write_value(0);
                r.events_ccabusy().write_value(0);
                trace!("TX done poll");
                return Poll::Ready(TransmitResult::Success);
            } else if r.events_ccabusy().read() != 0 {
                r.events_ccabusy().write_value(0);
                trace!("TX no CCA");
                return Poll::Ready(TransmitResult::ChannelInUse);
            }

            r.intenset().write(|w| {
                w.set_phyend(true);
                w.set_ccabusy(true);
            });

            Poll::Pending
        })
        .await;

        match result {
            TransmitResult::Success => Ok(()),
            TransmitResult::ChannelInUse => Err(Error::ChannelInUse),
        }
    }
}

/// An IEEE 802.15.4 packet
///
/// This `Packet` is a PHY layer packet. It's made up of the physical header (PHR) and the PSDU
/// (PHY service data unit). The PSDU of this `Packet` will always include the MAC level CRC, AKA
/// the FCS (Frame Control Sequence) -- the CRC is fully computed in hardware and automatically
/// appended on transmission and verified on reception.
///
/// The API lets users modify the usable part (not the CRC) of the PSDU via the `deref` and
/// `copy_from_slice` methods. These methods will automatically update the PHR.
///
/// See figure 119 in the Product Specification of the nRF52840 for more details
pub struct Packet {
    buffer: [u8; Self::SIZE],
}

// See figure 124 in nRF52840-PS
impl Packet {
    // for indexing purposes
    const PHY_HDR: usize = 0;
    const DATA: core::ops::RangeFrom<usize> = 1..;

    /// Maximum amount of usable payload (CRC excluded) a single packet can contain, in bytes
    pub const CAPACITY: u8 = 125;
    const CRC: u8 = 2; // size of the CRC, which is *never* copied to / from RAM
    const MAX_PSDU_LEN: u8 = Self::CAPACITY + Self::CRC;
    const SIZE: usize = 1 /* PHR */ + Self::MAX_PSDU_LEN as usize;

    /// Returns an empty packet (length = 0)
    pub fn new() -> Self {
        let mut packet = Self {
            buffer: [0; Self::SIZE],
        };
        packet.set_len(0);
        packet
    }

    /// Fills the packet payload with given `src` data
    ///
    /// # Panics
    ///
    /// This function panics if `src` is larger than `Self::CAPACITY`
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        assert!(src.len() <= Self::CAPACITY as usize);
        let len = src.len() as u8;
        self.buffer[Self::DATA][..len as usize].copy_from_slice(&src[..len.into()]);
        self.set_len(len);
    }

    /// Returns the size of this packet's payload
    pub fn len(&self) -> u8 {
        self.buffer[Self::PHY_HDR] - Self::CRC
    }

    /// Changes the size of the packet's payload
    ///
    /// # Panics
    ///
    /// This function panics if `len` is larger than `Self::CAPACITY`
    pub fn set_len(&mut self, len: u8) {
        assert!(len <= Self::CAPACITY);
        self.buffer[Self::PHY_HDR] = len + Self::CRC;
    }

    /// Returns the LQI (Link Quality Indicator) of the received packet
    ///
    /// Note that the LQI is stored in the `Packet`'s internal buffer by the hardware so the value
    /// returned by this method is only valid after a `Radio.recv` operation. Operations that
    /// modify the `Packet`, like `copy_from_slice` or `set_len`+`deref_mut`, will overwrite the
    /// stored LQI value.
    ///
    /// Also note that the hardware will *not* compute a LQI for packets smaller than 3 bytes so
    /// this method will return an invalid value for those packets.
    pub fn lqi(&self) -> u8 {
        self.buffer[1 /* PHY_HDR */ + self.len() as usize /* data */]
    }
}

impl core::ops::Deref for Packet {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buffer[Self::DATA][..self.len() as usize]
    }
}

impl core::ops::DerefMut for Packet {
    fn deref_mut(&mut self) -> &mut [u8] {
        let len = self.len();
        &mut self.buffer[Self::DATA][..len as usize]
    }
}

/// NOTE must be followed by a volatile write operation
fn dma_start_fence() {
    compiler_fence(Ordering::Release);
}

/// NOTE must be preceded by a volatile read operation
fn dma_end_fence() {
    compiler_fence(Ordering::Acquire);
}
