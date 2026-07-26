//! HID Report Descriptor parser.
//!
//! Parses a USB HID report descriptor byte stream and provides typed access to
//! field values inside raw report buffers, without heap allocation.
//!
//! # Usage
//!
//! ```ignore
//! use embassy_usb_host::class::hid::{HidHost, PROTOCOL_BOOT};
//! use embassy_usb_host::class::hid_report::{ReportDescriptor, usage_page, usage};
//!
//! // After enumeration, fetch the HID report descriptor:
//! let mut desc_buf = [0u8; 256];
//! let desc_bytes = hid.fetch_report_descriptor(&mut desc_buf).await?;
//! let report: ReportDescriptor<32> = ReportDescriptor::parse(desc_bytes);
//!
//! // In the read loop:
//! let mut buf = [0u8; 64];
//! hid.read(&mut buf).await?;
//!
//! let x = report.extract_i32(&buf, 0, usage_page::GENERIC_DESKTOP, usage::X);
//! let y = report.extract_i32(&buf, 0, usage_page::GENERIC_DESKTOP, usage::Y);
//! let btn1 = report.extract_bool(&buf, 0, usage_page::BUTTON, 1);
//! ```

// ── Usage page constants ───────────────────────────────────────────────────────

/// Common HID usage page identifiers.
pub mod usage_page {
    /// Generic Desktop Controls (pointers, mice, joysticks, gamepads, keyboards).
    pub const GENERIC_DESKTOP: u16 = 0x01;
    /// Keyboard / Keypad.
    pub const KEYBOARD: u16 = 0x07;
    /// LEDs.
    pub const LED: u16 = 0x08;
    /// Buttons. Usage 1 = Button 1, usage 2 = Button 2, etc.
    pub const BUTTON: u16 = 0x09;
    /// Consumer Controls (media keys, volume, etc.).
    pub const CONSUMER: u16 = 0x0C;
}

/// Generic Desktop usages (usage page [`usage_page::GENERIC_DESKTOP`]).
pub mod usage {
    /// Pointer collection.
    pub const POINTER: u16 = 0x01;
    /// Mouse collection.
    pub const MOUSE: u16 = 0x02;
    /// Joystick collection.
    pub const JOYSTICK: u16 = 0x04;
    /// Gamepad collection.
    pub const GAMEPAD: u16 = 0x05;
    /// Keyboard collection.
    pub const KEYBOARD: u16 = 0x06;
    /// X axis.
    pub const X: u16 = 0x30;
    /// Y axis.
    pub const Y: u16 = 0x31;
    /// Z axis.
    pub const Z: u16 = 0x32;
    /// X rotation axis.
    pub const RX: u16 = 0x33;
    /// Y rotation axis.
    pub const RY: u16 = 0x34;
    /// Z rotation axis.
    pub const RZ: u16 = 0x35;
    /// Slider control.
    pub const SLIDER: u16 = 0x36;
    /// Dial / rotary control.
    pub const DIAL: u16 = 0x37;
    /// Scroll wheel.
    pub const WHEEL: u16 = 0x38;
    /// Hat switch (POV).
    pub const HAT_SWITCH: u16 = 0x39;
    /// D-pad up.
    pub const DPAD_UP: u16 = 0x90;
    /// D-pad down.
    pub const DPAD_DOWN: u16 = 0x91;
    /// D-pad right.
    pub const DPAD_RIGHT: u16 = 0x92;
    /// D-pad left.
    pub const DPAD_LEFT: u16 = 0x93;
}

// ── Input item flag constants ──────────────────────────────────────────────────

/// Bit flags from a HID Input/Output/Feature item (bits 0–7 of the item data).
pub mod flags {
    /// The field carries no meaningful data (padding / filler).
    pub const CONSTANT: u8 = 1 << 0;
    /// Variable: each element represents one control. When clear, the field is
    /// an Array (each element *contains* a usage code rather than a state value).
    pub const VARIABLE: u8 = 1 << 1;
    /// Values are relative (e.g. mouse delta). When clear, absolute.
    pub const RELATIVE: u8 = 1 << 2;
    /// Values wrap around at the logical extents.
    pub const WRAP: u8 = 1 << 3;
    /// The control has a null state (out-of-range value means "not engaged").
    pub const NULL_STATE: u8 = 1 << 6;
}

