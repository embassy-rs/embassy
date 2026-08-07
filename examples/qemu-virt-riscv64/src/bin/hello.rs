#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use semihosting::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);
    semihosting::process::exit(1);
}

static DONE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
async fn say_hello_from_task() {
    println!("Hello from a spawned task!");
    DONE.signal(());
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    println!("Hello from embassy-executor on RISC-V 64!");
    spawner.spawn(say_hello_from_task().unwrap());
    // Wait for the spawned task to actually run; this also exercises
    // cross-task wakeups through the executor's pender.
    DONE.wait().await;
    println!("Goodbye.");
    semihosting::process::exit(0);
}
