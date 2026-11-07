//! A NOR flash array with the write rules of one family, power cuts at any program or erase, and a log of
//! every operation.

use std::fmt::Write as _;
use std::marker::PhantomData;

use embedded_storage::nor_flash::{ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash};

/// How a program unit may be written once it is not erased.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    /// The unit must be erased, except that all zeros can always be programmed. Anything else is refused.
    OnceOrZero,
    /// As `OnceOrZero`, and all zeros over a unit programmed to all ones is refused too (errata).
    OnceOrZeroErrata,
    /// The unit must be erased, or all zeros go over a unit that is not all zero yet: all zeros over all zeros
    /// is refused ("already fully programmed to 0").
    ZeroOnce,
    /// Never refused; a program over a unit that is not erased leaves an inconsistent ECC that faults on every
    /// read until the next erase, unless it writes the value the unit already holds.
    EccOverwrite,
    /// Bits only go from 1 to 0, any number of times; a 1 over a 0 silently stays 0.
    BitsToZero,
    /// Erases to 0x00. The unit must be zero; a write to a unit that is not zero is refused.
    NotZeroRefused,
    /// Erases to 0x00. A write over programmed data ORs into it.
    Or,
    /// Any value over anything, no erase needed (RRAM).
    Overwrite,
}

/// A flash family.
pub trait Model: 'static {
    /// Name in reports and for `POWER_FAIL_MODEL`.
    const NAME: &'static str;
    /// Where the rules come from.
    const SOURCE: &'static str;
    /// `NorFlash::WRITE_SIZE` of the embassy driver.
    const WRITE_SIZE: usize;
    /// `NorFlash::ERASE_SIZE` of the embassy driver.
    const ERASE_SIZE: usize;
    /// The unit the write rule and ECC apply to.
    const UNIT: usize;
    /// What an erase leaves.
    const ERASED: u8;
    /// The write rule.
    const RULE: Rule;
    /// A unit left half-programmed or half-erased by a cut faults on read.
    const ECC: bool;
    /// Physical sector sizes from address 0, the last one repeating; empty: all `ERASE_SIZE`. One `erase` of the
    /// driver then is several sector erases, each of which a cut can hit.
    const SECTORS: &'static [usize] = &[];
    /// Writes of one `WRITE_SIZE` word allowed between erases.
    const MAX_WRITES: usize = usize::MAX;
    /// Writes allowed per block of this many bytes between erases: (block, writes); (0, 0) for no limit.
    const BLOCK_WRITES: (usize, usize) = (0, 0);
}

/// Error of a [`Nor`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimError {
    /// Power is off.
    PowerCut,
    /// Uncorrectable ECC error on read.
    Ecc,
    /// The family's controller refused the program (PGERR, PROGERR, NOTZEROERR).
    Refused,
    /// Out of bounds or misaligned.
    Bounds,
}

impl NorFlashError for SimError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            SimError::Bounds => NorFlashErrorKind::OutOfBounds,
            _ => NorFlashErrorKind::Other,
        }
    }
}

