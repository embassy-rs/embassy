#[doc = "Register `SYS_CTLR` reader"]
pub type R = crate::R<SysCtlrSpec>;
#[doc = "Register `SYS_CTLR` writer"]
pub type W = crate::W<SysCtlrSpec>;
#[doc = "Automatic Clock Gating Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcgEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<AcgEn> for bool {
    #[inline(always)]
    fn from(variant: AcgEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ACG_EN` reader - Automatic Clock Gating Enable"]
pub type AcgEnR = crate::BitReader<AcgEn>;
impl AcgEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> AcgEn {
        match self.bits {
            false => AcgEn::Disable,
            true => AcgEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == AcgEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == AcgEn::Enable
    }
}
#[doc = "Field `ACG_EN` writer - Automatic Clock Gating Enable"]
pub type AcgEnW<'a, REG> = crate::BitWriter<'a, REG, AcgEn>;
impl<'a, REG> AcgEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(AcgEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(AcgEn::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Automatic Clock Gating Enable"]
    #[inline(always)]
    pub fn acg_en(&self) -> AcgEnR {
        AcgEnR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Automatic Clock Gating Enable"]
    #[inline(always)]
    pub fn acg_en(&mut self) -> AcgEnW<SysCtlrSpec> {
        AcgEnW::new(self, 0)
    }
}
#[doc = "System Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sys_ctlr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sys_ctlr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SysCtlrSpec;
impl crate::RegisterSpec for SysCtlrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sys_ctlr::R`](R) reader structure"]
impl crate::Readable for SysCtlrSpec {}
#[doc = "`write(|w| ..)` method takes [`sys_ctlr::W`](W) writer structure"]
impl crate::Writable for SysCtlrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SYS_CTLR to value 0x01"]
impl crate::Resettable for SysCtlrSpec {
    const RESET_VALUE: u32 = 0x01;
}
