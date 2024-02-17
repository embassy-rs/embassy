// Note: This file is copied and modified from fdcan crate by Richard Meadows

use core::convert::Infallible;
use core::slice;

use crate::can::fd::config::*;
use crate::can::fd::message_ram::enums::*;
use crate::can::fd::message_ram::{RegisterBlock, RxFifoElement, TxBufferElement};
use crate::can::frame::*;

/// Loopback Mode
#[derive(Clone, Copy, Debug)]
enum LoopbackMode {
    None,
    Internal,
    External,
}

pub struct Registers {
    pub regs: &'static crate::pac::can::Fdcan,
    pub msgram: &'static crate::pac::fdcanram::Fdcanram,
}

impl Registers {
    fn tx_buffer_element(&self, bufidx: usize) -> &mut TxBufferElement {
        &mut self.msg_ram_mut().transmit.tbsa[bufidx]
    }
    pub fn msg_ram_mut(&self) -> &mut RegisterBlock {
        let ptr = self.msgram.as_ptr() as *mut RegisterBlock;
        unsafe { &mut (*ptr) }
    }

    fn rx_fifo_element(&self, fifonr: usize, bufnum: usize) -> &mut RxFifoElement {
        &mut self.msg_ram_mut().receive[fifonr].fxsa[bufnum]
    }

    pub fn read_classic(&self, fifonr: usize) -> Option<(ClassicFrame, u16)> {
        // Fill level - do we have a msg?
        if self.regs.rxfs(fifonr).read().ffl() < 1 {
            return None;
        }

        let read_idx = self.regs.rxfs(fifonr).read().fgi();
        let mailbox = self.rx_fifo_element(fifonr, read_idx as usize);

        let mut buffer: [u8; 8] = [0; 8];
        let maybe_header = extract_frame(mailbox, &mut buffer);

        // Clear FIFO, reduces count and increments read buf
        self.regs.rxfa(fifonr).modify(|w| w.set_fai(read_idx));

        match maybe_header {
            Some((header, ts)) => {
                let data = ClassicData::new(&buffer[0..header.len() as usize]);
                Some((ClassicFrame::new(header, data.unwrap()), ts))
            }
            None => None,
        }
    }

    pub fn read_fd(&self, fifonr: usize) -> Option<(FdFrame, u16)> {
        // Fill level - do we have a msg?
        if self.regs.rxfs(fifonr).read().ffl() < 1 {
            return None;
        }

        let read_idx = self.regs.rxfs(fifonr).read().fgi();
        let mailbox = self.rx_fifo_element(fifonr, read_idx as usize);

        let mut buffer: [u8; 64] = [0; 64];
        let maybe_header = extract_frame(mailbox, &mut buffer);

        // Clear FIFO, reduces count and increments read buf
        self.regs.rxfa(fifonr).modify(|w| w.set_fai(read_idx));

        match maybe_header {
            Some((header, ts)) => {
                let data = FdData::new(&buffer[0..header.len() as usize]);
                Some((FdFrame::new(header, data.unwrap()), ts))
            }
            None => None,
        }
    }

    pub fn put_tx_frame(&self, bufidx: usize, header: &Header, buffer: &[u8]) {
        // Fill level - do we have a msg?
        //if self.regs.rxfs(fifonr).read().ffl() < 1 { return None; }

        //let read_idx = self.regs.rxfs(fifonr).read().fgi();

        let mailbox = self.tx_buffer_element(bufidx);

        mailbox.reset();
        put_tx_header(mailbox, header);
        put_tx_data(mailbox, &buffer[..header.len() as usize]);

        // Set <idx as Mailbox> as ready to transmit
        self.regs.txbar().modify(|w| w.set_ar(bufidx, true));
    }

    /// Returns if the tx queue is able to accept new messages without having to cancel an existing one
    #[inline]
    pub fn tx_queue_is_full(&self) -> bool {
        self.regs.txfqs().read().tfqf()
    }

    #[inline]
    pub fn has_pending_frame(&self, idx: usize) -> bool {
        self.regs.txbrp().read().trp(idx)
    }

