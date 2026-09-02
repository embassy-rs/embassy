On the associated-functions question first, briefly: **yes, keep it, but with a hygiene rule.** The ISR has no handle — only the type — so `Filter::<T, M, Enabled>::end_of_regular_conversion()` (static, register-only) is the only clean way for `on_interrupt` to reach the hardware without fabricating a `&mut self` or dropping to raw PAC calls. Embassy's own handlers do exactly this via `T::regs()` statics. The hygiene rule to add: **associated functions are the "register primitive" layer, methods are the "exclusive access" layer, and one delegates to the other** — `fn try_get(&mut self)` calls `fn try_get_impl()` (static). Then the ISR never invents logic that diverges from the thread path; both call the same primitive. Mark the primitives `pub(crate)` (or leave private), keep the *public* surface `&mut self`-only, and split the primitives by register class: ISR-legal ones (read ISR/flags, ICR writes, latch) vs thread-only ones (CR2 RMWs — those the ISR may call only inside the same critical section the thread side uses).

---

# `todo_typing.md`

## 0. Design principles (enforcement tiers)

| Tier | Invariant class | Mechanism |
|---|---|---|
| Structural | miswiring, aliasing, wrong on every path | types, consumption, `'d` lifetimes |
| Sequencing | register order (DMAEN→DFEN→…) | constructor bodies |
| Temporal | depends on program history (who got there first) | runtime `Result`, atomics |
| Empirical | silicon violates RM | documented contract |

Panic/assert only for arg-local programmer bugs; `Result` for contention; **never** runtime checks for things types already carry.

## 1. Hardware ground truths (empirical + RM0455 facts driving the API)

1. **DMAEN quirk (confirmed on hardware):** with both `RDMAEN` and `JDMAEN` set, the request line fires for **both** groups — it does not discriminate. → Only one enable bit ever set; encoded structurally.
2. `DMAEN` bits writable only before `DFEN` (filter enable).
3. **SCD + CKAB are instance-global via FLT0** (`FLT0CR2.SCDIE/CKABIE`, `FLT0ISR.SCDF/CKABF`, `FLT0ICR` clears — only in filter 0, all filters/channels). They are channel-scoped conditions → reported once per instance.
4. **AWD is fully per-filter**: `AWDIE`/`AWDF`/`AWSR`/`AWCFR`/`AWHTR`/`AWLTR` per filter. One threshold pair per filter → one AWD config per filter.
5. `REOCF`/`JEOCF` have **no ICR clear** — cleared only by reading RDATAR/JDATAR → EOC reads are cancel-safe with data retention.
6. Overrun + sideband flags clear via write-1 ICR (fire-and-forget, no RMW hazard). CR2 is the **only** RMW-shared register ISR↔thread → critical_section mandatory there and only there.
7. `RCH` is a shadow register, applies at next `RSWSTART` → runtime-assignable.
8. `RDATAR` always transfers 32-bit: RDATA[31:8] + RDATACH + RPEND. No DMA bitfield extraction exists. Data = 24-bit MSB-aligned (`(raw as i32) >> 8`). Channel byte is load-bearing for injected scans.
9. No CR1 set/clear registers → cross-half CR1 writes forbidden from ISR context; thread-side halves are safe (cooperative single-core), dual-core → halves `!Send`.

## 2. Type hierarchy & evolution

```
Dfsdm<'d, T, C>
  └─ .configure_pins(closure)                    [enables DFSDMEN + RCC; returns split]
      DfsdmSplit<N> {
          common: DfsdmCommon<'d, T>,            // implicit Enabled, Drop → rcc disable
          ch0..chN: TransceiverBuilder<'d, T, TcvN, C, S_N, S_neighbor>,
          flt0..fltM: Filter<'d, T, FltM>,
      }

TransceiverBuilder  ──(terminal method: new_parallel_adc / new_parallel_dma / 
                       new_parallel_dma_dual / new_spi_ext / new_manchester / 
                       new_spi_int [+ _neighbor variants])──▶
      Transceiver<'a, 'd, T, M, S, MODE>          // CHEN=1 written as last step;
                                                  // MODE carries packing/mux (incl. DATPACK typemark)
          ├─ .configure(&cfg, &online_cfg)  → self          // pre-CHEN values in same call chain
          ├─ .configure_online(&online_cfg) → &mut self     // runtime reconfig (OFFSET, SCDT, BKSCD, CKABEN-area)
          └─ sideband guards take &Transceiver (SCDEN/CKABEN + channel index)

Filter<'d, T, M>   // no PowerState param; enable sequences live in constructors
  ├─ .enable(cfg)                          → SplitFilter<Cpu, Cpu>
  ├─ .enable_with_regular_dma(ch, irq, buf) → SplitFilter<RegularRing, InjectedCpu>
  ├─ .enable_with_injected_dma(ch, irq, buf) → SplitFilter<RegularCpu, InjectedRing>
  │     // each: RDMAEN/JDMAEN (exactly one) → FCR/CR1/JCHGR from cfg → DFEN=1 → build ring
  └─ (pre-enable config folded into FilterConfig; no Disabled state is public)

SplitFilter<'d, T, M, R, J> {
    regular:  R,   // RegularCpu<'d,T,M>  | RegularRing<'d,T,M,CH>
    injected: J,   // InjectedCpu<'d,T,M> | InjectedRing<'d,T,M,CH>
}
  // NO inherent methods except Drop (DFEN=0 + CR2 IER cleanup) and sideband constructors (&self)

SplitFilter  ──(&self)──▶  AwdGuard<'d, T, M>          // per filter, no slot
           ──(&self)──▶  ScdGuard<'d, T>              // instance singleton slot → Result
           ──(&self)──▶  CkabGuard<'d, T>             // instance singleton slot → Result

RegularRing/InjectedRing ──.stop()──▶ RegularCpu/InjectedCpu?  (optional; min: Drop = pause DMA, keep DFEN)
```

