#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use core::future::Future;
use core::pin::pin;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_time::{Duration, Timer, with_timeout};
use hal::eim::{Bit, Eim, Fault, MaskError};
use hal::erm::{Async, Erm, Events, Mode, MultiBitInterruptHandler, Report, SingleBitInterruptHandler, SramEvents};
use hal::interrupt::typelevel::{ERM0_MULTI_BIT, ERM0_SINGLE_BIT, Handler, Interrupt};
use hal::{bind_interrupts, pac};
use panic_probe as _;

static SINGLE_IRQS: AtomicU32 = AtomicU32::new(0);
static MULTI_IRQS: AtomicU32 = AtomicU32::new(0);

struct CountSingle;
impl Handler<ERM0_SINGLE_BIT> for CountSingle {
    unsafe fn on_interrupt() {
        SINGLE_IRQS.fetch_add(1, Ordering::Relaxed);
        unsafe { <SingleBitInterruptHandler as Handler<ERM0_SINGLE_BIT>>::on_interrupt() };
    }
}

struct CountMulti;
impl Handler<ERM0_MULTI_BIT> for CountMulti {
    unsafe fn on_interrupt() {
        MULTI_IRQS.fetch_add(1, Ordering::Relaxed);
        unsafe { <MultiBitInterruptHandler as Handler<ERM0_MULTI_BIT>>::on_interrupt() };
    }
}

bind_interrupts!(struct Irqs {
    ERM0_SINGLE_BIT => CountSingle;
    ERM0_MULTI_BIT => CountMulti;
});

// The wrappers forward to the production handlers, so these bindings prove
// exactly the same ISR contract while allowing real IRQ delivery assertions.
unsafe impl hal::interrupt::typelevel::Binding<ERM0_SINGLE_BIT, SingleBitInterruptHandler> for Irqs {}
unsafe impl hal::interrupt::typelevel::Binding<ERM0_MULTI_BIT, MultiBitInterruptHandler> for Irqs {}

unsafe extern "C" {
    static mut __ecc_scratch: u32;
}

#[used]
#[unsafe(link_section = ".ecc_scratch")]
static mut ECC_SCRATCH: [u32; 2] = [0; 2];

const WORDS: [u32; 2] = [0x1234_5678, 0xa5a5_5a5a];
const TIMEOUT: Duration = Duration::from_millis(100);

fn assert_disabled(eim: &Eim<'_>) {
    assert!(!eim.is_enabled());
    assert!(!pac::EIM0.eimcr().read().geien());
    assert!(!pac::EIM0.eichen().read().eich0en());
}

fn assert_notifications_masked() {
    let cr = pac::ERM0.cr0().read();
    assert!(!cr.escie0());
    assert!(!cr.encie0());
    assert!(!cr.encie1());
}

fn assert_descriptor(eim: &Eim<'_>, fault: Fault) {
    assert_disabled(eim);
    assert_eq!(pac::EIM0.eichd0_word0().read().chkbit_mask(), fault.checkbit_mask());
    assert_eq!(pac::EIM0.eichd0_word1().read().b0_3data_mask(), fault.data_mask());
}

