# DFSDM TODO v2 — consolidated

Audited against: RM0455 ch.33 (H7A3/H7B3), RM0468 (H723+) break bits, metapac
(dfsdm_v1, timer_v1/v3), current embassy-stm32 code.
Supersedes the old AI TODO docs (removed); their still-valid intent is absorbed here.

---

## VALIDATED CORRECT — no action (reference)

- All DFEN=0-gated bits are written pre-enable (correct order in `enable_int`/`configure`):
  FAST, RDMAEN, RSYNC, JEXTEN, JEXTSEL, JDMAEN, JSCAN, JSYNC.
- All CHEN=0-gated bits written on `Transceiver<Disabled>` (correct):
  DATMPX, DATPACK, CHINSEL, SPICKSEL, DTRBS, AWFORD, AWFOSR.
- DFSDMEN=0-gated: CKOUTSRC/CKOUTDIV set in `new_ckout`/`new` before
  `configure_pins -> DfsdmCommon::enable`. Correct.
- Runtime-writable (online APIs are legal as implemented): OFFSET, CKABEN, SCDEN,
  BKSCD, SCDT, AWDCH, EXCH, all CR2 IEs, RCONT, RSWSTART, JSWSTART, PLSSKP.
- REOCF/JEOCF cleared only by reading RDATAR/JDATAR → EOC reads are cancel-safe
  with data retention.
- Request precedence: injected > regular; requests ignored while RCIP/JCIP;
  interrupted regular restarts, flagged via RPEND. Matches code docs.

---

## EMPIRICAL — silicon-verified findings (TRM contradicts where noted)

- [ ] **E1 — Gain limit = `i32::MAX` (2^31−1), confirmed by derivation and
  on-silicon test (TRM divergence — RM is wrong twice).** Enforced at `config_types::MAX_GAIN`
  (types.rs:1300). The TRM is wrong twice, identically in all 15 revisions:
  - §33.4.13 states the inclusive condition `FOSR^FORD·IOSR ≤ 2^31` — off by
    one: a 32-bit signed accumulator tops out at +2^31−1, so gain exactly
    2^31 with positive full-scale input wraps to −2^31.
  - §33.4.9 Table 253 even lists a peak of `±2^32` (FOSR=256, Sinc3,
    IOSR=256), unrepresentable in the 32-bit signed datapath.
  - [ ] Rewrite the garbled comment at types.rs:1297-1299 (currently mixes
    "TRM states <=2^32" with `i32::MIN.unsigned_abs()` = 2^31): state the
    actual chain — 32-bit signed accumulator → +2^31−1 max; TRM §33.4.13
    (≤2^31) and §33.4.9 (±2^32) are both disproven; bound is empirical
    (derived + HW-tested).
  - [ ] Boundary test: `FilterParameters::try_new(Sinc3, fosr=1024, iosr=2)`
    (gain = 2^31) must be rejected; the largest config ≤ 2^31−1 must be
    accepted.
- [x] **E2 — Disabled channels hold their clock-absence flag set.** (TRM-correct,
  not a divergence — the driver misread it.) rm0455 §"Clock absence detection"
  (identical wording in all 15 TRMs):
  "CKABF[y] is set also by hardware when corresponding channel y is disabled
  (if CHEN[y] = 0 then CKABF[y] is held in set state)". Corroborated on
  silicon during bring-up (flags on unused channels while the enabled channel
  with a clock stayed clean). The detector itself is per-channel opt-in
  (CKABEN, CHyCFGR1 bit 6, reset 0) — not default-on. Consequence: raw
  CKABF[7:0] reads are meaningless unless masked to channels with
  CKABEN=1 && CHEN=1. Handled by FT12 / FT11 / FT3.
- [x] **E3 — SCD semantics + detector startup latches (RM0399 §31.4.11 +
  §"Manchester/SPI code synchronization"; silicon-verified on the H755 bench
  during FT12 bring-up).**
  - SCD is a **saturation detector**: per-channel up-counter of consecutive
    identical bits on the **data** stream (channel transceiver outputs, not
    CKIN), restarted on every 1↔0 transition; SCDF fires when it reaches SCDT.
    It catches stuck/open-circuit analog inputs; clock faults are CKAB's job.
  - **SCDT=0 fires constantly** — the counter starts at 0 and trivially
    "reaches" 0. Hit on silicon when the threshold write was lost in the FT12
    migration; now structural (FT12 writes SCDT before SCDEN; a validated
    threshold newtype was declined — consider a `debug_assert!(threshold != 0)`).
  - **SCDF is hardware-cleared when CHEN=0** (RM0399 §31.4.11) — the exact
    opposite of CKABF (E2). Drop-time de-arm + disable self-cleans stale SCD
    flags; only CKAB needs the E2 masking discipline.
  - **CKABF startup latch**: while the transceiver is unsynchronized, CKABF is
    held set and `CLRCKABF` writes are ignored; after sync it stays set until
    software clears it. Observed as a one-shot event at the first
    `wait_for_event` — expected hardware behavior; motivating case for FT3
    (poll CKABF=0 as the sync-done indicator) and FT4 (`clear_flags` startup
    residue).
  - SCD **cannot** be used in parallel input mode (DATMPX≠0, §31.4.11) — not
    enforced yet; candidate doc note or `debug_assert` on
    `ShortCircuitDetector::assign_transceivers`.

---

## FIX