// ── Parsed report field ────────────────────────────────────────────────────────

/// A single logical field decoded from a HID report descriptor.
///
/// Fields that share a usage page / page range but represent a **single control**
/// (axes, single-bit buttons with individual usages) have `count == 1`.
/// Fields that cover a **range** of usages (e.g., buttons 1–16) have `count > 1`
/// and sequential `usage_min..=usage_max`; use [`extract_u32`] / [`extract_bool`]
/// with the appropriate element index.
///
/// [`extract_u32`]: ReportField::extract_u32
/// [`extract_bool`]: ReportField::extract_bool
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReportField {
    /// Report ID this field belongs to (0 when the descriptor has no report IDs).
    pub report_id: u8,
    /// HID usage page (e.g. [`usage_page::GENERIC_DESKTOP`]).
    pub usage_page: u16,
    /// First (or only) usage for this field.
    pub usage_min: u16,
    /// Last usage for this field. Equals `usage_min` for single-usage fields.
    pub usage_max: u16,
    /// Bit offset of the first element from the start of the report payload
    /// (i.e. *after* the report-ID byte when [`ReportDescriptor::has_report_ids`] is true).
    pub bit_offset: u32,
    /// Bits per element.
    pub bit_size: u8,
    /// Number of consecutive elements (≥ 1).
    pub count: u16,
    /// Logical minimum (signed).
    pub logical_min: i32,
    /// Logical maximum (signed).
    pub logical_max: i32,
    /// Input item flags — see the [`flags`] module.
    pub flags: u8,
}

impl ReportField {
    /// True if this field is padding / carries no data.
    pub fn is_constant(&self) -> bool {
        self.flags & flags::CONSTANT != 0
    }
    /// True if each element represents one control (Variable). False ⇒ Array.
    pub fn is_variable(&self) -> bool {
        self.flags & flags::VARIABLE != 0
    }
    /// True if values are relative (e.g. mouse motion delta).
    pub fn is_relative(&self) -> bool {
        self.flags & flags::RELATIVE != 0
    }
    /// True if values should be interpreted as signed (`logical_min < 0`).
    pub fn is_signed(&self) -> bool {
        self.logical_min < 0
    }

    /// For a range field, return the element index (0-based) corresponding to `usage`.
    /// Returns `None` if `usage` is outside `usage_min..=usage_max`.
    pub fn index_of_usage(&self, usage: u16) -> Option<usize> {
        if usage >= self.usage_min && usage <= self.usage_max {
            Some((usage - self.usage_min) as usize)
        } else {
            None
        }
    }

    /// Extract the `index`-th element as an unsigned integer from `report_payload`.
    ///
    /// `report_payload` is the raw report bytes **without** any leading report-ID byte.
    /// Returns `None` if `index >= count` or the buffer is too short.
    pub fn extract_u32(&self, report_payload: &[u8], index: usize) -> Option<u32> {
        if index >= self.count as usize {
            return None;
        }
        let bit_start = self.bit_offset as usize + index * (self.bit_size as usize);
        extract_bits(report_payload, bit_start, self.bit_size as usize)
    }

    /// Extract the `index`-th element as a **signed** integer.
    ///
    /// Sign-extends from `bit_size` bits when `logical_min < 0`.
    pub fn extract_i32(&self, report_payload: &[u8], index: usize) -> Option<i32> {
        let raw = self.extract_u32(report_payload, index)?;
        if self.is_signed() && self.bit_size < 32 {
            let sign_bit = 1u32 << (self.bit_size - 1);
            if raw & sign_bit != 0 {
                return Some((raw | !((sign_bit << 1).wrapping_sub(1))) as i32);
            }
        }
        Some(raw as i32)
    }

    /// Extract the `index`-th element as a `bool` (non-zero ⇒ true).
    pub fn extract_bool(&self, report_payload: &[u8], index: usize) -> Option<bool> {
        self.extract_u32(report_payload, index).map(|v| v != 0)
    }
}

