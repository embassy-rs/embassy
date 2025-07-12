# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.3.0 - 2025-01-05

- Update `embassy-time` to 0.4.0
- Add Bluetooth support.
- Add WPA3 support.
- Expand wifi security configuration options.

## 0.2.0 - 2024-08-05

- Update to new versions of embassy-{time,sync}
- Add more fields to the BssInfo packet struct #2461
- Extend the Scan API #2282
- Reuse buf to reduce stack usage #2580
- Add MAC address getter to cyw43 controller #2818
- Add function to join WPA2 network with precomputed PSK. #2885
- Add function to close soft AP. #3042
- Fixing missing re-export #3211

## 0.1.0 - 2024-01-11

- First release
