#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    mbc0_mem0_glbcfg: Mbc0Mem0Glbcfg,
    mbc0_mem1_glbcfg: Mbc0Mem1Glbcfg,
    mbc0_mem2_glbcfg: Mbc0Mem2Glbcfg,
    mbc0_mem3_glbcfg: Mbc0Mem3Glbcfg,
    _reserved4: [u8; 0x10],
    mbc0_memn_glbac0: Mbc0MemnGlbac0,
    mbc0_memn_glbac1: Mbc0MemnGlbac1,
    mbc0_memn_glbac2: Mbc0MemnGlbac2,
    mbc0_memn_glbac3: Mbc0MemnGlbac3,
    mbc0_memn_glbac4: Mbc0MemnGlbac4,
    mbc0_memn_glbac5: Mbc0MemnGlbac5,
    mbc0_memn_glbac6: Mbc0MemnGlbac6,
    mbc0_memn_glbac7: Mbc0MemnGlbac7,
    mbc0_dom0_mem0_blk_cfg_w0: Mbc0Dom0Mem0BlkCfgW0,
    mbc0_dom0_mem0_blk_cfg_w1: Mbc0Dom0Mem0BlkCfgW1,
    mbc0_dom0_mem0_blk_cfg_w2: Mbc0Dom0Mem0BlkCfgW2,
    mbc0_dom0_mem0_blk_cfg_w3: Mbc0Dom0Mem0BlkCfgW3,
    mbc0_dom0_mem0_blk_cfg_w4: Mbc0Dom0Mem0BlkCfgW4,
    mbc0_dom0_mem0_blk_cfg_w5: Mbc0Dom0Mem0BlkCfgW5,
    mbc0_dom0_mem0_blk_cfg_w6: Mbc0Dom0Mem0BlkCfgW6,
    mbc0_dom0_mem0_blk_cfg_w7: Mbc0Dom0Mem0BlkCfgW7,
    mbc0_dom0_mem0_blk_cfg_w8: Mbc0Dom0Mem0BlkCfgW8,
    mbc0_dom0_mem0_blk_cfg_w9: Mbc0Dom0Mem0BlkCfgW9,
    mbc0_dom0_mem0_blk_cfg_w10: Mbc0Dom0Mem0BlkCfgW10,
    mbc0_dom0_mem0_blk_cfg_w11: Mbc0Dom0Mem0BlkCfgW11,
    mbc0_dom0_mem0_blk_cfg_w12: Mbc0Dom0Mem0BlkCfgW12,
    mbc0_dom0_mem0_blk_cfg_w13: Mbc0Dom0Mem0BlkCfgW13,
    mbc0_dom0_mem0_blk_cfg_w14: Mbc0Dom0Mem0BlkCfgW14,
    mbc0_dom0_mem0_blk_cfg_w15: Mbc0Dom0Mem0BlkCfgW15,
    _reserved28: [u8; 0x0100],
    mbc0_dom0_mem1_blk_cfg_w0: Mbc0Dom0Mem1BlkCfgW0,
    mbc0_dom0_mem1_blk_cfg_w1: Mbc0Dom0Mem1BlkCfgW1,
    _reserved30: [u8; 0x20],
    mbc0_dom0_mem2_blk_cfg_w0: Mbc0Dom0Mem2BlkCfgW0,
}
impl RegisterBlock {
    #[doc = "0x00 - MBC Global Configuration Register"]
    #[inline(always)]
    pub const fn mbc0_mem0_glbcfg(&self) -> &Mbc0Mem0Glbcfg {
        &self.mbc0_mem0_glbcfg
    }
    #[doc = "0x04 - MBC Global Configuration Register"]
    #[inline(always)]
    pub const fn mbc0_mem1_glbcfg(&self) -> &Mbc0Mem1Glbcfg {
        &self.mbc0_mem1_glbcfg
    }
    #[doc = "0x08 - MBC Global Configuration Register"]
    #[inline(always)]
    pub const fn mbc0_mem2_glbcfg(&self) -> &Mbc0Mem2Glbcfg {
        &self.mbc0_mem2_glbcfg
    }
    #[doc = "0x0c - MBC Global Configuration Register"]
    #[inline(always)]
    pub const fn mbc0_mem3_glbcfg(&self) -> &Mbc0Mem3Glbcfg {
        &self.mbc0_mem3_glbcfg
    }
    #[doc = "0x20 - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac0(&self) -> &Mbc0MemnGlbac0 {
        &self.mbc0_memn_glbac0
    }
    #[doc = "0x24 - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac1(&self) -> &Mbc0MemnGlbac1 {
        &self.mbc0_memn_glbac1
    }
    #[doc = "0x28 - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac2(&self) -> &Mbc0MemnGlbac2 {
        &self.mbc0_memn_glbac2
    }
    #[doc = "0x2c - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac3(&self) -> &Mbc0MemnGlbac3 {
        &self.mbc0_memn_glbac3
    }
    #[doc = "0x30 - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac4(&self) -> &Mbc0MemnGlbac4 {
        &self.mbc0_memn_glbac4
    }
    #[doc = "0x34 - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac5(&self) -> &Mbc0MemnGlbac5 {
        &self.mbc0_memn_glbac5
    }
    #[doc = "0x38 - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac6(&self) -> &Mbc0MemnGlbac6 {
        &self.mbc0_memn_glbac6
    }
    #[doc = "0x3c - MBC Global Access Control"]
    #[inline(always)]
    pub const fn mbc0_memn_glbac7(&self) -> &Mbc0MemnGlbac7 {
        &self.mbc0_memn_glbac7
    }
    #[doc = "0x40 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w0(&self) -> &Mbc0Dom0Mem0BlkCfgW0 {
        &self.mbc0_dom0_mem0_blk_cfg_w0
    }
    #[doc = "0x44 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w1(&self) -> &Mbc0Dom0Mem0BlkCfgW1 {
        &self.mbc0_dom0_mem0_blk_cfg_w1
    }
    #[doc = "0x48 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w2(&self) -> &Mbc0Dom0Mem0BlkCfgW2 {
        &self.mbc0_dom0_mem0_blk_cfg_w2
    }
    #[doc = "0x4c - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w3(&self) -> &Mbc0Dom0Mem0BlkCfgW3 {
        &self.mbc0_dom0_mem0_blk_cfg_w3
    }
    #[doc = "0x50 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w4(&self) -> &Mbc0Dom0Mem0BlkCfgW4 {
        &self.mbc0_dom0_mem0_blk_cfg_w4
    }
    #[doc = "0x54 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w5(&self) -> &Mbc0Dom0Mem0BlkCfgW5 {
        &self.mbc0_dom0_mem0_blk_cfg_w5
    }
    #[doc = "0x58 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w6(&self) -> &Mbc0Dom0Mem0BlkCfgW6 {
        &self.mbc0_dom0_mem0_blk_cfg_w6
    }
    #[doc = "0x5c - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w7(&self) -> &Mbc0Dom0Mem0BlkCfgW7 {
        &self.mbc0_dom0_mem0_blk_cfg_w7
    }
    #[doc = "0x60 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w8(&self) -> &Mbc0Dom0Mem0BlkCfgW8 {
        &self.mbc0_dom0_mem0_blk_cfg_w8
    }
    #[doc = "0x64 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w9(&self) -> &Mbc0Dom0Mem0BlkCfgW9 {
        &self.mbc0_dom0_mem0_blk_cfg_w9
    }
    #[doc = "0x68 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w10(&self) -> &Mbc0Dom0Mem0BlkCfgW10 {
        &self.mbc0_dom0_mem0_blk_cfg_w10
    }
    #[doc = "0x6c - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w11(&self) -> &Mbc0Dom0Mem0BlkCfgW11 {
        &self.mbc0_dom0_mem0_blk_cfg_w11
    }
    #[doc = "0x70 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w12(&self) -> &Mbc0Dom0Mem0BlkCfgW12 {
        &self.mbc0_dom0_mem0_blk_cfg_w12
    }
    #[doc = "0x74 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w13(&self) -> &Mbc0Dom0Mem0BlkCfgW13 {
        &self.mbc0_dom0_mem0_blk_cfg_w13
    }
    #[doc = "0x78 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w14(&self) -> &Mbc0Dom0Mem0BlkCfgW14 {
        &self.mbc0_dom0_mem0_blk_cfg_w14
    }
    #[doc = "0x7c - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem0_blk_cfg_w15(&self) -> &Mbc0Dom0Mem0BlkCfgW15 {
        &self.mbc0_dom0_mem0_blk_cfg_w15
    }
    #[doc = "0x180 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem1_blk_cfg_w0(&self) -> &Mbc0Dom0Mem1BlkCfgW0 {
        &self.mbc0_dom0_mem1_blk_cfg_w0
    }
    #[doc = "0x184 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem1_blk_cfg_w1(&self) -> &Mbc0Dom0Mem1BlkCfgW1 {
        &self.mbc0_dom0_mem1_blk_cfg_w1
    }
    #[doc = "0x1a8 - MBC Memory Block Configuration Word"]
    #[inline(always)]
    pub const fn mbc0_dom0_mem2_blk_cfg_w0(&self) -> &Mbc0Dom0Mem2BlkCfgW0 {
        &self.mbc0_dom0_mem2_blk_cfg_w0
    }
}
#[doc = "MBC0_MEM0_GLBCFG (r) register accessor: MBC Global Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_mem0_glbcfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_mem0_glbcfg`] module"]
#[doc(alias = "MBC0_MEM0_GLBCFG")]
pub type Mbc0Mem0Glbcfg = crate::Reg<mbc0_mem0_glbcfg::Mbc0Mem0GlbcfgSpec>;
#[doc = "MBC Global Configuration Register"]
pub mod mbc0_mem0_glbcfg;
#[doc = "MBC0_MEM1_GLBCFG (r) register accessor: MBC Global Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_mem1_glbcfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_mem1_glbcfg`] module"]
#[doc(alias = "MBC0_MEM1_GLBCFG")]
pub type Mbc0Mem1Glbcfg = crate::Reg<mbc0_mem1_glbcfg::Mbc0Mem1GlbcfgSpec>;
#[doc = "MBC Global Configuration Register"]
pub mod mbc0_mem1_glbcfg;
#[doc = "MBC0_MEM2_GLBCFG (r) register accessor: MBC Global Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_mem2_glbcfg::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_mem2_glbcfg`] module"]
#[doc(alias = "MBC0_MEM2_GLBCFG")]
pub type Mbc0Mem2Glbcfg = crate::Reg<mbc0_mem2_glbcfg::Mbc0Mem2GlbcfgSpec>;
#[doc = "MBC Global Configuration Register"]
pub mod mbc0_mem2_glbcfg;
#[doc = "MBC0_MEM3_GLBCFG (rw) register accessor: MBC Global Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_mem3_glbcfg::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_mem3_glbcfg::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_mem3_glbcfg`] module"]
#[doc(alias = "MBC0_MEM3_GLBCFG")]
pub type Mbc0Mem3Glbcfg = crate::Reg<mbc0_mem3_glbcfg::Mbc0Mem3GlbcfgSpec>;
#[doc = "MBC Global Configuration Register"]
pub mod mbc0_mem3_glbcfg;
#[doc = "MBC0_MEMN_GLBAC0 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac0`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC0")]
pub type Mbc0MemnGlbac0 = crate::Reg<mbc0_memn_glbac0::Mbc0MemnGlbac0Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac0;
#[doc = "MBC0_MEMN_GLBAC1 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac1`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC1")]
pub type Mbc0MemnGlbac1 = crate::Reg<mbc0_memn_glbac1::Mbc0MemnGlbac1Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac1;
#[doc = "MBC0_MEMN_GLBAC2 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac2`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC2")]
pub type Mbc0MemnGlbac2 = crate::Reg<mbc0_memn_glbac2::Mbc0MemnGlbac2Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac2;
#[doc = "MBC0_MEMN_GLBAC3 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac3`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC3")]
pub type Mbc0MemnGlbac3 = crate::Reg<mbc0_memn_glbac3::Mbc0MemnGlbac3Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac3;
#[doc = "MBC0_MEMN_GLBAC4 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac4`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC4")]
pub type Mbc0MemnGlbac4 = crate::Reg<mbc0_memn_glbac4::Mbc0MemnGlbac4Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac4;
#[doc = "MBC0_MEMN_GLBAC5 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac5`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC5")]
pub type Mbc0MemnGlbac5 = crate::Reg<mbc0_memn_glbac5::Mbc0MemnGlbac5Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac5;
#[doc = "MBC0_MEMN_GLBAC6 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac6`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC6")]
pub type Mbc0MemnGlbac6 = crate::Reg<mbc0_memn_glbac6::Mbc0MemnGlbac6Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac6;
#[doc = "MBC0_MEMN_GLBAC7 (rw) register accessor: MBC Global Access Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_memn_glbac7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_memn_glbac7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_memn_glbac7`] module"]
#[doc(alias = "MBC0_MEMN_GLBAC7")]
pub type Mbc0MemnGlbac7 = crate::Reg<mbc0_memn_glbac7::Mbc0MemnGlbac7Spec>;
#[doc = "MBC Global Access Control"]
pub mod mbc0_memn_glbac7;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W0 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w0`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W0")]
pub type Mbc0Dom0Mem0BlkCfgW0 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w0::Mbc0Dom0Mem0BlkCfgW0Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w0;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W1 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w1`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W1")]
pub type Mbc0Dom0Mem0BlkCfgW1 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w1::Mbc0Dom0Mem0BlkCfgW1Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w1;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W2 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w2`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W2")]
pub type Mbc0Dom0Mem0BlkCfgW2 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w2::Mbc0Dom0Mem0BlkCfgW2Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w2;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W3 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w3`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W3")]
pub type Mbc0Dom0Mem0BlkCfgW3 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w3::Mbc0Dom0Mem0BlkCfgW3Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w3;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W4 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w4`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W4")]
pub type Mbc0Dom0Mem0BlkCfgW4 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w4::Mbc0Dom0Mem0BlkCfgW4Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w4;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W5 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w5`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W5")]
pub type Mbc0Dom0Mem0BlkCfgW5 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w5::Mbc0Dom0Mem0BlkCfgW5Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w5;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W6 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w6::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w6::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w6`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W6")]
pub type Mbc0Dom0Mem0BlkCfgW6 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w6::Mbc0Dom0Mem0BlkCfgW6Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w6;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W7 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w7::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w7::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w7`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W7")]
pub type Mbc0Dom0Mem0BlkCfgW7 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w7::Mbc0Dom0Mem0BlkCfgW7Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w7;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W8 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w8::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w8::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w8`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W8")]
pub type Mbc0Dom0Mem0BlkCfgW8 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w8::Mbc0Dom0Mem0BlkCfgW8Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w8;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W9 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w9::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w9::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w9`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W9")]
pub type Mbc0Dom0Mem0BlkCfgW9 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w9::Mbc0Dom0Mem0BlkCfgW9Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w9;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W10 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w10::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w10::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w10`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W10")]
pub type Mbc0Dom0Mem0BlkCfgW10 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w10::Mbc0Dom0Mem0BlkCfgW10Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w10;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W11 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w11::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w11::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w11`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W11")]
pub type Mbc0Dom0Mem0BlkCfgW11 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w11::Mbc0Dom0Mem0BlkCfgW11Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w11;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W12 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w12::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w12::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w12`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W12")]
pub type Mbc0Dom0Mem0BlkCfgW12 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w12::Mbc0Dom0Mem0BlkCfgW12Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w12;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W13 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w13::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w13::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w13`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W13")]
pub type Mbc0Dom0Mem0BlkCfgW13 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w13::Mbc0Dom0Mem0BlkCfgW13Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w13;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W14 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w14::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w14::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w14`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W14")]
pub type Mbc0Dom0Mem0BlkCfgW14 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w14::Mbc0Dom0Mem0BlkCfgW14Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w14;
#[doc = "MBC0_DOM0_MEM0_BLK_CFG_W15 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem0_blk_cfg_w15::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem0_blk_cfg_w15::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem0_blk_cfg_w15`] module"]
#[doc(alias = "MBC0_DOM0_MEM0_BLK_CFG_W15")]
pub type Mbc0Dom0Mem0BlkCfgW15 = crate::Reg<mbc0_dom0_mem0_blk_cfg_w15::Mbc0Dom0Mem0BlkCfgW15Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem0_blk_cfg_w15;
#[doc = "MBC0_DOM0_MEM1_BLK_CFG_W0 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem1_blk_cfg_w0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem1_blk_cfg_w0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem1_blk_cfg_w0`] module"]
#[doc(alias = "MBC0_DOM0_MEM1_BLK_CFG_W0")]
pub type Mbc0Dom0Mem1BlkCfgW0 = crate::Reg<mbc0_dom0_mem1_blk_cfg_w0::Mbc0Dom0Mem1BlkCfgW0Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem1_blk_cfg_w0;
#[doc = "MBC0_DOM0_MEM1_BLK_CFG_W1 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem1_blk_cfg_w1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem1_blk_cfg_w1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem1_blk_cfg_w1`] module"]
#[doc(alias = "MBC0_DOM0_MEM1_BLK_CFG_W1")]
pub type Mbc0Dom0Mem1BlkCfgW1 = crate::Reg<mbc0_dom0_mem1_blk_cfg_w1::Mbc0Dom0Mem1BlkCfgW1Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem1_blk_cfg_w1;
#[doc = "MBC0_DOM0_MEM2_BLK_CFG_W0 (rw) register accessor: MBC Memory Block Configuration Word\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_dom0_mem2_blk_cfg_w0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_dom0_mem2_blk_cfg_w0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mbc0_dom0_mem2_blk_cfg_w0`] module"]
#[doc(alias = "MBC0_DOM0_MEM2_BLK_CFG_W0")]
pub type Mbc0Dom0Mem2BlkCfgW0 = crate::Reg<mbc0_dom0_mem2_blk_cfg_w0::Mbc0Dom0Mem2BlkCfgW0Spec>;
#[doc = "MBC Memory Block Configuration Word"]
pub mod mbc0_dom0_mem2_blk_cfg_w0;