fn exercise_descriptors(eim: &mut Eim<'_>) {
    assert_disabled(eim);
    assert_eq!(pac::EIM0.eichd0_word0().read().chkbit_mask(), 0);
    assert_eq!(pac::EIM0.eichd0_word1().read().b0_3data_mask(), 0);
    for index in 0..=u8::MAX {
        assert_eq!(Bit::data(index).is_some(), index < 32);
        assert_eq!(Bit::checkbit(index).is_some(), index < 7);
        if let Some(bit) = Bit::data(index) {
            let fault = Fault::single(bit);
            assert_eq!(fault.data_mask(), 1u32 << index);
            assert_eq!(fault.checkbit_mask(), 0);
            eim.configure(fault);
            assert_descriptor(eim, fault);
            assert_eq!(Fault::double(bit, bit), Err(MaskError::DuplicateBit));
        }
        if let Some(bit) = Bit::checkbit(index) {
            let fault = Fault::single(bit);
            assert_eq!(fault.data_mask(), 0);
            assert_eq!(fault.checkbit_mask(), 1u8 << index);
            eim.configure(fault);
            assert_descriptor(eim, fault);
            assert_eq!(Fault::double(bit, bit), Err(MaskError::DuplicateBit));
        }
    }
    assert_ne!(Bit::data(0), Bit::checkbit(0));
    // Double-bit descriptors are tested ONLY with both enables off.
    for (first, second, data, check) in [
        (Bit::data(0).unwrap(), Bit::data(31).unwrap(), 0x8000_0001, 0),
        (Bit::checkbit(0).unwrap(), Bit::checkbit(6).unwrap(), 0, 0x41),
        (Bit::data(0).unwrap(), Bit::checkbit(0).unwrap(), 1, 1),
    ] {
        let fault = Fault::double(first, second).unwrap();
        assert_eq!(fault.data_mask(), data);
        assert_eq!(fault.checkbit_mask(), check);
        assert_eq!(Fault::double(second, first).unwrap(), fault);
        eim.configure(fault);
        assert_descriptor(eim, fault);
    }
    eim.disable();
    eim.disable();
    assert_disabled(eim);
}

fn pulse(eim: &mut Eim<'_>, index: usize, bit: Bit) {
    let scratch = core::ptr::addr_of!(__ecc_scratch);
    assert!(index < WORDS.len());
    let primask = cortex_m::register::primask::read();
    // SAFETY: two initialized, aligned linker-reserved ECC words; code, vectors,
    // stack and RTT are above SRAM A0. No DMA/other tasks or debugger memory
    // views access SRAM A0. inject_read disarms before any assertions or awaits.
    let actual = unsafe { eim.inject_read(scratch.add(index), bit) };
    assert_eq!(cortex_m::register::primask::read(), primask);
    assert_eq!(actual, WORDS[index]);
    assert_disabled(eim);
    assert_descriptor(eim, Fault::single(bit));
}

fn assert_corrected<M: Mode>(erm: &Erm<'_, M>, index: usize, count: u8) -> Report {
    let status = erm.status();
    assert_eq!(status.sram, Some(SramEvents::Corrected));
    assert!(!status.flash_uncorrectable);
    assert_eq!(status.events(), Some(Events::SRAM_CORRECTED));
    assert!(!status.intersects(Events::SRAM_UNCORRECTABLE.union(Events::FLASH_UNCORRECTABLE)));
    assert_eq!(erm.corrected_count(), count);
    let report = erm.snapshot(Events::ALL).unwrap();
    assert_eq!(report.events(), Events::SRAM_CORRECTED);
    match report {
        Report::Sram {
            events,
            address,
            syndrome,
        } => {
            assert_eq!(events, SramEvents::Corrected);
            assert_eq!(address, 0x2000_0000 + index as u32 * 4);
            assert_eq!(syndrome, pac::ERM0.syn0().read().syndrome());
        }
        _ => panic!("expected a corrected SRAM report"),
    }
    assert_eq!(erm.snapshot(Events::SRAM_CORRECTED), Some(report));
    assert!(
        erm.snapshot(Events::SRAM_UNCORRECTABLE.union(Events::FLASH_UNCORRECTABLE))
            .is_none()
    );
    report
}

fn assert_clean<M: Mode>(erm: &Erm<'_, M>) {
    assert_eq!(erm.status().events(), None);
    assert!(erm.snapshot(Events::ALL).is_none());
}

fn assert_stored_words<M: Mode>(erm: &Erm<'_, M>) {
    let before = erm.snapshot(Events::ALL);
    let count = erm.corrected_count();
    let scratch = core::ptr::addr_of!(__ecc_scratch);
    for (index, expected) in WORDS.into_iter().enumerate() {
        // SAFETY: reserved words initialized by main; injection is off.
        assert_eq!(unsafe { read_volatile(scratch.add(index)) }, expected);
    }
    assert_eq!(erm.snapshot(Events::ALL), before);
    assert_eq!(erm.corrected_count(), count);
}

