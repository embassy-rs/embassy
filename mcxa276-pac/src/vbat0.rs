#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    _reserved1: [u8; 0x01fc],
    froctla: Froctla,
    _reserved2: [u8; 0x14],
    frolcka: Frolcka,
    _reserved3: [u8; 0x04],
    froclke: Froclke,
    _reserved4: [u8; 0x04dc],
    wakeup: (),
    _reserved5: [u8; 0xf8],
    waklcka: Waklcka,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x200 - FRO16K Control A"]
    #[inline(always)]
    pub const fn froctla(&self) -> &Froctla {
        &self.froctla
    }
    #[doc = "0x218 - FRO16K Lock A"]
    #[inline(always)]
    pub const fn frolcka(&self) -> &Frolcka {
        &self.frolcka
    }
    #[doc = "0x220 - FRO16K Clock Enable"]
    #[inline(always)]
    pub const fn froclke(&self) -> &Froclke {
        &self.froclke
    }
    #[doc = "0x700..0x708 - Array of registers: WAKEUPA"]
    #[inline(always)]
    pub const fn wakeup(&self, n: usize) -> &Wakeup {
        #[allow(clippy::no_effect)]
        [(); 2][n];
        unsafe {
            &*core::ptr::from_ref(self)
                .cast::<u8>()
                .add(1792)
                .add(8 * n)
                .cast()
        }
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x700..0x708 - Array of registers: WAKEUPA"]
    #[inline(always)]
    pub fn wakeup_iter(&self) -> impl Iterator<Item = &Wakeup> {
        (0..2).map(move |n| unsafe {
            &*core::ptr::from_ref(self)
                .cast::<u8>()
                .add(1792)
                .add(8 * n)
                .cast()
        })
    }
    #[doc = "0x7f8 - Wakeup Lock A"]
    #[inline(always)]
    pub const fn waklcka(&self) -> &Waklcka {
        &self.waklcka
    }
}
#[doc = "VERID (r) register accessor: Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID"]
pub mod verid;
#[doc = "FROCTLA (rw) register accessor: FRO16K Control A\n\nYou can [`read`](crate::Reg::read) this register and get [`froctla::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`froctla::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@froctla`] module"]
#[doc(alias = "FROCTLA")]
pub type Froctla = crate::Reg<froctla::FroctlaSpec>;
#[doc = "FRO16K Control A"]
pub mod froctla;
#[doc = "FROLCKA (rw) register accessor: FRO16K Lock A\n\nYou can [`read`](crate::Reg::read) this register and get [`frolcka::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frolcka::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@frolcka`] module"]
#[doc(alias = "FROLCKA")]
pub type Frolcka = crate::Reg<frolcka::FrolckaSpec>;
#[doc = "FRO16K Lock A"]
pub mod frolcka;
#[doc = "FROCLKE (rw) register accessor: FRO16K Clock Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`froclke::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`froclke::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@froclke`] module"]
#[doc(alias = "FROCLKE")]
pub type Froclke = crate::Reg<froclke::FroclkeSpec>;
#[doc = "FRO16K Clock Enable"]
pub mod froclke;
#[doc = "Array of registers: WAKEUPA"]
pub use self::wakeup::Wakeup;
#[doc = r"Cluster"]
#[doc = "Array of registers: WAKEUPA"]
pub mod wakeup;
#[doc = "WAKLCKA (rw) register accessor: Wakeup Lock A\n\nYou can [`read`](crate::Reg::read) this register and get [`waklcka::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`waklcka::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@waklcka`] module"]
#[doc(alias = "WAKLCKA")]
pub type Waklcka = crate::Reg<waklcka::WaklckaSpec>;
#[doc = "Wakeup Lock A"]
pub mod waklcka;
