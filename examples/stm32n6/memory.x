/* STM32N6x7 AXI SRAM map (secure alias used throughout; non-secure 0x24xxxxxx is
 * an alias of the same RAM).
 *
 *   FLEXRAM  0x34000000   400 KB
 *   AXISRAM1 0x34064000   624 KB   (needs RCC.memenr.axisram1en = 1; off at reset)
 *   AXISRAM2 0x34100000  1024 KB   (enabled by boot ROM — always safe)
 *   AXISRAM3 0x34200000   448 KB
 *   AXISRAM4 0x34270000   448 KB
 *   AXISRAM5 0x342E0000   448 KB
 *   AXISRAM6 0x34350000   448 KB
 *   NPURAM   0x343C0000   256 KB
 *   VENCRAM  0x34400000   128 KB
 *
 * Boot ROM dev-mode only leaves [0x34180000, 0x34200000) (the top half of
 * AXISRAM2) safe for anything the Reset handler touches before `main()`
 * runs: addresses below 0x34180000 aren't CPU-accessible yet (hard-faults
 * at reset, PC=0, before even the vector table runs), and AXISRAM3 and
 * above (0x34200000+) isn't clocked yet (RCC.memenr axisram3..6en, done by
 * our own `enable_all_sram()` early in `main()` — too late for anything the
 * Reset handler copies/zeroes itself). FLASH/RAM must stay exactly in that
 * 512 KB window, as in every other example here.
 *
 * `npu_mobilenet` needs more room than that window has, for two statics
 * that are real (non-eliminable) content but are never touched until deep
 * inside `main()`, after `enable_all_sram()`: EC_BLOB (~300 KB of NPU
 * microcode, `.rodata`) and BLOB_STORAGE (its writable, aligned working
 * copy, ~300 KB of `.bss`). Those two are placed via `#[link_section]` into
 * NPUMEM instead of FLASH/RAM, specifically so they *don't* land in the
 * Reset handler's `.data`/`.bss` init copy — which is what broke when
 * EC_BLOB simply grew FLASH's footprint and pushed `.data`'s load address
 * into unclocked AXISRAM3.
 *
 * NPUMEM = AXISRAM1, *not* AXISRAM3/4/5/6: those four are the network's own
 * hardware-absolute activation pools (see `network.c`'s `global pool N`
 * comments — indices 0-3 map to AXISRAM6/5/4/3, all real, nonzero-sized
 * usage). Putting NPUMEM there (an earlier version of this file did) let
 * the running network's *own activation writes* overwrite our blob's
 * later instructions mid-execution — the epoch controller would run
 * correctly for a while and then hit "unknown opcode" (EPOCHCTRL_IRQ
 * ERR_UNKOP) partway through, once execution reached instructions that sat
 * under an activation address the network had by then written to.
 * AXISRAM1 is the one bank `network.c` declares `size=0` (genuinely
 * unused) for this model.
 */
MEMORY
{
  FLASH  : ORIGIN = 0x34180000, LENGTH = 256K
  RAM    : ORIGIN = 0x341C0000, LENGTH = 256K
  NPUMEM : ORIGIN = 0x34064000, LENGTH = 624K
}
