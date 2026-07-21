//! Blocking I3C controller driver for STM32N6, STM32H5, and STM32U3.

use core::marker::PhantomData;

use embassy_hal_internal::Peri;
#[cfg(feature = "time")]
use embassy_time::{Duration, Instant};

use super::config::Config;
use super::{
    DIR_READ, DIR_WRITE, DIRECT_WITHOUT_DEFBYTE_RESTART, DIRECT_WITHOUT_DEFBYTE_STOP, Error, GENERATE_RESTART,
    GENERATE_STOP, Info, Instance, MTYPE_CCC, MTYPE_DIRECT, MTYPE_PRIVATE, PRIVATE_WITHOUT_ARB_RESTART,
    PRIVATE_WITHOUT_ARB_STOP, SclPin, SdaPin,
};
use crate::gpio::Flex;
use crate::mode::Blocking;
use crate::pac::i3c::regs::Cr;
use crate::pac::i3c::vals::{Crinit, Mend, Thres};
use crate::rcc;

/// I3C controller in blocking mode.
pub struct Controller<'d, T: Instance> {
    info: &'static Info,
    _peri: Peri<'d, T>,
    _scl: Option<Flex<'d>>,
    _sda: Option<Flex<'d>>,
    #[cfg(feature = "time")]
    timeout: Duration,
    _mode: PhantomData<Blocking>,
}

impl<'d, T: Instance> Controller<'d, T> {
    /// Create a new blocking I3C controller.
    pub fn new_blocking(
        peri: Peri<'d, T>,
        scl: Peri<'d, impl SclPin<T>>,
        sda: Peri<'d, impl SdaPin<T>>,
        config: Config,
    ) -> Self {
        Self::new_inner(
            peri,
            new_pin!(scl, config.scl_af()),
            new_pin!(sda, config.sda_af()),
            config,
        )
    }

    fn new_inner(peri: Peri<'d, T>, scl: Option<Flex<'d>>, sda: Option<Flex<'d>>, config: Config) -> Self {
        rcc::enable_and_reset::<T>();

        let info = T::info();
        init_controller(info, config);

        Self {
            info,
            _peri: peri,
            _scl: scl,
            _sda: sda,
            #[cfg(feature = "time")]
            timeout: Duration::from_millis(1000),
            _mode: PhantomData,
        }
    }

    /// Set the timeout for blocking operations.
    #[cfg(feature = "time")]
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Read data from a target using a direct CCC command.
    pub fn direct_ccc_read(&mut self, target_addr: u8, ccc: u8, buf: &mut [u8]) -> Result<(), Error> {
        let mut ctrl = [0u32; 2];
        prepare_direct_ccc_control(&mut ctrl, target_addr, ccc, 0, buf.len(), true, false);
        let rx_len = buf.len();

        let mut ctx = XferContext {
            ctrl: &mut ctrl,
            ctrl_idx: 0,
            ctrl_remaining: 2,
            tx: &[],
            tx_idx: 0,
            tx_remaining: 0,
            rx: buf,
            rx_idx: 0,
            rx_remaining: rx_len,
        };
        self.blocking_transfer(&mut ctx)
    }

    /// Write data to a target using a direct CCC command.
    pub fn direct_ccc_write(&mut self, target_addr: u8, ccc: u8, data: &[u8]) -> Result<(), Error> {
        let mut ctrl = [0u32; 2];
        prepare_direct_ccc_control(&mut ctrl, target_addr, ccc, 0, data.len(), false, false);

        let mut ctx = XferContext {
            ctrl: &mut ctrl,
            ctrl_idx: 0,
            ctrl_remaining: 2,
            tx: data,
            tx_idx: 0,
            tx_remaining: data.len(),
            rx: &mut [],
            rx_idx: 0,
            rx_remaining: 0,
        };
        self.blocking_transfer(&mut ctx)
    }

