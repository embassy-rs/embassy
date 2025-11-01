#[doc = "Register `GEXP_STATUS_IE` reader"]
pub type R = crate::R<GexpStatusIeSpec>;
#[doc = "Register `GEXP_STATUS_IE` writer"]
pub type W = crate::W<GexpStatusIeSpec>;
#[doc = "Direct operation Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorIe {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<ErrorIe> for bool {
    #[inline(always)]
    fn from(variant: ErrorIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERROR_IE` reader - Direct operation Error Interrupt Enable"]
pub type ErrorIeR = crate::BitReader<ErrorIe>;
impl ErrorIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ErrorIe {
        match self.bits {
            false => ErrorIe::Disable,
            true => ErrorIe::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == ErrorIe::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == ErrorIe::Enable
    }
}
#[doc = "Field `ERROR_IE` writer - Direct operation Error Interrupt Enable"]
pub type ErrorIeW<'a, REG> = crate::BitWriter<'a, REG, ErrorIe>;
impl<'a, REG> ErrorIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(ErrorIe::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(ErrorIe::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Direct operation Error Interrupt Enable"]
    #[inline(always)]
    pub fn error_ie(&self) -> ErrorIeR {
        ErrorIeR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Direct operation Error Interrupt Enable"]
    #[inline(always)]
    pub fn error_ie(&mut self) -> ErrorIeW<GexpStatusIeSpec> {
        ErrorIeW::new(self, 0)
    }
}
#[doc = "General Exception Status Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`gexp_status_ie::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gexp_status_ie::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GexpStatusIeSpec;
impl crate::RegisterSpec for GexpStatusIeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gexp_status_ie::R`](R) reader structure"]
impl crate::Readable for GexpStatusIeSpec {}
#[doc = "`write(|w| ..)` method takes [`gexp_status_ie::W`](W) writer structure"]
impl crate::Writable for GexpStatusIeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GEXP_STATUS_IE to value 0"]
impl crate::Resettable for GexpStatusIeSpec {}
