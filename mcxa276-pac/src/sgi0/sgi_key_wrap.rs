#[doc = "Register `sgi_key_wrap` reader"]
pub type R = crate::R<SgiKeyWrapSpec>;
#[doc = "Field `kw_data` reader - Field contains wrapped key, auto-updated by HW for each word"]
pub type KwDataR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Field contains wrapped key, auto-updated by HW for each word"]
    #[inline(always)]
    pub fn kw_data(&self) -> KwDataR {
        KwDataR::new(self.bits)
    }
}
#[doc = "Wrapped key read SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_key_wrap::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiKeyWrapSpec;
impl crate::RegisterSpec for SgiKeyWrapSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_key_wrap::R`](R) reader structure"]
impl crate::Readable for SgiKeyWrapSpec {}
#[doc = "`reset()` method sets sgi_key_wrap to value 0"]
impl crate::Resettable for SgiKeyWrapSpec {}
