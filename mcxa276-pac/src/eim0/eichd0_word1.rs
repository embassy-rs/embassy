#[doc = "Register `EICHD0_WORD1` reader"]
pub type R = crate::R<Eichd0Word1Spec>;
#[doc = "Register `EICHD0_WORD1` writer"]
pub type W = crate::W<Eichd0Word1Spec>;
#[doc = "Field `B0_3DATA_MASK` reader - Data Mask Bytes 0-3"]
pub type B0_3dataMaskR = crate::FieldReader<u32>;
#[doc = "Field `B0_3DATA_MASK` writer - Data Mask Bytes 0-3"]
pub type B0_3dataMaskW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Data Mask Bytes 0-3"]
    #[inline(always)]
    pub fn b0_3data_mask(&self) -> B0_3dataMaskR {
        B0_3dataMaskR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Data Mask Bytes 0-3"]
    #[inline(always)]
    pub fn b0_3data_mask(&mut self) -> B0_3dataMaskW<Eichd0Word1Spec> {
        B0_3dataMaskW::new(self, 0)
    }
}
#[doc = "Error Injection Channel Descriptor 0, Word1\n\nYou can [`read`](crate::Reg::read) this register and get [`eichd0_word1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`eichd0_word1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Eichd0Word1Spec;
impl crate::RegisterSpec for Eichd0Word1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`eichd0_word1::R`](R) reader structure"]
impl crate::Readable for Eichd0Word1Spec {}
#[doc = "`write(|w| ..)` method takes [`eichd0_word1::W`](W) writer structure"]
impl crate::Writable for Eichd0Word1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EICHD0_WORD1 to value 0"]
impl crate::Resettable for Eichd0Word1Spec {}
