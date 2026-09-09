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

## EMPIRICAL — TRM divergences (silicon contradicts RM; TRM not authoritative here)

- [ ] **E1 — Gain limit = `i32::MAX` (2^31−1), confirmed by derivation and
  on-silicon test.** Enforced at `FilterParameters::MAX_GAIN`
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
- [x] **E2 — Disabled channels hold their clock-absence flag set.**
  rm0455 §"Clock absence detection" (identical wording in all 15 TRMs):
  "CKABF[y] is set also by hardware when corresponding channel y is disabled
  (if CHEN[y] = 0 then CKABF[y] is held in set state)". Corroborated on
  silicon during bring-up (flags on unused channels while the enabled channel
  with a clock stayed clean). The detector itself is per-channel opt-in
  (CKABEN, CHyCFGR1 bit 6, reset 0) — not default-on. Consequence: raw
  CKABF[7:0] reads are meaningless unless masked to channels with
  CKABEN=1 && CHEN=1. Handled by F7 / FT11 / FT3.

---

## FIX

- [ ] **F1 — CR2 RMW race.** CR2 is the only register RMW'd by both ISR and
  thread. All interrupt-enable `modify()`s must run inside
  `critical_section::with` (lost-update hazard today):
  `FilterRegs::set_{regular,injected}_end_of_conversion_interrupt` (mod.rs:1029),
  `AnalogWatchdog::set_analog_watchdog_interrupt` (mod.rs:1935),
  `ShortCircuitDetector::set_short_circuit_detector_interrupt` (mod.rs:2096),
  `ClockAbsenceDetector::set_clock_absence_interrupt` (mod.rs:2154), and the new
  ROVRIE/JOVRIE setters. CR1 is never touched by the ISR → no guard needed.