fn poll_pending<F: Future>(future: core::pin::Pin<&mut F>) {
    let mut cx = Context::from_waker(Waker::noop());
    assert!(matches!(future.poll(&mut cx), Poll::Pending));
}

async fn irq_after(counter: &AtomicU32, before: u32) {
    with_timeout(TIMEOUT, async {
        while counter.load(Ordering::Relaxed) == before {
            Timer::after_micros(100).await;
        }
    })
    .await
    .unwrap();
}

async fn exercise_async(erm: &mut Erm<'_, Async>, eim: &mut Eim<'_>) {
    assert_clean(erm);
    assert_eq!(erm.corrected_count(), 0);
    assert_notifications_masked();

    // A real, software-pended NVIC interrupt is only a wakeup, not an event.
    {
        let mut wait = pin!(erm.wait_report(Events::ALL));
        poll_pending(wait.as_mut());
        let single = SINGLE_IRQS.load(Ordering::Relaxed);
        let multi = MULTI_IRQS.load(Ordering::Relaxed);
        ERM0_SINGLE_BIT::pend();
        ERM0_MULTI_BIT::pend();
        irq_after(&SINGLE_IRQS, single).await;
        irq_after(&MULTI_IRQS, multi).await;
        poll_pending(wait.as_mut());
    }
    assert_notifications_masked();
    assert_clean(erm);
    assert!(
        with_timeout(Duration::from_millis(2), erm.wait_report(Events::ALL))
            .await
            .is_err()
    );
    assert_notifications_masked();

    // An event latched before the first poll is a valid, non-consuming result.
    pulse(eim, 0, Bit::checkbit(6).unwrap());
    let expected = assert_corrected(erm, 0, 1);
    let report = with_timeout(TIMEOUT, erm.wait_report(Events::ALL))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(report, expected);
    assert_notifications_masked();
    assert_eq!(erm.snapshot(Events::ALL), Some(expected));
    erm.acknowledge(Events::SRAM_CORRECTED);

    // Pending wait, real peripheral IRQ, then completion. IRQ count proves
    // this was not solely the future's status recheck.
    {
        let mut wait = pin!(erm.wait_report(Events::SRAM_CORRECTED));
        poll_pending(wait.as_mut());
        let before = SINGLE_IRQS.load(Ordering::Relaxed);
        pulse(eim, 1, Bit::data(31).unwrap());
        irq_after(&SINGLE_IRQS, before).await;
        let report = with_timeout(TIMEOUT, wait).await.unwrap().unwrap();
        match report {
            Report::Sram { events, address, .. } => {
                assert_eq!(events, SramEvents::Corrected);
                assert_eq!(address, 0x2000_0004);
            }
            _ => panic!("expected SRAM report after IRQ"),
        }
    }
    assert_corrected(erm, 1, 2);
    assert_notifications_masked();
    erm.acknowledge(Events::ALL);

    // Cancellation after IRQ delivery but before polling preserves evidence
    // and does not permanently close the shared wait cell.
    {
        let mut wait = pin!(erm.wait_report(Events::SRAM_CORRECTED));
        poll_pending(wait.as_mut());
        let before = SINGLE_IRQS.load(Ordering::Relaxed);
        pulse(eim, 0, Bit::data(7).unwrap());
        irq_after(&SINGLE_IRQS, before).await;
    }
    assert_notifications_masked();
    let expected = assert_corrected(erm, 0, 3);
    assert_eq!(
        with_timeout(TIMEOUT, erm.wait_report(Events::ALL))
            .await
            .unwrap()
            .unwrap(),
        expected
    );
    assert_notifications_masked();

    // A nonmatching latched flag must neither complete nor be consumed.
    {
        let mut wait = pin!(erm.wait_report(Events::FLASH_UNCORRECTABLE));
        poll_pending(wait.as_mut());
    }
    assert_notifications_masked();
    assert_eq!(erm.snapshot(Events::ALL), Some(expected));
    erm.acknowledge(Events::ALL);
    assert_stored_words(erm);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = hal::init(Default::default());
    assert!(
        pac::SYSCON.ram_ctrl().read().rama_ecc_enable(),
        "RAM A ECC must already be enabled"
    );
    let mut eim = Eim::new(p.EIM0.reborrow()).unwrap();
    assert!(pac::MRCC0.mrcc_glb_cc2().read().rama());
    exercise_descriptors(&mut eim);
    let scratch = core::ptr::addr_of_mut!(__ecc_scratch);
    assert_eq!(scratch as usize, 0x2000_0000);
    // SAFETY: dedicated NOLOAD reservation, both enables off; full aligned
    // stores initialize data and its checkbits before any injected read.
    unsafe {
        write_volatile(scratch, WORDS[0]);
        write_volatile(scratch.add(1), WORDS[1]);
    }
    cortex_m::asm::dsb();
    let mut erm = Erm::new_blocking(p.ERM0.reborrow()).unwrap();
    assert_clean(&erm);
    assert_eq!(erm.corrected_count(), 0);
    assert_notifications_masked();

    // Last descriptor above was double-bit; the pulse MUST replace it with a
    // single-bit fault. No double-bit descriptor is ever armed in this test.
    critical_section::with(|_| pulse(&mut eim, 0, Bit::data(0).unwrap()));
    assert_corrected(&erm, 0, 1);
    assert_stored_words(&erm);
    pulse(&mut eim, 1, Bit::checkbit(0).unwrap());
    let report = assert_corrected(&erm, 1, 2);
    assert_stored_words(&erm);

    // Selective acknowledgement and counter reset are independent operations.
    erm.acknowledge(Events::SRAM_UNCORRECTABLE.union(Events::FLASH_UNCORRECTABLE));
    assert_eq!(erm.snapshot(Events::ALL), Some(report));
    erm.reset_corrected_count();
    assert_eq!(erm.corrected_count(), 0);
    assert_eq!(erm.snapshot(Events::ALL), Some(report));
    pulse(&mut eim, 0, Bit::data(1).unwrap());
    assert_corrected(&erm, 0, 1);
    erm.acknowledge(Events::SRAM_CORRECTED);
    assert_clean(&erm);
    assert_eq!(erm.corrected_count(), 1);
    erm.reset_corrected_count();

    for index in 0..260 {
        pulse(&mut eim, index % 2, Bit::data(0).unwrap());
        assert_eq!(erm.corrected_count(), (index + 1).min(255) as u8);
    }
    assert_corrected(&erm, 1, 255);
    assert_stored_words(&erm);
    erm.acknowledge(Events::ALL);
    assert_clean(&erm);
    assert_eq!(erm.corrected_count(), 255);

    // Reconstruction starts a fresh hardware epoch, including flags/count.
    pulse(&mut eim, 0, Bit::data(0).unwrap());
    drop(erm);
    drop(eim);
    let mut eim = Eim::new(p.EIM0.reborrow()).unwrap();
    assert_disabled(&eim);
    assert_eq!(pac::EIM0.eichd0_word0().read().chkbit_mask(), 0);
    assert_eq!(pac::EIM0.eichd0_word1().read().b0_3data_mask(), 0);
    let mut erm = Erm::new_async(p.ERM0.reborrow(), Irqs).unwrap();
    exercise_async(&mut erm, &mut eim).await;
    pulse(&mut eim, 1, Bit::data(0).unwrap());
    drop(erm);
    assert_notifications_masked();
    let mut erm = Erm::new_async(p.ERM0.reborrow(), Irqs).unwrap();
    assert_clean(&erm);
    assert_eq!(erm.corrected_count(), 0);
    {
        let mut wait = pin!(erm.wait_report(Events::ALL));
        poll_pending(wait.as_mut());
    }
    pulse(&mut eim, 1, Bit::data(0).unwrap());
    let expected = assert_corrected(&erm, 1, 1);
    assert_eq!(
        with_timeout(TIMEOUT, erm.wait_report(Events::ALL))
            .await
            .unwrap()
            .unwrap(),
        expected
    );
    assert_notifications_masked();
    erm.acknowledge(Events::ALL);
    erm.reset_corrected_count();
    assert_clean(&erm);
    assert_stored_words(&erm);
    assert_disabled(&eim);
    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}
