#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_halt as _};
use nxp_pac::*;

fn init() {
    info!("Init");
    // SPI 7 at FLEXCOMM 7 Setup (spi instance is the same as the flexcomm instance)
    // Enable iocon and flexcomm
    SYSCON.ahbclkctrl0().modify(|w| {
        w.set_iocon(true);
    } );
    SYSCON.ahbclkctrl1().modify(|w| {
        w.set_fc(7, true);
    } );

    // Reset Flexcomm 7
    SYSCON.presetctrl1().modify(|w| {
        w.set_fc_rst(7, syscon::vals::FcRst::ASSERTED);
    });
    SYSCON.presetctrl1().modify(|w| {
        w.set_fc_rst(7, syscon::vals::FcRst::RELEASED);
    });

    // CLK SEL
    // Select Main Clock (ENUM_0X2 is FRO 12Mhz)
    SYSCON.fcclksel(7).modify(|w|
        w.set_sel(syscon::vals::FcclkselSel::ENUM_0X2)
    );
    // Set flexcomm to SPI
    FLEXCOMM7.pselid().modify(|w|{
        w.set_persel(flexcomm::vals::Persel::SPI);
    });
    
    // IOCON Setup
    // All pins are configured according to the standard settings in the SPI config documentation
    // SSEL1 at func 1 iocon
    IOCON.pio1(20).modify(|w|{
        w.set_func(iocon::vals::PioFunc::ALT1);
        w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
        w.set_slew(iocon::vals::PioSlew::STANDARD);
        w.set_mode(iocon::vals::PioMode::INACTIVE);
        w.set_invert(false);
        w.set_od(iocon::vals::PioOd::NORMAL);
    });
    // MOSI at func 7 iocon
    IOCON.pio0(20).modify(|w|{
        w.set_func(iocon::vals::PioFunc::ALT7);
        w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
        w.set_slew(iocon::vals::PioSlew::STANDARD);
        w.set_mode(iocon::vals::PioMode::INACTIVE);
        w.set_invert(false);
        w.set_od(iocon::vals::PioOd::NORMAL);
    });
    // MISO at func 7 iocon
    IOCON.pio0(19).modify(|w|{
        w.set_func(iocon::vals::PioFunc::ALT7);
        w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
        w.set_slew(iocon::vals::PioSlew::STANDARD);
        w.set_mode(iocon::vals::PioMode::INACTIVE);
        w.set_invert(false);
        w.set_od(iocon::vals::PioOd::NORMAL);
    });
    // SCKL at func 7 iocon
    IOCON.pio0(21).modify(|w|{
        w.set_func(iocon::vals::PioFunc::ALT7);
        w.set_digimode(iocon::vals::PioDigimode::DIGITAL);
        w.set_slew(iocon::vals::PioSlew::STANDARD);
        w.set_mode(iocon::vals::PioMode::INACTIVE);
        w.set_invert(false);
        w.set_od(iocon::vals::PioOd::NORMAL);
    });

    // FlexcommFRG divider (div is 0xFF by default in the documentation meaning it can be divided by either 1 or 2)
    SYSCON.flexfrgctrl(7).modify(|w|{
        w.set_div(0xFF);
        w.set_mult(0);
    });
    // SPI clock divider can go from 1 to 255
    SPI7.div().modify(|w|{
        w.set_divval(0);
    });
    // Final Clock is 12 MHz

    // SPI Master config using ssel_1
    SPI7.cfg().modify(|w|{
        w.set_enable(true);
        w.set_master(spi::vals::Master::MASTER_MODE);
        w.set_lsbf(spi::vals::Lsbf::STANDARD);
        w.set_cpha(spi::vals::Cpha::CHANGE);
        w.set_cpol(spi::vals::Cpol::LOW);
        w.set_loop_(false);
        w.set_spol1(spi::vals::Spol1::LOW);
    });
    // FIFO Config disabling DMA and enabling TX and RX
    SPI7.fifocfg().modify(|w| {
        w.set_dmatx(false);
        w.set_dmarx(false);
        w.set_enabletx(true);
        w.set_enablerx(true);
        //w.set_emptytx(true);
        //w.set_emptyrx(true);
    });
    // Disabling RX is recommended in the documentation if you are not expecting to receive data
    SPI7.fifowr().write(|w|{
        w.set_rxignore(spi::vals::Rxignore::IGNORE);
    });

    loop {
        SPI7.fifowr().write(|w|w.set_txssel1_n(spi::vals::Txssel1N::ASSERTED)); // Assert the SSEL you are transferring data to
        for _ in 0..100_000 {
            nop();}
        SPI7.fifowr().write(|w|{
            w.set_txdata(0x04); // Data to be transfered
            w.set_len(8);   // !! IMPORTANT !! If length isn't specified data won't be shifted out
        });
        for _ in 0..100_000 {
            nop();}
        let fifostat = SPI7.fifostat().read();
        info!("Tx full? {}", !fifostat.txnotfull());
        info!("Tx level: {}", fifostat.txlvl());
        info!("Tx empty? {}", fifostat.txempty());
        SPI7.fifowr().write(|w|{
            w.set_eot(true); // Mark end of transfer
            w.set_txssel1_n(spi::vals::Txssel1N::NOT_ASSERTED); // Deassert our SSEL
        });
        for _ in 0..100_000 {
            nop();}
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    init();
}