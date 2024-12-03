#![no_std]
#![no_main]

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::pubsub::{DynSubscriber, PubSubChannel, Subscriber};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Create the message bus. It has a queue of 4, supports 3 subscribers and 1 publisher
static MESSAGE_BUS: PubSubChannel<ThreadModeRawMutex, Message, 4, 3, 1> = PubSubChannel::new();

#[derive(Clone, defmt::Format)]
enum Message {
    A,
    B,
    C,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _p = embassy_nrf::init(Default::default());
    defmt::info!("Hello World!");

    // It's good to set up the subscribers before publishing anything.
    // A subscriber will only yield messages that have been published after its creation.

    spawner.must_spawn(fast_logger(unwrap!(MESSAGE_BUS.subscriber())));
    spawner.must_spawn(slow_logger(unwrap!(MESSAGE_BUS.dyn_subscriber())));
    spawner.must_spawn(slow_logger_pure(unwrap!(MESSAGE_BUS.dyn_subscriber())));

    // Get a publisher
    let message_publisher = unwrap!(MESSAGE_BUS.publisher());
    // We can't get more (normal) publishers
    // We can have an infinite amount of immediate publishers. They can't await a publish, only do an immediate publish
    defmt::assert!(MESSAGE_BUS.publisher().is_err());

    let mut index = 0;
    loop {
        Timer::after_millis(500).await;

        let message = match index % 3 {
            0 => Message::A,
            1 => Message::B,
            2..=u32::MAX => Message::C,
        };

        // We publish immediately and don't await anything.
        // If the queue is full, it will cause the oldest message to not be received by some/all subscribers
        message_publisher.publish_immediate(message);

        // Try to comment out the last one and uncomment this line below.
        // The behaviour will change:
        // - The subscribers won't miss any messages any more
        // - Trying to publish now has some wait time when the queue is full

        // message_publisher.publish(message).await;

        index += 1;
    }
}

/// A logger task that just awaits the messages it receives
///
/// This takes the generic `Subscriber`. This is most performant, but requires you to write down all of the generics
#[embassy_executor::task]
async fn fast_logger(mut messages: Subscriber<'static, ThreadModeRawMutex, Message, 4, 3, 1>) {
    loop {
        let message = messages.next_message().await;
        defmt::info!("Received message at fast logger: {:?}", message);
    }
}

/// A logger task that awaits the messages, but also does some other work.
/// Because of this, depending on how the messages were published, the subscriber might miss some messages.
///
/// This takes the dynamic `DynSubscriber`. This is not as performant as the generic version, but let's you ignore some of the generics.
#[embassy_executor::task]
async fn slow_logger(mut messages: DynSubscriber<'static, Message>) {
    loop {
        // Do some work
        Timer::after_millis(2000).await;

        // If the publisher has used the `publish_immediate` function, then we may receive a lag message here
        let message = messages.next_message().await;
        defmt::info!("Received message at slow logger: {:?}", message);

        // If the previous one was a lag message, then we should receive the next message here immediately
        let message = messages.next_message().await;
        defmt::info!("Received message at slow logger: {:?}", message);
    }
}

/// Same as `slow_logger` but it ignores lag results
#[embassy_executor::task]
async fn slow_logger_pure(mut messages: DynSubscriber<'static, Message>) {
    loop {
        // Do some work
        Timer::after_millis(2000).await;

        // Instead of receiving lags here, we just ignore that and read the next message
        let message = messages.next_message_pure().await;
        defmt::info!("Received message at slow logger pure: {:?}", message);
    }
}
