# embassy-net-driver-channel

This crate provides a toolkit for implementing [`embassy-net`](https://crates.io/crates/embassy-net) drivers in a
higher level way than implementing the [`embassy-net-driver`](https://crates.io/crates/embassy-net-driver) trait directly.

The `embassy-net-driver` trait is polling-based. To implement it, you must write the packet receive/transmit state machines by
hand, and hook up the `Waker`s provided by `embassy-net` to the right interrupt handlers so that `embassy-net`
knows when to poll your driver again to make more progress.

With `embassy-net-driver-channel` you get a "channel-like" interface instead, where you can send/receive packets
to/from embassy-net. The intended usage is to spawn a "driver task" in the background that does this, passing
packets between the hardware and the channel.

## A note about deadlocks

When implementing a driver using this crate, it might be tempting to write it in the most straightforward way:

```rust,ignore
loop {
    // Wait for either..
    match select(
        // ... the chip signaling an interrupt, indicating a packet is available to receive, or
        irq_pin.wait_for_low(),
        // ... a TX buffer becoming available, i.e. embassy-net wants to send a packet
        tx_chan.tx_buf(),
    ).await {
        Either::First(_) => {
            // a packet is ready to be received!
            let buf = rx_chan.rx_buf().await; // allocate a rx buf from the packet queue
            let n = receive_packet_over_spi(buf).await;
            rx_chan.rx_done(n);
        }
        Either::Second(buf) => {
            // a packet is ready to be sent!
            send_packet_over_spi(buf).await;
            tx_chan.tx_done();
        }
    }
}
```

However, this code has a latent deadlock bug. The symptom is it can hang at `rx_chan.rx_buf().await` under load.

The reason is that, under load, both the TX and RX queues can get full at the same time. When this happens, the `embassy-net` task stalls trying to send because the TX queue is full, therefore it stops processing packets in the RX queue. Your driver task also stalls because the RX queue is full, therefore it stops processing packets in the TX queue.

The fix is to make sure to always service the TX queue while you're waiting for space to become available in the RX queue. For example, select on either "tx_chan.tx_buf() available" or "INT is low AND rx_chan.rx_buf() available":

```rust,ignore
loop {
    // Wait for either..
    match select(
        async {
            // ... the chip signaling an interrupt, indicating a packet is available to receive
            irq_pin.wait_for_low().await;
            // *AND* the buffer is ready...
            rx_chan.rx_buf().await
        },
        // ... or a TX buffer becoming available, i.e. embassy-net wants to send a packet
        tx_chan.tx_buf(),
    ).await {
        Either::First(buf) => {
            // a packet is ready to be received!
            let n = receive_packet_over_spi(buf).await;
            rx_chan.rx_done(n);
        }
        Either::Second(buf) => {
            // a packet is ready to be sent!
            send_packet_over_spi(buf).await;
            tx_chan.tx_done();
        }
    }
}
```

## Examples

These `embassy-net` drivers are implemented using this crate. You can look at them for inspiration.

- [`cyw43`](https://github.com/embassy-rs/embassy/tree/main/cyw43) for WiFi on CYW43xx chips, used in the Raspberry Pi Pico W
- [`embassy-usb`](https://github.com/embassy-rs/embassy/tree/main/embassy-usb) for Ethernet-over-USB (CDC NCM) support.
- [`embassy-net-wiznet`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-wiznet) for Wiznet SPI Ethernet MAC+PHY chips.
- [`embassy-net-esp-hosted`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-esp-hosted) for using ESP32 chips with the [`esp-hosted`](https://github.com/espressif/esp-hosted) firmware as WiFi adapters for another non-ESP32 MCU.

## Interoperability

This crate can run on any executor.