    /// Perform a private I3C write to `target_addr`.
    pub fn private_write(&mut self, target_addr: u8, data: &[u8]) -> Result<(), Error> {
        let mut ctrl = [0u32; 1];
        prepare_private_control(&mut ctrl, target_addr, data.len(), false, false);

        let mut ctx = XferContext {
            ctrl: &mut ctrl,
            ctrl_idx: 0,
            ctrl_remaining: 1,
            tx: data,
            tx_idx: 0,
            tx_remaining: data.len(),
            rx: &mut [],
            rx_idx: 0,
            rx_remaining: 0,
        };
        self.blocking_transfer(&mut ctx)
    }

    /// Perform a private I3C read from `target_addr`.
    pub fn private_read(&mut self, target_addr: u8, buf: &mut [u8]) -> Result<(), Error> {
        let mut ctrl = [0u32; 1];
        prepare_private_control(&mut ctrl, target_addr, buf.len(), true, false);
        let rx_len = buf.len();

        let mut ctx = XferContext {
            ctrl: &mut ctrl,
            ctrl_idx: 0,
            ctrl_remaining: 1,
            tx: &[],
            tx_idx: 0,
            tx_remaining: 0,
            rx: buf,
            rx_idx: 0,
            rx_remaining: rx_len,
        };
        self.blocking_transfer(&mut ctx)
    }

    /// Broadcast a CCC command (no target address).
    pub fn broadcast_ccc(&mut self, ccc: u8, data: &[u8]) -> Result<(), Error> {
        let mut ctrl = [0u32; 1];
        ctrl[0] = (data.len() as u32) | ((ccc as u32) << 16) | MTYPE_CCC | GENERATE_STOP;

        let mut ctx = XferContext {
            ctrl: &mut ctrl,
            ctrl_idx: 0,
            ctrl_remaining: 1,
            tx: data,
            tx_idx: 0,
            tx_remaining: data.len(),
            rx: &mut [],
            rx_idx: 0,
            rx_remaining: 0,
        };
        self.blocking_transfer(&mut ctx)
    }

    /// Assign dynamic addresses to all targets on the bus (ENTDAA).
    ///
    /// Returns `(dynamic_address, provisioned_id)` pairs. Addresses start at `0x08`.
    pub fn dyn_addr_assign(&mut self) -> Result<heapless::Vec<(u8, u64), 8>, Error> {
        self.dyn_addr_assign_with_reset(false)
    }

    /// Assign dynamic addresses, optionally sending RSTDAA before ENTDAA.
    pub fn dyn_addr_assign_with_reset(&mut self, reset_first: bool) -> Result<heapless::Vec<(u8, u64), 8>, Error> {
        let regs = self.info.regs;
        let mut targets = heapless::Vec::new();
        let mut next_addr = 0x08u8;

        regs.cfgr().modify(|w| w.set_noarbh(false));

        if reset_first {
            self.broadcast_ccc(0x06, &[])?; // RSTDAA
        }

        write_ccc_cr(regs, 0x07, GENERATE_STOP); // ENTDAA

        loop {
            self.wait_daa_event()?;

            let evr = regs.evr().read();
            if evr.fcf() {
                regs.cevr().write(|w| w.set_cfcf(true));
                break;
            }

            if evr.txfnff() {
                let payload = read_entdaa_payload(regs);
                let addr = next_addr;
                next_addr = next_addr.wrapping_add(1);
                if next_addr > 0x7E {
                    return Err(Error::AddressOutOfRange);
                }

                regs.tx_data_regs().dr().write(|w| w.set_db(addr));
                targets.push((addr, payload)).map_err(|_| Error::Other)?;
            }
        }

        Ok(targets)
    }

