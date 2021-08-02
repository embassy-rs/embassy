
pub mod spi {
    use embassy_stm32::Peripherals;
    use embassy_stm32::spi::{Spi, Config};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::peripherals::{SPI1, DMA2_CH2, DMA2_CH3};

    pub fn new_spi<'a>(p: Peripherals) -> Spi<'a, SPI1, DMA2_CH3, DMA2_CH2>{
        Spi::new(
            p.SPI1,
            p.PB3,
            p.PB5,
            p.PB4,
            p.DMA2_CH3,
            p.DMA2_CH2,
            Hertz(1_000_000),
            Config::default(),
        )
    }

}