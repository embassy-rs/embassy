#[doc = "Register `sgi_int_status_set` reader"]
pub type R = crate::R<SgiIntStatusSetSpec>;
#[doc = "Register `sgi_int_status_set` writer"]
pub type W = crate::W<SgiIntStatusSetSpec>;
#[doc = "Field `int_set` reader - Write to set interrupt status flag (SGI_INT_STATUS.INT_PDONE=1) to trigger a SGI interrupt via software, e.g. for debug purposes."]
pub type IntSetR = crate::BitReader;
#[doc = "Field `int_set` writer - Write to set interrupt status flag (SGI_INT_STATUS.INT_PDONE=1) to trigger a SGI interrupt via software, e.g. for debug purposes."]
pub type IntSetW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `int_stss_rsvd` reader - reserved"]
pub type IntStssRsvdR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bit 0 - Write to set interrupt status flag (SGI_INT_STATUS.INT_PDONE=1) to trigger a SGI interrupt via software, e.g. for debug purposes."]
    #[inline(always)]
    pub fn int_set(&self) -> IntSetR {
        IntSetR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:31 - reserved"]
    #[inline(always)]
    pub fn int_stss_rsvd(&self) -> IntStssRsvdR {
        IntStssRsvdR::new((self.bits >> 1) & 0x7fff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - Write to set interrupt status flag (SGI_INT_STATUS.INT_PDONE=1) to trigger a SGI interrupt via software, e.g. for debug purposes."]
    #[inline(always)]
    pub fn int_set(&mut self) -> IntSetW<SgiIntStatusSetSpec> {
        IntSetW::new(self, 0)
    }
}
#[doc = "Interrupt status set\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_int_status_set::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_int_status_set::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiIntStatusSetSpec;
impl crate::RegisterSpec for SgiIntStatusSetSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_int_status_set::R`](R) reader structure"]
impl crate::Readable for SgiIntStatusSetSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_int_status_set::W`](W) writer structure"]
impl crate::Writable for SgiIntStatusSetSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_int_status_set to value 0"]
impl crate::Resettable for SgiIntStatusSetSpec {}
