//! This example shows some common strategies for sharing resources between tasks.
//!
//! We demonstrate five different ways of sharing, covering different use cases:
//! - Atomics: This method is used for simple values, such as bool and u8..u32
//! - Blocking Mutex: This is used for sharing non-async things, using Cell/RefCell for interior mutability.
//! - Async Mutex: This is used for sharing async resources, where you need to hold the lock across await points.
//!   The async Mutex has interior mutability built-in, so no RefCell is needed.
//! - Cell: For sharing Copy types between tasks running on the same executor.
//! - RefCell: When you want &mut access to a value shared between tasks running on the same executor.
//!
//! More information: https://embassy.dev/book/#_sharing_peripherals_between_tasks

#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use defmt::info;
use embassy_executor::{Executor, InterruptExecutor};
use embassy_rp::clocks::RoscRng;
use embassy_rp::interrupt::{InterruptExt, Priority};
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{self, InterruptHandler, UartTx};
use embassy_rp::{bind_interrupts, interrupt};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::{blocking_mutex, mutex};
use embassy_time::{Duration, Ticker};
use static_cell::{ConstStaticCell, StaticCell};
use {defmt_rtt as _, panic_probe as _};

type UartAsyncMutex = mutex::Mutex<CriticalSectionRawMutex, UartTx<'static, uart::Async>>;

struct MyType {
    inner: u32,
}

static EXECUTOR_HI: InterruptExecutor = InterruptExecutor::new();
static EXECUTOR_LOW: StaticCell<Executor> = StaticCell::new();

// Use Atomics for simple values
static ATOMIC: AtomicU32 = AtomicU32::new(0);

// Use blocking Mutex with Cell/RefCell for sharing non-async things
static MUTEX_BLOCKING: blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<MyType>> =
    blocking_mutex::Mutex::new(RefCell::new(MyType { inner: 0 }));

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
});

#[interrupt]
unsafe fn SWI_IRQ_0() {
    unsafe { EXECUTOR_HI.on_interrupt() }
}

#[entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    info!("Here we go!");

    let uart = UartTx::new(p.UART0, p.PIN_0, p.DMA_CH0, uart::Config::default());
    // Use the async Mutex for sharing async things (built-in interior mutability)
    static UART: StaticCell<UartAsyncMutex> = StaticCell::new();
    let uart = UART.init(mutex::Mutex::new(uart));

    // High-priority executor: runs in interrupt mode
    interrupt::SWI_IRQ_0.set_priority(Priority::P3);
    let spawner = EXECUTOR_HI.start(interrupt::SWI_IRQ_0);
    spawner.spawn(task_a(uart).unwrap());

    // Low priority executor: runs in thread mode
    let executor = EXECUTOR_LOW.init(Executor::new());
    executor.run(|spawner| {
        // No Mutex needed when sharing between tasks running on the same executor

        // Use Cell for Copy-types
        static CELL: ConstStaticCell<Cell<[u8; 4]>> = ConstStaticCell::new(Cell::new([0; 4]));
        let cell = CELL.take();

        // Use RefCell for &mut access
        static REF_CELL: ConstStaticCell<RefCell<MyType>> = ConstStaticCell::new(RefCell::new(MyType { inner: 0 }));
        let ref_cell = REF_CELL.take();

        spawner.spawn(task_b(uart, cell, ref_cell).unwrap());
        spawner.spawn(task_c(cell, ref_cell).unwrap());
    });
}

#[embassy_executor::task]
async fn task_a(uart: &'static UartAsyncMutex) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        let random = RoscRng.next_u32();

        {
            let mut uart = uart.lock().await;
            uart.write(b"task a").await.unwrap();
            // The uart lock is released when it goes out of scope
        }

        ATOMIC.store(random, Ordering::Relaxed);

        MUTEX_BLOCKING.lock(|x| x.borrow_mut().inner = random);

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn task_b(uart: &'static UartAsyncMutex, cell: &'static Cell<[u8; 4]>, ref_cell: &'static RefCell<MyType>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        let random = RoscRng.next_u32();

        uart.lock().await.write(b"task b").await.unwrap();

        cell.set(random.to_be_bytes());

        ref_cell.borrow_mut().inner = random;

        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn task_c(cell: &'static Cell<[u8; 4]>, ref_cell: &'static RefCell<MyType>) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        info!("=======================");

        let atomic_val = ATOMIC.load(Ordering::Relaxed);
        info!("atomic: {}", atomic_val);

        MUTEX_BLOCKING.lock(|x| {
            let val = x.borrow().inner;
            info!("blocking mutex: {}", val);
        });

        let cell_val = cell.get();
        info!("cell: {:?}", cell_val);

        let ref_cell_val = ref_cell.borrow().inner;
        info!("ref_cell: {:?}", ref_cell_val);

        ticker.next().await;
    }
}
