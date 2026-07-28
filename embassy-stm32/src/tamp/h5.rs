//! H5-specific internal tamper field access.
//!
//! Every other supported TAMP version exposes internal tampers through an
//! indexed array accessor (`itampe(n)`, `itampf(n)`, ...). H5 instead exposes
//! them as individually named bit fields (`itamp1e()`, `itamp2e()`, ...), with
//! gaps at ITAMP10 and ITAMP14 (reserved, not physically present). This shims
//! them behind the same 0-based index used everywhere else in this driver,
//! where index `n` means ITAMP`(n + 1)`.

use super::itamp_fields;

pub const INTERNAL_TAMPERS: u8 = 15;

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
    11 => (itamp12e, set_itamp12e, itamp12ie, set_itamp12ie, itamp12f, itamp12mf, set_citamp12f),
    12 => (itamp13e, set_itamp13e, itamp13ie, set_itamp13ie, itamp13f, itamp13mf, set_citamp13f),
    14 => (itamp15e, set_itamp15e, itamp15ie, set_itamp15ie, itamp15f, itamp15mf, set_citamp15f),
}
