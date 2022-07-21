# cyw43

WIP driver for the CYW43439 wifi chip, used in the Raspberry Pi Pico W. Implementation based on [Infineon/wifi-host-driver](https://github.com/Infineon/wifi-host-driver).

## Current status

Working:

- Station mode (joining an AP).
- Sending and receiving Ethernet frames.
- Using the default MAC address.
- [`embassy-net`](https://embassy.dev) integration.

TODO:

- AP mode (creating an AP)
- GPIO support (used for the Pico W LED)
- Scanning
- Setting a custom MAC address.
- RP2040 PIO driver for the nonstandard half-duplex SPI used in the Pico W. Probably porting [this](https://github.com/raspberrypi/pico-sdk/tree/master/src/rp2_common/cyw43_driver). (Currently bitbanging is used).
- Using the IRQ pin instead of polling the bus.
- Bus sleep (unclear what the benefit is. Is it needed for IRQs? or is it just power consumption optimization?)

## Running the example

- `cargo install probe-run`
- `cd examples/rpi-pico-w`
- `WIFI_NETWORK=MyWifiNetwork WIFI_PASSWORD=MyWifiPassword cargo run --release`

After a few seconds, you should see that DHCP picks up an IP address like this

```
11.944489 DEBUG Acquired IP configuration:
11.944517 DEBUG    IP address:      192.168.0.250/24
11.944620 DEBUG    Default gateway: 192.168.0.33
11.944722 DEBUG    DNS server 0:    192.168.0.33
```

The example implements a TCP echo server on port 1234. You can try connecting to it with:

```
nc 192.168.0.250 1234
```

Send it some data, you should see it echoed back and printed in the firmware's logs.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

