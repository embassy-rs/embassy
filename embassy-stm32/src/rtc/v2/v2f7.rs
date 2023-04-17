use stm32_metapac::rcc::vals::Rtcsel;

pub const BACKUP_REGISTER_COUNT: usize = 20;

/// Unlock the backup domain
pub(super) unsafe fn unlock_backup_domain(clock_config: u8) {
    crate::pac::PWR.cr1().modify(|w| w.set_dbp(true));
    while !crate::pac::PWR.cr1().read().dbp() {}

    let reg = crate::pac::RCC.bdcr().read();
    assert!(!reg.lsecsson(), "RTC is not compatible with LSE CSS, yet.");

    if !reg.rtcen() || reg.rtcsel().0 != clock_config {
        crate::pac::RCC.bdcr().modify(|w| w.set_bdrst(true));

        crate::pac::RCC.bdcr().modify(|w| {
            // Reset
            w.set_bdrst(false);

            // Select RTC source
            w.set_rtcsel(Rtcsel(clock_config));
            w.set_rtcen(true);

            // Restore bcdr
            w.set_lscosel(reg.lscosel());
            w.set_lscoen(reg.lscoen());

            w.set_lseon(reg.lseon());
            w.set_lsedrv(reg.lsedrv());
            w.set_lsebyp(reg.lsebyp());
        });
    }
}

pub(crate) unsafe fn enable_peripheral_clk() {
    // enable peripheral clock for communication
    crate::pac::RCC.apb1enr1().modify(|w| w.set_rtcapben(true));

    // read to allow the pwr clock to enable
    crate::pac::PWR.cr1().read();
}