- [ ] **F2 — Delete `get_datinr_as_ref`** (mod.rs:1239). `&self -> &mut u32` is
  unsound. Keep `get_datinr_as_ptr` (MDMA loopback only, raw pointer, doc'd).
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
- [ ] **F5 — TODO.md break section correction.** Break-enable bits span
  TIM1_AF1 (BKDF1BK0E→BRK1←break0), TIM1_AF2 (BK2DF1BK1E→BRK2←break1),
  TIM8_AF1 (BKDF1BK2E→BRK1←break2), TIM8_AF2 (BK2DF1BK3E→BRK2←break3),
  TIM15/16/17_AF1 (BKDF1BKE→BRK←break0/1/2 per timer). DFSDM2 break[0] →
  LPTIM3_ETR (no TIM register involved; document only).
- [ ] **F6 — One ring per filter.** One DMA request line per filter
  (rm0455 §33.6; serves JDATAR or RDATAR) — a filter runs an injected ring
  OR a regular ring, never both; F3's disjoint borrows don't prevent both.
  Guard: per-instance owner slot in driver `State`
  (`RingOwner {None, Regular, Injected}`), set on ring creation, cleared on
  Drop, panic/debug_assert on contention. Doc on both `ring_buffered` fns:
  "use two filters for both".
- [ ] **F7 — Clock-absence flag masking + CKABEN lifecycle.** Reader chain
  is raw today: `clock_absence_detector_channel_flags()` (mod.rs:2159) reads
  bare `CKABF[7:0]`; `wait_for_event` (mod.rs:2132) and the IRQ handler
  (mod.rs:1374) treat `!= 0` as "event" → with any CHEN=0 channel (held-set,
  E2) they fire spuriously, return garbage masks, and the whole-mask
  CLRCKABF write swallows real flags mixed into the same read. Fix:
  - Derive the armed mask at call time from hardware truth: read
    CKABEN[y] & CHEN[y] from all 8 CHyCFGR1s (no driver state to maintain).
  - Mask the flag read, the `!= 0` checks (wait_for_event + IRQ handler)
    and the CLRCKABF write with it; return only armed-channel bits.
  - CKABEN lifecycle: clear CKABEN when the channel is disabled
    (CKABEN=1 while CHEN=0 → held-set flag + IRQ spam). CKABIE itself is a
    single global bit (FLTCR2 bit 6; superset yaml bit_size 1 is
    TRM-correct); hardware gates the IRQ per channel by CKABEN (rm0455:
    "on channels selected by CKABEN").
  - Same armed-mask treatment for the SCD sibling
    (`short_circuit_detector_channel_flags`, mod.rs:2101; no held-set rule
    there, keep the discipline consistent).
  - Fix copy-paste docstrings ("short-circuit-detector" on CKAB fns).

---

## FEATURE

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
  - Tick TODO.md register list items: ROVRIE, JOVRIE, ROVRF, JOVRF, CLRROVRF,
    CLRJOVRF.
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
  Doc: CKAB is valid only with CKOUTSRC=0 (system clock).
  Masking rule (E2): only read/clear `CKABF[y]` for channels with
  CKABEN=1 && CHEN=1 — disabled channels hold their flags set; the sync wait
  must never inspect the raw 8-bit mask (F7).
- [ ] **FT4 — Detector `clear_flags()`** on `ShortCircuitDetector` /
  `ClockAbsenceDetector` (FLT0 ICR). Explicit "arm" primitive so an app can
  clear startup residue after its own settle delay; complements FT3 (SCD is
  not expected spurious at startup, but symmetric API is cheap).
- [ ] **FT5 — TIM break enables, `#[cfg(dfsdm)]`.** Fields exist only in
  metapac `timer_v1`/`timer_v3`; every DFSDM chip uses one of those (F4/F7/L4/L5
  → v1, H7 → v3); timer_v2 families (G4/H5/N6/U5/WBA) have no DFSDM → cfg is
  both compile-safe and semantically correct. See F5 for the bit map.
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
- [ ] **FT8 — CNVTIMR liveness probe doc**: `get_cnv_cnt` (mod.rs:641) —
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
  masked by the armed set (E2/F7) and documented as meaningless for disabled
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
  and resets ISR + AWSR (all flags cleared).
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
  meaningless"; raw 8-bit mask reads need the armed mask (F7).

---

## NITS

1. Doc typo "pendiong" ×2 (mod.rs:824, 966).
2. Receiver inconsistency: `end_of_*_conversion(&mut self)` delegate to
   statics (mod.rs:1019/1024); `*_conversion_in_progress()` statics
   (mod.rs:825/967 — valid, but inconsistent); `set_enabled()` static
   (mod.rs:1014). Unify shape (public = `&mut self`, statics internal).
3. `Config` struct empty with `//TODO` (mod.rs:44-48) — populate or remove.
4. `Error` enum stray `//TODO` (mod.rs:36) — resolve with FT1.
5. mod.rs:130 AFS critical-section question — fold into F1 (one
   critical_section strategy for all RMW: CR2 + AF assignment).
6. mod.rs:1074 — merge `break_signals` into short-circuit config.
7. mod.rs:1639/1742 — missing docstrings `build_spi_ext`/`build_spi_int`.
8. mod.rs:1816 — config-types module: docstrings, bitmap type, split.
9. types.rs:770 `dma_trait!` TODO — resolved by F4 rewrite.
10. types.rs:808 `FilterTrait` "generify for TODO?" — delete or define
    during F4.
11. `new_pin!(...).unwrap()` ×3 (mod.rs:2189/2201/2202) — verify vs embassy
    conventions.
12. types.rs:1389 `total_gain().unwrap()` — invariant documented; verify
    try_new covers it once.
13. Copy-paste docstrings on CKAB fns — fix with F7.

---

## DATA PIPELINE — stm32-data side

Full detail lives in the stm32-data repo: `in_progress/DFSDMx/TODO.md`.
Summary (each blocks DFSDM availability for whole chip groups):

- [ ] `header.rs` ALT_PERI_DEFINES: `DFSDM1 → DFSDM1_BASE / DFSDM1_BASE_NS`
  (unlocks L552/562; L5 headers define only the `_NS` alias — TrustZone
  attribution, `DFSDM1SEC`).
- [ ] `perimap.rs`: `F7[6]` → `F7[67]` (F777/778/779); L4x1
  (`dfsdm1_v1_0_4ch_L4x1`) → `DFSDM_4CH_2FLT_TRG3` (plain — rm0394 says no
  ADC); L47x/48x (`dfsdm1_v1_0_Cube`) → `DFSDM_8CH_4FLT_TRG3`.
- [ ] `trigger.rs`: `H7(A|B)3` → `H7(A|B)` (H7B0); fix F413 JTRG signal names
  (orphaned footnote digits — resolved mapping in stm32-data TODO SD5;
  RM0430 has no MMS2).
- [ ] Regenerate data + metapac; after this, the variants exist for
  F777-779, L451/452/462, L471/475/476/485/486, L552/562, H7B0.

---

## EXAMPLES

- [ ] `dfsdm_parallel_dma_to_dma.rs` → method-built ring (`flt0.reg
  .ring_buffered(..)`) + async `read()` / `blocking_read` with `Err(Overrun)`
  handling.
- [ ] `dfsdm_it.rs` → `read_regular(..)?`/Result handling.
- [ ] New (stretch): AWD + SCD/CKAB guard example incl. `wait_for_sync()` arm
  sequence.
- [ ] `dfsdm_3phase_motor.rs` (stretch): 3-phase PWM + DFSDM — injected
  conversions TRGO-triggered → injected ring (circular), controller reads in
  PWM-center ISR; regular continuous + manual latest reads (ignore overrun,
  D10); AWD fast-mode high threshold → break0 → TIMx BRK (hardware
  overcurrent break) on one filter/channel, AWD IRQ-only on others (graceful
  shutdown); CKAB via FT3/F7.

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
- [ ] `cargo fmt` per repo config.
- [ ] FT5 gate: the same matrix doubles as the gate for the optional
  TIM15/16/17 break impl.

---

## HOUSEKEEPING

- [ ] This file replaces the old register checklist (old TODO.md content).
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
