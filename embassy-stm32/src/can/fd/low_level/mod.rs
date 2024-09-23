use cfg_if::cfg_if;
use message_ram::{HeaderElement, MessageRamConfig, RxFifoConfig};
use stm32_metapac::can::regs::{Ndat1, Ndat2, Txbcr};

use crate::can::{
    enums::BusError,
    fd::{
        message_ram::RxFifoElementHeader,
        peripheral::{extract_frame, put_tx_data, put_tx_header},
    },
    frame::Header,
};

use super::{
    config::{DataBitTiming, FdCanConfig, FrameTransmissionConfig, GlobalFilter, NominalBitTiming},
    peripheral::LoopbackMode,
};

mod message_ram;

#[repr(u8)]
enum TimestampSource {
    Zero = 0b00,
    Internal = 0b01,
    /// tim3.cnt[0:15] used as source
    External = 0b11,
}

struct CanLowLevel {
    pub(crate) regs: crate::pac::can::Fdcan,
    pub(crate) msgram: crate::pac::fdcanram::Fdcanram,

    pub(crate) message_ram: message_ram::MessageRam,

    #[allow(dead_code)]
    pub msg_ram_offset: usize,
}

/// Mode management
impl CanLowLevel {
    fn enter_init_mode(&self) {
        self.regs.cccr().modify(|w| w.set_init(true));
        while self.regs.cccr().read().init() == false {}
        self.regs.cccr().modify(|w| w.set_cce(true));
    }

    fn leave_init_mode(&self) {
        self.regs.cccr().modify(|w| w.set_cce(false));
        self.regs.cccr().modify(|w| w.set_init(false));
        while self.regs.cccr().read().init() == true {}
    }
}

/// Configuration.
/// Can only be called when in config mode (CCCR.INIT=1, CCCR.CCE=1).
impl CanLowLevel {
    /// Applies the settings of a new FdCanConfig See [`FdCanConfig`]
    /// Returns a new instance, the old one must not be used.
    pub fn apply_config(&self, config: FdCanConfig) -> Self {
        // self.set_tx_buffer_mode(config.tx_buffer_mode);

        let message_ram = MessageRamConfig {
            base_offset: self.msg_ram_offset,
            available_space: None,
            standard_id_filter_size: 28,
            extended_id_filter_size: 8,
            rx_fifo_0: message_ram::RxFifoConfig {
                operation_mode: message_ram::RxFifoOperationMode::Blocking,
                watermark_interrupt_level: 3,
                fifo_size: 3,
                data_field_size: message_ram::DataFieldSize::B64,
            },
            rx_fifo_1: message_ram::RxFifoConfig::DISABLED,
            rx_buffer: message_ram::RxBufferConfig::DISABLED,
            tx: message_ram::TxConfig {
                queue_operation_mode: message_ram::TxQueueOperationMode::FIFO,
                queue_size: 3,
                dedicated_size: 0,
                data_field_size: message_ram::DataFieldSize::B64,
            },
        }
        .apply_config(&self.regs, &self.msgram);

        let ll = Self {
            regs: self.regs,
            msgram: self.msgram,
            message_ram,
            msg_ram_offset: self.msg_ram_offset,
        };

        // Does not work for FDCAN, internal timer has inconsistent timebase.
        ll.set_timestamp_source(TimestampSource::Internal, 0);

        // TXBTIE bits need to be set for each tx element in order for
        // interrupts to fire.
        ll.regs.txbtie().write(|w| w.0 = 0xffff_ffff);
        ll.regs.ie().modify(|w| {
            w.set_rfne(0, true); // Rx Fifo 0 New Msg
            w.set_rfne(1, true); // Rx Fifo 1 New Msg
            w.set_tce(true); //  Tx Complete
            w.set_boe(true); // Bus-Off Status Changed
        });
        ll.regs.ile().modify(|w| {
            w.set_eint0(true); // Interrupt Line 0
            w.set_eint1(true); // Interrupt Line 1
        });

        ll.set_data_bit_timing(config.dbtr);
        ll.set_nominal_bit_timing(config.nbtr);
        ll.set_automatic_retransmit(config.automatic_retransmit);
        ll.set_transmit_pause(config.transmit_pause);
        ll.set_frame_transmit(config.frame_transmit);
        ll.set_non_iso_mode(config.non_iso_mode);
        ll.set_edge_filtering(config.edge_filtering);
        ll.set_protocol_exception_handling(config.protocol_exception_handling);
        ll.set_global_filter(config.global_filter);

        ll
    }

