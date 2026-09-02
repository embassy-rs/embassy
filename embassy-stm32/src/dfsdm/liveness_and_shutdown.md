## Shutdown & liveness semantics for DFSDM sources

### 1. The core hardware fact

DFSDM is a **passive sink with no input-side underrun detection**. A starved filter (source stops delivering samples) never completes a conversion: `REOCF`/`JEOCF` never set, no DMA request, no interrupt, `RCIP` stays high. Failure mode = **silent infinite hang** of `read().await` / ring `read()`. No DFSDM status flag says "source stopped" in general — detection must be composed from several mechanisms, layered by what they can actually see.

### 2. Prevention: transceiver connection is a borrow, not a register copy

Transceivers are non-exclusive data sources (any filter → any channel), which is exactly shared-borrow semantics. Encode it:

```rust
impl RegularCpu<'d, T, M, 'a> {
    /// Consumes self, returns half holding the new borrow. Reassignment
    /// releases the previous borrow. No unassign exists — detaching IS
    /// reassigning (RCH/JCHGR are selectors, not links).
    pub fn assign_transceiver<'b>(self, tcv: &'b Transceiver<...>) -> RegularCpu<'d, T, M, 'b>;
}
// injected: &'a [&'a dyn TransceiverTrait<T>] or fixed array (arity = selected channels)
```

This makes the following **compile errors** while any filter is connected:

- `drop(tcv)` / transceiver `Drop` (pin unconfigure)
- `tcv.disable()` (`CHEN=0`)
- `tcv.configure_online()` (`&mut self` paths)

→ The driver **cannot cause** starvation internally. Drop order is forced correct: halves/rings/guards → transceivers → common.

Costs (accepted): online reconfig of a connected channel frozen (feature — no mid-integration mutation); `'a` lifetime plumbing through `SplitFilter` (mechanical; fixed-size borrow array as fallback for injected).

"Asign-as-overwrite" is hardware-faithful — keep no-`unassign`. Document asymmetry: JCHGR reassignment = instant + resets scan position; RCH = shadow, takes effect next `RSWSTART`.

### 3. Detection layer 1: CKAB — channel clock presence

Semantics (RM0455): `CKABF[y]` trips when **no clock present on CKINy**, including channel **disabled (`CHEN=0`) or not yet synchronized** (flag reads 1). So CKAB covers:

| Case | CKAB sees it? |
|---|---|
| Dead external clock source (serial ext modes) | ✅ primary use |
| Channel disabled / disconnected while filter connected | ✅ (flag = 1) |
| Channel not yet synchronized after enable | ✅ (flag = 1) → **requires blanking period** after enable before arming the guard |
| Enabled + clocked, but data content dead (internal-clock mode, mute mic) | ❌ clock is ours, runs fine |
| Parallel input (ADC / DMA writes to DATINR) | ❌ no clock pin involved |

Guards: `CkabGuard` (instance-scoped via FLT0, singleton slot, OR-accumulating latch) — unchanged from sideband design. **Add: blanking** — guard constructor takes (or `start()` implies) a settle delay after channel enable before `wait()` is meaningful, otherwise it fires instantly on the not-yet-synchronized state. Implementation: either a `Timer::after(settle)` before first check, or document that the first `take_event()` right after enable may report the sync-in-progress trip.

### 4. Detection layer 2: CNVTIMR — universal liveness probe

`FLTxCNVTIMR.CNVCNT` counts completed conversions, free-running. **Stalled filter ⇔ frozen counter.** Mode-independent — covers everything CKAB structurally cannot (parallel ADC/DMA sources, data-dead-but-clocked).

```rust
impl RegularCpu {
    /// Completed-conversion counter (CNVTIMR). Stops advancing iff the
    /// filter is starved. Diagnoses source death in ALL input modes.
    pub fn conversions_completed(&mut self) -> u32;   // &mut self: single reader, no exclusivity issue (read-only reg actually — any state OK)
}
```

Application-level composition (stock embassy, no driver machinery):

```rust
match embassy_time::with_timeout(10.millis(), regular.read(Irqs)).await {
    Ok(s) => ...,
    Err(_) => info!("stalled, CNVTIMR={}", regular.conversions_completed()),
}
// two reads Δt apart: frozen = starved, advancing = alive-but-slow
```

**Scope caveat (doc):** CNVTIMR measures **filter activity**, not consumer progress — in continuous mode it advances faster than results are consumed. Source starvation → CNVTIMR; consumer falling behind → overrun counter. Complementary probes, not redundant.

### 5. The layered contract (summary)

| Layer | Mechanism | Covers |
|---|---|---|
| Prevent (internal) | shared-borrow assignment | driver-caused disconnect/drop/disable — **unrepresentable** |
| Detect (electrical) | `CkabGuard` (+ blanking) | dead/absent/unsynchronized clock on clocked pins, disabled channel |
| Detect (general) | `conversions_completed()` + app timeout | any-mode starvation, incl. parallel & data-dead |
| Detect (consumer) | overrun counter / `Err(Overrun)` | consumer too slow (output side, has hardware flags) |

Tier mapping (consistent with taxonomy): prevention = structural (types), CKAB = hardware detector (empirical), CNVTIMR+timeout = temporal (application level), overrun = temporal (driver level).

### 6. TODO additions

- [ ] `'a` borrow in `assign_transceiver`/`assign_transceivers`, plumbed through `SplitFilter`
- [ ] Blanking/settle handling in `CkabGuard` (constructor param or first-`wait` delay; document unsynchronized-trip)
- [ ] `conversions_completed()` accessor (+ doc: activity-not-consumption caveat)
- [ ] Doc: "read() hangs silently iff source starved — see liveness layers"; connect CKAB guard ↔ CNVTIMR as diagnosis path
- [ ] Doc: assign-as-overwrite asymmetry (JCHGR instant+scan-reset vs RCH shadowed)