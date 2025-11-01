#[doc = "Register `SPLLSTAT` reader"]
pub type R = crate::R<SpllstatSpec>;
#[doc = "Pre-divider (N) ratio change acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ndivack {
    #[doc = "0: The Pre-divider (N) ratio change is not accepted by the analog PLL."]
    Disabled = 0,
    #[doc = "1: The Pre-divider (N) ratio change is accepted by the analog PLL."]
    Enabled = 1,
}
impl From<Ndivack> for bool {
    #[inline(always)]
    fn from(variant: Ndivack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NDIVACK` reader - Pre-divider (N) ratio change acknowledge"]
pub type NdivackR = crate::BitReader<Ndivack>;
impl NdivackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ndivack {
        match self.bits {
            false => Ndivack::Disabled,
            true => Ndivack::Enabled,
        }
    }
    #[doc = "The Pre-divider (N) ratio change is not accepted by the analog PLL."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ndivack::Disabled
    }
    #[doc = "The Pre-divider (N) ratio change is accepted by the analog PLL."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ndivack::Enabled
    }
}
#[doc = "Feedback (M) divider ratio change acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mdivack {
    #[doc = "0: The Feedback (M) ratio change is not accepted by the analog PLL."]
    Disabled = 0,
    #[doc = "1: The Feedback (M) ratio change is accepted by the analog PLL."]
    Enabled = 1,
}
impl From<Mdivack> for bool {
    #[inline(always)]
    fn from(variant: Mdivack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MDIVACK` reader - Feedback (M) divider ratio change acknowledge"]
pub type MdivackR = crate::BitReader<Mdivack>;
impl MdivackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mdivack {
        match self.bits {
            false => Mdivack::Disabled,
            true => Mdivack::Enabled,
        }
    }
    #[doc = "The Feedback (M) ratio change is not accepted by the analog PLL."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Mdivack::Disabled
    }
    #[doc = "The Feedback (M) ratio change is accepted by the analog PLL."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Mdivack::Enabled
    }
}
#[doc = "Post-divider (P) ratio change acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pdivack {
    #[doc = "0: The Post-divider (P) ratio change is not accepted by the analog PLL"]
    Disabled = 0,
    #[doc = "1: The Post-divider (P) ratio change is accepted by the analog PLL"]
    Enabled = 1,
}
impl From<Pdivack> for bool {
    #[inline(always)]
    fn from(variant: Pdivack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PDIVACK` reader - Post-divider (P) ratio change acknowledge"]
pub type PdivackR = crate::BitReader<Pdivack>;
impl PdivackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pdivack {
        match self.bits {
            false => Pdivack::Disabled,
            true => Pdivack::Enabled,
        }
    }
    #[doc = "The Post-divider (P) ratio change is not accepted by the analog PLL"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pdivack::Disabled
    }
    #[doc = "The Post-divider (P) ratio change is accepted by the analog PLL"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Pdivack::Enabled
    }
}
#[doc = "Free running detector (active high)\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frmdet {
    #[doc = "0: Free running is not detected"]
    Disabled = 0,
    #[doc = "1: Free running is detected"]
    Enabled = 1,
}
impl From<Frmdet> for bool {
    #[inline(always)]
    fn from(variant: Frmdet) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRMDET` reader - Free running detector (active high)"]
pub type FrmdetR = crate::BitReader<Frmdet>;
impl FrmdetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Frmdet {
        match self.bits {
            false => Frmdet::Disabled,
            true => Frmdet::Enabled,
        }
    }
    #[doc = "Free running is not detected"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Frmdet::Disabled
    }
    #[doc = "Free running is detected"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Frmdet::Enabled
    }
}
impl R {
    #[doc = "Bit 1 - Pre-divider (N) ratio change acknowledge"]
    #[inline(always)]
    pub fn ndivack(&self) -> NdivackR {
        NdivackR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Feedback (M) divider ratio change acknowledge"]
    #[inline(always)]
    pub fn mdivack(&self) -> MdivackR {
        MdivackR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Post-divider (P) ratio change acknowledge"]
    #[inline(always)]
    pub fn pdivack(&self) -> PdivackR {
        PdivackR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Free running detector (active high)"]
    #[inline(always)]
    pub fn frmdet(&self) -> FrmdetR {
        FrmdetR::new(((self.bits >> 4) & 1) != 0)
    }
}
#[doc = "SPLL Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllstat::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllstatSpec;
impl crate::RegisterSpec for SpllstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllstat::R`](R) reader structure"]
impl crate::Readable for SpllstatSpec {}
#[doc = "`reset()` method sets SPLLSTAT to value 0"]
impl crate::Resettable for SpllstatSpec {}
