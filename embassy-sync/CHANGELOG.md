# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0 - 2023-04-13

- pubsub: Fix messages not getting popped when the last subscriber that needed them gets dropped.
- pubsub: Move instead of clone messages when the last subscriber pops them.
- pubsub: Pop messages which count is 0 after unsubscribe.
- Update `embedded-io` from `0.3` to `0.4` (uses `async fn` in traits).
- impl `Default` for `WakerRegistration`
- impl `Default` for `Signal`
- Remove unnecessary uses of `atomic-polyfill`
- Add `#[must_use]` to all futures.


## 0.1.0 - 2022-08-26

- First release