/// Extract `bit_count` bits starting at `bit_offset` from `data` (HID little-endian bit order).
fn extract_bits(data: &[u8], bit_offset: usize, bit_count: usize) -> Option<u32> {
    if bit_count == 0 || bit_count > 32 {
        return None;
    }
    let byte_start = bit_offset / 8;
    let byte_end = (bit_offset + bit_count + 7) / 8;
    if byte_end > data.len() {
        return None;
    }
    // Accumulate up to 5 bytes into a u64 (a 32-bit field can span 5 bytes).
    let mut val = 0u64;
    for i in 0..(byte_end - byte_start) {
        val |= (data[byte_start + i] as u64) << (i * 8);
    }
    val >>= bit_offset % 8;
    val &= (1u64 << bit_count) - 1;
    Some(val as u32)
}

// ── Parsed descriptor ─────────────────────────────────────────────────────────

/// Parsed HID report descriptor.
///
/// `N` is the maximum number of input fields to store. A typical gamepad
/// descriptor produces 8–20 fields; **32 is sufficient for most devices**.
///
/// Obtain one by calling [`ReportDescriptor::parse`].
#[derive(Debug)]
pub struct ReportDescriptor<const N: usize> {
    fields: [Option<ReportField>; N],
    count: usize,
    /// `true` when the descriptor uses Report ID items.
    /// Each packet from the device will then begin with a one-byte report ID.
    pub has_report_ids: bool,
}

impl<const N: usize> ReportDescriptor<N> {
    /// Parse a raw HID report descriptor byte slice.
    ///
    /// Fields beyond the `N`-th are silently dropped — increase `N` if needed.
    pub fn parse(descriptor: &[u8]) -> Self {
        let mut global = GlobalState::default();
        let mut local = LocalState::default();
        let mut stack = [GlobalState::DEFAULT; 4];
        let mut stack_depth: usize = 0;

        // Per-report-ID bit offsets for Input fields.
        // Index 0 is always for report_id = 0 (no report IDs used).
        let mut offsets: [(u8, u32); 16] = [(0, 0); 16];
        let mut offset_count: usize = 1;

        let mut result = ReportDescriptor {
            fields: [const { None }; N],
            count: 0,
            has_report_ids: false,
        };

        for item in ItemIter::new(descriptor) {
            match (item.item_type, item.tag) {
                // ── Global items ──────────────────────────────────────────
                (1, 0) => global.usage_page = item.data as u16,
                (1, 1) => global.logical_min = item.data_signed,
                (1, 2) => global.logical_max = item.data_signed,
                (1, 7) => global.report_size = item.data as u8,
                (1, 8) => {
                    global.report_id = item.data as u8;
                    result.has_report_ids = true;
                    let id = global.report_id;
                    if !offsets[..offset_count].iter().any(|(i, _)| *i == id) && offset_count < 16 {
                        offsets[offset_count] = (id, 0);
                        offset_count += 1;
                    }
                }
                (1, 9) => global.report_count = item.data as u16,
                (1, 10) => {
                    // Push
                    if stack_depth < stack.len() {
                        stack[stack_depth] = global.clone();
                        stack_depth += 1;
                    }
                }
                (1, 11) => {
                    // Pop
                    if stack_depth > 0 {
                        stack_depth -= 1;
                        global = stack[stack_depth].clone();
                    }
                }

                // ── Local items ───────────────────────────────────────────
                (2, 0) => {
                    // Usage (may encode page in high 16 bits for extended usages)
                    let (page, u) = if item.size > 2 {
                        ((item.data >> 16) as u16, (item.data & 0xFFFF) as u16)
                    } else {
                        (0, item.data as u16)
                    };
                    if local.usage_count < local.usages.len() {
                        // Pack page into high 16 bits (0 means "use global page")
                        local.usages[local.usage_count] = ((page as u32) << 16) | u as u32;
                        local.usage_count += 1;
                    }
                }
                (2, 1) => {
                    local.usage_min = item.data as u16;
                    local.has_usage_range = true;
                }
                (2, 2) => {
                    local.usage_max = item.data as u16;
                    local.has_usage_range = true;
                }

                // ── Main: Input ───────────────────────────────────────────
                (0, 8) => {
                    let item_flags = item.data as u8;
                    let rc = global.report_count as usize;
                    let rs = global.report_size;

                    if rc > 0 && rs > 0 {
                        // Advance the bit offset for this report_id, get the starting offset.
                        let bit_offset = offsets[..offset_count]
                            .iter_mut()
                            .find(|(id, _)| *id == global.report_id)
                            .map(|(_, off)| {
                                let start = *off;
                                *off += rc as u32 * rs as u32;
                                start
                            })
                            .unwrap_or(0);

                        // Decide how to emit fields.
                        // Rule: expand a Variable field into per-usage individual fields when
                        // we have an explicit usage list that exactly matches the count.
                        // This correctly handles non-sequential usages (e.g. X/Y/Z/Rz axes).
                        let expand = (item_flags & flags::VARIABLE != 0)
                            && local.usage_count > 0
                            && local.usage_count == rc
                            && local.usage_count <= local.usages.len();

                        if expand {
                            for (i, &packed) in local.usages[..local.usage_count].iter().enumerate() {
                                if result.count >= N {
                                    break;
                                }
                                let page = {
                                    let p = (packed >> 16) as u16;
                                    if p != 0 { p } else { global.usage_page }
                                };
                                let u = (packed & 0xFFFF) as u16;
                                result.fields[result.count] = Some(ReportField {
                                    report_id: global.report_id,
                                    usage_page: page,
                                    usage_min: u,
                                    usage_max: u,
                                    bit_offset: bit_offset + i as u32 * rs as u32,
                                    bit_size: rs,
                                    count: 1,
                                    logical_min: global.logical_min,
                                    logical_max: global.logical_max,
                                    flags: item_flags,
                                });
                                result.count += 1;
                            }
                        } else if result.count < N {
                            // Emit one field group (range-based or constant/array).
                            let (page, umin, umax) = if local.has_usage_range {
                                (global.usage_page, local.usage_min, local.usage_max)
                            } else if local.usage_count > 0 {
                                let first = local.usages[0];
                                let last = local.usages[local.usage_count - 1];
                                let p = {
                                    let hp = (first >> 16) as u16;
                                    if hp != 0 { hp } else { global.usage_page }
                                };
                                (p, (first & 0xFFFF) as u16, (last & 0xFFFF) as u16)
                            } else {
                                (global.usage_page, 0, 0)
                            };
                            result.fields[result.count] = Some(ReportField {
                                report_id: global.report_id,
                                usage_page: page,
                                usage_min: umin,
                                usage_max: umax,
                                bit_offset,
                                bit_size: rs,
                                count: rc as u16,
                                logical_min: global.logical_min,
                                logical_max: global.logical_max,
                                flags: item_flags,
                            });
                            result.count += 1;
                        }
                    }
                    local = LocalState::default();
                }

                // ── Main: Output / Feature — advance offsets, ignore ──────
                // (We only care about Input fields for device→host reading.)
                (0, 9) | (0, 11) => {
                    local = LocalState::default();
                }

                // ── Main: Collection / End Collection ─────────────────────
                (0, 10) | (0, 12) => {
                    local = LocalState::default();
                }

                _ => {}
            }
        }

        result
    }

