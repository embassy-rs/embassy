# embassy-ptp-driver

This crate defines the common timestamp type and adjustable clock trait for PTP
hardware clocks. Hardware implementations and clock-synchronization engines can
depend on this crate without depending on each other or on `embassy-net`.

Packet timestamp transport and its timestamp type remain independently owned by
`xarxa-driver`. An integration crate may convert between timestamps when
the packet timestamp source and adjustable clock are the same hardware time
domain.

The clock here is also distinct from `embassy-time-driver`: executor time is a
global, monotonic scheduling clock, while a PTP hardware clock has an arbitrary
epoch and is adjusted while it is being synchronized.

## Interoperability

This crate can run on any executor.
