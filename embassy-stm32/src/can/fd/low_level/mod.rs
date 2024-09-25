use stm32_metapac::can::regs::{Ndat1, Ndat2, Txbcr};
use util::{RxElementData, TxElementData};

use crate::can::frame::Header;

mod configuration;
mod filter;

pub(crate) mod message_ram;

pub(crate) mod util;

/// Loopback Mode
#[derive(Clone, Copy, Debug)]
pub(crate) enum LoopbackMode {
    None,
    Internal,
    External,
}

#[repr(u8)]
#[allow(dead_code)]
pub(crate) enum TimestampSource {
    /// Timestamp always set to 0
    Zero = 0b00,
    Internal = 0b01,
    /// tim3.cnt[0:15] used as source
    External = 0b11,
}

#[derive(defmt::Format)]
pub(crate) struct RxLevels {
    pub rx_0_level: u8,
    pub rx_1_level: u8,
    pub buf_mask: u64,
}

pub(crate) struct CanLowLevel {
    pub(crate) regs: crate::pac::can::Fdcan,
    pub(crate) msgram: crate::pac::fdcanram::Fdcanram,

    pub(crate) message_ram: message_ram::MessageRam,

    #[allow(dead_code)]
    pub msg_ram_offset: usize,
    #[allow(dead_code)]
    pub msg_ram_size: usize,
}

/// Mode management
impl CanLowLevel {
    pub fn enter_init_mode(&self) {
        self.regs.cccr().modify(|w| w.set_init(true));
        while self.regs.cccr().read().init() == false {}
        self.regs.cccr().modify(|w| w.set_cce(true));
    }

    pub fn leave_init_mode(&self) {
        self.regs.cccr().modify(|w| w.set_cce(false));
        self.regs.cccr().modify(|w| w.set_init(false));
        while self.regs.cccr().read().init() == true {}
    }

    /// Moves out of ConfigMode and into specified mode
    #[inline]
    pub fn into_mode(&self, mode: crate::can::_version::OperatingMode) {
        match mode {
            crate::can::OperatingMode::InternalLoopbackMode => self.set_loopback_mode(LoopbackMode::Internal),
            crate::can::OperatingMode::ExternalLoopbackMode => self.set_loopback_mode(LoopbackMode::External),
            crate::can::OperatingMode::NormalOperationMode => self.set_normal_operations(true),
            crate::can::OperatingMode::RestrictedOperationMode => self.set_restricted_operations(true),
            crate::can::OperatingMode::BusMonitoringMode => self.set_bus_monitoring_mode(true),
        }
        self.leave_init_mode();
    }
}

/// Tx
impl CanLowLevel {
    fn tx_element_set(&self, idx: u8, header: &Header, data: &[u8]) {
        let element = self.message_ram.tx_elements.get_mut(idx as usize);

        let mut n_data = [0u8; 64];
        n_data.iter_mut().zip(data.iter()).for_each(|(to, from)| *to = *from);

        TxElementData {
            header: *header,
            data: n_data,
            marker: 0,
            tx_event: false,
        }
        .put(element);
    }

    #[allow(dead_code)]
    pub fn tx_buffer_add(&self, idx: u8, header: &Header, data: &[u8], before_add: impl FnOnce(u8)) -> Option<u8> {
        // TODO validate that only dedicated buffers are passed.
        // TODO change to bit mask

        if self.regs.txbrp().read().trp(idx as usize) {
            // Transmit already pending for this buffer
            return None;
        }

        before_add(idx);

        // Write message to message RAM
        self.tx_element_set(idx, header, data);

        // Set buffer available bit to indicate ready to TX.
        self.regs.txbar().write(|v| v.set_ar(idx as usize, true));

        Some(idx)
    }

