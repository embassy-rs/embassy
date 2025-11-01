#[doc = "Register `SWD_ACCESS_CPU0` reader"]
pub type R = crate::R<SwdAccessCpu0Spec>;
#[doc = "Register `SWD_ACCESS_CPU0` writer"]
pub type W = crate::W<SwdAccessCpu0Spec>;
#[doc = "CPU0 SWD-AP: 0x12345678\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SecCode {
    #[doc = "0: CPU0 DAP is not allowed. Reading back register is read as 0x5."]
    Disable = 0,
    #[doc = "305419896: Value to write to enable CPU0 SWD access. Reading back register is read as 0xA."]
    Enable = 305419896,
}
impl From<SecCode> for u32 {
    #[inline(always)]
    fn from(variant: SecCode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for SecCode {
    type Ux = u32;
}
impl crate::IsEnum for SecCode {}
#[doc = "Field `SEC_CODE` reader - CPU0 SWD-AP: 0x12345678"]
pub type SecCodeR = crate::FieldReader<SecCode>;
impl SecCodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<SecCode> {
        match self.bits {
            0 => Some(SecCode::Disable),
            305419896 => Some(SecCode::Enable),
            _ => None,
        }
    }
    #[doc = "CPU0 DAP is not allowed. Reading back register is read as 0x5."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SecCode::Disable
    }
    #[doc = "Value to write to enable CPU0 SWD access. Reading back register is read as 0xA."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SecCode::Enable
    }
}
#[doc = "Field `SEC_CODE` writer - CPU0 SWD-AP: 0x12345678"]
pub type SecCodeW<'a, REG> = crate::FieldWriter<'a, REG, 32, SecCode>;
impl<'a, REG> SecCodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u32>,
{
    #[doc = "CPU0 DAP is not allowed. Reading back register is read as 0x5."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SecCode::Disable)
    }
    #[doc = "Value to write to enable CPU0 SWD access. Reading back register is read as 0xA."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SecCode::Enable)
    }
}
impl R {
    #[doc = "Bits 0:31 - CPU0 SWD-AP: 0x12345678"]
    #[inline(always)]
    pub fn sec_code(&self) -> SecCodeR {
        SecCodeR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - CPU0 SWD-AP: 0x12345678"]
    #[inline(always)]
    pub fn sec_code(&mut self) -> SecCodeW<SwdAccessCpu0Spec> {
        SecCodeW::new(self, 0)
    }
}
#[doc = "CPU0 Software Debug Access\n\nYou can [`read`](crate::Reg::read) this register and get [`swd_access_cpu0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swd_access_cpu0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwdAccessCpu0Spec;
impl crate::RegisterSpec for SwdAccessCpu0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`swd_access_cpu0::R`](R) reader structure"]
impl crate::Readable for SwdAccessCpu0Spec {}
#[doc = "`write(|w| ..)` method takes [`swd_access_cpu0::W`](W) writer structure"]
impl crate::Writable for SwdAccessCpu0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWD_ACCESS_CPU0 to value 0"]
impl crate::Resettable for SwdAccessCpu0Spec {}