- [ ] **F3 — Register ownership via implicit `&mut` gating (TRM-mandated).**
  RM0455 §33.8.7/33.8.8: "firmware must not read JDATAR/RDATAR if DMA is
  activated to read it". Enforce with the borrow, not typestate:
  - Manual data-read methods (`read_regular`, `try_get_regular_result`,
    `get_regular_result_unchecked`, mod.rs:755-827; injected equivalents,
    mod.rs:903-969) stay available on **all** DmaMode halves — the user can
    always manually read whenever no ring exists.
  - `ring_buffered(dma, irq, buf)` takes `&mut self` of the half; while the
    ring object lives, every `&mut self` path on that half is compile-time
    blocked — including paused/stopped rings. "Read what's there" goes through
    the ring's `read`/`read_latest`/`blocking_read`. Dropping the ring releases
    the borrow (and the DMA channel) and manual reads return.
  - NoDma halves cannot create a ring — manual reads are the only path.
  - CR1 touchers (`start_*_conversion`, `set_continuous`, transceiver
    reassignment) go through the ring (ring `start()` covers conversion start)
    or happen before ring creation.
  - `DmaMode` typestate remains only as the RDMAEN/JDMAEN config carrier
    (hardware quirk: only one of the two enable bits is ever set).
  - No SplitFilter type — a filter half is DMA or not; DMA and CPU-data
    functions never coexist in time, enforced by the exclusive borrow.
  Field-level disjoint borrows keep `flt.inj`/`awd`/`extremes` usable while
  the regular ring borrows `flt.reg`.
- [ ] **F4 — RingBufferedFilter ownership.** Replace `filter: &'e dyn
  FilterDma<T, M>` (dma.rs:12) with `&'e mut` of the concrete half. Ring is built
  by methods on the halves: `flt.reg.ring_buffered(dma, irq, buf)` /
  `flt.inj.ring_buffered(..)` (field-level reborrow keeps `flt.inj`/`awd`/
  `extremes` usable while the regular ring lives). Delete the `FilterDma` trait
  (types.rs:796) and the free `new_regular/new_injected` constructors;
  `data_register()` becomes an inherent fn per half.
- [ ] **F5 — Break-enable bit map.** Break-enable bits span
  TIM1_AF1 (BKDF1BK0E→BRK1←break0), TIM1_AF2 (BK2DF1BK1E→BRK2←break1),
  TIM8_AF1 (BKDF1BK2E→BRK1←break2), TIM8_AF2 (BK2DF1BK3E→BRK2←break3),
  TIM15/16/17_AF1 (BKDF1BKE→BRK←break0/1/2 per timer). DFSDM2 break[0] →
  LPTIM3_ETR (no TIM register involved; document only).
- [x] **F6 — One ring per filter (already enforced by typestate, no code).**
  One DMA request line per filter (rm0455 §33.6; serves JDATAR or RDATAR), so
  a filter can't DMA both halves. Already guaranteed at compile time: `Filter`
  carries a single per-filter `D` (mod.rs:416-424), and the ring constructors
  are gated on `FilterRegular<.., RegDma>` / `FilterInjected<.., InjDma>`, so
  a second ring on the same filter is unconstructable. Doc only (D11).
- [ ] **F7 — 2FLT/6FLT capability gap.** `capability::Flt2`/`Flt6` are referenced
  in associations.rs (4CH_2FLT variants, `filters: Flt2`; 8CH_6FLT `filters:
  Flt6`) but `capability` only defines `Flt1/Flt4/Flt8` (types.rs:97-102) and
  `FilterCount` has no impls for them. 2FLT is broken three ways (L451/452/462 =
  `DFSDM_4CH_2FLT_TRG3`, real hardware): missing `capability::Flt2` +
  `FilterCount`; missing `FLT1 => Flt1` in the "single-channel IRQs" list
  (associations.rs:332-343 lists only `FLT0 => Flt0`); missing split struct +
  `Flt2Ready`. 6FLT (`DFSDM_8CH_6FLT_DLY_TRG5_ADC_HWID`, MP13-only) has IRQ
  impls FLT0..5 but missing `capability::Flt6` + `FilterCount` + split +
  `Flt6Ready`.
  - Verify per-variant first: 2FLT must be fixed (L4x1 chips exist); 6FLT is
    MP13-only with no chip in the db yet — fix for completeness or prune (ties
    into HOUSEKEEPING "chip-less variants" and FT10, which adds a different
    2CH_1FLT block).

---

## FEATURE


- [ ] **FT13 — Input-width-aware gain ceiling (serial vs parallel).** E1's
  `MAX_GAIN = 2^31−1` assumes 1-bit serial input; parallel (DATMPX ADC /
  CPU-DMA DATINR) is 16-bit, so the safe FOSR/IOSR ceiling is far lower (~`MAX_GAIN
  >> 16`). Two needs:
  - override/force escape hatch for users who know their headroom;
  - input-aware ceiling — EITHER an input marker typestate (serial / parallel +
    bit width) on `FilterParameters`, OR a separate method/constructor taking the
    input width (pick whichever is simpler).
  Verify the parallel-input gain model in the TRM first (§33.4.5–33.4.6).
- [ ] **FT14 (optional) — AWD-filter gain ceiling.** The AWD fast filter
  (`AWFORD`/`AWFOSR`, → `AwdFilterOrder`/`AwdFilterOsr` per D14) input is
  always 1-bit serial (no parallel case), so no input-width ceiling is needed;
  but the fast filter's own gain (FOSR^FORD, max 32³) vs 16-bit WDATR is
  undocumented in the TRM — investigate only if a fast-mode AWD overflow is
  observed on silicon (symmetric with E1).
- [ ] **FT15 — API self-documentation polish.** Replace tuple returns with named
  result structs (`RegularResult { data, channel, pending }`, `InjectedResult`,
  `Extremum { value, channel }`); connect `data_right_shift` to
  `FilterParameters::recommended_shift()` (derive-by-default, raw override);
  rename `read_maxima`/`read_minima` → read-and-clear variants (or a combined
  `Extremes` snapshot); consider `regular`/`injected` over `reg`/`inj`;
  `get_cnv_cnt` → `conversion_time()` with liveness doc; expose public i32
  sign-extension (u32-vs-i32 + typed value accessors). Goal: no TRM needed for
  the common paths.
