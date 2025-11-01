#[doc = "Register `LASTEDGE` reader"]
pub type R = crate::R<LastedgeSpec>;
#[doc = "Field `LASTEDGE` reader - Last Edge Time Counter"]
pub type LastedgeR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Last Edge Time Counter"]
    #[inline(always)]
    pub fn lastedge(&self) -> LastedgeR {
        LastedgeR::new(self.bits)
    }
}
#[doc = "Last Edge Time Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lastedge::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LastedgeSpec;
impl crate::RegisterSpec for LastedgeSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lastedge::R`](R) reader structure"]
impl crate::Readable for LastedgeSpec {}
#[doc = "`reset()` method sets LASTEDGE to value 0xffff"]
impl crate::Resettable for LastedgeSpec {
    const RESET_VALUE: u16 = 0xffff;
}
