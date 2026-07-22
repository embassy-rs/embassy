//! N6-specific internal tamper field access.
//!
//! Like H5, N6 exposes internal tampers as individually named bit fields
//! (`itamp1e()`, `itamp2e()`, …) rather than an indexed array. Index `n`
//! maps to ITAMP`(n + 1)` with a gap at index 9 (ITAMP10 is not present).

use super::itamp_fields;

pub const INTERNAL_TAMPERS: u8 = 11;

itamp_fields! {
    0 => (itamp1e, set_itamp1e, itamp1ie, set_itamp1ie, itamp1f, itamp1mf, set_citamp1f),
    1 => (itamp2e, set_itamp2e, itamp2ie, set_itamp2ie, itamp2f, itamp2mf, set_citamp2f),
    2 => (itamp3e, set_itamp3e, itamp3ie, set_itamp3ie, itamp3f, itamp3mf, set_citamp3f),
    3 => (itamp4e, set_itamp4e, itamp4ie, set_itamp4ie, itamp4f, itamp4mf, set_citamp4f),
    4 => (itamp5e, set_itamp5e, itamp5ie, set_itamp5ie, itamp5f, itamp5mf, set_citamp5f),
    5 => (itamp6e, set_itamp6e, itamp6ie, set_itamp6ie, itamp6f, itamp6mf, set_citamp6f),
    6 => (itamp7e, set_itamp7e, itamp7ie, set_itamp7ie, itamp7f, itamp7mf, set_citamp7f),
    7 => (itamp8e, set_itamp8e, itamp8ie, set_itamp8ie, itamp8f, itamp8mf, set_citamp8f),
    8 => (itamp9e, set_itamp9e, itamp9ie, set_itamp9ie, itamp9f, itamp9mf, set_citamp9f),
    10 => (itamp11e, set_itamp11e, itamp11ie, set_itamp11ie, itamp11f, itamp11mf, set_citamp11f),
}
