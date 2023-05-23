use embassy_hal_common::{into_ref, Peripheral, PeripheralRef};

use crate::ipcc::sealed::Instance;
use crate::peripherals::IPCC;
use crate::rcc::sealed::RccPeripheral;

#[non_exhaustive]
#[derive(Clone, Copy, Default)]
pub struct Config {
    // TODO: add IPCC peripheral configuration, if any, here
    // reserved for future use
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum IpccChannel {
    Channel1 = 0,
    Channel2 = 1,
    Channel3 = 2,
    Channel4 = 3,
    Channel5 = 4,
    Channel6 = 5,
}

pub(crate) mod sealed {
    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs() -> crate::pac::ipcc::Ipcc;
        fn set_cpu2(enabled: bool);
    }
}

pub struct Ipcc<'d> {
    _peri: PeripheralRef<'d, IPCC>,
}

impl<'d> Ipcc<'d> {
    pub fn new(peri: impl Peripheral<P = IPCC> + 'd, _config: Config) -> Self {
        Self::new_inner(peri)
    }

    pub(crate) fn new_inner(peri: impl Peripheral<P = IPCC> + 'd) -> Self {
        into_ref!(peri);

        Self { _peri: peri }
    }

    pub fn init(&mut self) {
        IPCC::enable();
        IPCC::reset();
        IPCC::set_cpu2(true);

        unsafe { _configure_pwr() };

        let regs = IPCC::regs();

        unsafe {
            regs.cpu(0).cr().modify(|w| {
                w.set_rxoie(true);
                w.set_txfie(true);
            })
        }
    }

    pub fn c1_set_rx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(0).mr().modify(|w| w.set_chom(channel as usize, !enabled)) }
    }

    pub fn c1_get_rx_channel(&self, channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(0).mr().read().chom(channel as usize) }
    }

    pub fn c2_set_rx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(1).mr().modify(|w| w.set_chom(channel as usize, !enabled)) }
    }

    pub fn c2_get_rx_channel(&self, channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(1).mr().read().chom(channel as usize) }
    }

    pub fn c1_set_tx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(0).mr().modify(|w| w.set_chfm(channel as usize, !enabled)) }
    }

    pub fn c1_get_tx_channel(&self, channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(0).mr().read().chfm(channel as usize) }
    }

    pub fn c2_set_tx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(1).mr().modify(|w| w.set_chfm(channel as usize, !enabled)) }
    }

    pub fn c2_get_tx_channel(&self, channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(1).mr().read().chfm(channel as usize) }
    }

    /// clears IPCC receive channel status for CPU1
    pub fn c1_clear_flag_channel(&mut self, channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(0).scr().write(|w| w.set_chc(channel as usize, true)) }
    }

    /// clears IPCC receive channel status for CPU2
    pub fn c2_clear_flag_channel(&mut self, channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(1).scr().write(|w| w.set_chc(channel as usize, true)) }
    }

    pub fn c1_set_flag_channel(&mut self, channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(0).scr().write(|w| w.set_chs(channel as usize, true)) }
    }

    pub fn c2_set_flag_channel(&mut self, channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(1).scr().write(|w| w.set_chs(channel as usize, true)) }
    }

    pub fn c1_is_active_flag(&self, channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        unsafe { regs.cpu(0).sr().read().chf(channel as usize) }
    }

    pub fn c2_is_active_flag(&self, channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        unsafe { regs.cpu(1).sr().read().chf(channel as usize) }
    }

    pub fn is_tx_pending(&self, channel: IpccChannel) -> bool {
        !self.c1_is_active_flag(channel) && self.c1_get_tx_channel(channel)
    }

    pub fn is_rx_pending(&self, channel: IpccChannel) -> bool {
        self.c2_is_active_flag(channel) && self.c1_get_rx_channel(channel)
    }

    pub fn as_mut_ptr(&self) -> *mut Self {
        unsafe { &mut core::ptr::read(self) as *mut _ }
    }
}

impl sealed::Instance for crate::peripherals::IPCC {
    fn regs() -> crate::pac::ipcc::Ipcc {
        crate::pac::IPCC
    }

    fn set_cpu2(enabled: bool) {
        unsafe { crate::pac::PWR.cr4().modify(|w| w.set_c2boot(enabled)) }
    }
}

unsafe fn _configure_pwr() {
    let rcc = crate::pac::RCC;

    // set RF wake-up clock = LSE
    rcc.csr().modify(|w| w.set_rfwkpsel(0b01));
}
