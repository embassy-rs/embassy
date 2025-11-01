#[doc = "Register `sgi_prng_sw_seed` reader"]
pub type R = crate::R<SgiPrngSwSeedSpec>;
#[doc = "Register `sgi_prng_sw_seed` writer"]
pub type W = crate::W<SgiPrngSwSeedSpec>;
#[doc = "Field `seed` reader - 32-bits SEED field. A write to the SEED field"]
pub type SeedR = crate::FieldReader<u32>;
#[doc = "Field `seed` writer - 32-bits SEED field. A write to the SEED field"]
pub type SeedW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - 32-bits SEED field. A write to the SEED field"]
    #[inline(always)]
    pub fn seed(&self) -> SeedR {
        SeedR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - 32-bits SEED field. A write to the SEED field"]
    #[inline(always)]
    pub fn seed(&mut self) -> SeedW<SgiPrngSwSeedSpec> {
        SeedW::new(self, 0)
    }
}
#[doc = "SGI internal PRNG SW seeding register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_prng_sw_seed::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_prng_sw_seed::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiPrngSwSeedSpec;
impl crate::RegisterSpec for SgiPrngSwSeedSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_prng_sw_seed::R`](R) reader structure"]
impl crate::Readable for SgiPrngSwSeedSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_prng_sw_seed::W`](W) writer structure"]
impl crate::Writable for SgiPrngSwSeedSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_prng_sw_seed to value 0"]
impl crate::Resettable for SgiPrngSwSeedSpec {}
