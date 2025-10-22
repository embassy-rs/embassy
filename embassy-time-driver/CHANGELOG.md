# Changelog for embassy-time-driver

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## Unreleased - ReleaseDate

## 0.2.1 - 2025-08-26

- Allow inlining on time driver boundary
- add 133MHz tick rate to support PR2040 @ 133MHz when `TIMERx`'s `SOURCE` is set to `SYSCLK`

## 0.2.0 - 2025-01-02

- The `allocate_alarm`, `set_alarm_callback`, `set_alarm` functions have been removed.
- `schedule_wake` has been added to the `Driver` trait.

## 0.1.0 - 2024-01-11

Initial release