    /// Iterate over all parsed Input fields (skips empty slots).
    pub fn fields(&self) -> impl Iterator<Item = &ReportField> {
        self.fields[..self.count].iter().filter_map(|f| f.as_ref())
    }

    /// Find the Input field whose usage range contains `usage` on the given
    /// `usage_page`, for the given `report_id`.
    ///
    /// Returns a reference to the field and the element index within it.
    pub fn find(&self, report_id: u8, page: u16, usage: u16) -> Option<(&ReportField, usize)> {
        self.fields().find_map(|f| {
            if f.report_id == report_id && f.usage_page == page {
                f.index_of_usage(usage).map(|idx| (f, idx))
            } else {
                None
            }
        })
    }

    /// Extract a **signed** value for the given usage from a complete report packet.
    ///
    /// `report` is the full packet received from the interrupt IN endpoint,
    /// including the leading report-ID byte if [`has_report_ids`] is `true`.
    ///
    /// Returns `None` if the field is not found, the buffer is too short, or
    /// (when `has_report_ids`) `report[0]` does not match `report_id`.
    ///
    /// [`has_report_ids`]: ReportDescriptor::has_report_ids
    pub fn extract_i32(&self, report: &[u8], report_id: u8, page: u16, usage: u16) -> Option<i32> {
        let payload = self.payload(report, report_id)?;
        let (field, idx) = self.find(report_id, page, usage)?;
        field.extract_i32(payload, idx)
    }