    /// Configures the global filter settings
    #[inline]
    pub fn set_global_filter(&self, filter: GlobalFilter) {
        let anfs = match filter.handle_standard_frames {
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo0 => 0,
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo1 => 1,
            crate::can::fd::config::NonMatchingFilter::Reject => 2,
        };

        let anfe = match filter.handle_extended_frames {
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo0 => 0,
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo1 => 1,
            crate::can::fd::config::NonMatchingFilter::Reject => 2,
        };

        self.regs.gfc().modify(|w| {
            w.set_anfs(anfs);
            w.set_anfe(anfe);
            w.set_rrfs(filter.reject_remote_standard_frames);
            w.set_rrfe(filter.reject_remote_extended_frames);
        });
    }

    /// Configures the bit timings.
    ///
    /// You can use <http://www.bittiming.can-wiki.info/> to calculate the `btr` parameter. Enter
    /// parameters as follows:
    ///
    /// - *Clock Rate*: The input clock speed to the CAN peripheral (*not* the CPU clock speed).
    ///   This is the clock rate of the peripheral bus the CAN peripheral is attached to (eg. APB1).
    /// - *Sample Point*: Should normally be left at the default value of 87.5%.
    /// - *SJW*: Should normally be left at the default value of 1.
    ///
    /// Then copy the `CAN_BUS_TIME` register value from the table and pass it as the `btr`
    /// parameter to this method.
    #[inline]
    pub fn set_nominal_bit_timing(&self, btr: NominalBitTiming) {
        self.regs.nbtp().write(|w| {
            w.set_nbrp(btr.nbrp() - 1);
            w.set_ntseg1(btr.ntseg1() - 1);
            w.set_ntseg2(btr.ntseg2() - 1);
            w.set_nsjw(btr.nsjw() - 1);
        });
    }

    /// Configures the data bit timings for the FdCan Variable Bitrates.
    /// This is not used when frame_transmit is set to anything other than AllowFdCanAndBRS.
    #[inline]
    pub fn set_data_bit_timing(&self, btr: DataBitTiming) {
        self.regs.dbtp().write(|w| {
            w.set_dbrp(btr.dbrp() - 1);
            w.set_dtseg1(btr.dtseg1() - 1);
            w.set_dtseg2(btr.dtseg2() - 1);
            w.set_dsjw(btr.dsjw() - 1);
        });
    }

