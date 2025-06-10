# cyw43

Rust driver for the CYW43439 wifi+bluetooth chip. Implementation based on [Infineon/wifi-host-driver](https://github.com/Infineon/wifi-host-driver).

Works on the following boards:

- Raspberry Pi Pico W (RP2040)
- Raspberry Pi Pico 2 W (RP2350A)
- Pimoroni Pico Plus 2 W (RP2350B)
- Any board with Raspberry Pi RM2 radio module.
- Any board with the CYW43439 chip, and possibly others if the protocol is similar enough.

## Features

Working:

- WiFi support
    - Station mode (joining an AP).
    - AP mode (creating an AP)
    - Scanning
    - Sending and receiving Ethernet frames.
    - Using the default MAC address.
    - [`embassy-net`](https://embassy.dev) integration.
    - RP2040 PIO driver for the nonstandard half-duplex SPI used in the Pico W.
    - Using IRQ for device events, no busy polling.
    - GPIO support (for LED on the Pico W).
- Bluetooth support
    - Bluetooth Classic + LE HCI commands.
    - Concurrent operation with WiFi.
    - Implements the [bt-hci](https://crates.io/crates/bt-hci) controller traits.
    - Works with the [TrouBLE](https://github.com/embassy-rs/trouble) bluetooth LE stack. Check its repo for examples using `cyw43`.

## Running the WiFi examples

- Install `probe-rs` following the instructions at <https://probe.rs>.
- `cd examples/rp`
### Example 1: Scan the wifi stations
- `cargo run --release --bin wifi_scan`
### Example 2: Create an access point (IP and credentials in the code)
- `cargo run --release --bin wifi_ap_tcp_server`
### Example 3: Connect to an existing network and create a server
- `cargo run --release --bin wifi_tcp_server`

After a few seconds, you should see that DHCP picks up an IP address like this
```
11.944489 DEBUG Acquired IP configuration:
11.944517 DEBUG    IP address:      192.168.0.250/24
11.944620 DEBUG    Default gateway: 192.168.0.33
11.944722 DEBUG    DNS server 0:    192.168.0.33
```
This example implements a TCP echo server on port 1234. You can try connecting to it with:
```
nc 192.168.0.250 1234
```
Send it some data, you should see it echoed back and printed in the firmware's logs.