    fn blocking_transfer(&mut self, ctx: &mut XferContext<'_>) -> Result<(), Error> {
        let regs = self.info.regs;

        if ctx.ctrl_remaining > 0 {
            ctx.ctrl_remaining -= 1;
            let word = ctx.next_word()?;
            regs.cr().write_value(Cr(word));
        }

        loop {
            service_control_fifo(regs, ctx)?;
            service_tx_fifo(regs, ctx);
            service_rx_fifo(regs, ctx);

            let evr = regs.evr().read();
            let exit = evr.0 & (1 << 9 | 1 << 22); // FCF | ERRF

            if evr.fcf() && ctx.ctrl_remaining > 0 {
                regs.cevr().write(|w| w.set_cfcf(true));
                regs.cfgr().modify(|w| w.set_tsfset(true));
            }

            if exit != 0 {
                break;
            }
        }

        let evr = regs.evr().read();
        if evr.fcf() {
            regs.cevr().write(|w| w.set_cfcf(true));
        }

        if evr.errf() {
            regs.cevr().write(|w| w.set_cerrf(true));
            return Err(read_error(regs));
        }

        if ctx.tx_remaining != 0 || ctx.rx_remaining != 0 {
            return Err(Error::Size);
        }

        Ok(())
    }

    fn wait_daa_event(&self) -> Result<(), Error> {
        #[cfg(feature = "time")]
        {
            let deadline = Instant::now() + self.timeout;
            loop {
                let evr = self.info.regs.evr().read();
                if evr.fcf() || evr.txfnff() {
                    return Ok(());
                }
                if evr.errf() {
                    self.info.regs.cevr().write(|w| w.set_cerrf(true));
                    return Err(read_error(self.info.regs));
                }
                if Instant::now() >= deadline {
                    return Err(Error::Timeout);
                }
            }
        }

        #[cfg(not(feature = "time"))]
        loop {
            let evr = self.info.regs.evr().read();
            if evr.fcf() || evr.txfnff() {
                return Ok(());
            }
            if evr.errf() {
                self.info.regs.cevr().write(|w| w.set_cerrf(true));
                return Err(read_error(self.info.regs));
            }
        }
    }
}

impl<'d, T: Instance> Drop for Controller<'d, T> {
    fn drop(&mut self) {
        self.info.rcc.disable();
    }
}

struct XferContext<'a> {
    ctrl: &'a mut [u32],
    ctrl_idx: usize,
    ctrl_remaining: usize,
    tx: &'a [u8],
    tx_idx: usize,
    tx_remaining: usize,
    rx: &'a mut [u8],
    rx_idx: usize,
    rx_remaining: usize,
}

impl XferContext<'_> {
    fn next_word(&mut self) -> Result<u32, Error> {
        if self.ctrl_idx >= self.ctrl.len() {
            return Err(Error::InvalidParam);
        }
        let word = self.ctrl[self.ctrl_idx];
        self.ctrl_idx += 1;
        Ok(word)
    }
}

fn init_controller(info: &Info, config: Config) {
    let regs = info.regs;
    let t = config.timing;
    let c = config.controller;

    regs.cfgr().modify(|w| w.set_en(false));
    regs.cfgr().modify(|w| w.set_crinit(Crinit::Controller));

    regs.timingr0().write(|w| {
        w.set_scll_pp(t.scl_pp_low);
        w.set_sclh_i3c(t.scl_i3c_high);
        w.set_scll_od(t.scl_od_low);
        w.set_sclh_i2c(t.scl_i2c_high);
    });

    regs.timingr1().write(|w| {
        w.set_aval(t.bus_idle);
        w.set_asncr(t.wait_time);
        w.set_free(t.bus_free);
        w.set_sda_hd(t.sda_hold_1_5);
    });

    regs.timingr2().write(|w| {
        w.set_stall(c.stall_time);
        w.set_stalla(false);
        w.set_stallc(false);
        w.set_stalld(false);
        w.set_stallt(false);
    });

    regs.cfgr().modify(|w| {
        w.set_hksdaen(c.high_keeper_sda);
        w.set_hjack(c.hot_join_allowed);
        w.set_rxthres(Thres::Byte);
        w.set_txthres(Thres::Byte);
        w.set_tmode(false);
        w.set_rmode(false);
    });

    if c.dynamic_addr != 0 {
        regs.devr0().modify(|w| w.set_da(c.dynamic_addr));
    }

    regs.cfgr().modify(|w| w.set_en(true));
}

