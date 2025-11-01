#[doc = "Register `SRAMRETLDO_CNTRL` reader"]
pub type R = crate::R<SramretldoCntrlSpec>;
#[doc = "Register `SRAMRETLDO_CNTRL` writer"]
pub type W = crate::W<SramretldoCntrlSpec>;
#[doc = "SRAM LDO Regulator Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SramldoOn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<SramldoOn> for bool {
    #[inline(always)]
    fn from(variant: SramldoOn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SRAMLDO_ON` reader - SRAM LDO Regulator Enable"]
pub type SramldoOnR = crate::BitReader<SramldoOn>;
impl SramldoOnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SramldoOn {
        match self.bits {
            false => SramldoOn::Disable,
            true => SramldoOn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SramldoOn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SramldoOn::Enable
    }
}
#[doc = "Field `SRAMLDO_ON` writer - SRAM LDO Regulator Enable"]
pub type SramldoOnW<'a, REG> = crate::BitWriter<'a, REG, SramldoOn>;
impl<'a, REG> SramldoOnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SramldoOn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SramldoOn::Enable)
    }
}
#[doc = "Field `SRAM_RET_EN` reader - SRAM Retention"]
pub type SramRetEnR = crate::FieldReader;
#[doc = "Field `SRAM_RET_EN` writer - SRAM Retention"]
pub type SramRetEnW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bit 0 - SRAM LDO Regulator Enable"]
    #[inline(always)]
    pub fn sramldo_on(&self) -> SramldoOnR {
        SramldoOnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 8:11 - SRAM Retention"]
    #[inline(always)]
    pub fn sram_ret_en(&self) -> SramRetEnR {
        SramRetEnR::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - SRAM LDO Regulator Enable"]
    #[inline(always)]
    pub fn sramldo_on(&mut self) -> SramldoOnW<SramretldoCntrlSpec> {
        SramldoOnW::new(self, 0)
    }
    #[doc = "Bits 8:11 - SRAM Retention"]
    #[inline(always)]
    pub fn sram_ret_en(&mut self) -> SramRetEnW<SramretldoCntrlSpec> {
        SramRetEnW::new(self, 8)
    }
}
#[doc = "SRAM Retention LDO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sramretldo_cntrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sramretldo_cntrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SramretldoCntrlSpec;
impl crate::RegisterSpec for SramretldoCntrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sramretldo_cntrl::R`](R) reader structure"]
impl crate::Readable for SramretldoCntrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sramretldo_cntrl::W`](W) writer structure"]
impl crate::Writable for SramretldoCntrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SRAMRETLDO_CNTRL to value 0x0f01"]
impl crate::Resettable for SramretldoCntrlSpec {
    const RESET_VALUE: u32 = 0x0f01;
}
