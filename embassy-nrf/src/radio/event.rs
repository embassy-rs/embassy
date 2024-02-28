use crate::pac;
use bitflags;

bitflags::bitflags! {
    /// Event as bit flags
    pub struct Event : u32 {
        /// Radio ready
        const READY = 1u32 << 0;
        /// Address operation done
        const ADDRESS = 1u32 << 1;
        /// Payload operation done
        const PAYLOAD = 1u32 << 2;
        /// Packet operation done
        const END = 1u32 << 3;
        /// Radio has been disabled
        const DISABLED = 1u32 << 4;
        /// Device address match in last received packet
        const DEV_MATCH = 1u32 << 5;
        /// No device address match in last received packet
        const DEV_MISS = 1u32 << 6;
        /// RSSI sampling complete
        const RSSI_END = 1u32 << 7;
        /// Bit counter reached target
        const BC_MATCH = 1u32 << 10;
        /// CRC ok in last received packet
        const CRC_OK = 1u32 << 12;
        /// CRC error in last received packet
        const CRC_ERROR = 1u32 << 13;
        /// IEEE 802.15.4 length field received
        const FRAME_START = 1u32 << 14;
        /// Sampling of energy detect complete
        const ED_END = 1u32 << 15;
        /// Sampling of energy detect stopped
        const ED_STOPPED = 1u32 << 16;
        /// Wireless medium in idle, ready to sent
        const CCA_IDLE = 1u32 << 17;
        /// Wireless medium busy, do not send
        const CCA_BUSY = 1u32 << 18;
        /// Clear channel assessment stopped
        const CCA_STOPPED = 1u32 << 19;
        /// BLE LR rate boost received
        const RATE_BOOST = 1u32 << 20;
        /// Radio has ramped up transmitter
        const TX_READY = 1u32 << 21;
        /// Radio has ramped up receiver
        const RX_READY = 1u32 << 22;
        /// MAC header match found
        const MHR_MATCH = 1u32 << 23;
        /// Preamble received, possible false triggering
        const SYNC = 1u32 << 26;
        /// Last bit sent / received
        const PHY_END = 1u32 << 27;
        /// Continuous tone extension is present
        const CTE_PRESENT = 1u32 << 28;
    }
}

