#[doc = "Register `MIN` reader"]
pub type R = crate::R<MinSpec>;
#[doc = "Register `MIN` writer"]
pub type W = crate::W<MinSpec>;
#[doc = "Field `MIN_VALUE` reader - Minimum Value"]
pub type MinValueR = crate::FieldReader<u32>;
#[doc = "Field `MIN_VALUE` writer - Minimum Value"]
pub type MinValueW<'a, REG> = crate::FieldWriter<'a, REG, 31, u32>;
impl R {
    #[doc = "Bits 0:30 - Minimum Value"]
    #[inline(always)]
    pub fn min_value(&self) -> MinValueR {
        MinValueR::new(self.bits & 0x7fff_ffff)
    }
}
impl W {
    #[doc = "Bits 0:30 - Minimum Value"]
    #[inline(always)]
    pub fn min_value(&mut self) -> MinValueW<MinSpec> {
        MinValueW::new(self, 0)
    }
}
#[doc = "Minimum\n\nYou can [`read`](crate::Reg::read) this register and get [`min::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`min::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MinSpec;
impl crate::RegisterSpec for MinSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`min::R`](R) reader structure"]
impl crate::Readable for MinSpec {}
#[doc = "`write(|w| ..)` method takes [`min::W`](W) writer structure"]
impl crate::Writable for MinSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MIN to value 0"]
impl crate::Resettable for MinSpec {}
