# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
