# Embassy NXP MCX-A MCUs HAL

A Hardware Abstraction Layer (HAL) for the NXP MCX-A family of
microcontrollers using the Embassy async framework. This HAL provides
safe, idiomatic Rust interfaces for GPIO, UART, and OSTIMER
peripherals.

## Retained sleep with a stopped OSTIMER

The optional `sleep-time-bridge` feature selects `time-driver-ostimer` and
exposes `ostimer::sleep_snapshot` and `ostimer::resume_from_sleep`. It cannot
be combined with `time-driver-lptmr`. Embassy remains the owner of the
monotonic clock and timer queue; the feature does not select or enter a
hardware sleep mode.

Keep interrupts masked from the snapshot through sleep, clock recovery, and
reconciliation. The snapshot reports the current monotonic time and earliest
task deadline (`u64::MAX` for none), excluding internal counter-wrap
maintenance alarms. Use an independently retained monotonic source to measure
the entire interval, including recovery, and supply its drift and sampling
uncertainty in microseconds.

`resume_from_sleep` subtracts OSTIMER progress from that measurement, adds
only stopped time beyond the uncertainty, and services expired alarms and
rearms future alarms before returning. It preserves the driver's extended
42-bit raw-counter epoch and translates future compare values through the
sleep offset. `None` indicates an inconsistent observation or overflow; do
not resume timer-dependent handlers with an unreconciled clock.

The caller owns peripheral eligibility, retention, independent wake routing,
elapsed-time accuracy, and restoring clocks before reconciliation. Neither
mutable RTC wall time nor software sleep statistics are a substitute for the
retained elapsed-time source.