    /// Enables or disables automatic retransmission of messages
    ///
    /// If this is enabled, the CAN peripheral will automatically try to retransmit each frame
    /// util it can be sent. Otherwise, it will try only once to send each frame.
    ///
    /// Automatic retransmission is enabled by default.
    #[inline]
    pub fn set_automatic_retransmit(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_dar(!enabled));
    }

    /// Configures the transmit pause feature. See
    /// [`FdCanConfig::set_transmit_pause`]
    #[inline]
    pub fn set_transmit_pause(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_txp(!enabled));
    }

    /// Configures non-iso mode. See [`FdCanConfig::set_non_iso_mode`]
    #[inline]
    pub fn set_non_iso_mode(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_niso(enabled));
    }

    /// Configures edge filtering. See [`FdCanConfig::set_edge_filtering`]
    #[inline]
    pub fn set_edge_filtering(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_efbi(enabled));
    }

    /// Configures frame transmission mode. See
    /// [`FdCanConfig::set_frame_transmit`]
    #[inline]
    pub fn set_frame_transmit(&self, fts: FrameTransmissionConfig) {
        let (fdoe, brse) = match fts {
            FrameTransmissionConfig::ClassicCanOnly => (false, false),
            FrameTransmissionConfig::AllowFdCan => (true, false),
            FrameTransmissionConfig::AllowFdCanAndBRS => (true, true),
        };

        self.regs.cccr().modify(|w| {
            w.set_fdoe(fdoe);
            #[cfg(can_fdcan_h7)]
            w.set_bse(brse);
            #[cfg(not(can_fdcan_h7))]
            w.set_brse(brse);
        });
    }

    /// Sets the protocol exception handling on/off
    #[inline]
    pub fn set_protocol_exception_handling(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_pxhd(!enabled));
    }

    #[inline]
    fn set_timestamp_source(&self, source: TimestampSource, prescaler: u8) {
        self.regs.tscc().write(|v| {
            v.set_tss(source as u8);
            v.set_tcp(prescaler);
        });
    }

    /// Enables or disables loopback mode: internally connects the TX and RX signals together.
    #[inline]
    fn set_loopback_mode(&self, mode: LoopbackMode) {
        let (test, mon, lbck) = match mode {
            LoopbackMode::None => (false, false, false),
            LoopbackMode::Internal => (true, true, true),
            LoopbackMode::External => (true, false, true),
        };

        self.set_test_mode(test);
        self.set_bus_monitoring_mode(mon);
        self.regs.test().modify(|w| w.set_lbck(lbck));
    }

    /// Enables or disables silent mode: Disconnects the TX signal from the pin.
    #[inline]
    fn set_bus_monitoring_mode(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_mon(enabled));
    }

    #[inline]
    fn set_test_mode(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_test(enabled));
    }

    #[inline]
    fn set_clock_stop(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_csr(enabled));
        while self.regs.cccr().read().csa() != enabled {}
    }
}

/// Tx
impl CanLowLevel {
    fn tx_element_set(&self, idx: u8, header: &Header, data: &[u8]) {
        let element = self.message_ram.tx_elements.get_mut(idx as usize);
        put_tx_header(&mut element.header, &header);
        put_tx_data(&mut element.data, data);
    }

    fn tx_element_get(&self, idx: u8) -> (Header, [u8; 64]) {
        todo!()
    }

    fn tx_buffer_add(&self, idx: u8, header: &Header, data: &[u8]) -> Option<u8> {
        if self.regs.txbrp().read().trp(idx as usize) {
            // Transmit already pending for this buffer
            return None;
        }

        // Write message to message RAM
        self.tx_element_set(idx, header, data);

        // Set buffer available bit to indicate ready to TX.
        self.regs.txbar().write(|v| v.set_ar(idx as usize, true));

        Some(idx)
    }

    fn tx_queue_add(&self, header: &Header, data: &[u8]) -> Option<u8> {
        // We could use TXFQS.TFQPI here (same as for FIFO mode), but this
        // causes us to lose slots on TX cancellations.
        // Instead we find the first slot in the queue and insert the message
        // there, this is not a very expensive operation in terms of processor
        // time, it's mostly bitwise stuff.

        let trailing_mask = !u32::MAX
            .overflowing_shl(self.message_ram.tx_elements.len() as u32 & 0b11111)
            .0;
        let free_idx = (self.regs.txbrp().read().0 | trailing_mask).trailing_ones();
        let full = free_idx < 32;
        let free_idx = free_idx as u8;

        if full {
            return None;
        }

        // Write message to message RAM
        self.tx_element_set(free_idx, header, data);

        // Set buffer available bit to indicate ready to TX.
        self.regs.txbar().write(|v| v.set_ar(free_idx as usize, true));

        Some(free_idx)
    }

