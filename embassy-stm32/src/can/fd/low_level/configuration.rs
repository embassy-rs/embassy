use stm32_metapac::can::vals::Tfqm;

use super::{
    message_ram::{MessageRam, MessageRamSegment},
    CanLowLevel, LoopbackMode, TimestampSource,
};
use crate::can::config::{
    CanFdMode, DataBitTiming, FdCanConfig, GlobalFilter, MessageRamConfig, NominalBitTiming, TxBufferMode,
};

/// Configuration.
/// Can only be called when in config mode (CCCR.INIT=1, CCCR.CCE=1).
impl CanLowLevel {
    pub const unsafe fn new(
        regs: crate::pac::can::Fdcan,
        msgram: crate::pac::fdcanram::Fdcanram,
        msg_ram_offset: usize,
        msg_ram_size: usize,
    ) -> Self {
        Self {
            regs,
            msgram,
            message_ram: MessageRam::DEFAULT,
            msg_ram_offset,
            msg_ram_size,
        }
    }

    pub fn apply_message_ram_config(&mut self, config: MessageRamConfig) {
        let segment = MessageRamSegment {
            base_offset: self.msg_ram_offset,
            available_space: Some(self.msg_ram_size),
        };
        let message_ram = unsafe { config.apply_config(&segment, &self.regs, &self.msgram) };
        self.message_ram = message_ram;
    }

    /// Applies the settings of a new FdCanConfig See [`FdCanConfig`]
    pub fn apply_config(&self, config: &FdCanConfig) {
        self.set_tx_buffer_mode(config.tx_buffer_mode);

        // Does not work for FDCAN, internal timer has inconsistent timebase.
        self.set_timestamp_source(TimestampSource::Internal, 0);

        // TXBTIE and TXBCIE bits need to be set for each tx element in order for
        // interrupts to fire.
        self.regs.txbtie().write(|w| w.0 = 0xffff_ffff);
        self.regs.txbcie().write(|w| w.0 = 0xffff_ffff);
        self.regs.ie().modify(|w| {
            w.set_rfne(0, true); // Rx Fifo 0 New Msg
            w.set_rfne(1, true); // Rx Fifo 1 New Msg
            w.set_drxe(true); // Rx Dedicated Buffer New Msg
            w.set_tce(true); //  Tx Complete
            w.set_tcfe(true); // Tx Cancel Finished
            w.set_boe(true); // Bus-Off Status Changed
        });
        self.regs.ile().modify(|w| {
            w.set_eint0(true); // Interrupt Line 0
            w.set_eint1(true); // Interrupt Line 1
        });

        self.set_data_bit_timing(config.dbtr);
        self.set_nominal_bit_timing(config.nbtr);
        self.set_automatic_retransmit(config.automatic_retransmit);
        self.set_transmit_pause(config.transmit_pause);
        self.set_can_fd_mode(config.can_fd_mode);
        self.set_edge_filtering(config.edge_filtering);
        self.set_protocol_exception_handling(config.protocol_exception_handling);
        self.set_global_filter(config.global_filter);
    }

    #[inline]
    pub fn set_tx_buffer_mode(&self, mode: TxBufferMode) {
        let mode = match mode {
            TxBufferMode::Fifo => Tfqm::FIFO,
            TxBufferMode::Priority => Tfqm::QUEUE,
        };
        self.regs.txbc().modify(|v| v.set_tfqm(mode));
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

    /// Configures edge filtering. See [`FdCanConfig::set_edge_filtering`]
    #[inline]
    pub fn set_edge_filtering(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_efbi(enabled));
    }

    /// Configures frame transmission mode. See
    /// [`FdCanConfig::set_frame_transmit`]
    #[inline]
    pub fn set_can_fd_mode(&self, fts: CanFdMode) {
        let (niso, fdoe, brse) = match fts {
            CanFdMode::ClassicCanOnly => (false, false, false),
            CanFdMode::AllowFdCan => (true, true, false),
            CanFdMode::AllowFdCanAndBRS => (true, true, true),
        };

        self.regs.cccr().modify(|w| {
            w.set_niso(niso);
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
    pub fn set_timestamp_source(&self, source: TimestampSource, prescaler: u8) {
        self.regs.tscc().write(|v| {
            v.set_tss(source as u8);
            v.set_tcp(prescaler);
        });
    }

    /// Enables or disables loopback mode: internally connects the TX and RX signals together.
    #[inline]
    pub fn set_loopback_mode(&self, mode: LoopbackMode) {
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
    pub fn set_bus_monitoring_mode(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_mon(enabled));
    }

    #[inline]
    pub fn set_test_mode(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_test(enabled));
    }

    //#[inline]
    //pub fn set_clock_stop(&self, enabled: bool) {
    //    self.regs.cccr().modify(|w| w.set_csr(enabled));
    //    while self.regs.cccr().read().csa() != enabled {}
    //}

    #[inline]
    pub fn set_restricted_operations(&self, enabled: bool) {
        self.regs.cccr().modify(|w| w.set_asm(enabled));
    }

    #[inline]
    pub fn set_normal_operations(&self, _enabled: bool) {
        self.set_loopback_mode(LoopbackMode::None);
    }
}
