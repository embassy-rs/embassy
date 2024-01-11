# embassy-time

Timekeeping, delays and timeouts.

Timekeeping is done with elapsed time since system boot. Time is represented in
ticks, where the tick rate is defined either by the driver (in the case of a fixed-rate
tick) or chosen by the user with a [tick rate](#tick-rate) feature. The chosen
tick rate applies to everything in `embassy-time` and thus determines the maximum 
timing resolution of <code>(1 / tick_rate) seconds</code>.

Tick counts are 64 bits. The default tick rate of 1Mhz supports
representing time spans of up to ~584558 years, which is big enough for all practical
purposes and allows not having to worry about overflows.

## Global time driver

The `time` module is backed by a global "time driver" specified at build time.
Only one driver can be active in a program.

All methods and structs transparently call into the active driver. This makes it
possible for libraries to use `embassy_time` in a driver-agnostic way without
requiring generic parameters.

For more details, check the [`embassy_time_driver`](https://crates.io/crates/embassy-time-driver) crate.

## Instants and Durations

[`Instant`] represents a given instant of time (relative to system boot), and [`Duration`]
represents the duration of a span of time. They implement the math operations you'd expect,
like addition and substraction.

## Delays and timeouts

[`Timer`] allows performing async delays. [`Ticker`] allows periodic delays without drifting over time.

An implementation of the `embedded-hal` delay traits is provided by [`Delay`], for compatibility
with libraries from the ecosystem.

## Wall-clock time

The `time` module deals exclusively with a monotonically increasing tick count.
Therefore it has no direct support for wall-clock time ("real life" datetimes
like `2021-08-24 13:33:21`).

If persistence across reboots is not needed, support can be built on top of
`embassy_time` by storing the offset between "seconds elapsed since boot"
and "seconds since unix epoch".