Mode-in-return-type: the DMA choice is a **constructor**, not a type parameter or config bool. `FilterConfig` has **no** DMA fields.

## 3. APIs per type

### `FilterConfig` (values only — topology belongs to constructors)
```rust
pub struct FilterConfig {
    pub filter_params: FilterParameters,        // try_new, never panic
    pub continuous_regular: bool,               // CR1.RCONT
    pub fast_regular: bool,                     // CR1.FAST
    pub regular_synchronization: bool,          // CR1.RSYNC
    pub injected_synchronization: bool,         // CR1.JSYNC
    pub injected_trigger: Option<(InjectedDfsdmTrigger<T>, TriggerEdge)>, // CR1.JEXTSEL/JEXTEN
    pub injected_scanning: bool,                // CR1.JSCAN
}
```
Deliberately **absent**: RDMAEN/JDMAEN (constructor choice), AWD config (guard), JCHGR (runtime on half), RCH (runtime on half).

### `RegularCpu<'d, T, M>` / `InjectedCpu<'d, T, M>` (ZSTs, `&mut self` = exclusive data-register access)
| method | register | notes |
|---|---|---|
| `start()` | CR1.RSWSTART / JSWSTART | critical-sectioned CR2 RMW |
| `async read(irq) -> Result<(i32,u8[,bool]), Error>` | RDATAR / JDATAR | EOC-interrupt; cancel-safe w/ data retention; checks overrun first |
| `try_get() -> Option<Result<.., Error>>` | ISR + data reg | Err(Overrun) on ROVRF/JOVRF, clears via ICR |
| `get_unchecked()` | data reg | doc: valid only if EOCF was set |
| `is_eoc()` / `in_progress()` | ISR.REOCF/RCIP, JEOCF/JCIP | |
| `assign_transceiver(&Transceiver)` | CR1.RCH (regular) / JCHGR (injected) | RCH = shadow, next start; JCHGR resets scan position |
| `set_trigger(Option<...>)` (injected only) | CR1.JEXTSEL/JEXTEN | critical-sectioned |

Exist **only** on `Cpu` types. On the DMA half's counterpart group they are the only read path — no method anywhere touches the ring-owned register.

