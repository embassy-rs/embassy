# SPE ADIN1110 `embassy-net` integration

[`embassy-net`](https://crates.io/crates/embassy-net) integration for the `Analog ADIN1110` SPI SPE ethernet chips.

## What is SPE or Single Pair Ethernet / 10 BASE-T1L

SPE stands for Single Pair Ethernet. As the names implies, SPE uses differential signalling with 2 wires (a twisted-pair) in a cable as the physical medium.
SPE is full-duplex - it can transmit and receive ethernet packets at the same time. SPE is still ethernet, only the physical layer is different.

SPE also supports [`PoDL (Power over Data Line)`](https://www.ti.com/lit/an/snla395/snla395.pdf), power delivery from 0.5 up to 50 Watts, similar to [`PoE`](https://en.wikipedia.org/wiki/Power_over_Ethernet), but an additional hardware and handshake protocol are needed.

SPE has many link speeds but only `10 BASE-T1L` is able to reach cable lengths up to 1000 meters in `2.4 Vpp` transmit amplitude.
Currently in 2023, none of the standards are compatible with each other.
Thus `10 BASE-T1L` won't work with a `10 BASE-T1S`, `100 BASE-T1` or any standard `x BASE-T`.

In the industry SPE is also called [`APL (Advanced Physical Layer)`](https://www.ethernet-apl.org), and is based on the `10 BASE-T1L` standard.

APL can be used in [`intrinsic safety applications/explosion hazardous areas`](https://en.wikipedia.org/wiki/Electrical_equipment_in_hazardous_areas) which has its own name and standard called [`2-WISE (2-wire intrinsically safe ethernet) IEC TS 60079-47:2021`](https://webstore.iec.ch/publication/64292).

`10 BASE-T1L` and `ADIN1110` are designed to support intrinsic safety applications. The power supply energy is fixed and PDoL is not supported.

## Supported SPI modes

`ADIN1110` supports two SPI modes. `Generic` and [`OPEN Alliance 10BASE-T1x MAC-PHY serial interface`](https://opensig.org/download/document/OPEN_Alliance_10BASET1x_MAC-PHY_Serial_Interface_V1.1.pdf)

Both modes support with and without additional CRC.
Currently only `Generic` SPI with or without CRC is supported.

*NOTE:* SPI Mode is selected by the hardware pins `SPI_CFG0` and `SPI_CFG1`. Software can't detect nor change the mode.

## Hardware

- Tested on [`Analog Devices EVAL-ADIN1110EBZ`](https://www.analog.com/en/design-center/evaluation-hardware-and-software/evaluation-boards-kits/eval-adin1110.html) with an `STM32L4S5QII3P`, see [`spe_adin1110_http_server`](../examples/stm32l4/src/bin/spe_adin1110_http_server.rs) for an example.
- [`SparkFun MicroMod Single Pair Ethernet Function Board`](https://www.sparkfun.com/products/19038) or [`SparkFun MicroMod Single Pair Ethernet Kit (End Of Life)`](https://www.sparkfun.com/products/19628), supporting multiple microcontrollers. **Make sure to check if it's a microcontroller that is supported by Embassy!**

## Other SPE chips

* [`Analog ADIN2111`](https://www.analog.com/en/products/adin2111.html) 2 Port SPI version. Can work with this driver.
* [`Analog ADIN1100`](https://www.analog.com/en/products/adin1100.html) RGMII version.

## Testing

ADIN1110 library can tested on the host with a mock SPI driver.

$ `cargo test --target x86_64-unknown-linux-gnu`

## Benchmark

- Benchmarked on [`Analog Devices EVAL-ADIN1110EBZ`](https://www.analog.com/en/design-center/evaluation-hardware-and-software/evaluation-boards-kits/eval-adin1110.html), with [`spe_adin1110_http_server`](../examples/stm32l4/src/bin/spe_adin1110_http_server.rs) example.

Basic `ping` benchmark
```rust,ignore
# ping <IP> -c 60

60 packets transmitted, 60 received, 0% packet loss, time 59066ms
rtt min/avg/max/mdev = 1.089/1.161/1.237/0.018 ms

# ping <IP> -s 1472 -M do -c 60

60 packets transmitted, 60 received, 0% packet loss, time 59066ms
rtt min/avg/max/mdev = 5.122/5.162/6.177/0.133 ms
```

HTTP load generator benchmark with [`oha`](https://github.com/hatoo/oha)
```rust,ignore
# oha -c 1 http://<IP> -z 60s
Summary:
  Success rate: 50.00%
  Total:        60.0005 secs
  Slowest:      0.0055 secs
  Fastest:      0.0033 secs
  Average:      0.0034 secs
  Requests/sec: 362.1971

  Total data:   2.99 MiB
  Size/request: 289 B
  Size/sec:     51.11 KiB
```
