use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pubsub::{PubSubChannel, SendDynImmediatePublisher};

// NoopRawMutex is not Sync, so converting an ImmediatePublisher backed by it
// into a SendDynImmediatePublisher must fail to compile.
fn main() {
    let channel = PubSubChannel::<NoopRawMutex, u32, 4, 4, 4>::new();
    let _imm: SendDynImmediatePublisher<'_, u32> = channel.immediate_publisher().into();
}
