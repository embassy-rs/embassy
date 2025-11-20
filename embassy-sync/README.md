# embassy-sync

An [Embassy](https://embassy.dev) project.

Synchronization primitives and data structures with async support:

- [`Channel`](channel::Channel) - A Multiple Producer Multiple Consumer (MPMC) channel. Each message is only received by a single consumer.
- [`PriorityChannel`](priority_channel::PriorityChannel) - A Multiple Producer Multiple Consumer (MPMC) channel. Each message is only received by a single consumer. Higher priority items are shifted to the front of the channel.
- [`PubSubChannel`](pubsub::PubSubChannel) - A broadcast channel (publish-subscribe) channel. Each message is received by all consumers.
- [`Signal`](signal::Signal) - Signalling latest value to a single consumer.
- [`Watch`](watch::Watch) - Signalling latest value to multiple consumers.
- [`Mutex`](mutex::Mutex) - Mutex for synchronizing state between asynchronous tasks.
- [`Pipe`](pipe::Pipe) - Byte stream implementing `embedded_io` traits.
- [`WakerRegistration`](waitqueue::WakerRegistration) - Utility to register and wake a `Waker`.
- [`AtomicWaker`](waitqueue::AtomicWaker) - Utility to register and wake a `Waker` from interrupt context.
- [`MultiWakerRegistration`](waitqueue::MultiWakerRegistration) - Utility registering and waking multiple `Waker`'s.
- [`LazyLock`](lazy_lock::LazyLock) - A value which is initialized on the first access

## Interoperability

Futures from this crate can run on any executor.
