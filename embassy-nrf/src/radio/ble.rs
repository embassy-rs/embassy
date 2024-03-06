//! Radio driver implementation focused on Bluetooth Low-Energy transmission.

use core::future::poll_fn;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_internal::drop::OnDrop;
use embassy_hal_internal::{into_ref, PeripheralRef};
pub use pac::radio::mode::MODE_A as Mode;
#[cfg(not(feature = "nrf51"))]
use pac::radio::pcnf0::PLEN_A as PreambleLength;

use crate::interrupt::typelevel::Interrupt;
use crate::radio::*;
pub use crate::radio::{Error, TxPower};
use crate::util::slice_in_ram_or;

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
                // To simplify the implementation, It is setted as the maximum value
                // and the length of the packet is controlled only by the LENGTH field in the packet
                .maxlen()
                .bits(255)
                // Configure the length of the address field in the packet
                // The prefix after the address fields is always appended, so is always 1 byte less than the size of the address
                // The base address is truncated from the least significant byte if the BALEN is less than 4
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
                .set_bit()
        });

        // Configure CRC
        r.crccnf.write(|w| {
            // In BLE the CRC shall be calculated on the PDU of all Link Layer
            // packets (even if the packet is encrypted).
            // It skips the address field
            w.skipaddr()
                .skip()
                // In BLE  24-bit CRC = 3 bytes
                .len()
                .three()
        });

        // Ch map between 2400 MHZ .. 2500 MHz
        // All modes use this range
        #[cfg(not(feature = "nrf51"))]
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

        Self { _p: radio }
    }

    fn state(&self) -> RadioState {
        super::state(T::regs())
    }

    /// Set the radio mode
    ///
    /// The radio must be disabled before calling this function
    pub fn set_mode(&mut self, mode: Mode) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();
        r.mode.write(|w| w.mode().variant(mode));

        #[cfg(not(feature = "nrf51"))]
        r.pcnf0.write(|w| {
            w.plen().variant(match mode {
                Mode::BLE_1MBIT => PreambleLength::_8BIT,
                Mode::BLE_2MBIT => PreambleLength::_16BIT,
                #[cfg(any(
                    feature = "nrf52811",
                    feature = "nrf52820",
                    feature = "nrf52833",
                    feature = "nrf52840",
                    feature = "_nrf5340-net"
                ))]
                Mode::BLE_LR125KBIT | Mode::BLE_LR500KBIT => PreambleLength::LONG_RANGE,
                _ => unimplemented!(),
            })
        });
    }

    /// Set the header size changing the S1's len field
    ///
    /// The radio must be disabled before calling this function
    pub fn set_header_expansion(&mut self, use_s1_field: bool) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();

        // s1 len in bits
        let s1len: u8 = match use_s1_field {
            false => 0,
            true => 8,
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

    /// Set initial data whitening value
    /// Data whitening is used to avoid long sequences of zeros or ones, e.g., 0b0000000 or 0b1111111, in the data bit stream
    /// On BLE the initial value is the channel index | 0x40
    ///
    /// The radio must be disabled before calling this function
    pub fn set_whitening_init(&mut self, whitening_init: u8) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();

        r.datawhiteiv.write(|w| unsafe { w.datawhiteiv().bits(whitening_init) });
    }

    /// Set the central frequency to be used
    /// It should be in the range 2400..2500
    ///
    /// [The radio must be disabled before calling this function](https://devzone.nordicsemi.com/f/nordic-q-a/15829/radio-frequency-change)
    pub fn set_frequency(&mut self, frequency: u32) {
        assert!(self.state() == RadioState::DISABLED);
        assert!((2400..=2500).contains(&frequency));

        let r = T::regs();

        r.frequency
            .write(|w| unsafe { w.frequency().bits((frequency - 2400) as u8) });
    }

    /// Set the acess address
    /// This address is always constants for advertising
    /// And a random value generate on each connection
    /// It is used to filter the packages
    ///
    /// The radio must be disabled before calling this function
    pub fn set_access_address(&mut self, access_address: u32) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();

        // Configure logical address
        // The byte ordering on air is always least significant byte first for the address
        // So for the address 0xAA_BB_CC_DD, the address on air will be DD CC BB AA
        // The package order is BASE, PREFIX so BASE=0xBB_CC_DD and PREFIX=0xAA
        r.prefix0
            .write(|w| unsafe { w.ap0().bits((access_address >> 24) as u8) });

        // The base address is truncated from the least significant byte (because the BALEN is less than 4)
        // So it shifts the address to the right
        r.base0.write(|w| unsafe { w.bits(access_address << 8) });

        // Don't match tx address
        r.txaddress.write(|w| unsafe { w.txaddress().bits(0) });

        // Match on logical address
        // This config only filter the packets by the address,
        // so only packages send to the previous address
        // will finish the reception (TODO: check the explanation)
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

    /// Set the CRC polynomial
    /// It only uses the 24 least significant bits
    ///
    /// The radio must be disabled before calling this function
    pub fn set_crc_poly(&mut self, crc_poly: u32) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();

        r.crcpoly.write(|w| unsafe {
            // Configure the CRC polynomial
            // Each term in the CRC polynomial is mapped to a bit in this
            // register which index corresponds to the term's exponent.
            // The least significant term/bit is hard-wired internally to
            // 1, and bit number 0 of the register content is ignored by
            // the hardware. The following example is for an 8 bit CRC
            // polynomial: x8 + x7 + x3 + x2 + 1 = 1 1000 1101 .
            w.crcpoly().bits(crc_poly & 0xFFFFFF)
        });
    }

    /// Set the CRC init value
    /// It only uses the 24 least significant bits
    /// The CRC initial value varies depending of the PDU type
    ///
    /// The radio must be disabled before calling this function
    pub fn set_crc_init(&mut self, crc_init: u32) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();

        r.crcinit.write(|w| unsafe { w.crcinit().bits(crc_init & 0xFFFFFF) });
    }

    /// Set the radio tx power
    ///
    /// The radio must be disabled before calling this function
    pub fn set_tx_power(&mut self, tx_power: TxPower) {
        assert!(self.state() == RadioState::DISABLED);

        let r = T::regs();

        r.txpower.write(|w| w.txpower().variant(tx_power));
    }

    /// Set buffer to read/write
    ///
    /// This method is unsound. You should guarantee that the buffer will live
    /// for the life time of the transmission or if the buffer will be modified.
    /// Also if the buffer is smaller than the packet length, the radio will
    /// read/write memory out of the buffer bounds.
    fn set_buffer(&mut self, buffer: &[u8]) -> Result<(), Error> {
        slice_in_ram_or(buffer, Error::BufferNotInRAM)?;

        let r = T::regs();

        // Here it consider that the length of the packet is
        // correctly set in the buffer, otherwise it will send
        // unowned regions of memory
        let ptr = buffer.as_ptr();

        // Configure the payload
        r.packetptr.write(|w| unsafe { w.bits(ptr as u32) });

        Ok(())
    }

    /// Send packet
    /// If the length byte in the package is greater than the buffer length
    /// the radio will read memory out of the buffer bounds
    pub async fn transmit(&mut self, buffer: &[u8]) -> Result<(), Error> {
        self.set_buffer(buffer)?;

        let r = T::regs();
        self.trigger_and_wait_end(move || {
            // Initialize the transmission
            // trace!("txen");

            r.tasks_txen.write(|w| unsafe { w.bits(1) });
        })
        .await;

        Ok(())
    }

    /// Receive packet
    /// If the length byte in the received package is greater than the buffer length
    /// the radio will write memory out of the buffer bounds
    pub async fn receive(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.set_buffer(buffer)?;

        let r = T::regs();
        self.trigger_and_wait_end(move || {
            // Initialize the transmission
            // trace!("rxen");
            r.tasks_rxen.write(|w| unsafe { w.bits(1) });
        })
        .await;

        Ok(())
    }

    async fn trigger_and_wait_end(&mut self, trigger: impl FnOnce()) {
        //self.trace_state();

        let r = T::regs();
        let s = T::state();

        // If the Future is dropped before the end of the transmission
        // it disable the interrupt and stop the transmission
        // to keep the state consistent
        let drop = OnDrop::new(|| {
            trace!("radio drop: stopping");

            r.intenclr.write(|w| w.end().clear());
            r.events_end.reset();

            r.tasks_stop.write(|w| unsafe { w.bits(1) });

            // The docs don't explicitly mention any event to acknowledge the stop task
            while r.events_end.read().bits() == 0 {}

            trace!("radio drop: stopped");
        });

        // trace!("radio:enable interrupt");
        // Clear some remnant side-effects (TODO: check if this is necessary)
        r.events_end.reset();

        // Enable interrupt
        r.intenset.write(|w| w.end().set());

        compiler_fence(Ordering::SeqCst);

        // Trigger the transmission
        trigger();
        // self.trace_state();

        // On poll check if interrupt happen
        poll_fn(|cx| {
            s.event_waker.register(cx.waker());
            if r.events_end.read().bits() == 1 {
                // trace!("radio:end");
                return core::task::Poll::Ready(());
            }
            Poll::Pending
        })
        .await;

        compiler_fence(Ordering::SeqCst);
        r.events_disabled.reset(); // ACK

        // Everthing ends fine, so it disable the drop
        drop.defuse();
    }

    /// Disable the radio
    fn disable(&mut self) {
        let r = T::regs();

        compiler_fence(Ordering::SeqCst);
        // If it is already disabled, do nothing
        if self.state() != RadioState::DISABLED {
            trace!("radio:disable");
            // Trigger the disable task
            r.tasks_disable.write(|w| unsafe { w.bits(1) });

            // Wait until the radio is disabled
            while r.events_disabled.read().bits() == 0 {}

            compiler_fence(Ordering::SeqCst);

            // Acknowledge it
            r.events_disabled.reset();
        }
    }
}

impl<'d, T: Instance> Drop for Radio<'d, T> {
    fn drop(&mut self) {
        self.disable();
    }
}
