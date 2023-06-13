use self::sealed::Instance;
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

pub mod sealed {
    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs() -> crate::pac::ipcc::Ipcc;
        fn set_cpu2(enabled: bool);
    }
}

pub struct Ipcc;

impl Ipcc {
    pub fn enable(_config: Config) {
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

    pub fn c1_set_rx_channel(channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(0).mr().modify(|w| w.set_chom(channel as usize, !enabled)) }
    }

    pub fn c1_get_rx_channel(channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(0).mr().read().chom(channel as usize) }
    }

    #[allow(dead_code)]
    pub fn c2_set_rx_channel(channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(1).mr().modify(|w| w.set_chom(channel as usize, !enabled)) }
    }

    #[allow(dead_code)]
    pub fn c2_get_rx_channel(channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(1).mr().read().chom(channel as usize) }
    }

    pub fn c1_set_tx_channel(channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(0).mr().modify(|w| w.set_chfm(channel as usize, !enabled)) }
    }

    pub fn c1_get_tx_channel(channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(0).mr().read().chfm(channel as usize) }
    }

    #[allow(dead_code)]
    pub fn c2_set_tx_channel(channel: IpccChannel, enabled: bool) {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { regs.cpu(1).mr().modify(|w| w.set_chfm(channel as usize, !enabled)) }
    }

    #[allow(dead_code)]
    pub fn c2_get_tx_channel(channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        // If bit is set to 1 then interrupt is disabled
        unsafe { !regs.cpu(1).mr().read().chfm(channel as usize) }
    }

    /// clears IPCC receive channel status for CPU1
    pub fn c1_clear_flag_channel(channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(0).scr().write(|w| w.set_chc(channel as usize, true)) }
    }

    #[allow(dead_code)]
    /// clears IPCC receive channel status for CPU2
    pub fn c2_clear_flag_channel(channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(1).scr().write(|w| w.set_chc(channel as usize, true)) }
    }

    pub fn c1_set_flag_channel(channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(0).scr().write(|w| w.set_chs(channel as usize, true)) }
    }

    #[allow(dead_code)]
    pub fn c2_set_flag_channel(channel: IpccChannel) {
        let regs = IPCC::regs();

        unsafe { regs.cpu(1).scr().write(|w| w.set_chs(channel as usize, true)) }
    }

    pub fn c1_is_active_flag(channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        unsafe { regs.cpu(0).sr().read().chf(channel as usize) }
    }

    pub fn c2_is_active_flag(channel: IpccChannel) -> bool {
        let regs = IPCC::regs();

        unsafe { regs.cpu(1).sr().read().chf(channel as usize) }
    }

    pub fn is_tx_pending(channel: IpccChannel) -> bool {
        !Self::c1_is_active_flag(channel) && Self::c1_get_tx_channel(channel)
    }

    pub fn is_rx_pending(channel: IpccChannel) -> bool {
        Self::c2_is_active_flag(channel) && Self::c1_get_rx_channel(channel)
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
    let pwr = crate::pac::PWR;
    let rcc = crate::pac::RCC;

    rcc.cfgr().modify(|w| w.set_stopwuck(true));

    pwr.cr1().modify(|w| w.set_dbp(true));
    pwr.cr1().modify(|w| w.set_dbp(true));

    // configure LSE
    rcc.bdcr().modify(|w| w.set_lseon(true));

    // select system clock source = PLL
    // set PLL coefficients
    // m: 2,
    // n: 12,
    // r: 3,
    // q: 4,
    // p: 3,
    let src_bits = 0b11;
    let pllp = (3 - 1) & 0b11111;
    let pllq = (4 - 1) & 0b111;
    let pllr = (3 - 1) & 0b111;
    let plln = 12 & 0b1111111;
    let pllm = (2 - 1) & 0b111;
    rcc.pllcfgr().modify(|w| {
        w.set_pllsrc(src_bits);
        w.set_pllm(pllm);
        w.set_plln(plln);
        w.set_pllr(pllr);
        w.set_pllp(pllp);
        w.set_pllpen(true);
        w.set_pllq(pllq);
        w.set_pllqen(true);
    });
    // enable PLL
    rcc.cr().modify(|w| w.set_pllon(true));
    rcc.cr().write(|w| w.set_hsion(false));
    // while !rcc.cr().read().pllrdy() {}

    // configure SYSCLK mux to use PLL clocl
    rcc.cfgr().modify(|w| w.set_sw(0b11));

    // configure CPU1 & CPU2 dividers
    rcc.cfgr().modify(|w| w.set_hpre(0)); // not divided
    rcc.extcfgr().modify(|w| {
        w.set_c2hpre(0b1000); // div2
        w.set_shdhpre(0); // not divided
    });

    // apply APB1 / APB2 values
    rcc.cfgr().modify(|w| {
        w.set_ppre1(0b000); // not divided
        w.set_ppre2(0b000); // not divided
    });

    // TODO: required
    // set RF wake-up clock = LSE
    rcc.csr().modify(|w| w.set_rfwkpsel(0b01));

    // set LPTIM1 & LPTIM2 clock source
    rcc.ccipr().modify(|w| {
        w.set_lptim1sel(0b00); // PCLK
        w.set_lptim2sel(0b00); // PCLK
    });
}
