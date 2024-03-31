# embassy-net

`embassy-net` is a no-std no-alloc async network stack, designed for embedded systems.

It builds on [`smoltcp`](https://github.com/smoltcp-rs/smoltcp). It provides a higher-level and more opinionated
API. It glues together the components provided by `smoltcp`, handling the low-level details with defaults and
memory management designed to work well for embedded systems, aiming for a more "Just Works" experience.

## Features

- IPv4, IPv6
- Ethernet and bare-IP mediums.
- TCP, UDP, DNS, DHCPv4, IGMPv4
- TCP sockets implement the `embedded-io` async traits.

See the [`smoltcp`](https://github.com/smoltcp-rs/smoltcp) README for a detailed list of implemented and 
unimplemented features of the network protocols. 

## Hardware support

- [`esp-wifi`](https://github.com/esp-rs/esp-wifi) for WiFi support on bare-metal ESP32 chips. Maintained by Espressif.
- [`cyw43`](https://github.com/embassy-rs/embassy/tree/main/cyw43) for WiFi on CYW43xx chips, used in the Raspberry Pi Pico W
- [`embassy-usb`](https://github.com/embassy-rs/embassy/tree/main/embassy-usb) for Ethernet-over-USB (CDC NCM) support.
- [`embassy-stm32`](https://github.com/embassy-rs/embassy/tree/main/embassy-stm32) for the builtin Ethernet MAC in all STM32 chips (STM32F1, STM32F2, STM32F4, STM32F7, STM32H7, STM32H5).
- [`embassy-net-wiznet`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-wiznet) for Wiznet SPI Ethernet MAC+PHY chips (W5100S, W5500)
- [`embassy-net-esp-hosted`](https://github.com/embassy-rs/embassy/tree/main/embassy-net-esp-hosted) for using ESP32 chips with the [`esp-hosted`](https://github.com/espressif/esp-hosted) firmware as WiFi adapters for another non-ESP32 MCU.

## Examples

- For usage with Embassy HALs and network chip drivers, search [here](https://github.com/embassy-rs/embassy/tree/main/examples) for `eth` or `wifi`.
- The [`esp-wifi` repo](https://github.com/esp-rs/esp-wifi) has examples for use on bare-metal ESP32 chips.
- For usage on `std` platforms, see [the `std` examples](https://github.com/embassy-rs/embassy/tree/main/examples/std/src/bin)

## Adding support for new hardware

To add `embassy-net` support for new hardware (i.e. a new Ethernet or WiFi chip, or
an Ethernet/WiFi MCU peripheral), you have to implement the [`embassy-net-driver`](https://crates.io/crates/embassy-net-driver)
traits.

Alternatively, [`embassy-net-driver-channel`](https://crates.io/crates/embassy-net-driver-channel) provides a higer-level API
to construct a driver that processes packets in its own background task and communicates with the `embassy-net` task via
packet queues for RX and TX.

Drivers should depend only on `embassy-net-driver` or `embassy-net-driver-channel`. Never on the main `embassy-net` crate.
This allows existing drivers to continue working for newer `embassy-net` major versions, without needing an update, if the driver
trait has not had breaking changes.

## Interoperability

This crate can run on any executor.

[`embassy-time`](https://crates.io/crates/embassy-time) is used for timekeeping and timeouts. You must
link an `embassy-time` driver in your project to use this crate.
