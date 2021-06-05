use core::sync::atomic::{AtomicU8, Ordering};
use core::task::Poll;
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::AtomicWaker;
use futures::future::poll_fn;

use super::*;
use crate::fmt::assert;
use crate::interrupt;
use crate::pac;
use crate::pac::dma::{regs, vals};

const DMAS: [pac::dma::Dma; 2] = [pac::DMA1, pac::DMA2];

const CH_COUNT: usize = 16;
const CH_STATUS_NONE: u8 = 0;
const CH_STATUS_COMPLETED: u8 = 1;
const CH_STATUS_ERROR: u8 = 2;

struct State {
    ch_wakers: [AtomicWaker; CH_COUNT],
    ch_status: [AtomicU8; CH_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        const AU: AtomicU8 = AtomicU8::new(CH_STATUS_NONE);
        Self {
            ch_wakers: [AW; CH_COUNT],
            ch_status: [AU; CH_COUNT],
        }
    }
}

static STATE: State = State::new();

#[allow(unused)] // Used by usart/v1.rs which may or may not be enabled
pub(crate) async unsafe fn transfer_m2p(
    ch: &mut impl Channel,
    ch_func: u8,
    src: &[u8],
    dst: *mut u8,
) {
    let n = ch.num() as usize;
    let r = ch.regs();
    let c = r.st(ch.ch_num() as _);

    // ndtr is max 16 bits.
    assert!(src.len() <= 0xFFFF);

    // Reset status
    STATE.ch_status[n].store(CH_STATUS_NONE, Ordering::Relaxed);

    unsafe {
        c.par().write_value(dst as _);
        c.m0ar().write_value(src.as_ptr() as _);
        c.ndtr().write_value(regs::Ndtr(src.len() as _));
        c.cr().write(|w| {
            w.set_dir(vals::Dir::MEMORYTOPERIPHERAL);
            w.set_msize(vals::Size::BITS8);
            w.set_psize(vals::Size::BITS8);
            w.set_minc(vals::Inc::INCREMENTED);
            w.set_pinc(vals::Inc::FIXED);
            w.set_chsel(ch_func);
            w.set_teie(true);
            w.set_tcie(true);
            w.set_en(true);
        });
    }

    let res = poll_fn(|cx| {
        STATE.ch_wakers[n].register(cx.waker());
        match STATE.ch_status[n].load(Ordering::Relaxed) {
            CH_STATUS_NONE => Poll::Pending,
            x => Poll::Ready(x),
        }
    })
    .await;

    // TODO handle error
    assert!(res == CH_STATUS_COMPLETED);
}

unsafe fn on_irq() {
    for (dman, &dma) in DMAS.iter().enumerate() {
        for isrn in 0..2 {
            let isr = dma.isr(isrn).read();
            dma.ifcr(isrn).write_value(isr);

            for chn in 0..4 {
                let n = dman * 8 + isrn * 4 + chn;
                if isr.teif(chn) {
                    STATE.ch_status[n].store(CH_STATUS_ERROR, Ordering::Relaxed);
                    STATE.ch_wakers[n].wake();
                } else if isr.tcif(chn) {
                    STATE.ch_status[n].store(CH_STATUS_COMPLETED, Ordering::Relaxed);
                    STATE.ch_wakers[n].wake();
                }
            }
        }
    }
}

#[interrupt]
unsafe fn DMA1_STREAM0() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM1() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM2() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM3() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM4() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM5() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM6() {
    on_irq()
}
#[interrupt]
unsafe fn DMA1_STREAM7() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM0() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM1() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM2() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM3() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM4() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM5() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM6() {
    on_irq()
}
#[interrupt]
unsafe fn DMA2_STREAM7() {
    on_irq()
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    interrupt::DMA1_STREAM0::steal().enable();
    interrupt::DMA1_STREAM1::steal().enable();
    interrupt::DMA1_STREAM2::steal().enable();
    interrupt::DMA1_STREAM3::steal().enable();
    interrupt::DMA1_STREAM4::steal().enable();
    interrupt::DMA1_STREAM5::steal().enable();
    interrupt::DMA1_STREAM6::steal().enable();
    interrupt::DMA1_STREAM7::steal().enable();
    interrupt::DMA2_STREAM0::steal().enable();
    interrupt::DMA2_STREAM1::steal().enable();
    interrupt::DMA2_STREAM2::steal().enable();
    interrupt::DMA2_STREAM3::steal().enable();
    interrupt::DMA2_STREAM4::steal().enable();
    interrupt::DMA2_STREAM5::steal().enable();
    interrupt::DMA2_STREAM6::steal().enable();
    interrupt::DMA2_STREAM7::steal().enable();
}
