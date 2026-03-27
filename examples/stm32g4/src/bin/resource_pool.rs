#![no_std]
#![no_main]

use core::fmt::Write;

use defmt::info;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::resource_pool::{MappedResourceGuard, ResourceGuard, ResourcePool};
use embassy_time::Timer;
use heapless::String;
use static_cell::{ConstStaticCell, StaticCell};
use {defmt_rtt as _, panic_probe as _};

const N_BUFFERS: usize = 3;
const N_BYTES: usize = 256;

static BUFFERS: ConstStaticCell<[String<N_BYTES>; N_BUFFERS]> =
    ConstStaticCell::new([String::new(), String::new(), String::new()]);

static SHARED_CHANNEL: Channel<
    ThreadModeRawMutex,
    MappedResourceGuard<'static, 'static, ThreadModeRawMutex, String<N_BYTES>, str, N_BUFFERS>,
    8,
> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());

    static POOL: StaticCell<ResourcePool<'static, ThreadModeRawMutex, String<N_BYTES>, N_BUFFERS>> = StaticCell::new();
    let pool = POOL.init(ResourcePool::new(BUFFERS.take()));

    spawner.spawn(produce_data(pool, 0).unwrap());
    Timer::after_millis(100).await;
    spawner.spawn(produce_data(pool, 1).unwrap());
    Timer::after_millis(100).await;
    spawner.spawn(produce_data(pool, 2).unwrap());
    Timer::after_millis(100).await;
    spawner.spawn(produce_data(pool, 3).unwrap());

    info!("started producers");

    let receiver = SHARED_CHANNEL.receiver();

    loop {
        let guard = receiver.receive().await;

        defmt::info!("received: {} at addr {}", &*guard, guard.as_ptr() as usize);

        // keep buffer for a while so it is not immediately returned to the pool
        Timer::after_millis(1500).await;

        // extra verbose, this happens automatically
        // core::mem::drop(guard);
    }
}

#[embassy_executor::task(pool_size = 4)]
async fn produce_data(pool: &'static ResourcePool<'static, ThreadModeRawMutex, String<N_BYTES>, N_BUFFERS>, num: u32) {
    let sender = SHARED_CHANNEL.sender();

    let mut n = 0;
    loop {
        Timer::after_secs(3).await;

        // acquire one buffer
        let mut guard = pool.take().await;

        // write to buffer
        guard.clear();
        write!(&mut *guard, "hello {} from task {}", n, num).unwrap();

        // map
        let guard = ResourceGuard::map(guard, |g| g.as_mut_str());

        let addr = guard.as_ptr() as usize;

        // send buffer to main loop
        sender.try_send(guard).ok().unwrap();

        info!("task {} sent buffer with addr {}", num, addr);

        n += 1;
    }
}