    /// Returns `Ok` when the mailbox is free or if it contains pending frame with a
    /// lower priority (higher ID) than the identifier `id`.
    #[inline]
    pub fn is_available(&self, bufidx: usize, id: &embedded_can::Id) -> bool {
        if self.has_pending_frame(bufidx) {
            let mailbox = self.tx_buffer_element(bufidx);

            let header_reg = mailbox.header.read();
            let old_id = make_id(header_reg.id().bits(), header_reg.xtd().bits());

            *id > old_id
        } else {
            true
        }
    }

    /// Attempts to abort the sending of a frame that is pending in a mailbox.
    ///
    /// If there is no frame in the provided mailbox, or its transmission succeeds before it can be
    /// aborted, this function has no effect and returns `false`.
    ///
    /// If there is a frame in the provided mailbox, and it is canceled successfully, this function
    /// returns `true`.
    #[inline]
    pub fn abort(&self, bufidx: usize) -> bool {
        let can = self.regs;

        // Check if there is a request pending to abort
        if self.has_pending_frame(bufidx) {
            // Abort Request
            can.txbcr().write(|w| w.set_cr(bufidx, true));

            // Wait for the abort request to be finished.
            loop {
                if can.txbcf().read().cf(bufidx) {
                    // Return false when a transmission has occured
                    break can.txbto().read().to(bufidx) == false;
                }
            }
        } else {
            false
        }
    }

    #[inline]
    //fn abort_pending_mailbox<PTX, R>(&mut self, idx: Mailbox, pending: PTX) -> Option<R>
    pub fn abort_pending_mailbox(&self, bufidx: usize) -> Option<ClassicFrame>
//where
    //    PTX: FnOnce(Mailbox, TxFrameHeader, &[u32]) -> R,
    {
        if self.abort(bufidx) {
            let mailbox = self.tx_buffer_element(bufidx);

            let header_reg = mailbox.header.read();
            let id = make_id(header_reg.id().bits(), header_reg.xtd().bits());

            let len = match header_reg.to_data_length() {
                DataLength::Fdcan(len) => len,
                DataLength::Classic(len) => len,
            };
            if len as usize > ClassicFrame::MAX_DATA_LEN {
                return None;
            }

            //let tx_ram = self.tx_msg_ram();
            let mut data = [0u8; 64];
            data_from_tx_buffer(&mut data, mailbox, len as usize);

            let cd = ClassicData::new(&data).unwrap();
            Some(ClassicFrame::new(Header::new(id, len, header_reg.rtr().bit()), cd))
        } else {
            // Abort request failed because the frame was already sent (or being sent) on
            // the bus. All mailboxes are now free. This can happen for small prescaler
            // values (e.g. 1MBit/s bit timing with a source clock of 8MHz) or when an ISR
            // has preempted the execution.
            None
        }
    }

    #[inline]
    //fn abort_pending_mailbox<PTX, R>(&mut self, idx: Mailbox, pending: PTX) -> Option<R>
    pub fn abort_pending_fd_mailbox(&self, bufidx: usize) -> Option<FdFrame>
//where
    //    PTX: FnOnce(Mailbox, TxFrameHeader, &[u32]) -> R,
    {
        if self.abort(bufidx) {
            let mailbox = self.tx_buffer_element(bufidx);

            let header_reg = mailbox.header.read();
            let id = make_id(header_reg.id().bits(), header_reg.xtd().bits());

            let len = match header_reg.to_data_length() {
                DataLength::Fdcan(len) => len,
                DataLength::Classic(len) => len,
            };
            if len as usize > FdFrame::MAX_DATA_LEN {
                return None;
            }

            //let tx_ram = self.tx_msg_ram();
            let mut data = [0u8; 64];
            data_from_tx_buffer(&mut data, mailbox, len as usize);

            let cd = FdData::new(&data).unwrap();

            let header = if header_reg.fdf().frame_format() == FrameFormat::Fdcan {
                Header::new_fd(id, len, header_reg.rtr().bit(), header_reg.brs().bit())
            } else {
                Header::new(id, len, header_reg.rtr().bit())
            };

            Some(FdFrame::new(header, cd))
        } else {
            // Abort request failed because the frame was already sent (or being sent) on
            // the bus. All mailboxes are now free. This can happen for small prescaler
            // values (e.g. 1MBit/s bit timing with a source clock of 8MHz) or when an ISR
            // has preempted the execution.
            None
        }
    }

