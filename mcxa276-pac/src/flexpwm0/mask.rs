#[doc = "Register `MASK` reader"]
pub type R = crate::R<MaskSpec>;
#[doc = "Register `MASK` writer"]
pub type W = crate::W<MaskSpec>;
#[doc = "Field `MASKX` reader - PWM_X Masks"]
pub type MaskxR = crate::FieldReader;
#[doc = "Field `MASKX` writer - PWM_X Masks"]
pub type MaskxW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `MASKB` reader - PWM_B Masks"]
pub type MaskbR = crate::FieldReader;
#[doc = "Field `MASKB` writer - PWM_B Masks"]
pub type MaskbW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `MASKA` reader - PWM_A Masks"]
pub type MaskaR = crate::FieldReader;
#[doc = "Field `MASKA` writer - PWM_A Masks"]
pub type MaskaW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `UPDATE_MASK` writer - Update Mask Bits Immediately"]
pub type UpdateMaskW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - PWM_X Masks"]
    #[inline(always)]
    pub fn maskx(&self) -> MaskxR {
        MaskxR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - PWM_B Masks"]
    #[inline(always)]
    pub fn maskb(&self) -> MaskbR {
        MaskbR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - PWM_A Masks"]
    #[inline(always)]
    pub fn maska(&self) -> MaskaR {
        MaskaR::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - PWM_X Masks"]
    #[inline(always)]
    pub fn maskx(&mut self) -> MaskxW<MaskSpec> {
        MaskxW::new(self, 0)
    }
    #[doc = "Bits 4:7 - PWM_B Masks"]
    #[inline(always)]
    pub fn maskb(&mut self) -> MaskbW<MaskSpec> {
        MaskbW::new(self, 4)
    }
    #[doc = "Bits 8:11 - PWM_A Masks"]
    #[inline(always)]
    pub fn maska(&mut self) -> MaskaW<MaskSpec> {
        MaskaW::new(self, 8)
    }
    #[doc = "Bits 12:15 - Update Mask Bits Immediately"]
    #[inline(always)]
    pub fn update_mask(&mut self) -> UpdateMaskW<MaskSpec> {
        UpdateMaskW::new(self, 12)
    }
}
#[doc = "Mask Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MaskSpec;
impl crate::RegisterSpec for MaskSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`mask::R`](R) reader structure"]
impl crate::Readable for MaskSpec {}
#[doc = "`write(|w| ..)` method takes [`mask::W`](W) writer structure"]
impl crate::Writable for MaskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MASK to value 0"]
impl crate::Resettable for MaskSpec {}
