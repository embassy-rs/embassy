#[doc = "Register `ADDINFO` reader"]
pub type R = crate::R<AddinfoSpec>;
#[doc = "Host Mode Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Iehost {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Iehost> for bool {
    #[inline(always)]
    fn from(variant: Iehost) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IEHOST` reader - Host Mode Enable"]
pub type IehostR = crate::BitReader<Iehost>;
impl IehostR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Iehost {
        match self.bits {
            false => Iehost::Disabled,
            true => Iehost::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Iehost::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Iehost::Enabled
    }
}
impl R {
    #[doc = "Bit 0 - Host Mode Enable"]
    #[inline(always)]
    pub fn iehost(&self) -> IehostR {
        IehostR::new((self.bits & 1) != 0)
    }
}
#[doc = "Peripheral Additional Information\n\nYou can [`read`](crate::Reg::read) this register and get [`addinfo::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct AddinfoSpec;
impl crate::RegisterSpec for AddinfoSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`addinfo::R`](R) reader structure"]
impl crate::Readable for AddinfoSpec {}
#[doc = "`reset()` method sets ADDINFO to value 0x01"]
impl crate::Resettable for AddinfoSpec {
    const RESET_VALUE: u8 = 0x01;
}
