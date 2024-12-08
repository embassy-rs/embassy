# Changelog for embassy-time-queue-driver

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Added `integrated-timers` and `generic-queue-N` features
- Added `queue_generic` module which contains `Queue` (configured via the `generic-queue-N` features) and  `ConstGenericQueue<SIZE>`.
- Added `GenericTimerQueue` and `GlobalTimerQueue` structs that can be used to implement timer queues.

## 0.1.0 - 2024-01-11

Initial release
