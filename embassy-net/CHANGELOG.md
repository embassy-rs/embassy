# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

No unreleased changes yet... Quick, go send a PR!

## 0.7 - 2025-02-14

- don't infinite loop if udp::send methods receive a buffer too large to ever be sent
- add ICMP sockets and a ping utility

## 0.6 - 2025-01-05

- Make `Config` constructors `const`
- The `std` feature has been removed
- Updated `embassy-time` to v0.4

## 0.5 - 2024-11-28

- Refactor the API structure, simplifying lifetimes and generics.
    - Stack is now a thin handle that implements `Copy+Clone`. Instead of passing `&Stack` around, you can now pass `Stack`.
    - `Stack` and `DnsSocket` no longer need a generic parameter for the device driver.
    - The `run()` method has been moved to a new `Runner` struct.
    - Sockets are covariant wrt their lifetime.
    - An implication of the refactor is now you need only one `StaticCell` instead of two if you need to share the network stack between tasks.
- Use standard `core::net` IP types instead of custom ones from smoltcp.
- Update to `smoltcp` v0.12.
- Add `mdns` Cargo feature.
- dns: properly handle `AddrType::Either` in `get_host_by_name()`
- dns: truncate instead of panic if the DHCP server gives us more DNS servers than the configured maximum.
- stack: add `wait_link_up()`, `wait_link_down()`, `wait_config_down()`.
- tcp: Add `recv_queue()`, `send_queue()`.
- tcp: Add `wait_read_ready()`, `wait_write_ready()`.
- tcp: allow setting timeout through `embedded-nal` client.
- tcp: fix `flush()` hanging forever if socket is closed with pending data.
- tcp: fix `flush()` not waiting for ACK of FIN.
- tcp: implement `ReadReady`, `WriteReady` traits from `embedded-io`.
- udp, raw: Add `wait_send_ready()`, `wait_recv_ready()`, `flush()`.
- udp: add `recv_from_with()`, `send_to_with()` methods, allowing for IO with one less copy.
- udp: send/recv now takes/returns full `UdpMetadata` instead of just the remote `IpEndpoint`.
- raw: add raw sockets.


## 0.4 - 2024-01-11

- Update to `embassy-time` v0.3.

## 0.3 - 2024-01-04

- Added `ReadReady` and `WriteReady` impls on `TcpSocket`.
- Avoid never resolving `TcpIo::read` when the output buffer is empty.
- Update to `smoltcp` v0.11.
- Forward constants from `smoltcp` in DNS query results so changing DNS result size in `smoltcp` properly propagates.
- Removed the nightly feature.

## 0.2.1 - 2023-10-31

- Re-add impl_trait_projections
- Fix: Reset DHCP socket when the link up is detected 

## 0.2.0 - 2023-10-18

- Re-export `smoltcp::wire::IpEndpoint`
- Add poll functions on UdpSocket
- Make dual-stack work in embassy-net
- Fix multicast support
- Allow ethernet and 802.15.4 to coexist
- Add IEEE802.15.4 address to embassy net Stack
- Use HardwareAddress in Driver
- Add async versions of smoltcp's `send` and `recv` closure based API
- add error translation to tcp errors
- Forward TCP/UDP socket capacity impls
- allow changing IP config at runtime
- allow non-'static drivers
- Remove impl_trait_projections
- update embedded-io, embedded-nal-async
- add support for dhcp hostname option
- Wake stack's task after queueing a DNS query

## 0.1.0 - 2023-06-29

- First release
