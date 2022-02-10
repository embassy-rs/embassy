#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::dcmi::{self, *};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::interrupt;
use embassy_stm32::rcc::{Mco, Mco1Source, McoClock};
use embassy_stm32::time::U32Ext;
use embassy_stm32::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

use core::sync::atomic::{AtomicUsize, Ordering};
use embassy_stm32::Config;

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

#[allow(unused)]
pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(400.mhz().into());
    config.rcc.hclk = Some(400.mhz().into());
    config.rcc.pll1.q_ck = Some(100.mhz().into());
    config.rcc.pclk1 = Some(100.mhz().into());
    config.rcc.pclk2 = Some(100.mhz().into());
    config.rcc.pclk3 = Some(100.mhz().into());
    config.rcc.pclk4 = Some(100.mhz().into());
    config
}

use ov7725::*;

const WIDTH: usize = 100;
const HEIGHT: usize = 100;

static mut FRAME: [u32; WIDTH * HEIGHT / 2] = [0u32; WIDTH * HEIGHT / 2];

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    defmt::info!("Hello World!");
    let mco = Mco::new(p.MCO1, p.PA8, Mco1Source::Hsi, McoClock::Divided(3));

    let mut led = Output::new(p.PE3, Level::High, Speed::Low);
    let i2c_irq = interrupt::take!(I2C1_EV);
    let cam_i2c = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        i2c_irq,
        p.DMA1_CH1,
        p.DMA1_CH2,
        100u32.khz(),
    );

    let mut camera = Ov7725::new(cam_i2c, mco);

    defmt::unwrap!(camera.init().await);

    let manufacturer_id = defmt::unwrap!(camera.read_manufacturer_id().await);
    let camera_id = defmt::unwrap!(camera.read_product_id().await);

    defmt::info!(
        "manufacturer: 0x{:x}, pid: 0x{:x}",
        manufacturer_id,
        camera_id
    );

    let dcmi_irq = interrupt::take!(DCMI);
    let config = dcmi::Config::default();
    let mut dcmi = Dcmi::new_8bit(
        p.DCMI, p.DMA1_CH0, dcmi_irq, p.PC6, p.PC7, p.PE0, p.PE1, p.PE4, p.PD3, p.PE5, p.PE6,
        p.PB7, p.PA4, p.PA6, config,
    );

    defmt::info!("attempting capture");
    defmt::unwrap!(dcmi.capture(unsafe { &mut FRAME }).await);

    defmt::info!("captured frame: {:x}", unsafe { &FRAME });

    defmt::info!("main loop running");
    loop {
        defmt::info!("high");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;

        defmt::info!("low");
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}

mod ov7725 {
    use core::marker::PhantomData;

    use defmt::Format;
    use embassy::time::{Duration, Timer};
    use embassy_stm32::rcc::{Mco, McoInstance};
    use embedded_hal_async::i2c::I2c;

    #[repr(u8)]
    pub enum RgbFormat {
        Gbr422 = 0,
        RGB565 = 1,
        RGB555 = 2,
        RGB444 = 3,
    }
    pub enum PixelFormat {
        Yuv,
        ProcessedRawBayer,
        Rgb(RgbFormat),
        RawBayer,
    }

    impl From<PixelFormat> for u8 {
        fn from(raw: PixelFormat) -> Self {
            match raw {
                PixelFormat::Yuv => 0,
                PixelFormat::ProcessedRawBayer => 1,
                PixelFormat::Rgb(mode) => 2 | ((mode as u8) << 2),
                PixelFormat::RawBayer => 3,
            }
        }
    }

    #[derive(Clone, Copy)]
    #[repr(u8)]
    #[allow(unused)]
    pub enum Register {
        Gain = 0x00,
        Blue = 0x01,
        Red = 0x02,
        Green = 0x03,
        BAvg = 0x05,
        GAvg = 0x06,
        RAvg = 0x07,
        Aech = 0x08,
        Com2 = 0x09,
        PId = 0x0a,
        Ver = 0x0b,
        Com3 = 0x0c,
        Com4 = 0x0d,
        Com5 = 0x0e,
        Com6 = 0x0f,
        Aec = 0x10,
        ClkRc = 0x11,
        Com7 = 0x12,
        Com8 = 0x13,
        Com9 = 0x14,
        Com10 = 0x15,
        Reg16 = 0x16,
        HStart = 0x17,
        HSize = 0x18,
        VStart = 0x19,
        VSize = 0x1a,
        PShift = 0x1b,
        MidH = 0x1c,
        MidL = 0x1d,
        Laec = 0x1f,
        Com11 = 0x20,
        BdBase = 0x22,
        BdMStep = 0x23,
        Aew = 0x24,
        Aeb = 0x25,
        Vpt = 0x26,
        Reg28 = 0x28,
        HOutSize = 0x29,
        EXHCH = 0x2a,
        EXHCL = 0x2b,
        VOutSize = 0x2c,
        Advfl = 0x2d,
        Advfh = 0x2e,
        Yave = 0x2f,
        LumHTh = 0x30,
        LumLTh = 0x31,
        HRef = 0x32,
        DspCtrl4 = 0x67,
        DspAuto = 0xac,
    }

