#![no_std]
#![no_main]

use embassy_executor::Spawner;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Don't do anything, just make sure it compiles.
    loop {}
}