impl Event {
    /// Read events from radio
    #[cfg(not(feature = "nrf52832"))]
    pub fn from_radio(radio: &pac::radio::RegisterBlock) -> Self {
        let mut value = Self::empty();
        if radio.events_ready.read().events_ready().bit_is_set() {
            value |= Self::READY;
        }
        if radio.events_address.read().events_address().bit_is_set() {
            value |= Self::ADDRESS;
        }
        if radio.events_payload.read().events_payload().bit_is_set() {
            value |= Self::PAYLOAD;
        }
        if radio.events_end.read().events_end().bit_is_set() {
            value |= Self::END;
        }
        if radio.events_disabled.read().events_disabled().bit_is_set() {
            value |= Self::DISABLED;
        }
        if radio.events_devmatch.read().events_devmatch().bit_is_set() {
            value |= Self::DEV_MATCH;
        }
        if radio.events_devmiss.read().events_devmiss().bit_is_set() {
            value |= Self::DEV_MISS;
        }
        if radio.events_rssiend.read().events_rssiend().bit_is_set() {
            value |= Self::RSSI_END;
        }
        if radio.events_bcmatch.read().events_bcmatch().bit_is_set() {
            value |= Self::BC_MATCH;
        }
        if radio.events_crcok.read().events_crcok().bit_is_set() {
            value |= Self::CRC_OK;
        }
        if radio.events_crcerror.read().events_crcerror().bit_is_set() {
            value |= Self::CRC_ERROR;
        }
        #[cfg(any(
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_framestart.read().events_framestart().bit_is_set() {
            value |= Self::FRAME_START;
        }
        #[cfg(any(
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_edend.read().events_edend().bit_is_set() {
            value |= Self::ED_END;
        }
        #[cfg(any(
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_edstopped.read().events_edstopped().bit_is_set() {
            value |= Self::ED_STOPPED;
        }
        #[cfg(any(feature = "nrf52820", feature = "nrf52833", feature = "_nrf5340-net"))]
        if radio.events_ccaidle.read().events_ccaidle().bit_is_set() {
            value |= Self::CCA_IDLE;
        }
        #[cfg(any(feature = "nrf52820", feature = "nrf52833", feature = "_nrf5340-net"))]
        if radio.events_ccabusy.read().events_ccabusy().bit_is_set() {
            value |= Self::CCA_BUSY;
        }
        #[cfg(any(feature = "nrf52820", feature = "nrf52833", feature = "_nrf5340-net"))]
        if radio.events_ccastopped.read().events_ccastopped().bit_is_set() {
            value |= Self::CCA_STOPPED;
        }
        #[cfg(any(
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_rateboost.read().events_rateboost().bit_is_set() {
            value |= Self::RATE_BOOST;
        }
        #[cfg(any(
            feature = "nrf52805",
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_txready.read().events_txready().bit_is_set() {
            value |= Self::TX_READY;
        }
        #[cfg(any(
            feature = "nrf52805",
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_rxready.read().events_rxready().bit_is_set() {
            value |= Self::RX_READY;
        }
        #[cfg(any(
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_mhrmatch.read().events_mhrmatch().bit_is_set() {
            value |= Self::MHR_MATCH;
        }
        #[cfg(any(feature = "nrf52820", feature = "nrf52833", feature = "_nrf5340-net"))]
        if radio.events_sync.read().events_sync().bit_is_set() {
            value |= Self::SYNC;
        }
        #[cfg(any(
            feature = "nrf52805",
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_phyend.read().events_phyend().bit_is_set() {
            value |= Self::PHY_END;
        }
        #[cfg(any(
            feature = "nrf52811",
            feature = "nrf52820",
            feature = "nrf52833",
            feature = "_nrf5340-net"
        ))]
        if radio.events_ctepresent.read().events_ctepresent().bit_is_set() {
            value |= Self::CTE_PRESENT;
        }
        value
    }
    // The nRF52832 SVD probably is a bit broken
    /// Read events from radio
    #[cfg(feature = "nrf52832")]
    pub fn from_radio(radio: &pac::radio::RegisterBlock) -> Self {
        let mut value = Self::empty();
        if radio.events_ready.read().bits() == 1 {
            value |= Self::READY;
        }
        if radio.events_address.read().bits() == 1 {
            value |= Self::ADDRESS;
        }
        if radio.events_payload.read().bits() == 1 {
            value |= Self::PAYLOAD;
        }
        if radio.events_end.read().bits() == 1 {
            value |= Self::END;
        }
        if radio.events_disabled.read().bits() == 1 {
            value |= Self::DISABLED;
        }
        if radio.events_devmatch.read().bits() == 1 {
            value |= Self::DEV_MATCH;
        }
        if radio.events_devmiss.read().bits() == 1 {
            value |= Self::DEV_MISS;
        }
        if radio.events_rssiend.read().bits() == 1 {
            value |= Self::RSSI_END;
        }
        if radio.events_bcmatch.read().bits() == 1 {
            value |= Self::BC_MATCH;
        }
        if radio.events_crcok.read().bits() == 1 {
            value |= Self::CRC_OK;
        }
        if radio.events_crcerror.read().bits() == 1 {
            value |= Self::CRC_ERROR;
        }
        value
    }

    /// Read events from radio, mask with set interrupts
    pub fn from_radio_masked(radio: &pac::radio::RegisterBlock) -> Self {
        Self::from_radio(radio) & Self::from_bits_truncate(radio.intenset.read().bits())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Event {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            if self.contains(Self::READY) { "RD" } else { "__" },
            if self.contains(Self::ADDRESS) { "AD" } else { "__" },
            if self.contains(Self::PAYLOAD) { "PL" } else { "__" },
            if self.contains(Self::END) { " E" } else { "__" },
            if self.contains(Self::DISABLED) { "DI" } else { "__" },
            if self.contains(Self::DEV_MATCH) { "D+" } else { "__" },
            if self.contains(Self::DEV_MISS) { "D-" } else { "__" },
            if self.contains(Self::RSSI_END) { "RE" } else { "__" },
            if self.contains(Self::BC_MATCH) { "CM" } else { "__" },
            if self.contains(Self::CRC_OK) { "CO" } else { "__" },
            if self.contains(Self::CRC_ERROR) { "CE" } else { "__" },
            if self.contains(Self::FRAME_START) { "FS" } else { "__" },
            if self.contains(Self::ED_END) { "EE" } else { "__" },
            if self.contains(Self::ED_STOPPED) { "ES" } else { "__" },
            if self.contains(Self::CCA_IDLE) { "CI" } else { "__" },
            if self.contains(Self::CCA_BUSY) { "CB" } else { "__" },
            if self.contains(Self::CCA_STOPPED) { "CS" } else { "__" },
            if self.contains(Self::RATE_BOOST) { "RB" } else { "__" },
            if self.contains(Self::TX_READY) { "TX" } else { "__" },
            if self.contains(Self::RX_READY) { "RX" } else { "__" },
            if self.contains(Self::MHR_MATCH) { "MM" } else { "__" },
            if self.contains(Self::SYNC) { "SY" } else { "__" },
            if self.contains(Self::PHY_END) { "PE" } else { "__" },
            if self.contains(Self::CTE_PRESENT) { "CP" } else { "__" },
        )
    }
}

impl core::fmt::Display for Event {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            fmt,
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            if self.contains(Self::READY) { "RD" } else { "__" },
            if self.contains(Self::ADDRESS) { "AD" } else { "__" },
            if self.contains(Self::PAYLOAD) { "PL" } else { "__" },
            if self.contains(Self::END) { " E" } else { "__" },
            if self.contains(Self::DISABLED) { "DI" } else { "__" },
            if self.contains(Self::DEV_MATCH) { "D+" } else { "__" },
            if self.contains(Self::DEV_MISS) { "D-" } else { "__" },
            if self.contains(Self::RSSI_END) { "RE" } else { "__" },
            if self.contains(Self::BC_MATCH) { "CM" } else { "__" },
            if self.contains(Self::CRC_OK) { "CO" } else { "__" },
            if self.contains(Self::CRC_ERROR) { "CE" } else { "__" },
            if self.contains(Self::FRAME_START) { "FS" } else { "__" },
            if self.contains(Self::ED_END) { "EE" } else { "__" },
            if self.contains(Self::ED_STOPPED) { "ES" } else { "__" },
            if self.contains(Self::CCA_IDLE) { "CI" } else { "__" },
            if self.contains(Self::CCA_BUSY) { "CB" } else { "__" },
            if self.contains(Self::CCA_STOPPED) { "CS" } else { "__" },
            if self.contains(Self::RATE_BOOST) { "RB" } else { "__" },
            if self.contains(Self::TX_READY) { "TX" } else { "__" },
            if self.contains(Self::RX_READY) { "RX" } else { "__" },
            if self.contains(Self::MHR_MATCH) { "MM" } else { "__" },
            if self.contains(Self::SYNC) { "SY" } else { "__" },
            if self.contains(Self::PHY_END) { "PE" } else { "__" },
            if self.contains(Self::CTE_PRESENT) { "CP" } else { "__" },
        )
    }
}