    const CAM_ADDR: u8 = 0x21;

    #[derive(Format)]
    pub enum Error<I2cError: Format> {
        I2c(I2cError),
    }

    pub struct Ov7725<'d, Bus: I2c> {
        phantom: PhantomData<&'d ()>,
        bus: Bus,
    }

    impl<'d, Bus> Ov7725<'d, Bus>
    where
        Bus: I2c,
        Bus::Error: Format,
    {
        pub fn new<T>(bus: Bus, _mco: Mco<T>) -> Self
        where
            T: McoInstance,
        {
            Self {
                phantom: PhantomData,
                bus,
            }
        }

        pub async fn init(&mut self) -> Result<(), Error<Bus::Error>> {
            Timer::after(Duration::from_millis(500)).await;
            self.reset_regs().await?;
            Timer::after(Duration::from_millis(500)).await;
            self.set_pixformat().await?;
            self.set_resolution().await?;
            Ok(())
        }

        pub async fn read_manufacturer_id(&mut self) -> Result<u16, Error<Bus::Error>> {
            Ok(u16::from_le_bytes([
                self.read(Register::MidL).await?,
                self.read(Register::MidH).await?,
            ]))
        }

        pub async fn read_product_id(&mut self) -> Result<u16, Error<Bus::Error>> {
            Ok(u16::from_le_bytes([
                self.read(Register::Ver).await?,
                self.read(Register::PId).await?,
            ]))
        }

        async fn reset_regs(&mut self) -> Result<(), Error<Bus::Error>> {
            self.write(Register::Com7, 0x80).await
        }

        async fn set_pixformat(&mut self) -> Result<(), Error<Bus::Error>> {
            self.write(Register::DspCtrl4, 0).await?;
            let mut com7 = self.read(Register::Com7).await?;
            com7 |= u8::from(PixelFormat::Rgb(RgbFormat::RGB565));
            self.write(Register::Com7, com7).await?;
            Ok(())
        }

        async fn set_resolution(&mut self) -> Result<(), Error<Bus::Error>> {
            let horizontal: u16 = super::WIDTH as u16;
            let vertical: u16 = super::HEIGHT as u16;

            let h_high = (horizontal >> 2) as u8;
            let v_high = (vertical >> 1) as u8;
            let h_low = (horizontal & 0x03) as u8;
            let v_low = (vertical & 0x01) as u8;

            self.write(Register::HOutSize, h_high).await?;
            self.write(Register::VOutSize, v_high).await?;
            self.write(Register::EXHCH, h_low | (v_low << 2)).await?;

            self.write(Register::Com3, 0xd1).await?;

            let com3 = self.read(Register::Com3).await?;
            let vflip = com3 & 0x80 > 0;

            self.modify(Register::HRef, |reg| {
                reg & 0xbf | if vflip { 0x40 } else { 0x40 }
            })
            .await?;

            if horizontal <= 320 || vertical <= 240 {
                self.write(Register::HStart, 0x3f).await?;
                self.write(Register::HSize, 0x50).await?;
                self.write(Register::VStart, 0x02).await?; // TODO vflip is subtracted in the original code
                self.write(Register::VSize, 0x78).await?;
            } else {
                defmt::panic!("VGA resolutions not yet supported.");
            }

            Ok(())
        }

        async fn read(&mut self, register: Register) -> Result<u8, Error<Bus::Error>> {
            let mut buffer = [0u8; 1];
            self.bus
                .write_read(CAM_ADDR, &[register as u8], &mut buffer[..1])
                .await
                .map_err(Error::I2c)?;
            Ok(buffer[0])
        }

        async fn write(&mut self, register: Register, value: u8) -> Result<(), Error<Bus::Error>> {
            self.bus
                .write(CAM_ADDR, &[register as u8, value])
                .await
                .map_err(Error::I2c)
        }

        async fn modify<F: FnOnce(u8) -> u8>(
            &mut self,
            register: Register,
            f: F,
        ) -> Result<(), Error<Bus::Error>> {
            let value = self.read(register).await?;
            let value = f(value);
            self.write(register, value).await
        }
    }
}