    /// As Transmit, but if there is a pending frame, `pending` will be called so that the frame can
    /// be preserved.
    //pub fn transmit_preserve<PTX, P>(
    pub fn write_classic(&self, frame: &ClassicFrame) -> nb::Result<Option<ClassicFrame>, Infallible> {
        let queue_is_full = self.tx_queue_is_full();

        let id = frame.header().id();

        // If the queue is full,
        // Discard the first slot with a lower priority message
        let (idx, pending_frame) = if queue_is_full {
            if self.is_available(0, id) {
                (0, self.abort_pending_mailbox(0))
            } else if self.is_available(1, id) {
                (1, self.abort_pending_mailbox(1))
            } else if self.is_available(2, id) {
                (2, self.abort_pending_mailbox(2))
            } else {
                // For now we bail when there is no lower priority slot available
                // Can this lead to priority inversion?
                return Err(nb::Error::WouldBlock);
            }
        } else {
            // Read the Write Pointer
            let idx = self.regs.txfqs().read().tfqpi();

            (idx, None)
        };

        self.put_tx_frame(idx as usize, frame.header(), frame.data());

        Ok(pending_frame)
    }

    /// As Transmit, but if there is a pending frame, `pending` will be called so that the frame can
    /// be preserved.
    //pub fn transmit_preserve<PTX, P>(
    pub fn write_fd(&self, frame: &FdFrame) -> nb::Result<Option<FdFrame>, Infallible> {
        let queue_is_full = self.tx_queue_is_full();

        let id = frame.header().id();

        // If the queue is full,
        // Discard the first slot with a lower priority message
        let (idx, pending_frame) = if queue_is_full {
            if self.is_available(0, id) {
                (0, self.abort_pending_fd_mailbox(0))
            } else if self.is_available(1, id) {
                (1, self.abort_pending_fd_mailbox(1))
            } else if self.is_available(2, id) {
                (2, self.abort_pending_fd_mailbox(2))
            } else {
                // For now we bail when there is no lower priority slot available
                // Can this lead to priority inversion?
                return Err(nb::Error::WouldBlock);
            }
        } else {
            // Read the Write Pointer
            let idx = self.regs.txfqs().read().tfqpi();

            (idx, None)
        };

        self.put_tx_frame(idx as usize, frame.header(), frame.data());

        Ok(pending_frame)
    }

    #[inline]
    fn reset_msg_ram(&mut self) {
        self.msg_ram_mut().reset();
    }

    #[inline]
    fn enter_init_mode(&mut self) {
        self.regs.cccr().modify(|w| w.set_init(true));
        while false == self.regs.cccr().read().init() {}
        self.regs.cccr().modify(|w| w.set_cce(true));
    }

    /// Enables or disables loopback mode: Internally connects the TX and RX
    /// signals together.
    #[inline]
    fn set_loopback_mode(&mut self, mode: LoopbackMode) {
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
    fn set_bus_monitoring_mode(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_mon(enabled));
    }

