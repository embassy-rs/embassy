# embassy-macros

An [Embassy](https://embassy.dev) project.

Macros for creating the main entry point and tasks that can be spawned by `embassy-executor`. 

NOTE: The macros are re-exported by the `embassy-executor` crate which should be used instead of adding a direct dependency on the `embassy-macros` crate.

## Minimum supported Rust version (MSRV)

The `task` and `main` macros require the type alias impl trait (TAIT) nightly feature in order to compile.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
