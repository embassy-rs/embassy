#[doc = "Register `EICHD0_WORD0` reader"]
pub type R = crate::R<Eichd0Word0Spec>;
#[doc = "Register `EICHD0_WORD0` writer"]
pub type W = crate::W<Eichd0Word0Spec>;
#[doc = "Field `CHKBIT_MASK` reader - Checkbit Mask"]
pub type ChkbitMaskR = crate::FieldReader;
#[doc = "Field `CHKBIT_MASK` writer - Checkbit Mask"]
pub type ChkbitMaskW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bits 25:31 - Checkbit Mask"]
    #[inline(always)]
    pub fn chkbit_mask(&self) -> ChkbitMaskR {
        ChkbitMaskR::new(((self.bits >> 25) & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 25:31 - Checkbit Mask"]
    #[inline(always)]
    pub fn chkbit_mask(&mut self) -> ChkbitMaskW<Eichd0Word0Spec> {
        ChkbitMaskW::new(self, 25)
    }
}
#[doc = "Error Injection Channel Descriptor 0, Word0\n\nYou can [`read`](crate::Reg::read) this register and get [`eichd0_word0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eichd0_word0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Eichd0Word0Spec;
impl crate::RegisterSpec for Eichd0Word0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`eichd0_word0::R`](R) reader structure"]
impl crate::Readable for Eichd0Word0Spec {}
#[doc = "`write(|w| ..)` method takes [`eichd0_word0::W`](W) writer structure"]
impl crate::Writable for Eichd0Word0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EICHD0_WORD0 to value 0"]
impl crate::Resettable for Eichd0Word0Spec {}