    /// Extract an **unsigned** value for the given usage.  See [`extract_i32`] for details.
    ///
    /// [`extract_i32`]: ReportDescriptor::extract_i32
    pub fn extract_u32(&self, report: &[u8], report_id: u8, page: u16, usage: u16) -> Option<u32> {
        let payload = self.payload(report, report_id)?;
        let (field, idx) = self.find(report_id, page, usage)?;
        field.extract_u32(payload, idx)
    }

    /// Extract a **boolean** value for the given usage (non-zero ⇒ `true`).
    /// Convenient for button fields.
    pub fn extract_bool(&self, report: &[u8], report_id: u8, page: u16, usage: u16) -> Option<bool> {
        let payload = self.payload(report, report_id)?;
        let (field, idx) = self.find(report_id, page, usage)?;
        field.extract_bool(payload, idx)
    }

    /// Return the report payload slice, stripping the report-ID prefix when present.
    fn payload<'a>(&self, report: &'a [u8], report_id: u8) -> Option<&'a [u8]> {
        if self.has_report_ids {
            if report.first() != Some(&report_id) {
                return None;
            }
            Some(&report[1..])
        } else {
            Some(report)
        }
    }
}

// ── Internal parsing state ────────────────────────────────────────────────────

#[derive(Clone)]
struct GlobalState {
    usage_page: u16,
    logical_min: i32,
    logical_max: i32,
    report_size: u8,
    report_id: u8,
    report_count: u16,
}

impl GlobalState {
    const DEFAULT: Self = Self {
        usage_page: 0,
        logical_min: 0,
        logical_max: 0,
        report_size: 0,
        report_id: 0,
        report_count: 0,
    };
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::DEFAULT
    }
}

struct LocalState {
    /// Packed usages: high 16 bits = page (0 ⇒ use global page), low 16 bits = usage.
    usages: [u32; 16],
    usage_count: usize,
    usage_min: u16,
    usage_max: u16,
    has_usage_range: bool,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            usages: [0; 16],
            usage_count: 0,
            usage_min: 0,
            usage_max: 0,
            has_usage_range: false,
        }
    }
}

// ── HID descriptor item iterator ──────────────────────────────────────────────

struct Item {
    item_type: u8, // 0 = Main, 1 = Global, 2 = Local
    tag: u8,
    data: u32,        // unsigned interpretation
    data_signed: i32, // sign-extended interpretation
    size: u8,         // data byte count (0, 1, 2, or 4)
}

struct ItemIter<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ItemIter<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
}

impl<'a> Iterator for ItemIter<'a> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos >= self.bytes.len() {
                return None;
            }
            let prefix = self.bytes[self.pos];
            self.pos += 1;

            // Long item (prefix == 0xFE): skip entirely.
            if prefix == 0xFE {
                if self.pos >= self.bytes.len() {
                    return None;
                }
                let data_size = self.bytes[self.pos] as usize;
                self.pos += 2 + data_size; // skip bDataSize + bLongItemTag + data
                continue;
            }

            let size: usize = match prefix & 0x03 {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 4,
                _ => unreachable!(),
            };
            let item_type = (prefix >> 2) & 0x03;
            let tag = (prefix >> 4) & 0x0F;

            if self.pos + size > self.bytes.len() {
                return None;
            }

            let (data, data_signed) = match size {
                0 => (0u32, 0i32),
                1 => (self.bytes[self.pos] as u32, self.bytes[self.pos] as i8 as i32),
                2 => {
                    let v = u16::from_le_bytes([self.bytes[self.pos], self.bytes[self.pos + 1]]);
                    (v as u32, v as i16 as i32)
                }
                4 => {
                    let v = u32::from_le_bytes([
                        self.bytes[self.pos],
                        self.bytes[self.pos + 1],
                        self.bytes[self.pos + 2],
                        self.bytes[self.pos + 3],
                    ]);
                    (v, v as i32)
                }
                _ => unreachable!(),
            };
            self.pos += size;

            return Some(Item {
                item_type,
                tag,
                data,
                data_signed,
                size: size as u8,
            });
        }
    }
}
