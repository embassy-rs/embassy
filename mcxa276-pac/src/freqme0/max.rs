#[doc = "Register `MAX` reader"]
pub type R = crate::R<MaxSpec>;
#[doc = "Register `MAX` writer"]
pub type W = crate::W<MaxSpec>;
#[doc = "Field `MAX_VALUE` reader - Maximum Value"]
pub type MaxValueR = crate::FieldReader<u32>;
#[doc = "Field `MAX_VALUE` writer - Maximum Value"]
pub type MaxValueW<'a, REG> = crate::FieldWriter<'a, REG, 31, u32>;
impl R {
    #[doc = "Bits 0:30 - Maximum Value"]
    #[inline(always)]
    pub fn max_value(&self) -> MaxValueR {
        MaxValueR::new(self.bits & 0x7fff_ffff)
    }
}
impl W {
    #[doc = "Bits 0:30 - Maximum Value"]
    #[inline(always)]
    pub fn max_value(&mut self) -> MaxValueW<MaxSpec> {
        MaxValueW::new(self, 0)
    }
}
#[doc = "Maximum\n\nYou can [`read`](crate::Reg::read) this register and get [`max::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`max::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MaxSpec;
impl crate::RegisterSpec for MaxSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`max::R`](R) reader structure"]
impl crate::Readable for MaxSpec {}
#[doc = "`write(|w| ..)` method takes [`max::W`](W) writer structure"]
impl crate::Writable for MaxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MAX to value 0x7fff_ffff"]
impl crate::Resettable for MaxSpec {
    const RESET_VALUE: u32 = 0x7fff_ffff;
}