fn prepare_direct_ccc_control(
    ctrl: &mut [u32],
    target_addr: u8,
    ccc: u8,
    defbyte: u32,
    data_len: usize,
    read: bool,
    stop_between: bool,
) {
    let stop = if stop_between {
        DIRECT_WITHOUT_DEFBYTE_STOP
    } else {
        DIRECT_WITHOUT_DEFBYTE_RESTART
    };

    ctrl[0] = defbyte | ((ccc as u32) << 16) | MTYPE_CCC | GENERATE_RESTART;
    ctrl[1] = ((data_len as u32).saturating_sub(defbyte))
        | if read { DIR_READ } else { DIR_WRITE }
        | ((target_addr as u32) << 17)
        | MTYPE_DIRECT
        | stop;
}

fn prepare_private_control(ctrl: &mut [u32], target_addr: u8, data_len: usize, read: bool, stop_between: bool) {
    let stop = if stop_between {
        PRIVATE_WITHOUT_ARB_STOP
    } else {
        PRIVATE_WITHOUT_ARB_RESTART
    };

    ctrl[0] = (data_len as u32)
        | if read { DIR_READ } else { DIR_WRITE }
        | ((target_addr as u32) << 17)
        | MTYPE_PRIVATE
        | stop;
}

fn write_ccc_cr(regs: crate::pac::i3c::I3c, ccc: u8, end: u32) {
    regs.cr_alternate().write(|w| {
        w.set_dcnt(0);
        w.set_ccc(ccc);
        w.set_mtype((MTYPE_CCC >> 27) as u8);
        w.set_mend(if end == GENERATE_STOP {
            Mend::Stop
        } else {
            Mend::RepeatedStart
        });
    });
}

fn read_entdaa_payload(regs: crate::pac::i3c::I3c) -> u64 {
    let mut payload = 0u64;
    for i in 0..8 {
        payload |= (regs.rx_data_regs().dr().read().db() as u64) << (i * 8);
    }
    payload
}

fn service_control_fifo(regs: crate::pac::i3c::I3c, ctx: &mut XferContext<'_>) -> Result<(), Error> {
    if regs.evr().read().cfnff() && ctx.ctrl_remaining > 0 {
        ctx.ctrl_remaining -= 1;
        let word = ctx.next_word()?;
        regs.cr().write_value(Cr(word));
    }
    Ok(())
}

fn service_tx_fifo(regs: crate::pac::i3c::I3c, ctx: &mut XferContext<'_>) {
    while regs.evr().read().txfnff() && ctx.tx_remaining > 0 {
        let byte = ctx.tx[ctx.tx_idx];
        ctx.tx_idx += 1;
        ctx.tx_remaining -= 1;
        regs.tx_data_regs().dr().write(|w| w.set_db(byte));
    }
}

fn service_rx_fifo(regs: crate::pac::i3c::I3c, ctx: &mut XferContext<'_>) {
    while regs.evr().read().rxfnef() && ctx.rx_remaining > 0 {
        let byte = regs.rx_data_regs().dr().read().db();
        ctx.rx[ctx.rx_idx] = byte;
        ctx.rx_idx += 1;
        ctx.rx_remaining -= 1;
    }
}

fn read_error(regs: crate::pac::i3c::I3c) -> Error {
    let ser = regs.ser().read();
    if ser.anack() {
        Error::AddressNack
    } else if ser.dnack() {
        Error::DataNack
    } else if ser.dovr() || ser.covr() {
        Error::FifoOverrun
    } else if ser.perr() {
        Error::Protocol
    } else if ser.derr() {
        Error::DataHandOff
    } else {
        Error::Other
    }
}
