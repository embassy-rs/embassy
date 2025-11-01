#[doc = "Register `CH_SBR` reader"]
pub type R = crate::R<ChSbrSpec>;
#[doc = "Register `CH_SBR` writer"]
pub type W = crate::W<ChSbrSpec>;
#[doc = "Field `MID` reader - Master ID"]
pub type MidR = crate::FieldReader;
#[doc = "Privileged Access Level\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pal {
    #[doc = "0: User protection level for DMA transfers"]
    UserProtection = 0,
    #[doc = "1: Privileged protection level for DMA transfers"]
    PrivilegedProtection = 1,
}
impl From<Pal> for bool {
    #[inline(always)]
    fn from(variant: Pal) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PAL` reader - Privileged Access Level"]
pub type PalR = crate::BitReader<Pal>;
impl PalR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pal {
        match self.bits {
            false => Pal::UserProtection,
            true => Pal::PrivilegedProtection,
        }
    }
    #[doc = "User protection level for DMA transfers"]
    #[inline(always)]
    pub fn is_user_protection(&self) -> bool {
        *self == Pal::UserProtection
    }
    #[doc = "Privileged protection level for DMA transfers"]
    #[inline(always)]
    pub fn is_privileged_protection(&self) -> bool {
        *self == Pal::PrivilegedProtection
    }
}
#[doc = "Enable Master ID Replication\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emi {
    #[doc = "0: Master ID replication is disabled"]
    Disable = 0,
    #[doc = "1: Master ID replication is enabled"]
    Enable = 1,
}
impl From<Emi> for bool {
    #[inline(always)]
    fn from(variant: Emi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EMI` reader - Enable Master ID Replication"]
pub type EmiR = crate::BitReader<Emi>;
impl EmiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Emi {
        match self.bits {
            false => Emi::Disable,
            true => Emi::Enable,
        }
    }
    #[doc = "Master ID replication is disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Emi::Disable
    }
    #[doc = "Master ID replication is enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Emi::Enable
    }
}
#[doc = "Field `EMI` writer - Enable Master ID Replication"]
pub type EmiW<'a, REG> = crate::BitWriter<'a, REG, Emi>;
impl<'a, REG> EmiW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Master ID replication is disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Emi::Disable)
    }
    #[doc = "Master ID replication is enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Emi::Enable)
    }
}
impl R {
    #[doc = "Bits 0:3 - Master ID"]
    #[inline(always)]
    pub fn mid(&self) -> MidR {
        MidR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 15 - Privileged Access Level"]
    #[inline(always)]
    pub fn pal(&self) -> PalR {
        PalR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Enable Master ID Replication"]
    #[inline(always)]
    pub fn emi(&self) -> EmiR {
        EmiR::new(((self.bits >> 16) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 16 - Enable Master ID Replication"]
    #[inline(always)]
    pub fn emi(&mut self) -> EmiW<ChSbrSpec> {
        EmiW::new(self, 16)
    }
}
#[doc = "Channel System Bus\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_sbr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_sbr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChSbrSpec;
impl crate::RegisterSpec for ChSbrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_sbr::R`](R) reader structure"]
impl crate::Readable for ChSbrSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_sbr::W`](W) writer structure"]
impl crate::Writable for ChSbrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CH_SBR to value 0x05"]
impl crate::Resettable for ChSbrSpec {
    const RESET_VALUE: u32 = 0x05;
}