### `RegularRing<'d, T, M, CH>` / `InjectedRing<'d, T, M, CH>` (owns `ReadableRingBuffer<'d, u32>` + channel)
| method | notes |
|---|---|
| `start()` / `stop()` | conversion start bit + ring start; order like ADC (stop conversions before pausing DMA) |
| `async read(&mut buf) -> Result<usize, Error>` | Err(Overrun) when DMA lapped (ADC's commented-out intent, done properly) |
| `read_latest(&mut buf) -> usize` | never errors, discards stale |
| `clear()` | |

Built on embassy `ReadableRingBuffer`; `Drop` → pause ring, disable own IER bits, leave DFEN to `SplitFilter::Drop`. **Never enables EOC interrupts** — DMA is the consumer; "who sets the IER bit" is the single point of mode truth.

### Sideband guards
| type | construction | exclusivity | payload |
|---|---|---|---|
| `AwdGuard<'d, T, M>` | `SplitFilter::analog_watchdog(&self, channels, low, high, awfsel) -> AwdGuard` (per filter) | none needed (per-filter hw); one config per filter = the guard holds it | `AwdEvent { channel, dir: High/Low }` from `AWSR.AWHTF/AWLTF` |
| `ScdGuard<'d, T>` | `SplitFilter::short_circuit_monitor(&self, &Transceiver) -> Result<_, SidebandInUse>` | instance atomic slot (`state.scd_in_use`) | `channel: u8` from `SCDF[x]` |
| `CkabGuard<'d, T>` | `SplitFilter::clock_absence_monitor(&self, &Transceiver) -> Result<_, SidebandInUse>` | instance atomic slot | `channels: u8` bitmask, **OR-accumulating** latch |

All: latched atomics + `AtomicWaker` in state → events survive unawaited periods; `wait(&mut self) -> Event` + `take_event() -> Option<Event>`; `Drop` → disable own IER (critical-sectioned), release channel-side `SCDEN`/`CKABEN`, **release slot last**. SCD/CKAB guards write FLT0.CR2/ISR/ICR regardless of which filter's `SplitFilter` created them; AWD guard writes its own filter's registers. AWD/SCD/CKAB guards coexist; each type singleton via `&self` constructor + CAS, no `&mut` borrow coupling. Cross-instance/per-filter doc note: one guard per detector type per instance; app-level fan-out via pubsub for multi-task needs.

### `Transceiver` additions
- Packing typemark: `DataPackingModeReduced` becomes part of `MODE` (or parallel marker) → `write_sample_standard(u16)` only for Standard, `write_indat1([u16;2])` only for Interleaved/Dual; future DMA-write path derives width from the marker.
- **Delete `get_datinr_as_ref`** (`&self → &mut u32` = unsound). Keep `get_datinr_as_ptr` for the MDMA loopback, doc'd, raw pointer into a transfer that owns the transceiver.
- `write_sample_standard`/`write_indat1` stay (CPU test path).

### Register primitives (associated functions — the ISR-accessible layer)
```rust
impl<T, M> Filter<'d, T, M> {
    // pub(crate), register-class split:
    fn end_of_regular_conversion() -> bool;        // ISR read  — ISR-legal
    fn set_regular_eoc_interrupt(bool);            // CR2 RMW   — thread + ISR(inside critical_section)
    fn clear_regular_overrun();                    // ICR write — ISR-legal
    // … mirrored for injected / sideband / FLT0-global (SCD/CKAB on flt(0)!)
}
```
Methods delegate to primitives; ISR and thread path share them. No logic in `on_interrupt` that isn't in a primitive.

## 4. Interrupt handling

```rust
unsafe fn on_interrupt() {                       // one Handler per line (bind_interrupts!)
    let isr = isr.read(); let cr2 = cr2.read();
    // EOC: no flag clear (data read clears). Disable own IER (critical_section), wake group waker.
    // Overrun (only if ROVRIE/JOVRIE set — i.e. CPU mode): latch counter++ in state, ICR clear, wake.
    // AWD (this filter's): latch AWSR payload into per-filter atomic, AWCFR clear, wake.
    // FLT0 handler additionally: SCDF/CKABF → latch into instance atomics (OR-accumulate CKAB),
    //                           FLT0ICR clear, wake instance wakers.
}
```
State layout: `T::state()` (instance) = group wakers, overrun counters, sideband latch atomics + wakers, `scd_in_use`/`ckab_in_use` slots. Per-filter AWD latch may live in instance state indexed by `M` or separate — instance state is simplest. Rings never enable EOC IERs → handler's IER-gating is the mode check.

## 5. TODO mapping (from the register list)

- [x] RCC/CKOUT/peripheral/channel/filter enable → all inside `Dfsdm::configure_pins` + builder terminal methods + filter constructors. **Move the pub enable fns out of `DfsdmCommon`** — construction is the enable point.
- [ ] FLT0-global placement fix: `DfsdmCommon`'s old `set_analog_watchdog_interrupt` moves onto the **filter** (AWD is per-filter!); `set_short_circuit_detector_interrupt`/`set_clock_absence_interrupt` stay FLT0-global → instance level.
- [ ] Three filter constructors + SplitFilter + 4 half types + Drop chain.
- [ ] Sideband guards + instance slots + `Error::Overrun` producers.
- [ ] Packing typemark → DATINR write restriction; kill `get_datinr_as_ref`.
- [ ] Overrun counters, `CNVTIMR` diagnostic read (any state), EXMAX/EXMIN accessors (**reading resets them → `&mut self` on halves**, single-consumer).
- [ ] TIM side: `set_break_dfsdm_enable`/`set_break2_dfsdm_enable` (TIM1_AF1 BKxDF1BKxE) — belongs to the TIM driver; DFSDM only exposes `BKSCD` via `BreakSignals` (done). Timer-triggered injected: works once `injected_trigger` is in config.
- [ ] `FilterConfig::default` → `try_new`-based, no panic.
- [ ] `#[diagnostic::on_unimplemented]` on `DmaChannelFor<T, M, Group>`: "DMAx_CHy cannot service DFSDM filter M {regular|injected} requests".
- [ ] Doc the DMAEN hardware quirk on the DMA constructors; doc SCD/CKAB FLT0-only scoping; doc cancel-safety of `read()`; doc RCH shadow semantics; doc JCHGR/scan-position reset.
- [ ] Dual-core check (H7, `init_primary`): if halves cross cores → `!Send` on halves or hardware semaphore; otherwise note single-core assumption for CR1 RMW safety.
- [ ] Examples: mem2mem loopback (self-contained), regular-DMA + injected-on-demand split, AWD/CKAB guard.
- [ ] Later (only on demand): `stop()` returning the `Cpu` half; async slot acquisition for sideband (`get_*`); `BothDma`-style nothing — **never**.