    #[inline]
    fn set_restricted_operations(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_asm(enabled));
    }

    #[inline]
    fn set_normal_operations(&mut self, _enabled: bool) {
        self.set_loopback_mode(LoopbackMode::None);
    }

    #[inline]
    fn set_test_mode(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_test(enabled));
    }

    #[inline]
    fn set_power_down_mode(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_csr(enabled));
        while self.regs.cccr().read().csa() != enabled {}
    }

    /// Moves out of PoweredDownMode and into ConfigMode
    #[inline]
    pub fn into_config_mode(mut self, _config: FdCanConfig) {
        self.set_power_down_mode(false);
        self.enter_init_mode();

        self.reset_msg_ram();

        // check the FDCAN core matches our expections
        assert!(
            self.regs.crel().read().rel() == 3,
            "Expected FDCAN core major release 3"
        );
        assert!(
            self.regs.endn().read().etv() == 0x87654321_u32,
            "Error reading endianness test value from FDCAN core"
        );

        // Framework specific settings are set here

        // set TxBuffer to Queue Mode
        self.regs.txbc().write(|w| w.set_tfqm(true));

        // set standard filters list size to 28
        // set extended filters list size to 8
        // REQUIRED: we use the memory map as if these settings are set
        // instead of re-calculating them.
        #[cfg(not(stm32h7))]
        {
            self.regs.rxgfc().modify(|w| {
                w.set_lss(crate::can::fd::message_ram::STANDARD_FILTER_MAX);
                w.set_lse(crate::can::fd::message_ram::EXTENDED_FILTER_MAX);
            });
        }
        #[cfg(stm32h7)]
        {
            self.regs
                .sidfc()
                .modify(|w| w.set_lss(crate::can::fd::message_ram::STANDARD_FILTER_MAX));
            self.regs
                .xidfc()
                .modify(|w| w.set_lse(crate::can::fd::message_ram::EXTENDED_FILTER_MAX));
        }

        /*
        for fid in 0..crate::can::message_ram::STANDARD_FILTER_MAX {
            self.set_standard_filter((fid as u8).into(), StandardFilter::disable());
        }
        for fid in 0..Ecrate::can::message_ram::XTENDED_FILTER_MAX {
            self.set_extended_filter(fid.into(), ExtendedFilter::disable());
        }
        */
    }

    /// Disables the CAN interface and returns back the raw peripheral it was created from.
    #[inline]
    pub fn free(mut self) {
        //self.disable_interrupts(Interrupts::all());

        //TODO check this!
        self.enter_init_mode();
        self.set_power_down_mode(true);
        //self.control.instance
    }

    /// Applies the settings of a new FdCanConfig See [`FdCanConfig`]
    #[inline]
    pub fn apply_config(&mut self, config: FdCanConfig) {
        self.set_data_bit_timing(config.dbtr);
        self.set_nominal_bit_timing(config.nbtr);
        self.set_automatic_retransmit(config.automatic_retransmit);
        self.set_transmit_pause(config.transmit_pause);
        self.set_frame_transmit(config.frame_transmit);
        //self.set_interrupt_line_config(config.interrupt_line_config);
        self.set_non_iso_mode(config.non_iso_mode);
        self.set_edge_filtering(config.edge_filtering);
        self.set_protocol_exception_handling(config.protocol_exception_handling);
        self.set_global_filter(config.global_filter);
    }

    #[inline]
    fn leave_init_mode(&mut self, config: FdCanConfig) {
        self.apply_config(config);

        self.regs.cccr().modify(|w| w.set_cce(false));
        self.regs.cccr().modify(|w| w.set_init(false));
        while self.regs.cccr().read().init() == true {}
    }

    /// Moves out of ConfigMode and into InternalLoopbackMode
    #[inline]
    pub fn into_internal_loopback(mut self, config: FdCanConfig) {
        self.set_loopback_mode(LoopbackMode::Internal);
        self.leave_init_mode(config);
    }

    /// Moves out of ConfigMode and into ExternalLoopbackMode
    #[inline]
    pub fn into_external_loopback(mut self, config: FdCanConfig) {
        self.set_loopback_mode(LoopbackMode::External);
        self.leave_init_mode(config);
    }

    /// Moves out of ConfigMode and into RestrictedOperationMode
    #[inline]
    pub fn into_restricted(mut self, config: FdCanConfig) {
        self.set_restricted_operations(true);
        self.leave_init_mode(config);
    }

    /// Moves out of ConfigMode and into NormalOperationMode
    #[inline]
    pub fn into_normal(mut self, config: FdCanConfig) {
        self.set_normal_operations(true);
        self.leave_init_mode(config);
    }

    /// Moves out of ConfigMode and into BusMonitoringMode
    #[inline]
    pub fn into_bus_monitoring(mut self, config: FdCanConfig) {
        self.set_bus_monitoring_mode(true);
        self.leave_init_mode(config);
    }

    /// Moves out of ConfigMode and into Testmode
    #[inline]
    pub fn into_test_mode(mut self, config: FdCanConfig) {
        self.set_test_mode(true);
        self.leave_init_mode(config);
    }

    /// Moves out of ConfigMode and into PoweredDownmode
    #[inline]
    pub fn into_powered_down(mut self, config: FdCanConfig) {
        self.set_power_down_mode(true);
        self.leave_init_mode(config);
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
    pub fn set_nominal_bit_timing(&mut self, btr: NominalBitTiming) {
        //self.control.config.nbtr = btr;

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
    pub fn set_data_bit_timing(&mut self, btr: DataBitTiming) {
        //self.control.config.dbtr = btr;

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
    pub fn set_automatic_retransmit(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_dar(!enabled));
        //self.control.config.automatic_retransmit = enabled;
    }

    /// Configures the transmit pause feature. See
    /// [`FdCanConfig::set_transmit_pause`]
    #[inline]
    pub fn set_transmit_pause(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_txp(!enabled));
        //self.control.config.transmit_pause = enabled;
    }

    /// Configures non-iso mode. See [`FdCanConfig::set_non_iso_mode`]
    #[inline]
    pub fn set_non_iso_mode(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_niso(enabled));
        //self.control.config.non_iso_mode = enabled;
    }

    /// Configures edge filtering. See [`FdCanConfig::set_edge_filtering`]
    #[inline]
    pub fn set_edge_filtering(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_efbi(enabled));
        //self.control.config.edge_filtering = enabled;
    }

    /// Configures frame transmission mode. See
    /// [`FdCanConfig::set_frame_transmit`]
    #[inline]
    pub fn set_frame_transmit(&mut self, fts: FrameTransmissionConfig) {
        let (fdoe, brse) = match fts {
            FrameTransmissionConfig::ClassicCanOnly => (false, false),
            FrameTransmissionConfig::AllowFdCan => (true, false),
            FrameTransmissionConfig::AllowFdCanAndBRS => (true, true),
        };

        self.regs.cccr().modify(|w| {
            w.set_fdoe(fdoe);
            #[cfg(stm32h7)]
            w.set_bse(brse);
            #[cfg(not(stm32h7))]
            w.set_brse(brse);
        });

        //self.control.config.frame_transmit = fts;
    }

    /// Sets the protocol exception handling on/off
    #[inline]
    pub fn set_protocol_exception_handling(&mut self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_pxhd(!enabled));

        //self.control.config.protocol_exception_handling = enabled;
    }

    /// Configures and resets the timestamp counter
    #[inline]
    pub fn set_timestamp_counter_source(&mut self, select: TimestampSource) {
        #[cfg(stm32h7)]
        let (tcp, tss) = match select {
            TimestampSource::None => (0, 0),
            TimestampSource::Prescaler(p) => (p as u8, 1),
            TimestampSource::FromTIM3 => (0, 2),
        };

        #[cfg(not(stm32h7))]
        let (tcp, tss) = match select {
            TimestampSource::None => (0, stm32_metapac::can::vals::Tss::ZERO),
            TimestampSource::Prescaler(p) => (p as u8, stm32_metapac::can::vals::Tss::INCREMENT),
            TimestampSource::FromTIM3 => (0, stm32_metapac::can::vals::Tss::EXTERNAL),
        };

        self.regs.tscc().write(|w| {
            w.set_tcp(tcp);
            w.set_tss(tss);
        });

        //self.control.config.timestamp_source = select;
    }

    #[cfg(not(stm32h7))]
    /// Configures the global filter settings
    #[inline]
    pub fn set_global_filter(&mut self, filter: GlobalFilter) {
        let anfs = match filter.handle_standard_frames {
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo0 => stm32_metapac::can::vals::Anfs::ACCEPT_FIFO_0,
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo1 => stm32_metapac::can::vals::Anfs::ACCEPT_FIFO_1,
            crate::can::fd::config::NonMatchingFilter::Reject => stm32_metapac::can::vals::Anfs::REJECT,
        };
        let anfe = match filter.handle_extended_frames {
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo0 => stm32_metapac::can::vals::Anfe::ACCEPT_FIFO_0,
            crate::can::fd::config::NonMatchingFilter::IntoRxFifo1 => stm32_metapac::can::vals::Anfe::ACCEPT_FIFO_1,
            crate::can::fd::config::NonMatchingFilter::Reject => stm32_metapac::can::vals::Anfe::REJECT,
        };

        self.regs.rxgfc().modify(|w| {
            w.set_anfs(anfs);
            w.set_anfe(anfe);
            w.set_rrfs(filter.reject_remote_standard_frames);
            w.set_rrfe(filter.reject_remote_extended_frames);
        });
    }

    #[cfg(stm32h7)]
    /// Configures the global filter settings
    #[inline]
    pub fn set_global_filter(&mut self, filter: GlobalFilter) {
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
}

