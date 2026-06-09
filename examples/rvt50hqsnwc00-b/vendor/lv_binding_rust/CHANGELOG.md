# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.2]

### Fixed

- Fix build in docs.rs

## [0.6.1]

### Fixed

- Excluded LVGL demos due to crates.io binary size limits

## [0.6.0]

### Added

- Support pointer input devices #62
- Enable interop with [`lv_drivers`](https://github.com/lvgl/lv_drivers) #64
- Allow using both custom and built-in fonts #76
- Support setting the LVGL timer from Rust #81
- Support building from Windows #55
- Enable using the vendored LVGL config #56
- Allow screen switching #57
- Add examples for #64 and #81
- Add a lot of documentation

### Fixed

- Example README now properly specifies how to run
- No more undefined behavior if LVGL is not properly initialized
- Fixed various miscompilations and counts of undefined behavior

### Changed

- Changed core API entirely #51
- Updated LVGL to 8.3.5 #67
- Updated dependencies #61

### Removed

- The `UI` struct and its related API

## [0.5.2] - 2021-03-06

### Added

- Expose RGB values from Color #29
- Make lvgl possible to compile to WASM using Emscripten #31 (complete example available at [lvgl-rs-wasm](https://github.com/rafaelcaricio/lvgl-rs-wasm) and [live](https://rafaelcaricio.github.io/lvgl-rs-wasm/) on the web)

### Fixed

- Fix documentation generation, now we will be visible in docs.rs website 🥳 #41 
- Fix compiler error when running the examples #40

### Changed

- Updated README:
  - Added a hint to install SDL2 before running the demos on macOS #36
  - Add system dependencies for compilation #41

## [0.4.0] - 2020-06-19

### Changed

- Simplify examples by removing the use of threads

### Removed

- Removes the dependency on `alloc` crate

## [0.3.1] - 2020-06-14

### Changed

- Replace `string.c` with implementation in Rust

## [0.3.0] - 2020-06-14

### Added

- New code generation for the safe bindings based on the [`syn`](https://docs.rs/syn/1.0.31/syn/index.html) crate. This uses `lvgl-codegen` directly, which implements code generation for known patterns. This avoids a lot of manual work to expose LVGL API as safe Rust API

### Changed

- Code generation is completely transparent to users
- The code in `lvgl-codegen` gets cleaner and intuitive to write, since now we are processing Rust code instead of C. C is completely abstracted at the `lvgl-sys`/`rust-bindgen` level

### Removed

- No (direct) dependency on `clang-rs`

[Unreleased]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.6.2..HEAD
[0.6.2]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.6.1..0.6.2
[0.6.1]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.6.0..0.6.1
[0.6.0]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.5.2..0.6.0
[0.5.2]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.4.0..0.5.2
[0.4.0]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.3.1..0.4.0
[0.3.1]: https://github.com/rafaelcaricio/lvgl-rs/compare/0.3.0..0.3.1
[0.3.0]: https://github.com/rafaelcaricio/lvgl-rs/releases/tag/0.3.0