/// splitmix64.
#[derive(Clone)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// Named address ranges, for the log.
#[derive(Clone, Default)]
pub struct Regions(pub Vec<(&'static str, usize, usize)>);

impl Regions {
    pub fn name(&self, at: usize) -> String {
        for &(name, start, len) in &self.0 {
            if (start..start + len).contains(&at) {
                return format!("{name}+{:#x}", at - start);
            }
        }
        format!("{at:#x}")
    }

    pub fn contains(&self, name: &str, at: usize) -> bool {
        self.0
            .iter()
            .any(|&(n, start, len)| n == name && (start..start + len).contains(&at))
    }
}

/// The flash array.
pub struct Nor<M> {
    pub mem: Vec<u8>,
    /// Per unit: faults on read.
    pub bad: Vec<bool>,
    /// Programs and erases so far (each sector of an erase counts once).
    pub ops: u64,
    cut: Option<(u64, Rng)>,
    pub dead: bool,
    /// The last error returned, with its address.
    pub last_error: Option<(SimError, usize)>,
    /// Programs a cut tore, until their sector is erased: where each started and what it was writing.
    pub torn: Vec<(usize, Vec<u8>)>,
    /// A family rule was broken (silently, as the chip would): a class and what and where.
    pub broken: Vec<(&'static str, String)>,
    /// Per unit: programmed since the last erase (whatever it holds).
    pub programmed: Vec<bool>,
    /// Per unit: faults because a program went over it (`EccOverwrite`), not because a cut tore it.
    pub poisoned: Vec<bool>,
    /// Writes per `WRITE_SIZE` word and per `BLOCK_WRITES` block since the last erase.
    writes: Vec<u32>,
    block_writes: Vec<u32>,
    pub log: Vec<String>,
    /// Leave plain programs and erases out of the log (cuts and refusals are always logged).
    pub quiet: bool,
    pub regions: Regions,
    ops_since_power_on: u64,
    model: PhantomData<M>,
}

/// More operations than this without a power cut is a loop.
const OP_BUDGET: u64 = 100_000;

impl<M: Model> Nor<M> {
    pub fn new(size: usize, regions: Regions) -> Self {
        Self {
            mem: vec![M::ERASED; size],
            bad: vec![false; size / M::UNIT],
            ops: 0,
            cut: None,
            dead: false,
            last_error: None,
            torn: Vec::new(),
            broken: Vec::new(),
            programmed: vec![false; size / M::UNIT],
            poisoned: vec![false; size / M::UNIT],
            writes: vec![0; size / M::WRITE_SIZE],
            block_writes: vec![0; size.checked_div(M::BLOCK_WRITES.0).unwrap_or(0)],
            log: Vec::new(),
            quiet: false,
            regions,
            ops_since_power_on: 0,
            model: PhantomData,
        }
    }

    /// Cuts power during operation number `at` (counted from the start), torn as `tear` says.
    pub fn arm(&mut self, at: u64, tear: u64) {
        self.cut = Some((at, Rng(tear)));
    }

    pub fn disarm(&mut self) {
        self.cut = None;
    }

    pub fn power_on(&mut self) {
        self.dead = false;
        self.ops_since_power_on = 0;
    }

    pub fn note(&mut self, line: String) {
        self.log.push(line);
    }

    fn fail(&mut self, error: SimError, at: usize) -> SimError {
        self.last_error = Some((error, at));
        error
    }

    /// Starts an operation: `Some(rng)` when the cut hits it.
    fn start(&mut self) -> Result<Option<Rng>, SimError> {
        if self.dead {
            return Err(SimError::PowerCut);
        }
        let n = self.ops;
        self.ops += 1;
        self.ops_since_power_on += 1;
        assert!(
            self.ops_since_power_on < OP_BUDGET,
            "{}: more than {OP_BUDGET} flash operations without a power cut: a loop",
            M::NAME
        );
        match self.cut.take() {
            Some((at, rng)) if at == n => Ok(Some(rng)),
            other => {
                self.cut = other;
                Ok(None)
            }
        }
    }

    /// Programming moves bits this way: toward 0 on 0xFF-erased flash, toward 1 on 0x00-erased.
    fn partial(old: u8, new: u8, r: u8) -> u8 {
        if M::ERASED == 0xFF {
            old & (new | r)
        } else {
            old | (new & r)
        }
    }

    fn program_unit(&mut self, at: usize, new: &[u8]) -> Result<(), SimError> {
        let unit = at / M::UNIT;
        let cur = &self.mem[at..at + M::UNIT];
        let erased = cur.iter().all(|&b| b == M::ERASED) && !self.bad[unit];
        let zeros = new.iter().all(|&b| b == 0);
        let cur_zero = cur.iter().all(|&b| b == 0);
        let cur_ones = cur.iter().all(|&b| b == 0xFF);
        let same = cur == new;
        let needs_one = cur.iter().zip(new).any(|(&c, &b)| c & b != b);
        match M::RULE {
            Rule::OnceOrZero | Rule::OnceOrZeroErrata | Rule::ZeroOnce => {
                let refused = match M::RULE {
                    Rule::OnceOrZero => !erased && !zeros,
                    // A unit programmed to all ones reads as erased, but the erratum still refuses zeros over it.
                    Rule::OnceOrZeroErrata => (!erased && !zeros) || (zeros && self.programmed[unit] && cur_ones),
                    _ => !erased && !(zeros && !cur_zero),
                };
                if refused {
                    return Err(SimError::Refused);
                }
                self.mem[at..at + M::UNIT].copy_from_slice(new);
                self.bad[unit] = false;
            }
            Rule::EccOverwrite => {
                if !erased && !same {
                    let line = format!(
                        "{}: program over a used unit ({:02X?} <- {new:02X?}), ECC now inconsistent",
                        self.regions.name(at),
                        &self.mem[at..at + M::UNIT]
                    );
                    self.bad[unit] = true;
                    self.poisoned[unit] = true;
                    self.broken.push(("ecc-overwrite", line));
                }
                for (i, &b) in new.iter().enumerate() {
                    self.mem[at + i] &= b;
                }
            }
            Rule::BitsToZero => {
                if needs_one {
                    let line = format!("{}: program would need a bit 0 -> 1", self.regions.name(at));
                    self.broken.push(("bits-0-to-1", line));
                }
                for (i, &b) in new.iter().enumerate() {
                    self.mem[at + i] &= b;
                }
            }
            Rule::NotZeroRefused => {
                if !cur_zero {
                    return Err(SimError::Refused);
                }
                self.mem[at..at + M::UNIT].copy_from_slice(new);
            }
            Rule::Or => {
                if !erased && !zeros {
                    self.broken.push((
                        "or-overwrite",
                        format!(
                            "{}: write over programmed data, the result is the OR of old and new",
                            self.regions.name(at)
                        ),
                    ));
                }
                for (i, &b) in new.iter().enumerate() {
                    self.mem[at + i] |= b;
                }
            }
            Rule::Overwrite => self.mem[at..at + M::UNIT].copy_from_slice(new),
        }
        self.programmed[unit] = true;
        Ok(())
    }

    /// Counts the writes of the words in `at..at + len` against `MAX_WRITES` and `BLOCK_WRITES`.
    fn count_writes(&mut self, at: usize, len: usize) {
        for word in (at..at + len).step_by(M::WRITE_SIZE) {
            let w = &mut self.writes[word / M::WRITE_SIZE];
            *w += 1;
            if *w as usize == M::MAX_WRITES.saturating_add(1) {
                let line = format!(
                    "{}: written {} times since the erase (n_WRITE {})",
                    self.regions.name(word),
                    *w,
                    M::MAX_WRITES
                );
                self.broken.push(("nwrite-exceeded", line));
            }
            let (block, limit) = M::BLOCK_WRITES;
            if let Some(index) = word.checked_div(block) {
                let b = &mut self.block_writes[index];
                *b += 1;
                if *b as usize == limit + 1 {
                    let line = format!(
                        "{}: block of {block} B written more than {limit} times",
                        self.regions.name(word)
                    );
                    self.broken.push(("block-writes-exceeded", line));
                }
            }
        }
    }

    /// The physical sectors that make up `from..to`.
    fn sectors(from: usize, to: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut start = 0;
        let mut i = 0;
        while start < to {
            let size = match M::SECTORS {
                [] => M::ERASE_SIZE,
                list => list[i.min(list.len() - 1)],
            };
            if start >= from {
                out.push((start, size));
            }
            start += size;
            i += 1;
        }
        out
    }

    fn clear(&mut self, start: usize, len: usize) {
        self.torn.retain(|(at, _)| !(start..start + len).contains(at));
        self.mem[start..start + len].fill(M::ERASED);
        self.bad[start / M::UNIT..(start + len) / M::UNIT].fill(false);
        self.programmed[start / M::UNIT..(start + len) / M::UNIT].fill(false);
        self.poisoned[start / M::UNIT..(start + len) / M::UNIT].fill(false);
        self.writes[start / M::WRITE_SIZE..(start + len) / M::WRITE_SIZE].fill(0);
        let block = M::BLOCK_WRITES.0;
        if let Some(first) = start.checked_div(block) {
            self.block_writes[first..(start + len).div_ceil(block)].fill(0);
        }
    }

    fn tear_program(&mut self, at: usize, new: &[u8], rng: &mut Rng) -> &'static str {
        if M::RULE == Rule::Overwrite {
            // A line is committed as 32-bit stores: some of them made it.
            for (i, word) in new.chunks(4).enumerate() {
                if rng.below(2) == 1 {
                    self.mem[at + 4 * i..at + 4 * i + word.len()].copy_from_slice(word);
                }
            }
            return "some 32-bit words of the line written";
        }
        match rng.below(3) {
            0 => "not started",
            v => {
                for (i, &b) in new.iter().enumerate() {
                    let r = rng.next() as u8;
                    self.mem[at + i] = Self::partial(self.mem[at + i], b, r);
                }
                if M::ECC && v == 2 {
                    self.bad[at / M::UNIT] = true;
                    "half programmed, ECC fault"
                } else {
                    "half programmed"
                }
            }
        }
    }

    fn tear_erase(&mut self, block: usize, len: usize, rng: &mut Rng) {
        for at in (block..block + len).step_by(M::UNIT) {
            match rng.below(4) {
                0 => {}
                1 => {
                    self.mem[at..at + M::UNIT].fill(M::ERASED);
                    self.bad[at / M::UNIT] = false;
                }
                v => {
                    for cell in &mut self.mem[at..at + M::UNIT] {
                        let r = rng.next() as u8;
                        *cell = if M::ERASED == 0xFF { *cell | r } else { *cell & r };
                    }
                    // ECC catches most half-erased units; one that is nearly erased can read wrong unflagged.
                    if M::ECC && v == 2 {
                        self.bad[at / M::UNIT] = true;
                    }
                }
            }
            let erased = self.mem[at..at + M::UNIT].iter().all(|&b| b == M::ERASED);
            self.programmed[at / M::UNIT] &= !erased;
            self.writes[at / M::WRITE_SIZE..(at + M::UNIT) / M::WRITE_SIZE].fill(0);
        }
    }

    /// The first byte of every `M::WRITE_SIZE` word, `!!` for a unit that faults.
    pub fn words(&self, start: usize, words: usize) -> String {
        let mut out = String::new();
        for w in 0..words {
            let at = start + w * M::WRITE_SIZE;
            if self.bad[at / M::UNIT] {
                out.push_str("!! ");
            } else {
                let _ = write!(out, "{:02X} ", self.mem[at]);
            }
        }
        out.trim_end().to_string()
    }
}

impl<M: Model> ErrorType for Nor<M> {
    type Error = SimError;
}

impl<M: Model> ReadNorFlash for Nor<M> {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), SimError> {
        let at = offset as usize;
        if self.dead {
            return Err(SimError::PowerCut);
        }
        if at + bytes.len() > self.mem.len() {
            return Err(self.fail(SimError::Bounds, at));
        }
        if bytes.is_empty() {
            return Ok(());
        }
        if let Some(unit) = (at / M::UNIT..(at + bytes.len()).div_ceil(M::UNIT)).find(|&u| self.bad[u]) {
            return Err(self.fail(SimError::Ecc, unit * M::UNIT));
        }
        bytes.copy_from_slice(&self.mem[at..at + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.mem.len()
    }
}

impl<M: Model> NorFlash for Nor<M> {
    const WRITE_SIZE: usize = M::WRITE_SIZE;
    const ERASE_SIZE: usize = M::ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), SimError> {
        let (from, to) = (from as usize, to as usize);
        if from > to || to > self.mem.len() || !from.is_multiple_of(M::ERASE_SIZE) || !to.is_multiple_of(M::ERASE_SIZE)
        {
            return Err(self.fail(SimError::Bounds, from));
        }
        for (block, len) in Self::sectors(from, to) {
            let n = self.ops;
            let cut = self.start()?;
            let name = self.regions.name(block);
            match cut {
                None => {
                    self.clear(block, len);
                    if !self.quiet {
                        self.log.push(format!("op {n:3}: erase   {name} ({len} B)"));
                    }
                }
                Some(mut rng) => {
                    self.tear_erase(block, len, &mut rng);
                    self.dead = true;
                    self.log
                        .push(format!("op {n:3}: erase   {name} ({len} B)  <-- POWER CUT, erase torn"));
                    return Err(self.fail(SimError::PowerCut, block));
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), SimError> {
        let at = offset as usize;
        if at + bytes.len() > self.mem.len()
            || !at.is_multiple_of(M::WRITE_SIZE)
            || !bytes.len().is_multiple_of(M::WRITE_SIZE)
        {
            return Err(self.fail(SimError::Bounds, at));
        }
        let n = self.ops;
        let cut = self.start()?;
        let name = self.regions.name(at);
        let units = bytes.len() / M::UNIT;
        let (first, last) = (at / M::UNIT, (at + bytes.len()) / M::UNIT);
        let blank = self.mem[at..at + bytes.len()].iter().all(|&b| b == M::ERASED)
            && !self.bad[first..last].contains(&true)
            && !self.programmed[first..last].contains(&true);
        if cut.is_none() && blank {
            // Into erased units every family takes any value: no rule to check unit by unit.
            self.mem[at..at + bytes.len()].copy_from_slice(bytes);
            self.programmed[first..last].fill(true);
            self.count_writes(at, bytes.len());
            if !self.quiet {
                self.log.push(format!("op {n:3}: program {name} {} bytes", bytes.len()));
            }
            return Ok(());
        }
        let mut cut = cut.map(|mut rng| (rng.below(units), rng));
        for (i, new) in bytes.chunks(M::UNIT).enumerate() {
            let unit_at = at + i * M::UNIT;
            if let Some((_, rng)) = cut.as_mut().filter(|(torn, _)| *torn == i) {
                let what = self.tear_program(unit_at, new, rng);
                // Units before the cut were programmed, and the one it hit too unless it never started.
                let done = if what == "not started" {
                    unit_at
                } else {
                    unit_at + M::UNIT
                };
                self.count_writes(at, done - at);
                self.torn.push((at, bytes.to_vec()));
                self.dead = true;
                self.log.push(format!(
                    "op {n:3}: program {name} {} bytes {:02X?}  <-- POWER CUT at unit {i}: {what}",
                    bytes.len(),
                    &bytes[..bytes.len().min(8)]
                ));
                return Err(self.fail(SimError::PowerCut, unit_at));
            }
            if let Err(e) = self.program_unit(unit_at, new) {
                self.count_writes(at, unit_at - at);
                let old = self.mem[unit_at..unit_at + M::UNIT].to_vec();
                self.log.push(format!(
                    "op {n:3}: program {name} {} bytes {:02X?}  <-- REFUSED by the controller at {} (holds {:02X?})",
                    bytes.len(),
                    &bytes[..bytes.len().min(8)],
                    self.regions.name(unit_at),
                    old
                ));
                return Err(self.fail(e, unit_at));
            }
        }
        self.count_writes(at, bytes.len());
        if self.quiet {
        } else if bytes.len() <= 2 * M::UNIT.max(8) {
            self.log.push(format!("op {n:3}: program {name} {:02X?}", bytes));
        } else {
            self.log.push(format!("op {n:3}: program {name} {} bytes", bytes.len()));
        }
        Ok(())
    }
}
