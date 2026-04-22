use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pubsub::{PubSubChannel, SendDynPublisher};

// NoopRawMutex is not Sync, so converting a Publisher backed by it into a
// SendDynPublisher must fail to compile.
fn main() {
    let channel = PubSubChannel::<NoopRawMutex, u32, 4, 4, 4>::new();
    let _pubb: SendDynPublisher<'_, u32> = channel.publisher().unwrap().into();
}
