#![no_std]
#![no_main]
#![feature(lang_items)]

use core::{
    panic::PanicInfo,
    sync::atomic::{compiler_fence, Ordering},
};

use cortex_m::singleton;
use rtic::app;
// use rtt_target::{rprintln, rtt_init_print};
use stm32f4xx_hal::{
    dma::{config::DmaConfig, Channel0, PeripheralToMemory, Stream0, StreamsTuple, Transfer},
    pac::{ADC1, DMA2, RCC},
    prelude::*,
};

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

type DmaTransfer =
    Transfer<Stream0<DMA2>, Channel0, ADC1, PeripheralToMemory, &'static mut [u16; 128]>;

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        transfer: DmaTransfer,
        triple_buffer: Option<&'static mut [u16; 128]>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let rcc = cx.device.RCC.constrain();

        // rtt_init_print!();
        // rprintln!("Init");

        let _clocks = rcc
            .cfgr
            .sysclk(84.mhz())
            .pclk2(28.mhz())
            .pclk1(28.mhz())
            .freeze();

        let gpioa = cx.device.GPIOA.split();
        let _pa0 = gpioa.pa0.into_analog();

        let stream_0 = StreamsTuple::new(cx.device.DMA2).0;
        let config = DmaConfig::default()
            .transfer_complete_interrupt(true)
            .memory_increment(true)
            .double_buffer(true);

        let rcc = unsafe { &*RCC::ptr() };
        rcc.apb2enr.modify(|_, w| w.adc1en().enabled());
        rcc.apb2rstr.modify(|_, w| w.adcrst().set_bit());
        rcc.apb2rstr.modify(|_, w| w.adcrst().clear_bit());
        let adc = cx.device.ADC1;
        adc.cr2.modify(|_, w| {
            w.dma()
                .enabled()
                .cont()
                .continuous()
                .dds()
                .continuous()
                .adon()
                .enabled()
        });

        let first_buffer = singleton!(: [u16; 128] = [0; 128]).unwrap();
        let second_buffer = singleton!(: [u16; 128] = [0; 128]).unwrap();
        let triple_buffer = Some(singleton!(: [u16; 128] = [0; 128]).unwrap());

        let transfer = Transfer::init(stream_0, adc, first_buffer, Some(second_buffer), config);

        // rprintln!("Finished init");
        init::LateResources {
            transfer,
            triple_buffer,
        }
    }

    #[idle(resources = [transfer])]
    fn idle(mut cx: idle::Context) -> ! {
        cx.resources.transfer.lock(|shared| {
            shared.start(|adc| {
                adc.cr2.modify(|_, w| w.swstart().start());
            });
        });
        // rprintln!("DMA started");
        loop {
            compiler_fence(Ordering::SeqCst);
        }
    }

    #[task(binds = DMA2_STREAM0, priority = 2, resources = [transfer, triple_buffer])]
    fn dma(cx: dma::Context) {
        static mut COUNT: usize = 0;

        let triple = cx.resources.triple_buffer.take().unwrap();
        let buf = cx
            .resources
            .transfer
            .next_transfer(triple)
            .map_err(|_| {})
            .unwrap()
            .0;
        if *COUNT % (1 << 14) == 0 {
            // rprintln!("Buf: {:?}", &buf[0..10]);
        }
        *COUNT += 1;
        *cx.resources.triple_buffer = Some(buf);
    }
};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    // rprintln!("{}", info);
    loop {
        compiler_fence(Ordering::SeqCst);
    }
}