    #[allow(dead_code)]
    pub fn tx_queue_add(&self, header: &Header, data: &[u8], before_add: impl FnOnce(u8)) -> Option<u8> {
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

        before_add(free_idx);

        // Write message to message RAM
        self.tx_element_set(free_idx, header, data);

        // Set buffer available bit to indicate ready to TX.
        self.regs.txbar().write(|v| v.set_ar(free_idx as usize, true));

        Some(free_idx)
    }

    pub fn tx_fifo_add(&self, header: &Header, data: &[u8], before_add: impl FnOnce(u8)) -> Option<u8> {
        let status = self.regs.txfqs().read();

        if status.tfqf() {
            // If full, return None.
            return None;
        }

        let free_idx = status.tfqpi();

        before_add(free_idx);

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
    pub fn tx_cancel(&self, mask: u32) {
        self.regs.txbcr().write_value(Txbcr(mask));
    }
}

impl CanLowLevel {
    pub fn rx_buffer_read(&self, buffer_mask: u64) -> Option<(u8, RxElementData)> {
        let ndat = self.get_ndat_mask();
        let available = ndat & buffer_mask;

        if available == 0 {
            return None;
        }

        let buffer_idx = available.leading_zeros();

        let element = self.message_ram.rx_buffer.get_mut(buffer_idx as usize);
        let ret = RxElementData::extract(element);

        match buffer_idx {
            idx if idx < 32 => self.regs.ndat1().write_value(Ndat1(buffer_idx << idx)),
            idx if idx < 64 => self.regs.ndat2().write_value(Ndat2(buffer_idx << (idx - 32))),
            _ => panic!(),
        };

        Some((buffer_idx as u8, ret))
    }

    pub fn rx_fifo_read(&self, fifo_num: u8) -> Option<RxElementData> {
        let status = self.regs.rxfs(fifo_num as usize).read();

        let fill_level = status.ffl();
        if fill_level == 0 {
            return None;
        }

        let get_index = self.regs.rxfs(fifo_num as usize).read().fgi();
        let element = self.message_ram.rx_fifos[fifo_num as usize].get_mut(get_index as usize);
        let ret = RxElementData::extract(element);

        self.regs.rxfa(fifo_num as usize).write(|v| v.set_fai(get_index));

        Some(ret)
    }

    #[inline]
    pub fn get_ndat_mask(&self) -> u64 {
        (self.regs.ndat1().read().nd() as u64) | ((self.regs.ndat2().read().nd() as u64) << 32)
    }

    #[inline]
    pub fn get_rx_levels(&self) -> RxLevels {
        RxLevels {
            rx_0_level: self.regs.rxfs(0).read().ffl(),
            rx_1_level: self.regs.rxfs(1).read().ffl(),
            buf_mask: self.get_ndat_mask(),
        }
    }
}

/// Error stuff
impl CanLowLevel {
    //fn reg_to_error(value: u8) -> Option<BusError> {
    //    match value {
    //        //0b000 => None,
    //        0b001 => Some(BusError::Stuff),
    //        0b010 => Some(BusError::Form),
    //        0b011 => Some(BusError::Acknowledge),
    //        0b100 => Some(BusError::BitRecessive),
    //        0b101 => Some(BusError::BitDominant),
    //        0b110 => Some(BusError::Crc),
    //        //0b111 => Some(BusError::NoError),
    //        _ => None,
    //    }
    //}

    //pub fn curr_error(&self) -> Option<BusError> {
    //    let err = { self.regs.psr().read() };
    //    if err.bo() {
    //        return Some(BusError::BusOff);
    //    } else if err.ep() {
    //        return Some(BusError::BusPassive);
    //    } else if err.ew() {
    //        return Some(BusError::BusWarning);
    //    } else {
    //        cfg_if! {
    //            if #[cfg(can_fdcan_h7)] {
    //                let lec = err.lec();
    //            } else {
    //                let lec = err.lec().to_bits();
    //            }
    //        }
    //        if let Some(err) = Self::reg_to_error(lec) {
    //            return Some(err);
    //        }
    //    }
    //    None
    //}
}
