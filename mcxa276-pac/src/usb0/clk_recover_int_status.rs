#[doc = "Register `CLK_RECOVER_INT_STATUS` reader"]
pub type R = crate::R<ClkRecoverIntStatusSpec>;
#[doc = "Register `CLK_RECOVER_INT_STATUS` writer"]
pub type W = crate::W<ClkRecoverIntStatusSpec>;
#[doc = "Overflow Error Interrupt Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvfError {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Unmasked interrupt occurred"]
    IntYes = 1,
}
impl From<OvfError> for bool {
    #[inline(always)]
    fn from(variant: OvfError) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OVF_ERROR` reader - Overflow Error Interrupt Status Flag"]
pub type OvfErrorR = crate::BitReader<OvfError>;
impl OvfErrorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvfError {
        match self.bits {
            false => OvfError::IntNo,
            true => OvfError::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == OvfError::IntNo
    }
    #[doc = "Unmasked interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == OvfError::IntYes
    }
}
#[doc = "Field `OVF_ERROR` writer - Overflow Error Interrupt Status Flag"]
pub type OvfErrorW<'a, REG> = crate::BitWriter1C<'a, REG, OvfError>;
impl<'a, REG> OvfErrorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(OvfError::IntNo)
    }
    #[doc = "Unmasked interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(OvfError::IntYes)
    }
}
impl R {
    #[doc = "Bit 4 - Overflow Error Interrupt Status Flag"]
    #[inline(always)]
    pub fn ovf_error(&self) -> OvfErrorR {
        OvfErrorR::new(((self.bits >> 4) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 4 - Overflow Error Interrupt Status Flag"]
    #[inline(always)]
    pub fn ovf_error(&mut self) -> OvfErrorW<ClkRecoverIntStatusSpec> {
        OvfErrorW::new(self, 4)
    }
}
#[doc = "Clock Recovery Separated Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_int_status::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_int_status::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ClkRecoverIntStatusSpec;
impl crate::RegisterSpec for ClkRecoverIntStatusSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`clk_recover_int_status::R`](R) reader structure"]
impl crate::Readable for ClkRecoverIntStatusSpec {}
#[doc = "`write(|w| ..)` method takes [`clk_recover_int_status::W`](W) writer structure"]
impl crate::Writable for ClkRecoverIntStatusSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u8 = 0x10;
}
#[doc = "`reset()` method sets CLK_RECOVER_INT_STATUS to value 0"]
impl crate::Resettable for ClkRecoverIntStatusSpec {}