    fn tx_fifo_add(&self, header: &Header, data: &[u8]) -> Option<u8> {
        let status = self.regs.txfqs().read();

        if status.tfqf() {
            // If full, return None.
            return None;
        }

        let free_idx = status.tfqpi();

        // Write message to message RAM
        self.tx_element_set(free_idx, header, data);

        // Set buffer available bit to indicate ready to TX.
        self.regs.txbar().write(|v| v.set_ar(free_idx as usize, true));

        Some(free_idx)
    }

    /// Request cancellation of the TX slots indicated in `mask`.
    /// If there was no pending messages in any of the slots,
    /// cancellation will be indicated for those slots imemdiately.
    /// Slots with pending messages will be cancelled asynchronously,
    /// and response to the cancellations should be handled in a later
    /// interrupt.
    fn tx_cancel(&self, mask: u32) {
        self.regs.txbcr().write_value(Txbcr(mask));
    }
}

impl CanLowLevel {
    fn rx_element_get(&self, element: &HeaderElement<RxFifoElementHeader>) -> Option<(Header, u16, [u8; 64])> {
        let mut buffer = [0u8; 64];
        // TODO allow read len to be lower than drl?
        let maybe_header = extract_frame(&element.header, &element.data, &mut buffer);

        let (header, ts) = maybe_header.unwrap();
        Some((header, ts, buffer))
    }

    fn rx_buffer_read(&self, buffer_idx: u8) -> Option<(Header, u16, [u8; 64])> {
        let bit_idx = buffer_idx & 0b11111;
        let bit = 1 << bit_idx;

        // TODO fix NDAT1 and NDAT2 should be indexed
        let has_data = match buffer_idx {
            idx if idx < 32 => self.regs.ndat1().read().nd() & bit != 0,
            idx if idx < 64 => self.regs.ndat2().read().nd() & bit != 0,
            _ => panic!(),
        };

        if !has_data {
            return None;
        }

        let element = self.message_ram.rx_buffer.get_mut(buffer_idx as usize);
        let ret = self.rx_element_get(element);

        match buffer_idx {
            idx if idx < 32 => self.regs.ndat1().write_value(Ndat1(bit)),
            idx if idx < 64 => self.regs.ndat2().write_value(Ndat2(bit)),
            _ => panic!(),
        };

        ret
    }

    fn rx_fifo_read(&self, fifo_num: u8) -> Option<(Header, u16, [u8; 64])> {
        let status = self.regs.rxfs(fifo_num as usize).read();

        let fill_level = status.ffl();
        if fill_level == 0 {
            return None;
        }

        let get_index = self.regs.rxfs(fifo_num as usize).read().fgi();
        let element = self.message_ram.rx_fifos[fifo_num as usize].get_mut(get_index as usize);
        let ret = self.rx_element_get(element);

        self.regs.rxfa(fifo_num as usize).write(|v| v.set_fai(get_index));

        ret
    }
}

/// Error stuff
impl CanLowLevel {
    fn reg_to_error(value: u8) -> Option<BusError> {
        match value {
            //0b000 => None,
            0b001 => Some(BusError::Stuff),
            0b010 => Some(BusError::Form),
            0b011 => Some(BusError::Acknowledge),
            0b100 => Some(BusError::BitRecessive),
            0b101 => Some(BusError::BitDominant),
            0b110 => Some(BusError::Crc),
            //0b111 => Some(BusError::NoError),
            _ => None,
        }
    }

    pub fn curr_error(&self) -> Option<BusError> {
        let err = { self.regs.psr().read() };
        if err.bo() {
            return Some(BusError::BusOff);
        } else if err.ep() {
            return Some(BusError::BusPassive);
        } else if err.ew() {
            return Some(BusError::BusWarning);
        } else {
            cfg_if! {
                if #[cfg(can_fdcan_h7)] {
                    let lec = err.lec();
                } else {
                    let lec = err.lec().to_bits();
                }
            }
            if let Some(err) = Self::reg_to_error(lec) {
                return Some(err);
            }
        }
        None
    }
}
