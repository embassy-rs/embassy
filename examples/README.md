# Examples for the rp2040 `WIZnet W5500-EVB-Pico` board

Examples are written for the [`WIZnet W5500-EVB-Pico`](https://www.wiznet.io/product-item/w5500-evb-pico/) board.

## Prerequisites
```bash
cargo install probe-rs-cli
```

## TCP server example
```bash
cargo run --bin tcp-server --release
```
This example implements a TCP echo server on port 1234 and using DHCP.
Send it some data, you should see it echoed back and printed in the console.

## Multi-socket example
```bash
cargo run --bin multisocket --release
```
This example shows how you can allow multiple simultaneous TCP connections, by having multiple sockets listening on the same port.

## TCP client example
```bash
cargo run --bin tcp-client --release
```
This example implements a TCP client that attempts to connect to a host on port 1234 and send it some data once per second.

## UDP server example
```bash
cargo run --bin udp --release
```
This example implements a UDP server listening on port 1234 and echoing back the data.
