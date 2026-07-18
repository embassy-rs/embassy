//! H5-specific internal tamper field access.
//!
//! Every other supported TAMP version exposes internal tampers through an
//! indexed array accessor (`itampe(n)`, `itampf(n)`, ...). H5 instead exposes
//! them as individually named bit fields (`itamp1e()`, `itamp2e()`, ...), with
//! gaps at ITAMP10 and ITAMP14 (reserved, not physically present). This shims
//! them behind the same 0-based index used everywhere else in this driver,
//! where index `n` means ITAMP`(n + 1)`.

use crate::pac::tamp::regs::{Cr1, Ier, Misr, Scr, Sr};

pub const INTERNAL_TAMPERS: u8 = 15;

pub fn cr1_itampe(r: Cr1, n: usize) -> bool {
    match n {
        0 => r.itamp1e(),
        1 => r.itamp2e(),
        2 => r.itamp3e(),
        3 => r.itamp4e(),
        4 => r.itamp5e(),
        5 => r.itamp6e(),
        6 => r.itamp7e(),
        7 => r.itamp8e(),
        8 => r.itamp9e(),
        10 => r.itamp11e(),
        11 => r.itamp12e(),
        12 => r.itamp13e(),
        14 => r.itamp15e(),
        _ => false,
    }
}

pub fn cr1_set_itampe(w: &mut Cr1, n: usize, val: bool) {
    match n {
        0 => w.set_itamp1e(val),
        1 => w.set_itamp2e(val),
        2 => w.set_itamp3e(val),
        3 => w.set_itamp4e(val),
        4 => w.set_itamp5e(val),
        5 => w.set_itamp6e(val),
        6 => w.set_itamp7e(val),
        7 => w.set_itamp8e(val),
        8 => w.set_itamp9e(val),
        10 => w.set_itamp11e(val),
        11 => w.set_itamp12e(val),
        12 => w.set_itamp13e(val),
        14 => w.set_itamp15e(val),
        _ => panic!("invalid internal tamper index {} for this chip", n),
    }
}

pub fn ier_itampie(r: Ier, n: usize) -> bool {
    match n {
        0 => r.itamp1ie(),
        1 => r.itamp2ie(),
        2 => r.itamp3ie(),
        3 => r.itamp4ie(),
        4 => r.itamp5ie(),
        5 => r.itamp6ie(),
        6 => r.itamp7ie(),
        7 => r.itamp8ie(),
        8 => r.itamp9ie(),
        10 => r.itamp11ie(),
        11 => r.itamp12ie(),
        12 => r.itamp13ie(),
        14 => r.itamp15ie(),
        _ => false,
    }
}

pub fn ier_set_itampie(w: &mut Ier, n: usize, val: bool) {
    match n {
        0 => w.set_itamp1ie(val),
        1 => w.set_itamp2ie(val),
        2 => w.set_itamp3ie(val),
        3 => w.set_itamp4ie(val),
        4 => w.set_itamp5ie(val),
        5 => w.set_itamp6ie(val),
        6 => w.set_itamp7ie(val),
        7 => w.set_itamp8ie(val),
        8 => w.set_itamp9ie(val),
        10 => w.set_itamp11ie(val),
        11 => w.set_itamp12ie(val),
        12 => w.set_itamp13ie(val),
        14 => w.set_itamp15ie(val),
        _ => {}
    }
}

pub fn sr_itampf(r: Sr, n: usize) -> bool {
    match n {
        0 => r.itamp1f(),
        1 => r.itamp2f(),
        2 => r.itamp3f(),
        3 => r.itamp4f(),
        4 => r.itamp5f(),
        5 => r.itamp6f(),
        6 => r.itamp7f(),
        7 => r.itamp8f(),
        8 => r.itamp9f(),
        10 => r.itamp11f(),
        11 => r.itamp12f(),
        12 => r.itamp13f(),
        14 => r.itamp15f(),
        _ => false,
    }
}

pub fn misr_itampmf(r: Misr, n: usize) -> bool {
    match n {
        0 => r.itamp1mf(),
        1 => r.itamp2mf(),
        2 => r.itamp3mf(),
        3 => r.itamp4mf(),
        4 => r.itamp5mf(),
        5 => r.itamp6mf(),
        6 => r.itamp7mf(),
        7 => r.itamp8mf(),
        8 => r.itamp9mf(),
        10 => r.itamp11mf(),
        11 => r.itamp12mf(),
        12 => r.itamp13mf(),
        14 => r.itamp15mf(),
        _ => false,
    }
}

pub fn scr_set_citampf(w: &mut Scr, n: usize, val: bool) {
    match n {
        0 => w.set_citamp1f(val),
        1 => w.set_citamp2f(val),
        2 => w.set_citamp3f(val),
        3 => w.set_citamp4f(val),
        4 => w.set_citamp5f(val),
        5 => w.set_citamp6f(val),
        6 => w.set_citamp7f(val),
        7 => w.set_citamp8f(val),
        8 => w.set_citamp9f(val),
        10 => w.set_citamp11f(val),
        11 => w.set_citamp12f(val),
        12 => w.set_citamp13f(val),
        14 => w.set_citamp15f(val),
        _ => {}
    }
}