- [x] **FT17 — Type-system consolidation (marker axes + where-clause bundles).**
  - TS1 → moved to DONE (PinSource axis, implemented).
  - [DECLINED]TS2 — Bundle `FilterInstance<M>` supertrait (wraps `Instance +
    FilterInterrupt<M>` + `M: InstanceEvents<T>`) replacing the cluster on
    `Filter`/`FilterDisabled`/`FilterRegular`/`FilterInjected`/`AnalogWatchdog`.
  - [DECLINED]TS3 — Bundle `TransceiverInstance<M>` (wraps `Instance` + `M:
    TransceiverMarker + NextChannelForInstance<T>`) replacing the cluster on
    `Transceiver`.
  - [DECLINED]TS4 — `#[diagnostic::on_unimplemented]` on both bundles.
  - [Redundant]TS5 (note) — do NOT merge `FilterInterrupt` + `InstanceEvents` (different
    impl-carrying axes); bundle only. The `FilterMarker` bound is redundant
    (implied by `FilterInterrupt<M>`).
  - [DECLINEDBYEMPIRICISM] TS6 — collapse `'a`/`'d` → single `'d`
    (`&'d DfsdmCommon<'d, …>`). Reverted: unifying forces `&'d` borrows of
    locals whose `Peri<'d>` contents outlive the binding, and immobilizes
    `common` under the object borrows (kills typestate `disable(self)` moves).
    The two-lifetime design is load-bearing.
  - [x] TS7 — SAFETY review of `DfsdmCommon::into_raw_parts` (mod.rs:271-285) and
    `Filter::replace_{regular,injected}_transceivers` (mod.rs:658-718)
    (`ManuallyDrop` + `ptr::read`). **Reviewed & sound.** Invariants: MD
    suppresses the source drop (single owner per field, no double-drop); nothing
    between the reads and struct construction can unwind (worst case = leak, not
    UB); `Peri` is a ghost type (no real `&mut` aliasing); `Filter`'s Drop is
    skipped intentionally and re-acquired by the returned value. Comments
    tightened ("bitwise-move") + `replace_regular` docstring fixed ("injected"→
    "regular").
