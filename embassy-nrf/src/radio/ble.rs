//! Radio driver implementation focused on Bluetooth Low-Energy transmission.
//!
//! The radio can calculate the CRC, perform data whitening,
//! automatically send the right preamble.
//! Most of the configuration is done automatically when you choose the mode and this driver.
//!
//! Some configuration can just be done when de device is disabled,
//! and the configuration varies depending if is a transmitter or a receiver.
//! Because of that we have a state machine to keep track of the state of the radio.
//! The Radio is the disable radio which configure the common parameters between
//! the bluetooth protocols, like the package format, the CRC and the whitening.
//! The TxRadio radio enable and configured as a transmitter with the specific parameters.

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
use jewel::phy::{Channel, ChannelTrait, HeaderSize, Mode, Radio as BleRadio, CRC_POLY, MAX_PDU_LENGTH};
use pac::radio::mode::MODE_A as PacMode;
use pac::radio::pcnf0::PLEN_A as PreambleLength;
// Re-export SVD variants to allow user to directly set values.
pub use pac::radio::{state::STATE_A as RadioState, txpower::TXPOWER_A as TxPower};

use crate::interrupt::typelevel::Interrupt;
use crate::radio::*;
use crate::util::slice_in_ram_or;

/// UART error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Buffer was too long.
    BufferTooLong,
    /// Buffer was to short.
    BufferTooShort,
    /// The buffer is not in data RAM. It's most likely in flash, and nRF's DMA cannot access flash.
    BufferNotInRAM,
}

/// Radio driver.
pub struct Radio<'d, T: Instance> {
    _p: PeripheralRef<'d, T>,
}

impl<'d, T: Instance> Radio<'d, T> {
    /// Create a new radio driver.
    pub fn new(
        radio: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        // From 5.4.1 of the nRF52840 Product Specification:
        // > The HFXO must be running to use the RADIO or  the calibration mechanism associated with the 32.768 kHz RC oscillator.
        // Currently the jewel crate don't implement the calibration mechanism, so we need to ensure that the HFXO is running
        utils::check_xtal();

        into_ref!(radio);

        let r = T::regs();

        r.pcnf1.write(|w| unsafe {
            // It is 0 bytes long in a standard BLE packet
            w.statlen()
                .bits(0)
                // MaxLen configures the maximum packet payload plus add-on size in
                // number of bytes that can be transmitted or received by the RADIO. This feature can be used to ensure
                // that the RADIO does not overwrite, or read beyond, the RAM assigned to the packet payload. This means
                // that if the packet payload length defined by PCNF1.STATLEN and the LENGTH field in the packet specifies a
                // packet larger than MAXLEN, the payload will be truncated at MAXLEN
                //
                // To simplify the implementation, I'm setting the max length to the maximum value
                // and I'm using only the length field to truncate the payload
                .maxlen()
                .bits(255)
                // Configure the length of the address field in the packet
                // The prefix after the address fields is always appended, so is always 1 byte less than the size of the address
                //  The base address is truncated from the least significant byte if the BALEN is less than 4
                //
                // BLE address is always 4 bytes long
                .balen()
                .bits(3) // 3 bytes base address (+ 1 prefix);
                // Configure the endianess
                // For BLE is always little endian (LSB first)
                .endian()
                .little()
                // Data whitening is used to avoid long sequences of zeros or
                // ones, e.g., 0b0000000 or 0b1111111, in the data bit stream.
                // The whitener and de-whitener are defined the same way,
                // using a 7-bit linear feedback shift register with the
                // polynomial x7 + x4 + 1.
                //
                // In BLE Whitening shall be applied on the PDU and CRC of all
                // Link Layer packets and is performed after the CRC generation
                // in the transmitter. No other parts of the packets are whitened.
                // De-whitening is performed before the CRC checking in the receiver
                // Before whitening or de-whitening, the shift register should be
                // initialized based on the channel index.
                .whiteen()
                .set_bit() // Enable whitening
        });

        // Configure CRC
        r.crccnf.write(|w| {
            // In BLE the CRC shall be calculated on the PDU of all Link Layer
            // packets (even if the packet is encrypted).
            // So here we skip the address field
            w.skipaddr()
                .skip()
                // In BLE  24-bit CRC = 3 bytes
                .len()
                .three()
        });

        r.crcpoly.write(|w| unsafe {
            // Configure the CRC polynomial
            // Each term in the CRC polynomial is mapped to a bit in this
            // register which index corresponds to the term's exponent.
            // The least significant term/bit is hard-wired internally to
            // 1, and bit number 0 of the register content is ignored by
            // the hardware. The following example is for an 8 bit CRC
            // polynomial: x8 + x7 + x3 + x2 + 1 = 1 1000 1101 .
            w.crcpoly().bits(CRC_POLY & 0xFFFFFF)
        });
        // The CRC initial value varies depending of the PDU type

        // Ch map between 2400 MHZ .. 2500 MHz
        // All modes use this range
        r.frequency.write(|w| w.map().default());

        // Configure shortcuts to simplify and speed up sending and receiving packets.
        r.shorts.write(|w| {
            // start transmission/recv immediately after ramp-up
            // disable radio when transmission/recv is done
            w.ready_start().enabled().end_disable().enabled()
        });

        // Enable NVIC interrupt
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        let mut radio = Self { _p: radio };

        radio
    }

