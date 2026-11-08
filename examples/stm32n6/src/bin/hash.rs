#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::hash::*;
use embassy_stm32::{Config, bind_interrupts, hash, peripherals};
use embassy_time::Instant;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    HASH => hash::InterruptHandler<peripherals::HASH>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // DK uses external SMPS (UM3300 Tab.6); embassy default = internal SMPS hangs init() at VOSRDY.
    let mut config = Config::default();
    config.rcc.supply_config = embassy_stm32::rcc::SupplyConfig::External;
    let p = embassy_stm32::init(config);

    let test_1: &[u8] = b"as;dfhaslfhas;oifvnasd;nifvnhasd;nifvhndlkfghsd;nvfnahssdfgsdafgsasdfasdfasdfasdfasdfghjklmnbvcalskdjghalskdjgfbaslkdjfgbalskdjgbalskdjbdfhsdfhsfghsfghfgh";
    let test_2: &[u8] = b"fdhalksdjfhlasdjkfhalskdjfhgal;skdjfgalskdhfjgalskdjfglafgadfgdfgdafgaadsfgfgdfgadrgsyfthxfgjfhklhjkfgukhulkvhlvhukgfhfsrghzdhxyfufynufyuszeradrtydyytserr";

    let mut hw_hasher = Hash::new_blocking(p.HASH, Irqs);

    let hw_start_time = Instant::now();

    // Compute a digest in hardware.
    let mut context = hw_hasher.start::<Sha256, NonHmac>(DataType::Width8, None);
    hw_hasher.update_blocking(&mut context, test_1);
    hw_hasher.update_blocking(&mut context, test_2);
    let mut hw_digest: [u8; 32] = [0; 32];
    hw_hasher.finish_blocking(context, &mut hw_digest);

    let hw_end_time = Instant::now();
    let hw_execution_time = hw_end_time - hw_start_time;

    info!("Hardware Digest: {:?}", hw_digest);
    info!("Hardware Execution Time: {:?}", hw_execution_time);

    let hmac_key: [u8; 64] = [0x55; 64];

    // Compute HMAC in hardware.
    let mut sha256hmac_context = hw_hasher.start::<Sha256, Hmac>(DataType::Width8, Some(&hmac_key));
    hw_hasher.update_blocking(&mut sha256hmac_context, test_1);
    hw_hasher.update_blocking(&mut sha256hmac_context, test_2);
    let mut hw_hmac: [u8; 32] = [0; 32];
    hw_hasher.finish_blocking(sha256hmac_context, &mut hw_hmac);

    info!("Hardware HMAC: {:?}", hw_hmac);

    loop {}
}
