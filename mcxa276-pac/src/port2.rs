#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    verid: Verid,
    _reserved1: [u8; 0x0c],
    gpclr: Gpclr,
    gpchr: Gpchr,
    _reserved3: [u8; 0x08],
    config: Config,
    _reserved4: [u8; 0x5c],
    pcr0: Pcr0,
    pcr1: Pcr1,
    pcr2: Pcr2,
    pcr3: Pcr3,
    pcr4: Pcr4,
    pcr5: Pcr5,
    pcr6: Pcr6,
    pcr7: Pcr7,
    pcr8: Pcr8,
    pcr9: Pcr9,
    pcr10: Pcr10,
    pcr11: Pcr11,
    pcr12: Pcr12,
    pcr13: Pcr13,
    pcr14: Pcr14,
    pcr15: Pcr15,
    pcr16: Pcr16,
    pcr17: Pcr17,
    pcr18: Pcr18,
    pcr19: Pcr19,
    pcr20: Pcr20,
    pcr21: Pcr21,
    pcr22: Pcr22,
    pcr23: Pcr23,
    pcr24: Pcr24,
    pcr25: Pcr25,
    pcr26: Pcr26,
    pcr27: Pcr27,
    pcr28: Pcr28,
    pcr29: Pcr29,
    pcr30: Pcr30,
    pcr31: Pcr31,
}
impl RegisterBlock {
    #[doc = "0x00 - Version ID"]
    #[inline(always)]
    pub const fn verid(&self) -> &Verid {
        &self.verid
    }
    #[doc = "0x10 - Global Pin Control Low"]
    #[inline(always)]
    pub const fn gpclr(&self) -> &Gpclr {
        &self.gpclr
    }
    #[doc = "0x14 - Global Pin Control High"]
    #[inline(always)]
    pub const fn gpchr(&self) -> &Gpchr {
        &self.gpchr
    }
    #[doc = "0x20 - Configuration"]
    #[inline(always)]
    pub const fn config(&self) -> &Config {
        &self.config
    }
    #[doc = "0x80 - Pin Control 0"]
    #[inline(always)]
    pub const fn pcr0(&self) -> &Pcr0 {
        &self.pcr0
    }
    #[doc = "0x84 - Pin Control 1"]
    #[inline(always)]
    pub const fn pcr1(&self) -> &Pcr1 {
        &self.pcr1
    }
    #[doc = "0x88 - Pin Control 2"]
    #[inline(always)]
    pub const fn pcr2(&self) -> &Pcr2 {
        &self.pcr2
    }
    #[doc = "0x8c - Pin Control 3"]
    #[inline(always)]
    pub const fn pcr3(&self) -> &Pcr3 {
        &self.pcr3
    }
    #[doc = "0x90 - Pin Control 4"]
    #[inline(always)]
    pub const fn pcr4(&self) -> &Pcr4 {
        &self.pcr4
    }
    #[doc = "0x94 - Pin Control 5"]
    #[inline(always)]
    pub const fn pcr5(&self) -> &Pcr5 {
        &self.pcr5
    }
    #[doc = "0x98 - Pin Control 6"]
    #[inline(always)]
    pub const fn pcr6(&self) -> &Pcr6 {
        &self.pcr6
    }
    #[doc = "0x9c - Pin Control 7"]
    #[inline(always)]
    pub const fn pcr7(&self) -> &Pcr7 {
        &self.pcr7
    }
    #[doc = "0xa0 - Pin Control 8"]
    #[inline(always)]
    pub const fn pcr8(&self) -> &Pcr8 {
        &self.pcr8
    }
    #[doc = "0xa4 - Pin Control 9"]
    #[inline(always)]
    pub const fn pcr9(&self) -> &Pcr9 {
        &self.pcr9
    }
    #[doc = "0xa8 - Pin Control 10"]
    #[inline(always)]
    pub const fn pcr10(&self) -> &Pcr10 {
        &self.pcr10
    }
    #[doc = "0xac - Pin Control 11"]
    #[inline(always)]
    pub const fn pcr11(&self) -> &Pcr11 {
        &self.pcr11
    }
    #[doc = "0xb0 - Pin Control 12"]
    #[inline(always)]
    pub const fn pcr12(&self) -> &Pcr12 {
        &self.pcr12
    }
    #[doc = "0xb4 - Pin Control 13"]
    #[inline(always)]
    pub const fn pcr13(&self) -> &Pcr13 {
        &self.pcr13
    }
    #[doc = "0xb8 - Pin Control 14"]
    #[inline(always)]
    pub const fn pcr14(&self) -> &Pcr14 {
        &self.pcr14
    }
    #[doc = "0xbc - Pin Control 15"]
    #[inline(always)]
    pub const fn pcr15(&self) -> &Pcr15 {
        &self.pcr15
    }
    #[doc = "0xc0 - Pin Control 16"]
    #[inline(always)]
    pub const fn pcr16(&self) -> &Pcr16 {
        &self.pcr16
    }
    #[doc = "0xc4 - Pin Control 17"]
    #[inline(always)]
    pub const fn pcr17(&self) -> &Pcr17 {
        &self.pcr17
    }
    #[doc = "0xc8 - Pin Control 18"]
    #[inline(always)]
    pub const fn pcr18(&self) -> &Pcr18 {
        &self.pcr18
    }
    #[doc = "0xcc - Pin Control 19"]
    #[inline(always)]
    pub const fn pcr19(&self) -> &Pcr19 {
        &self.pcr19
    }
    #[doc = "0xd0 - Pin Control 20"]
    #[inline(always)]
    pub const fn pcr20(&self) -> &Pcr20 {
        &self.pcr20
    }
    #[doc = "0xd4 - Pin Control 21"]
    #[inline(always)]
    pub const fn pcr21(&self) -> &Pcr21 {
        &self.pcr21
    }
    #[doc = "0xd8 - Pin Control 22"]
    #[inline(always)]
    pub const fn pcr22(&self) -> &Pcr22 {
        &self.pcr22
    }
    #[doc = "0xdc - Pin Control 23"]
    #[inline(always)]
    pub const fn pcr23(&self) -> &Pcr23 {
        &self.pcr23
    }
    #[doc = "0xe0 - Pin Control 24"]
    #[inline(always)]
    pub const fn pcr24(&self) -> &Pcr24 {
        &self.pcr24
    }
    #[doc = "0xe4 - Pin Control 25"]
    #[inline(always)]
    pub const fn pcr25(&self) -> &Pcr25 {
        &self.pcr25
    }
    #[doc = "0xe8 - Pin Control 26"]
    #[inline(always)]
    pub const fn pcr26(&self) -> &Pcr26 {
        &self.pcr26
    }
    #[doc = "0xec - Pin Control 27"]
    #[inline(always)]
    pub const fn pcr27(&self) -> &Pcr27 {
        &self.pcr27
    }
    #[doc = "0xf0 - Pin Control 28"]
    #[inline(always)]
    pub const fn pcr28(&self) -> &Pcr28 {
        &self.pcr28
    }
    #[doc = "0xf4 - Pin Control 29"]
    #[inline(always)]
    pub const fn pcr29(&self) -> &Pcr29 {
        &self.pcr29
    }
    #[doc = "0xf8 - Pin Control 30"]
    #[inline(always)]
    pub const fn pcr30(&self) -> &Pcr30 {
        &self.pcr30
    }
    #[doc = "0xfc - Pin Control 31"]
    #[inline(always)]
    pub const fn pcr31(&self) -> &Pcr31 {
        &self.pcr31
    }
}
#[doc = "VERID (r) register accessor: Version ID\n\nYou can [`read`](crate::Reg::read) this register and get [`verid::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@verid`] module"]
#[doc(alias = "VERID")]
pub type Verid = crate::Reg<verid::VeridSpec>;
#[doc = "Version ID"]
pub mod verid;
#[doc = "GPCLR (rw) register accessor: Global Pin Control Low\n\nYou can [`read`](crate::Reg::read) this register and get [`gpclr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpclr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gpclr`] module"]
#[doc(alias = "GPCLR")]
pub type Gpclr = crate::Reg<gpclr::GpclrSpec>;
#[doc = "Global Pin Control Low"]
pub mod gpclr;
#[doc = "GPCHR (rw) register accessor: Global Pin Control High\n\nYou can [`read`](crate::Reg::read) this register and get [`gpchr::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpchr::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@gpchr`] module"]
#[doc(alias = "GPCHR")]
pub type Gpchr = crate::Reg<gpchr::GpchrSpec>;
#[doc = "Global Pin Control High"]
pub mod gpchr;
#[doc = "CONFIG (rw) register accessor: Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`config::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`config::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@config`] module"]
#[doc(alias = "CONFIG")]
pub type Config = crate::Reg<config::ConfigSpec>;
#[doc = "Configuration"]
pub mod config;
#[doc = "PCR0 (rw) register accessor: Pin Control 0\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr0`] module"]
#[doc(alias = "PCR0")]
pub type Pcr0 = crate::Reg<pcr0::Pcr0Spec>;
#[doc = "Pin Control 0"]
pub mod pcr0;
#[doc = "PCR1 (rw) register accessor: Pin Control 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr1`] module"]
#[doc(alias = "PCR1")]
pub type Pcr1 = crate::Reg<pcr1::Pcr1Spec>;
#[doc = "Pin Control 1"]
pub mod pcr1;
#[doc = "PCR2 (rw) register accessor: Pin Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr2`] module"]
#[doc(alias = "PCR2")]
pub type Pcr2 = crate::Reg<pcr2::Pcr2Spec>;
#[doc = "Pin Control 2"]
pub mod pcr2;
#[doc = "PCR3 (rw) register accessor: Pin Control 3\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr3`] module"]
#[doc(alias = "PCR3")]
pub type Pcr3 = crate::Reg<pcr3::Pcr3Spec>;
#[doc = "Pin Control 3"]
pub mod pcr3;
#[doc = "PCR4 (rw) register accessor: Pin Control 4\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr4`] module"]
#[doc(alias = "PCR4")]
pub type Pcr4 = crate::Reg<pcr4::Pcr4Spec>;
#[doc = "Pin Control 4"]
pub mod pcr4;
#[doc = "PCR5 (rw) register accessor: Pin Control 5\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr5`] module"]
#[doc(alias = "PCR5")]
pub type Pcr5 = crate::Reg<pcr5::Pcr5Spec>;
#[doc = "Pin Control 5"]
pub mod pcr5;
#[doc = "PCR6 (rw) register accessor: Pin Control 6\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr6`] module"]
#[doc(alias = "PCR6")]
pub type Pcr6 = crate::Reg<pcr6::Pcr6Spec>;
#[doc = "Pin Control 6"]
pub mod pcr6;
#[doc = "PCR7 (rw) register accessor: Pin Control 7\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr7`] module"]
#[doc(alias = "PCR7")]
pub type Pcr7 = crate::Reg<pcr7::Pcr7Spec>;
#[doc = "Pin Control 7"]
pub mod pcr7;
#[doc = "PCR8 (rw) register accessor: Pin Control 8\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr8`] module"]
#[doc(alias = "PCR8")]
pub type Pcr8 = crate::Reg<pcr8::Pcr8Spec>;
#[doc = "Pin Control 8"]
pub mod pcr8;
#[doc = "PCR9 (rw) register accessor: Pin Control 9\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr9`] module"]
#[doc(alias = "PCR9")]
pub type Pcr9 = crate::Reg<pcr9::Pcr9Spec>;
#[doc = "Pin Control 9"]
pub mod pcr9;
#[doc = "PCR10 (rw) register accessor: Pin Control 10\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr10`] module"]
#[doc(alias = "PCR10")]
pub type Pcr10 = crate::Reg<pcr10::Pcr10Spec>;
#[doc = "Pin Control 10"]
pub mod pcr10;
#[doc = "PCR11 (rw) register accessor: Pin Control 11\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr11`] module"]
#[doc(alias = "PCR11")]
pub type Pcr11 = crate::Reg<pcr11::Pcr11Spec>;
#[doc = "Pin Control 11"]
pub mod pcr11;
#[doc = "PCR12 (rw) register accessor: Pin Control 12\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr12`] module"]
#[doc(alias = "PCR12")]
pub type Pcr12 = crate::Reg<pcr12::Pcr12Spec>;
#[doc = "Pin Control 12"]
pub mod pcr12;
#[doc = "PCR13 (rw) register accessor: Pin Control 13\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr13`] module"]
#[doc(alias = "PCR13")]
pub type Pcr13 = crate::Reg<pcr13::Pcr13Spec>;
#[doc = "Pin Control 13"]
pub mod pcr13;
#[doc = "PCR14 (rw) register accessor: Pin Control 14\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr14`] module"]
#[doc(alias = "PCR14")]
pub type Pcr14 = crate::Reg<pcr14::Pcr14Spec>;
#[doc = "Pin Control 14"]
pub mod pcr14;
#[doc = "PCR15 (rw) register accessor: Pin Control 15\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr15`] module"]
#[doc(alias = "PCR15")]
pub type Pcr15 = crate::Reg<pcr15::Pcr15Spec>;
#[doc = "Pin Control 15"]
pub mod pcr15;
#[doc = "PCR16 (rw) register accessor: Pin Control 16\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr16::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr16::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr16`] module"]
#[doc(alias = "PCR16")]
pub type Pcr16 = crate::Reg<pcr16::Pcr16Spec>;
#[doc = "Pin Control 16"]
pub mod pcr16;
#[doc = "PCR17 (rw) register accessor: Pin Control 17\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr17::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr17::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr17`] module"]
#[doc(alias = "PCR17")]
pub type Pcr17 = crate::Reg<pcr17::Pcr17Spec>;
#[doc = "Pin Control 17"]
pub mod pcr17;
#[doc = "PCR18 (rw) register accessor: Pin Control 18\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr18::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr18::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr18`] module"]
#[doc(alias = "PCR18")]
pub type Pcr18 = crate::Reg<pcr18::Pcr18Spec>;
#[doc = "Pin Control 18"]
pub mod pcr18;
#[doc = "PCR19 (rw) register accessor: Pin Control 19\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr19::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr19::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr19`] module"]
#[doc(alias = "PCR19")]
pub type Pcr19 = crate::Reg<pcr19::Pcr19Spec>;
#[doc = "Pin Control 19"]
pub mod pcr19;
#[doc = "PCR20 (rw) register accessor: Pin Control 20\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr20::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr20::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr20`] module"]
#[doc(alias = "PCR20")]
pub type Pcr20 = crate::Reg<pcr20::Pcr20Spec>;
#[doc = "Pin Control 20"]
pub mod pcr20;
#[doc = "PCR21 (rw) register accessor: Pin Control 21\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr21::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr21::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr21`] module"]
#[doc(alias = "PCR21")]
pub type Pcr21 = crate::Reg<pcr21::Pcr21Spec>;
#[doc = "Pin Control 21"]
pub mod pcr21;
#[doc = "PCR22 (rw) register accessor: Pin Control 22\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr22::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr22::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr22`] module"]
#[doc(alias = "PCR22")]
pub type Pcr22 = crate::Reg<pcr22::Pcr22Spec>;
#[doc = "Pin Control 22"]
pub mod pcr22;
#[doc = "PCR23 (rw) register accessor: Pin Control 23\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr23::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr23::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr23`] module"]
#[doc(alias = "PCR23")]
pub type Pcr23 = crate::Reg<pcr23::Pcr23Spec>;
#[doc = "Pin Control 23"]
pub mod pcr23;
#[doc = "PCR24 (rw) register accessor: Pin Control 24\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr24::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr24::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr24`] module"]
#[doc(alias = "PCR24")]
pub type Pcr24 = crate::Reg<pcr24::Pcr24Spec>;
#[doc = "Pin Control 24"]
pub mod pcr24;
#[doc = "PCR25 (rw) register accessor: Pin Control 25\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr25::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr25::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr25`] module"]
#[doc(alias = "PCR25")]
pub type Pcr25 = crate::Reg<pcr25::Pcr25Spec>;
#[doc = "Pin Control 25"]
pub mod pcr25;
#[doc = "PCR26 (rw) register accessor: Pin Control 26\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr26::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr26::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr26`] module"]
#[doc(alias = "PCR26")]
pub type Pcr26 = crate::Reg<pcr26::Pcr26Spec>;
#[doc = "Pin Control 26"]
pub mod pcr26;
#[doc = "PCR27 (rw) register accessor: Pin Control 27\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr27::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr27::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr27`] module"]
#[doc(alias = "PCR27")]
pub type Pcr27 = crate::Reg<pcr27::Pcr27Spec>;
#[doc = "Pin Control 27"]
pub mod pcr27;
#[doc = "PCR28 (rw) register accessor: Pin Control 28\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr28::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr28::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr28`] module"]
#[doc(alias = "PCR28")]
pub type Pcr28 = crate::Reg<pcr28::Pcr28Spec>;
#[doc = "Pin Control 28"]
pub mod pcr28;
#[doc = "PCR29 (rw) register accessor: Pin Control 29\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr29::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr29::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr29`] module"]
#[doc(alias = "PCR29")]
pub type Pcr29 = crate::Reg<pcr29::Pcr29Spec>;
#[doc = "Pin Control 29"]
pub mod pcr29;
#[doc = "PCR30 (rw) register accessor: Pin Control 30\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr30::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr30::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr30`] module"]
#[doc(alias = "PCR30")]
pub type Pcr30 = crate::Reg<pcr30::Pcr30Spec>;
#[doc = "Pin Control 30"]
pub mod pcr30;
#[doc = "PCR31 (rw) register accessor: Pin Control 31\n\nYou can [`read`](crate::Reg::read) this register and get [`pcr31::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pcr31::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@pcr31`] module"]
#[doc(alias = "PCR31")]
pub type Pcr31 = crate::Reg<pcr31::Pcr31Spec>;
#[doc = "Pin Control 31"]
pub mod pcr31;