fn make_id(id: u32, extended: bool) -> embedded_can::Id {
    if extended {
        embedded_can::Id::from(unsafe { embedded_can::ExtendedId::new_unchecked(id & 0x1FFFFFFF) })
    } else {
        embedded_can::Id::from(unsafe { embedded_can::StandardId::new_unchecked((id & 0x000007FF) as u16) })
    }
}

fn put_tx_header(mailbox: &mut TxBufferElement, header: &Header) {
    let (id, id_type) = match header.id() {
        embedded_can::Id::Standard(id) => (id.as_raw() as u32, IdType::StandardId),
        embedded_can::Id::Extended(id) => (id.as_raw() as u32, IdType::ExtendedId),
    };

    // Use FDCAN only for DLC > 8. FDCAN users can revise this if required.
    let frame_format = if header.len() > 8 || header.fdcan() {
        FrameFormat::Fdcan
    } else {
        FrameFormat::Classic
    };
    let brs = header.len() > 8 || header.bit_rate_switching();

    mailbox.header.write(|w| {
        unsafe { w.id().bits(id) }
            .rtr()
            .bit(header.len() == 0 && header.rtr())
            .xtd()
            .set_id_type(id_type)
            .set_len(DataLength::new(header.len(), frame_format))
            .set_event(Event::NoEvent)
            .fdf()
            .set_format(frame_format)
            .brs()
            .bit(brs)
        //esi.set_error_indicator(//TODO//)
    });
}