- [ ] **FT19 (low priority) — Bundle ergonomics, re-approach.** Decide later
  between two idioms for condensing the per-item `where` cluster
  (`T: Instance + FilterInterrupt<M>, M: FilterMarker + InstanceEvents<T>`):
  - **Mini-merge**: fold `InstanceEvents` into `FilterInterrupt` as an assoc fn
    `handle_instance_events()` (Flt0 real / Flt1..7 noop, emitted in
    `impl_dfsdm_filter_irq!`); deletes the sibling trait and the
    `impl_noop_instance_events!` macro; header becomes
    `T: FilterInstance<M>`-friendly. Overturns TS5's "don't merge" note.
  - **Marker-side bundle (optional alternative)**: keep `InstanceEvents`, but
    `trait FilterFlow<T>: FilterMarker + InstanceEvents<T> {}` + blanket impl,
    so `M: FilterFlow<T>` elaborates both via supertraits (rust#20671 behavior);
    headers read `T: Instance + FilterInterrupt<M>, M: FilterFlow<T>`.
  - Either is cosmetic; default to leaving the cluster as-is if neither earns
    its churn. Verify empirically (playground + chip matrix) before committing.
- [X] **FT18 — Interrupt binding + NVIC enable hygiene** (coordinate: detector
  side with FT12; ISR side with FT11). `Binding<I,H>` is a compile-time proof
  (Copy ZST); `InterruptExt::enable()` is a runtime NVIC unmask — keep them
  orthogonal: gate at construction, enable idempotently once.
  - [X] IR1 — `build(irqs: impl Binding<…>)`: require-and-discard at construction
    (`_irq`), no storage (the binding is a proof marker, never used at runtime).
    `FilterBuilder::build(irqs: impl Binding<T::Interrupt, InterruptHandler<T, M>>)`
    and `common.detectors(irqs: impl Binding<T::Interrupt,
    InterruptHandler<T, Flt0>>)` (per FT12, `DetectorsBuilder` is dropped).
  - [X] IR2 — drop `_irq` from `read_regular` (mod.rs:760), `read_injected` (:908),
    `AnalogWatchdog::wait_for_event` (:1848), `ShortCircuitDetector::wait_for_event`
    (:2074), `ClockAbsenceDetector::wait_for_event` (:2132) — construction already
    proved the binding.
  - [X] IR3 — host the enable at construction of each IRQ-using object: once in
    `DetectorsBuilder::build` (FLT0 line) and once in `FilterBuilder::build`
    (that filter's line). The enable is idempotent (`NVIC::unmask`), so the old
    `SCD::new`/`CKAB::new` enables were redundant, not dangerous — the removal is
    ownership/clarity cleanup only. Document that Flt0's filter and the detectors
    share the FLT0 IRQ line.
  - [X]IR4 — write `<T as FilterInterrupt<M>>::Interrupt::enable()` explicitly —
    `T::Interrupt` is unambiguous today only because `Instance` has no `Interrupt`
    associated type.
  - [X] IR5 — `unpend()` before `enable()` (ADC hygiene, adc/mod.rs:697-700) so a
    stale pending flag doesn't fire immediately.
- [ ] **FT1 — Overrun, propagated everywhere** (RM0455 §33.5, Table 254:
  "data not read and overwritten by a new conversion"; JOVRF/ROVRF cleared via
  ICR write-1, enabled by JOVRIE/ROVRIE):
  - `try_get_regular_result` → `Option<Result<(i32, u8, bool), Error>>`;
    `try_get_injected_result` → `Option<Result<(i32, u8), Error>>`. ROVRF/JOVRF
    checked first, cleared via `CLRROVRF`/`CLRJOVRF` before the read attempt.
  - `read_regular`/`read_injected` → `Result<.., Error>`.
  - ROVRIE/JOVRIE enabled while a read is armed; ISR overrun branch: flag && IE
    → ICR clear → wake regular/injected waker → waiting future resolves
    `Err(Overrun)`.
  - `get_*_unchecked` stay raw (documented).
  - Ring reads: `Err(Overrun)` on ring lapping (auto-reset inside ring core)
    AND on ROVRF/JOVRF pre-check (catches filter-side starvation the lapping
    check cannot see). Map ring `DmaUnsynced` → `Error::PeripheralError`.
  - Add ROVRIE/JOVRIE/ROVRF/JOVRF/CLRROVRF/CLRJOVRF accessors + handling.
- [ ] **FT2 — Ring read API on RingBufferedFilter** (mirrors
  `adc/ringbuffered.rs`, wake = DMA HTIF/TCIF via `set_waker`):
  - `read(&mut buf) -> Result<usize, Error>` async (`read_exact`-based; ring
    auto-resets after `Err(Overrun)`, next read continues fresh).
  - `blocking_read(&mut buf) -> Result<usize, Error>` (SPI-style: pause ring on
    overrun).
  - keep `read_latest` (never errors, discards stale).
  - `start`: `compiler_fence(SeqCst)` + ring start + conversion start for
    regular (continuous mode recommended, like ADC Repeated); injected rings
    drain JDATAR, trigger via JEXTEN (timer) or `start_injected_conversion()`
    on the ring.
  - `stop` = pause ring only (conversions keep running — DFSDM's "start" is the
    filter); `clear`, `is_running`, `capacity`.
  - alignment set at construction: default 1, popcount(JCHG) for injected scan
    rings (keeps scan frames coherent after overrun recovery).
  - `Drop`: pause ring before filter/RCC teardown (borrow order already
    enforces).
- [ ] **FT3 — `wait_for_sync()` / `synchronized()` on Transceiver** (replaces
  time-based "blanking"; RM0455 §33.4.4 Clock absence sequence): after
  `CHEN=1`, repeatedly write `CLRCKABF[y]` until `CKABF[y]` reads 0 — the flag
  is held set (and un-clearable) until the transceiver is synchronized; only
  then enable CKABEN=1 (+CKABIE). Non-blocking-sleep poll, no embassy-time dep.
  Silicon-observed (E3): without this wait, the unsynced latch surfaces as a
  one-shot event at the first `wait_for_event`.
  Doc: CKAB is valid only with CKOUTSRC=0 (system clock).
  Masking rule (E2): only read/clear `CKABF[y]` for channels with
  CKABEN=1 && CHEN=1 — disabled channels hold their flags set; the sync wait
  must never inspect the raw 8-bit mask (FT12).
- [ ] **FT4 — Detector `clear_flags()`** on `ShortCircuitDetector` /
  `ClockAbsenceDetector` (FLT0 ICR). Explicit "arm" primitive so an app can
  clear startup residue after its own settle delay; complements FT3 (SCD is
  not expected spurious at startup, but symmetric API is cheap). Motivating
  case observed on silicon (E3): the CKABF startup latch wakes the first
  `wait_for_event` exactly once — `clear_flags` after assign is the interim
  remedy until FT3 lands.
- [ ] **FT5 — TIM break enables,
  `#[cfg(all(dfsdm, any(timer_v1, timer_v3)))]`.** Fields exist only in
  metapac `timer_v1`/`timer_v3`; every DFSDM chip uses one of those (F4/F7/L4/L5
  → v1, H7 → v3); timer_v2 families (G4/H5/N6/U5/WBA) have no DFSDM. Gate with
  the build.rs-emitted `dfsdm` cfg (precedent: `lib.rs:139` `#[cfg(dfsdm)] pub
  mod dfsdm`) **and** the timer-version cfg in conjunction: `dfsdm` alone
  relies on the v1/v3 invariant, while the version cfg alone would expose dead
  API on non-DFSDM v1 chips (F407/F446…). See F5 for the bit map.
  - `timer/low_level.rs`, `impl<T: AdvancedInstance1Channel>` (TIM1/TIM8):
    `set_break_dfsdm_enable` → `af1().set_bkdf1bke`, `set_break2_dfsdm_enable`
    → `af2().set_bk2df1bk1e`, + getters. Style reference:
    `set_break_comparator_enable` (timer/low_level.rs:1376).
  - `timer/complementary_pwm.rs`: user-facing wrappers.
  - Optional: TIM15/16/17 (`Af11chCmp.bkdf1bke`) on the general-1ch impl; add
    only if cargo-check passes on the DFSDM chip set (H723's RM0468 confirms
    the field). F413 has no TIM15/16/17 at all (TIM1-14 only, chip db +
    rm0430) — the impl applies to other series only; F413 breaks are
    TIM1/TIM8 (rm0430 Table 90).
  - DFSDM side (BKSCD/BKAWH/BKAWL assignment) already implemented; cross-link
    docs only.
  - **stm32-data side (audited): integrated for all DFSDM timer versions.**
    `data/registers/timer_v1.yaml` has BKDF1BKE (AF1 fieldset, bit 8) +
    BK2DF1BK1E (AF2 fieldset); timer_v3.yaml has both; AF1_ADV extends
    AF1_1CH_CMP so TIM1/TIM8 inherit; TIM15/16/17 use AF1_1CH_CMP directly.
    timer_v2/timer_l0 lack the fields — correct, no DFSDM chip uses those
    versions. Nothing lacking at data level.
  - **Per-series break wiring (stm32-data-gen/src/trigger.rs) verified**
    against every TRM break-connection table for L4(789A), L4(1-6), F412, F7,
    F413 (DFSDM1+DFSDM2 on BRK1), L4(PQRS), H7(42/43/53/50), MP1, H7(A/B)3,
    H7(23/33/25/35/30), L5.
  - **F4/F7 silicon caveat (empirical)**: F4/F7 TRMs document no AF1 DFSDM
    bits (0 mentions); F7 headers have AF1 `BKDF1BKE` only (no AF2 `BK2DF`);
    F4 headers have neither — yet metapac exposes both bits on timer_v1.
    Break-connection tables (RM0402/RM0430/RM0410) prove the wires, so the
    enables presumably exist; HW-verify F4 (both bits) and F7 (break2 bit)
    before relying on them.
  - Driver side: **zero usage today** — `bkdf` appears in no embassy `.rs`
    file; the examples' "enable breakinput" comments (dfsdm_pwm*.rs) are
    exactly the use-case FT5 unlocks.
- [ ] **FT6 — HWID accessor**, gated on `capability::HasHwid`. Read-only
  version-register cluster @0x7F0 (`DfsdmSuperset::hwid()`), documented in
  RM0475 §29.9 (MP13) and RM0436/RM0441/RM0442 (MP15x) — exactly the chips
  whose blocks carry the HWID cluster (section title "DFSDM version
  registers"; found via register names, not a "HWID" string search):
  - `DFSDM_HWCFGR` @0x7F0 (reset 0x0000_0204): `NBF[15:8]` filters,
    `NBT[7:0]` transceivers — self-describing silicon (MP13: 2 filters /
    4 transceivers, matches the block shape). A runtime capability probe is
    possible, but embassy's compile-time capability tags stay the primary
    mechanism — doc note only.
  - `DFSDM_VERR` @0x7F4 (reset 0x21 = MAJREV 2 / MINREV 1)
  - `DFSDM_IPIDR` @0x7F8 (reset 0x0011_0031)
  - `DFSDM_SIDR` @0x7FC (reset 0xA3C5DD02 = fixed code 0xA3C5DD + 2 KB)
- [ ] **FT7 — Delay block (pulses skipper) polish** (RM0455 §33.4.4 + §33.7.6;
  DLYR present only on DFSDM1 — `HasDelay` mapping already correct):
  - `skip_progress() -> u8` — read PLSSKP; read-back = pulses *still to skip*,
    0 = done.
  - Gate `skip_pulses` to serial `ChannelMode`s (skipper acts on the serial
    stream only; excludes ParallelAdcMode/ParallelDmaMode).
  - Doc: write starts skipping immediately; updating mid-skip is allowed;
    ≤63 pulses per write, skip more by repeated writes; cumulative skipped
    count is the app's job.
- [ ] **FT8 — CNVTIMR liveness probe doc**: `get_cnv_cnt` (mod.rs:641, →
  `conversion_time()` per FT15) —
  document "measures filter activity, not consumer progress" + the
  app-level starvation recipe (timeout + two Δt reads: frozen = starved,
  advancing = alive-but-slow). Universal liveness probe (works for parallel
  inputs too, where CKAB structurally cannot).
- [ ] **FT9 (optional) — `CkoutDivider::for_manchester(rate)` helper** from the
  RM0455 Manchester formula:
  `(CKOUTDIV+1)·T_INCKOUT < T_manchester < 2·CKOUTDIV·T_INCKOUT`.
- [ ] **FT10 — Embassy impl for `DFSDM_2CH_1FLT_TRG5` (H7A/B DFSDM2).** The
  data already emits this block for H7A3/H7B3/H7B0 (2ch/1flt, 32 triggers,
  no DLY, no ADC) and the metapac repr `Dfsdm2ch1fltTrg5` exists — only the
  HAL entry is missing. Add one `mark_dfsdm_instances!` entry
  (repr `Dfsdm2ch1fltTrg5`, `Tcv2`, `Flt1`, delay/hwid/adc_input = false) plus
  `DFSDM_2CH_1FLT_TRG5` in the single-IRQ `impl_dfsdm_filter_irqs!`
  (FLT0 => Flt0) list in associations.rs.
- [ ] **FT11 — ISR accessor completeness** (extends FT4). Generic
  `get_flags()` / `clear_flags(mask)` over ROVRF/JOVRF/REOCF/JEOCF/AWDF/
  SCDF/CKABF (+ per-channel clears via AWCFR/CLRCKABF/CLRSCDF). CKABF bits
  masked by the armed set (E2/FT12) and documented as meaningless for disabled
  channels. ADC-parity "read all status" layer.

---

## TODO — documentation

- [ ] **D1 — Liveness contract doc** on the filter read paths: "read() hangs
  silently iff the source is starved" (DFSDM is a passive sink, no input-side
  underrun detection). Layered detection: prevention (borrow-connected
  transceivers — compile-error on drop/disable while connected), CKAB
  (electrical, needs FT3), CNVTIMR+timeout (universal), overrun
  (consumer-side, FT1).
- [ ] **D2 — Assign-as-overwrite asymmetry**: JCHGR reassignment is instant and
  resets scan position; RCH is a shadow register applied at next RSWSTART.
- [ ] **D3 — start_* semantics**: requests are ignored while RCIP/JCIP; a
  regular conversion interrupted by injected restarts later, flagged by RPEND.
- [ ] **D4 — DATINR notes**: data written before the conversion started is
  lost; 16- and 32-bit accesses both legal (packing-mode dependent).
- [ ] **D5 — CKOUT sequencing**: wait for CKOUT stopped before changing
  CKOUTSRC (glitch avoidance); stop timing 4 sysclk (CKOUTSRC=0) / 1 sysclk +
  3 audio clk (CKOUTSRC=1); CKOUT range 0-20 MHz.
- [ ] **D6 — Continuous-mode restart quirk**: writing CR1 with RCONT=1 while a
  continuous conversion runs restarts it from the next conversion cycle.
- [ ] **D7 — Filter disable semantics**: DFEN=0 immediately stops conversions
  and resets ISR + AWSR (all flags cleared). Doc whether RDATAR/JDATAR retain
  their last value for a post-shutdown read.
- [ ] **D8 — Ring data layout doc**: one u32 word per sample =
  `RDATA[23:8] | RPEND | RDATACH` (JDATA analog); channel byte is load-bearing
  for scan demux; DFSDM reads are 32-bit only (no DMA field extraction).
- [ ] **D9 — Break cross-link doc**: DFSDM = event→wire assignment (BKSCD,
  BKAWH/BKAWL, `BreakSignals`); TIM = wire→BRK consumption enable (FT5).
- [ ] **D10 — Ignore-overrun pattern** (regular AND injected):
  `get_*_result_unchecked` + EOC/JEOC poll is the intended "always-fresh,
  overruns don't matter" path; FT1's `Err(Overrun)` is for callers who care.
  Document on both halves.
- [ ] **D11 — Ring docs**: circular-only (no double-buffer/pingpong in
  `ReadableRingBuffer`); one-ring-per-filter (F6, "use two filters for
  both"). Document on the ring fns.
- [ ] **D12 — CKAB held-set doc** (E2): "CKAB flags on disabled channels are
  meaningless"; raw 8-bit mask reads need the armed mask (FT12).
- [ ] **D13 — Extremes read-to-clear doc**: `read_maxima`/`read_minima` reset
  EXMAX/EXMIN on read (and clear EXMAXCH/EXMINCH); document the semantics.
- [ ] **D14 — AWD naming + doc.** One feature, two layers (§33.4.10):
  per-channel *fast filter* (AWFORD/AWFOSR + WDATR) feeding the per-filter
  *comparator* (AWDCH/AWHT/AWLT/…AWHTF/AWLTF/BKAWH/BKAWL), mode-selected by
  AWFSEL (0 = final main-filter output, 1 = fast filter — the overcurrent path).
  Rename the per-channel cluster to `AwdFilter*` so "AnalogWatchdog" is
  unambiguous: `AnalogWatchdogFilterConfiguration`→`AwdFilterConfig`,
  `AnalogWatchdogFilterOrder`→`AwdFilterOrder`, `AnalogWatchdogOsr`→`AwdFilterOsr`
  (fix the "AWFORD"→"AWFOSR" docstring), `get_analog_watchdog_data`→
  `awd_filter_data`, and the pub consuming builders
  `select_analog_watchdog_*`→`set_awd_filter_*` (made pub when
  `TransceiverConfig`/`configure` were deleted in FT12 — the old
  `analog_watchdog_filter_config` field no longer exists; see NITS #22 for the
  voluntary-vs-mandatory question). Keep `AnalogWatchdog`,
  `AnalogWatchdogConfig`, `AnalogWatchdogEvent`, `flt.awd` unchanged. Doc the
  AWFSEL coupling (the per-channel filter is only meaningful in fastmode).

---

## NITS

3. `Config` struct empty with `//TODO` (mod.rs:44-48) — populate or remove.
4. `Error` enum stray `//TODO` (mod.rs:36) — resolve with FT1.
5. mod.rs:130 AFS critical-section question — fold into F1 (one
   critical_section strategy for all RMW: CR2 + AF assignment).
6. [x] mod.rs:1074 — `break_signals` in `TransceiverConfigOnline` → superseded by
   FT12 (`assign_break_signals` on `ShortCircuitDetector`). Executed with FT12:
   the whole `TransceiverConfigOnline` struct was deleted.
7. [x] mod.rs:1639/1742 — missing docstrings `build_spi_ext`/`build_spi_int`
   (done; also fixed `skips` intra-doc link + `tothe` typos in the same sweep).
8. mod.rs:1816 — config-types module: docstrings, bitmap type, split.
9. types.rs:770 `dma_trait!` TODO — resolved by F4 rewrite.
11. `new_pin!(...).unwrap()` ×3 (mod.rs:2189/2201/2202) — verify vs embassy
    conventions.
12. types.rs:1389 `total_gain().unwrap()` — invariant documented; verify
    try_new covers it once.
14. Reflect: rename `read_regular`/`read_injected` → `read_regular_it`/
    `read_injected_it`? (distinguishes the interrupt-based async read from
    ring/blocking reads; decide during FT1/FT15, not a directive).
15. Reflect: `DFSDMEN` (peripheral enable) currently lives on `DfsdmCommon` —
    consider whether the global enable belongs on the `Dfsdm` wrapper instead.
17. `FilterConfig::default()` (mod.rs:386) calls `FilterParameters::new(Disabled, 1)`
    — its `.expect` is provably unreachable (`Disabled` → fosr=1, gain=1, total
    gain=1 ≤ MAX_GAIN; iosr=1 in 1..=256), so the default can never panic. Add a
    comment documenting that invariant (or an infallible const default path).
18. Packing-mode DATINR write restriction: `write_sample_standard` (INDAT0) and
    `write_indat1` (INDAT0+INDAT1) are both exposed on every `ParallelDmaMode`
    transceiver regardless of DATPACK — `write_indat1` on a Standard-packed
    channel is a silent wrong write. Typemark the packing mode (or document
    which writer matches which `DataPackingModeReduced`).
19. Deferred hardening: `#[diagnostic::on_unimplemented]` on the DMA-channel
    binding ("DMAx_CHy cannot service DFSDM filter M {regular|injected}"); and a
    dual-core `!Send` note (CR1 RMW is safe single-core only). Low priority.
20. `set_continuous` straddles the config/runtime split: it's the one
    `FilterDisabled` static (mod.rs:566) that is also runtime-reachable, since
    RCONT is runtime-writable — `FilterRegular::set_continuous(&mut self)`
    (mod.rs:834) delegates back into the Disabled-scoped static, while the other
    six config statics (FAST/FORD/FOSR/IOSR/RSYNC/JSYNC/JSCAN/JEXTEN/JEXTSEL) are
    DFEN=0-gated only. Harmless (thin delegate); just the known exception.
21. `set_data_packing_mode` design musing (mod.rs:1749-1753, "// could make that
    explicit with a semantic constructor … idk"): consider a semantic dual-pair
    constructor — `new_parallel_dma_dual()` on the even channel meaning "this
    channel and its paired successor are configured as a dual-input pair" —
     folding the comment's intent into the API or deleting the comment. Decide
     during the FT7/FT18 API pass.
22. `select_analog_watchdog_filter_order`/`select_analog_watchdog_osr` are now
    pub consuming builders (voluntary — AWFORD/AWFOSR stay at reset if
    untouched). Think about whether the AWD fast-mode input-stage config
    should be **mandatory** at build time instead (required constructor param
    or configure step), so it can't be forgotten when AWFSEL fastmode is
    intended. Voluntary by decision for now; coordinate with D14 (renames +
    AWFSEL coupling doc) when revisiting.

---

## DATA PIPELINE — stm32-data side

Full detail lives in the stm32-data repo: `in_progress/DFSDMx/TODO.md`.
Summary (each blocks DFSDM availability for whole chip groups):

- [ ] **SD10 (PRIORITY 1) — `trigger.rs` 3-bit-JEXTSEL suffix renumbering.**
  F412/F413/L4-classic compact encodings (0x00-0x07 →
  jtrg{0,1,2,3,5,7,9,10}); the suffix written verbatim as JEXTSEL is wrong for
  3-bit chips. Renumber to compact 0-7; verify each chip's encoding from the
  PDF (esp. F413 DFSDM2's garbled 4-column table). Pure data rename; no
  embassy driver change.
- [ ] **SD1 — `header.rs` ALT_PERI_DEFINES:** `DFSDM1 → DFSDM1_BASE /
  DFSDM1_BASE_NS` (unlocks L552/562; L5 headers define only the `_NS` alias —
  TrustZone `DFSDM1SEC`).
- [ ] **SD2 — `perimap.rs`:** `F7[6]` → `F7[67]` (F777/778/779).
- [ ] **SD3 — `perimap.rs`:** replace the three dead L4 patterns
  (`L4[9]2`/`L4[10]`/`L4[11]`) with L4x1 (`dfsdm1_v1_0_4ch_L4x1`) →
  `DFSDM_4CH_2FLT_TRG3` (plain — rm0394 says no ADC) and L47x/48x
  (`dfsdm1_v1_0_Cube`) → `DFSDM_8CH_4FLT_TRG3`.
- [ ] **SD4 — `trigger.rs`:** `H7(A|B)3` → `H7(A|B)` (H7B0).
- [ ] **SD5 — `trigger.rs`:** fix F413 JTRG signal names (orphaned footnote
  digits — resolved mapping in stm32-data TODO SD5; RM0430 has no MMS2);
  DFSDM1 jtrg4/6/8 stay reserved.
- [ ] **SD6 (dormant) — LPTIM3_ETR ← DFSDM2_BREAK0 (H7A/B).** Blocked on
  unmodeled LPTIM ETR input signal; not blocking anything else.
- [ ] **SD7 (minor) — MP1 RCC `ADFSDMEN`/`ADFSDMLPEN`.** Needed for CKOUTSRC=
  audio on MP1; MP1 not embassy-supported.
- [ ] **SD8 (minor) — H7A/B DFSDM2 kernel clock mux.** Wire `DFSDM2SEL` when
  DFSDM2 support lands in embassy.
- [ ] **SD9 (info, no action) — MP13 chips absent.** Perimap regex correct but
  dormant.
- [ ] Regenerate data + metapac (after SD1-SD5, SD10); after this, the
  variants exist for F777-779, L451/452/462, L471/475/476/485/486, L552/562,
  H7B0.

---

## EXAMPLES

- [x] `dfsdm_short_circuit.rs` + `dfsdm_clock_absence.rs` — migrated to the
  FT12 detector API (`ShortCircuitAssignment` pair, masked waits; see E3 for
  the CKAB startup-latch note).
- [ ] `dfsdm_parallel_dma_to_dma.rs` → method-built ring (`flt0.reg
  .ring_buffered(..)`) + async `read()` / `blocking_read` with `Err(Overrun)`
  handling.
- [ ] `dfsdm_it.rs` → `read_regular(..)?`/Result handling.
- [ ] New (stretch): AWD + SCD/CKAB guard example incl. `wait_for_sync()` arm
  sequence.
- [ ] New: parallel-ADC example (`build_parallel_adc`) — internal-ADC input,
  complementing the existing `dfsdm_parallel_dma_to_dma.rs` CPU/DMA path.
- [ ] `dfsdm_3phase_motor.rs` (stretch): 3-phase PWM + DFSDM — injected
  conversions TRGO-triggered → injected ring (circular), controller reads in
  PWM-center ISR; regular continuous + manual latest reads (ignore overrun,
  D10); AWD fast-mode high threshold → break0 → TIMx BRK (hardware
  overcurrent break) on one filter/channel, AWD IRQ-only on others (graceful
  shutdown); CKAB via FT3/FT12.

---

## VERIFY

- [ ] `cargo check` + clippy on the DFSDM chip matrix (expanded after the
  stm32-data fixes + regeneration):
  - already working: stm32h755cm7 (8ch/4flt), stm32h7a3 (8ch/8flt + 2ch
    DFSDM2), stm32h7b3, stm32f412, stm32f413 (incl. DFSDM2), stm32f767,
    stm32l496, stm32l4a6, stm32l4p5, stm32l4q5, stm32l4r5, stm32l4s9,
    stm32mp157
  - need stm32-data fixes first: stm32f777 (F7[67]), stm32l476 + stm32l452
    (L4 regex), stm32l552 (header NS alias), stm32h7b0 (trigger H7(A|B))
  - SD6–SD9 (dormant/minor/info) are non-blocking — excluded from the
    regen-gate.
- [ ] `cargo fmt` per repo config.
- [ ] FT5 gate: the same matrix doubles as the gate for the optional
  TIM15/16/17 break impl.

---

## HOUSEKEEPING

- [x] Old register-checklist scratchpad (`TODO.md`) removed — content subsumed
  into this file.
- [x] `TODO refactoring.md` / `liveness_and_shutdown.md` removed —
  keep-alive items absorbed above (ownership F3/F4, overrun FT1, liveness
  FT3/FT8/D1); the rest (SplitFilter type layer, software latch atomics, CAS
  guard slots) was rejected by design review.
- [ ] Chip-less embassy variants stay for now — decision: **no pruning yet**.
  After the stm32-data fixes, these still have no owning chip:
  `DFSDM_2CH_1FLT_TRG3_ADC`, `DFSDM_2CH_1FLT_DLY_TRG5_ADC`,
  `DFSDM_4CH_2FLT_TRG3_ADC` (L451/452/462 are plain TRG3 per rm0394);
  `DFSDM_4CH_2FLT_DLY_TRG5_ADC_HWID` is MP13-only (no MP13 chips in the chip
  db). Revisit later.

---

## DONE

- [x] **TS1 — `PinSource` axis (ex-FT17).** Dedicated `trait PinSource { const
  FROM_NEIGHBOR: bool }` with `OwnPins`/`NeighborPins`; deleted
  `SpiExtNeighborMode`/`SpiCkoutNeighborMode`/`ManchesterNeighborMode` and
  `ChannelMode::USES_NEIGHBOR_PINS`. `Transceiver<'a,'d,T,M,S,MODE,PINSOURCE,P>`
  threads the new param; `Drop` releases pins on `<M::Next>`'s slot when
  `FROM_NEIGHBOR`; the three `build_*_neighbor` constructors acquire on the next
  channel's slot. (types.rs:672-687, mod.rs:1129.)
- [x] **NITS #16** — delete `DataSource`/`ExternalSource`/`InternalSource`,
  delete `NotFlt0`, rename `_datasource_marker` → `_channel_mode_marker`.
- [x] **F2** — delete `get_datinr_as_ref` (unsound `&self -> &mut u32`);
  `get_datinr_as_ptr` kept.
- [x] **NITS #10** — delete `FilterTrait`. (Reflect note retained: a filter
  slice-collection use case would want a non-generic `AnyFilter`, not this.)
- [x] **FT16** — replace `UInt<BITS,T>` with `DataRightShift` + `PulsesToSkip`;
  `UInt` deleted.
- [x] **NITS #1/#2/#13** — doc typo "pendiong"; receiver-consistency sweep
  (read-only queries → `&self`, mutators → `&mut self`, statics internal);
  CKAB copy-paste docstrings.
- [x] **TS7** — SAFETY review (see FT17).
- [X] **F1 — CR2 RMW race.** CR2 is the only register RMW'd by both ISR and
  thread. All interrupt-enable `modify()`s must run inside
  `critical_section::with` (lost-update hazard today):
  `FilterRegs::set_{regular,injected}_end_of_conversion_interrupt` (mod.rs:1029),
  `AnalogWatchdog::set_analog_watchdog_interrupt` (mod.rs:1935),
  `ShortCircuitDetector::set_short_circuit_detector_interrupt` (mod.rs:2096),
  `ClockAbsenceDetector::set_clock_absence_interrupt` (mod.rs:2154), and the new
  ROVRIE/JOVRIE setters. CR1 is never touched by the ISR → no guard needed.
  The AF-assignment path (mod.rs:130, currently commented/experimental) gets the
  same `critical_section` discipline if it becomes a runtime RMW.
  - [X] **FT12 — Detector API redesign: AWD-style objects for SCD & CKAB
  (PRIORITY 1; absorbs the old clock-absence-masking item).** DONE — record
  below is as-built (deviations from the original spec noted inline). SCD/CKAB
  are a hardware hybrid — per-channel *enable* (CHyCFGR1 SCDEN/CKABEN),
  instance-level *flags/IRQ* (FLT0 ISR/CR2) — now hidden behind two
  self-contained objects mirroring `flt.awd`/`flt.extremes`:
  - `ShortCircuitDetector` (instance-level, Flt0):
    - `assign_transceivers([ShortCircuitAssignment<'_, T>; N])` — each
      assignment pairs transceiver + threshold (plain `u8`; a validated
      newtype was declined by decision). The fold writes AWSCDR.SCDT **before**
      raising SCDEN (RM order): the SCDEN=1 ∧ SCDT=0 window that latches a
      spurious SCDF (E3) is structurally impossible.
    - `set_threshold(&tcv, u8)` (runtime re-tune), `assign_break_signals(&tcv,
      BreakSignals)` → BKSCD, `unassign_transceivers([&dyn; N])`,
      `wait_for_event()`.
    - Public `flags()`/`clear_flags()` renames still pending → FT4/FT11
      (today: `pub(crate) *_channel_flags[_masked]` / `clear_*`).
  - `ClockAbsenceDetector` (instance-level, Flt0): `assign_transceivers
    ([&dyn; N])` → CKABEN, `unassign_transceivers`, `wait_for_event()`. **No
    threshold/break counterpart** — CKAB has no SCDT/BKSCD hardware (its
    absence threshold is CKOUTDIV, fixed at construction); nothing to pair.
  - **Armed-mask tracking (decision A, as built)**: `short_circuit_armed` /
    `clock_absence_armed` `AtomicU8` in `InstanceState` beside the wakers.
    Sole-writer invariant via the `set_*_channels(mask)` authority helpers —
    plain, no `critical_section` (CFGR1 is thread-only per the F1 audit;
    supersedes the original "under critical_section" wording). Registers are
    the authority: unassign/drop compute `channel_word() & …` fresh from
    CFGR1, the helpers write all COUNT channels and refresh the mirror
    (Relaxed) after every write. ISR + `wait_for_event` read flags ∧ mirror —
    E2 handled, no raw full-mask reads anywhere.
    Implicit de-arm in `Transceiver::drop` for both detectors
    (`drop_transceiver(M::CHANNEL)` — detectors cover the transceiver's own
    channel; the FROM_NEIGHBOR adjustment is pin-links-only, see TS1/CHINSEL).
    `enable()` does NOT re-arm (FT3's sequencing decision).
  - Config removal went **further than specced**: `TransceiverConfig`,
    `TransceiverConfigOnline` and `configure`/`configure_online` were deleted
    entirely (not "leaves offset" — `set_offset(u32)` is the public runtime
    method; `set_data_right_shift` and `select_analog_watchdog_*` became pub
    consuming builder steps, see NITS #22). The four detector setters moved
    off `Transceiver` as planned; the dead `ShortCircuitDetectionConfig` enum
    was deleted (along with the stale cm4 `dfsdm_pwm` example — the cm4 set
    may be revisited later).
  - `common.detectors()` / drop `DetectorsBuilder` — **superseded (closed)**:
    `split.detectors.build(&common, Irqs)` satisfies binding-at-construction
    (FT18 IR1) with `DetectorsBuilder` retained.
  - Symmetry target stands: AWD/SCD/CKAB/extremes each expose
    `assign_transceivers` + `wait_for_event` (+ `flags`/`clear_flags` where
    hardware allows — pending FT4/FT11).