    #[allow(dead_code)]
    fn trace_state(&self) {
        let r = T::regs();

        match r.state.read().state().variant().unwrap() {
            RadioState::DISABLED => trace!("radio:state:DISABLED"),
            RadioState::RX_RU => trace!("radio:state:RX_RU"),
            RadioState::RX_IDLE => trace!("radio:state:RX_IDLE"),
            RadioState::RX => trace!("radio:state:RX"),
            RadioState::RX_DISABLE => trace!("radio:state:RX_DISABLE"),
            RadioState::TX_RU => trace!("radio:state:TX_RU"),
            RadioState::TX_IDLE => trace!("radio:state:TX_IDLE"),
            RadioState::TX => trace!("radio:state:TX"),
            RadioState::TX_DISABLE => trace!("radio:state:TX_DISABLE"),
        }
    }

    async fn trigger_and_wait_end(&mut self, trigger: impl FnOnce() -> ()) {
        //self.trace_state();

        let r = T::regs();
        let s = T::state();

        // If the Future is dropped before the end of the transmission
        // we need to disable the interrupt and stop the transmission
        // to keep the state consistent
        let drop = OnDrop::new(|| {
            trace!("radio drop: stopping");

            r.intenclr.write(|w| w.end().clear());
            r.events_end.reset();

            r.tasks_stop.write(|w| w.tasks_stop().set_bit());

            // The docs don't explicitly mention any event to acknowledge the stop task
            // So I guess it's the same as end
            while r.events_end.read().events_end().bit_is_clear() {}

            trace!("radio drop: stopped");
        });

        /* Config interrupt */
        // trace!("radio:enable interrupt");
        // Clear some remnant side-effects (I'm unsure if this is needed)
        r.events_end.reset();

        // Enable interrupt
        r.intenset.write(|w| w.end().set());

        compiler_fence(Ordering::SeqCst);

        // Trigger the transmission
        trigger();
        // self.trace_state();

        // On poll check if interrupt happen
        poll_fn(|cx| {
            s.end_waker.register(cx.waker());
            if r.events_end.read().events_end().bit_is_set() {
                // trace!("radio:end");
                return core::task::Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_disabled.reset(); // ACK

        // Everthing ends fine, so we can disable the drop
        drop.defuse();
    }

    /// Disable the radio.
    fn disable(&mut self) {
        let r = T::regs();

        compiler_fence(Ordering::SeqCst);
        // If is already disabled, do nothing
        if !r.state.read().state().is_disabled() {
            trace!("radio:disable");
            // Trigger the disable task
            r.tasks_disable.write(|w| w.tasks_disable().set_bit());

            // Wait until the radio is disabled
            while r.events_disabled.read().events_disabled().bit_is_clear() {}

            compiler_fence(Ordering::SeqCst);

            // Acknowledge it
            r.events_disabled.reset();
        }
    }
}

impl<'d, T: Instance> BleRadio for Radio<'d, T> {
    type Error = Error;

    fn set_mode(&mut self, mode: Mode) {
        let r = T::regs();
        r.mode.write(|w| {
            w.mode().variant(match mode {
                Mode::Ble1mbit => PacMode::BLE_1MBIT,
                //Mode::Ble2mbit => PacMode::BLE_2MBIT,
            })
        });

        r.pcnf0.write(|w| {
            w.plen().variant(match mode {
                Mode::Ble1mbit => PreambleLength::_8BIT,
                //Mode::Ble2mbit => PreambleLength::_16BIT,
            })
        });
    }

    fn set_header_size(&mut self, header_size: HeaderSize) {
        let r = T::regs();

        let s1len: u8 = match header_size {
            HeaderSize::TwoBytes => 0,
            HeaderSize::ThreeBytes => 8, // bits
        };

        r.pcnf0.write(|w| unsafe {
            w
                // Configure S0 to 1 byte length, this will represent the Data/Adv header flags
                .s0len()
                .set_bit()
                // Configure the length (in bits) field to 1 byte length, this will represent the length of the payload
                // and also be used to know how many bytes to read/write from/to the buffer
                .lflen()
                .bits(8)
                // Configure the lengh (in bits) of bits in the S1 field. It could be used to represent the CTEInfo for data packages in BLE.
                .s1len()
                .bits(s1len)
        });
    }

    fn set_channel(&mut self, channel: Channel) {
        let r = T::regs();

        r.frequency
            .write(|w| unsafe { w.frequency().bits((channel.central_frequency() - 2400) as u8) });
        r.datawhiteiv
            .write(|w| unsafe { w.datawhiteiv().bits(channel.whitening_init()) });
    }

    fn set_access_address(&mut self, access_address: u32) {
        let r = T::regs();

        // Configure logical address
        // The byte ordering on air is always least significant byte first for the address
        // So for the address 0xAA_BB_CC_DD, the address on air will be DD CC BB AA
        // The package order is BASE, PREFIX so BASE=0xBB_CC_DD and PREFIX=0xAA
        r.prefix0
            .write(|w| unsafe { w.ap0().bits((access_address >> 24) as u8) });

        // The base address is truncated from the least significant byte (because the BALEN is less than 4)
        // So we need to shift the address to the right
        r.base0.write(|w| unsafe { w.bits(access_address << 8) });

        // Don't match tx address
        r.txaddress.write(|w| unsafe { w.txaddress().bits(0) });

        // Match on logical address
        // For what I understand, this config only filter the packets
        // by the address, so only packages send to the previous address
        // will finish the reception
        r.rxaddresses.write(|w| {
            w.addr0()
                .enabled()
                .addr1()
                .enabled()
                .addr2()
                .enabled()
                .addr3()
                .enabled()
                .addr4()
                .enabled()
        });
    }

    fn set_crc_init(&mut self, crc_init: u32) {
        let r = T::regs();

        r.crcinit.write(|w| unsafe { w.crcinit().bits(crc_init & 0xFFFFFF) });
    }

    fn set_tx_power(&mut self, power_db: i8) {
        let r = T::regs();

        let tx_power: TxPower = match power_db {
            8..=i8::MAX => TxPower::POS8D_BM,
            7 => TxPower::POS7D_BM,
            6 => TxPower::POS6D_BM,
            5 => TxPower::POS5D_BM,
            4 => TxPower::POS4D_BM,
            3 => TxPower::POS3D_BM,
            1..=2 => TxPower::POS2D_BM,
            -3..=0 => TxPower::_0D_BM,
            -7..=-4 => TxPower::NEG4D_BM,
            -11..=-8 => TxPower::NEG8D_BM,
            -15..=-12 => TxPower::NEG12D_BM,
            -19..=-16 => TxPower::NEG16D_BM,
            -29..=-20 => TxPower::NEG20D_BM,
            -39..=-30 => TxPower::NEG30D_BM,
            i8::MIN..=-40 => TxPower::NEG40D_BM,
        };

        r.txpower.write(|w| w.txpower().variant(tx_power));
    }

    fn set_buffer(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        // Because we are serializing the buffer, we should always have the buffer in RAM
        slice_in_ram_or(buffer, Error::BufferNotInRAM)?;

        if buffer.len() > MAX_PDU_LENGTH {
            return Err(Error::BufferTooLong);
        }

        let r = T::regs();

        // Here we are considering that the length of the packet is
        // correctly set in the buffer, otherwise we will sending
        // unowned regions of memory
        let ptr = buffer.as_ptr();

        // Configure the payload
        r.packetptr.write(|w| unsafe { w.bits(ptr as u32) });

        Ok(())
    }

    /// Send packet
    async fn transmit(&mut self) {
        let r = T::regs();

        self.trigger_and_wait_end(move || {
            // Initialize the transmission
            // trace!("txen");
            r.tasks_txen.write(|w| w.tasks_txen().set_bit());
        })
        .await;
    }

    /// Send packet
    async fn receive(&mut self) {
        let r = T::regs();

        self.trigger_and_wait_end(move || {
            // Initialize the transmission
            // trace!("rxen");
            r.tasks_rxen.write(|w| w.tasks_rxen().set_bit());

            // Await until ready
            while r.events_ready.read().events_ready().bit_is_clear() {}

            compiler_fence(Ordering::SeqCst);

            // Acknowledge it
            r.events_ready.reset();

            // trace!("radio:start");
            r.tasks_start.write(|w| w.tasks_start().set_bit());
        })
        .await;
    }
}

impl<'d, T: Instance> Drop for Radio<'d, T> {
    fn drop(&mut self) {
        self.disable();
    }
}
