# embassy-net

`embassy-net` is a no-std no-alloc async network stack, designed for embedded systems.

It uses the [`xarxa`](https://github.com/embassy-rs/xarxa) network stack, and adds convenient
async wrappers on top of it and implements the main loop for you.

## Features

- IPv4, IPv6
- Ethernet, IP and IEEE 802.15.4 / 6LoWPAN mediums.
- TCP, UDP, raw sockets, DNS, DHCPv4
- TCP sockets implement the `embedded-io` async traits.
- Multicast
- Multiple interface support
- Timestamping sent and received packets for e.g PTP

See the [`xarxa`](https://github.com/embassy-rs/xarxa) README for a detailed list of implemented and
unimplemented features of the network protocols.

Embassy-net focuses on the network/transport layer protocols up to TCP. Higher-level application layer protocols live in other crates built on top of `embassy-net`:
- [`edge-net`](https://crates.io/crates/edge-net)
- [`sntpc`](https://crates.io/crates/sntpc)
- PTP implementation coming soon

## Hardware support

- [`esp-radio`](https://crates.io/crates/esp-radio) for WiFi support ESP32 chips. Maintained by Espressif.
- [`cyw43`](https://github.com/embassy-rs/embassy/tree/main/cyw43) for WiFi on CYW43xx chips, used in the Raspberry Pi Pico W
- [`embassy-usb`](https://github.com/embassy-rs/embassy/tree/main/embassy-usb) for Ethernet-over-USB (CDC NCM) support.
- [`embassy-stm32`](https://github.com/embassy-rs/embassy/tree/main/embassy-stm32) for the builtin Ethernet MAC in all STM32 chips (STM32F1, STM32F2, STM32F4, STM32F7, STM32H7, STM32H5).
- [`embassy-net-wiznet`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-wiznet) for Wiznet SPI Ethernet MAC+PHY chips (W5100S, W5500)
- [`embassy-net-enc28j60`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-enc28j60) for the Microchip ENC28J60 SPI Ethernet MAC+PHY chip.
- [`embassy-net-adin1110`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-adin1110) for the Analog Devices ADIN1110 SPI 10BASE-T1L single-pair Ethernet chip.
- [`embassy-net-esp-hosted`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-esp-hosted) for using ESP32 chips with the [`esp-hosted`](https://github.com/espressif/esp-hosted) firmware as WiFi adapters for another non-ESP32 MCU.
- [`embassy-net-nrf91`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-nrf91) for the cellular modem in Nordic nRF91-series chips.
- [`embassy-net-ppp`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-ppp) for PPP over Serial, useful with cellular modems or for a network link to a host computer.
- [`embassy-nrf`](https://github.com/embassy-rs/embassy/tree/main/embassy-nrf) for IEEE 802.15.4 support on nrf chips.
- [`embassy-stm32-wpan`](https://github.com/embassy-rs/embassy/tree/main/embassy-stm32-wpan) for IEEE 802.15.4 support on STM32WB chips.
- [`embassy-net-tuntap`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-tuntap) for Linux TUN/TAP interfaces, useful for running on `std` platforms.

## Examples

- For usage with Embassy HALs and network chip drivers, search [here](https://github.com/embassy-rs/embassy/tree/main/examples) for `eth` or `wifi`.
- The [`esp-hal` repo](https://github.com/esp-rs/esp-hal) has examples for use on bare-metal ESP32 chips.
- For usage on `std` platforms, see [the `std` examples](https://github.com/embassy-rs/embassy/tree/main/examples/std/src/bin)

## Adding support for new hardware

To add `embassy-net` support for new hardware (i.e. a new Ethernet or WiFi chip, or
an Ethernet/WiFi MCU peripheral), you have to implement the [`xarxa-driver`](https://crates.io/crates/xarxa-driver)
`Driver` trait. Enable its `async` feature and implement `Driver::register_waker`: `embassy-net`
needs it to sleep until the driver has something new.

Drivers should depend only on `xarxa-driver`. Never on the main `embassy-net` crate. This allows
existing drivers to continue working for newer `embassy-net` major versions, without needing an
update, if the driver trait has not had breaking changes.

## Interoperability

This crate can run on any executor.

[`embassy-time`](https://crates.io/crates/embassy-time) is used for timekeeping and timeouts. You must
link an `embassy-time` driver in your project to use this crate.