fn put_tx_data(mailbox: &mut TxBufferElement, buffer: &[u8]) {
    let mut lbuffer = [0_u32; 16];
    let len = buffer.len();
    let data = unsafe { slice::from_raw_parts_mut(lbuffer.as_mut_ptr() as *mut u8, len) };
    data[..len].copy_from_slice(&buffer[..len]);
    let data_len = ((len) + 3) / 4;
    for (register, byte) in mailbox.data.iter_mut().zip(lbuffer[..data_len].iter()) {
        unsafe { register.write(*byte) };
    }
}

fn data_from_fifo(buffer: &mut [u8], mailbox: &RxFifoElement, len: usize) {
    for (i, register) in mailbox.data.iter().enumerate() {
        let register_value = register.read();
        let register_bytes = unsafe { slice::from_raw_parts(&register_value as *const u32 as *const u8, 4) };
        let num_bytes = (len) - i * 4;
        if num_bytes <= 4 {
            buffer[i * 4..i * 4 + num_bytes].copy_from_slice(&register_bytes[..num_bytes]);
            break;
        }
        buffer[i * 4..(i + 1) * 4].copy_from_slice(register_bytes);
    }
}

fn data_from_tx_buffer(buffer: &mut [u8], mailbox: &TxBufferElement, len: usize) {
    for (i, register) in mailbox.data.iter().enumerate() {
        let register_value = register.read();
        let register_bytes = unsafe { slice::from_raw_parts(&register_value as *const u32 as *const u8, 4) };
        let num_bytes = (len) - i * 4;
        if num_bytes <= 4 {
            buffer[i * 4..i * 4 + num_bytes].copy_from_slice(&register_bytes[..num_bytes]);
            break;
        }
        buffer[i * 4..(i + 1) * 4].copy_from_slice(register_bytes);
    }
}

impl From<&RxFifoElement> for ClassicFrame {
    fn from(mailbox: &RxFifoElement) -> Self {
        let header_reg = mailbox.header.read();

        let id = make_id(header_reg.id().bits(), header_reg.xtd().bits());
        let dlc = header_reg.to_data_length().len();
        let len = dlc as usize;

        let mut buffer: [u8; 64] = [0; 64];
        data_from_fifo(&mut buffer, mailbox, len);
        let data = ClassicData::new(&buffer[0..len]);
        let header = Header::new(id, dlc, header_reg.rtr().bits());
        ClassicFrame::new(header, data.unwrap())
    }
}

fn extract_frame(mailbox: &RxFifoElement, buffer: &mut [u8]) -> Option<(Header, u16)> {
    let header_reg = mailbox.header.read();

    let id = make_id(header_reg.id().bits(), header_reg.xtd().bits());
    let dlc = header_reg.to_data_length().len();
    let len = dlc as usize;
    let timestamp = header_reg.txts().bits;
    if len > buffer.len() {
        return None;
    }
    data_from_fifo(buffer, mailbox, len);
    let header = if header_reg.fdf().bits {
        Header::new_fd(id, dlc, header_reg.rtr().bits(), header_reg.brs().bits())
    } else {
        Header::new(id, dlc, header_reg.rtr().bits())
    };
    Some((header, timestamp))
}
