#![doc = include_str!("../README.md")]
extern crate proc_macro;

use proc_macro::TokenStream;

mod macros;
mod util;
use macros::*;

/// Declares an async task that can be run by `embassy-executor`. The optional `pool_size` parameter can be used to specify how
/// many concurrent tasks can be spawned (default is 1) for the function.
///
///
/// The following restrictions apply:
///
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * The optional `pool_size` attribute must be 1 or greater.
///
///
/// ## Examples
///
/// Declaring a task taking no arguments:
///
/// ``` rust
/// #[embassy_executor::task]
/// async fn mytask() {
///     // Function body
/// }
/// ```
///
/// Declaring a task with a given pool size:
///
/// ``` rust
/// #[embassy_executor::task(pool_size = 4)]
/// async fn mytask() {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn task(args: TokenStream, item: TokenStream) -> TokenStream {
    task::run(args.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn main_avr(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_AVR).into()
}

/// Creates a new `executor` instance and declares an application entry point for Cortex-M spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_cortex_m(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_CORTEX_M).into()
}

/// Creates a new `executor` instance and declares an architecture agnostic application entry point spawning
/// the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// A user-defined entry macro must provided via the `entry` argument
///
/// ## Examples
/// Spawning a task:
/// ``` rust
/// #[embassy_executor::main(entry = "qingke_rt::entry")]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_spin(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_SPIN).into()
}

/// Creates a new `executor` instance and declares an application entry point for RISC-V spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// A user-defined entry macro can be optionally provided via the `entry` argument to override the default of `riscv_rt::entry`.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
///
/// Spawning a task using a custom entry macro:
/// ``` rust
/// #[embassy_executor::main(entry = "esp_riscv_rt::entry")]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_riscv(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_RISCV).into()
}

/// Creates a new `executor` instance and declares an application entry point for STD spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_std(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_STD).into()
}

/// Creates a new `executor` instance and declares an application entry point for WASM spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// ## Examples
/// Spawning a task:
///
/// ``` rust
/// #[embassy_executor::main]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_wasm(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_WASM).into()
}

/// Creates a new `executor` instance and declares an application entry point for an unspecified architecture, spawning the corresponding function body as an async task.
///
/// The following restrictions apply:
///
/// * The function must accept exactly 1 parameter, an `embassy_executor::Spawner` handle that it can use to spawn additional tasks.
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * Only a single `main` task may be declared.
///
/// A user-defined entry macro and executor type must be provided via the `entry` and `executor` arguments of the `main` macro.
///
/// ## Examples
/// Spawning a task:
/// ``` rust
/// #[embassy_executor::main(entry = "your_hal::entry", executor = "your_hal::Executor")]
/// async fn main(_s: embassy_executor::Spawner) {
///     // Function body
/// }
/// ```
#[proc_macro_attribute]
pub fn main_unspecified(args: TokenStream, item: TokenStream) -> TokenStream {
    main::run(args.into(), item.into(), &main::ARCH_UNSPECIFIED).into()
}

/// Declares an async test that can be run by `embassy-executor`.
///
/// This macro allows you to create asynchronous tests that use Embassy's executor capabilities.
/// The test runner will spawn the test function using Embassy's executor and wait for it to complete.
///
/// The following restrictions apply:
///
/// * The function must be declared `async`.
/// * The function must not use generics.
/// * The function must return a result type compatible with the standard test framework.
///
/// ## Examples
///
/// Creating a simple async test:
///
/// ``` rust
/// #[embassy_executor_macros::test]
/// async fn my_test() {
///     // Async test code
///     assert_eq!(1 + 1, 2);
/// }
/// ```
///
/// Tests can also use the Embassy executor features such as timers and tasks:
///
/// ``` rust
/// #[embassy_executor_macros::test]
/// async fn async_operations_test() {
///     let mut count = 0;
///     // Use async operations
///     embassy_time::Timer::after_millis(10).await;
///     count += 1;
///     assert_eq!(count, 1);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(args: TokenStream, input: TokenStream) -> TokenStream {
    macros::test::test(args, input)
}
