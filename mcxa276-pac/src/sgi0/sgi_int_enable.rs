#[doc = "Register `sgi_int_enable` reader"]
pub type R = crate::R<SgiIntEnableSpec>;
#[doc = "Register `sgi_int_enable` writer"]
pub type W = crate::W<SgiIntEnableSpec>;
#[doc = "Field `int_en` reader - Interrupt enable bit"]
pub type IntEnR = crate::BitReader;
#[doc = "Field `int_en` writer - Interrupt enable bit"]
pub type IntEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `int_ena_rsvd` reader - reserved"]
pub type IntEnaRsvdR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bit 0 - Interrupt enable bit"]
    #[inline(always)]
    pub fn int_en(&self) -> IntEnR {
        IntEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:31 - reserved"]
    #[inline(always)]
    pub fn int_ena_rsvd(&self) -> IntEnaRsvdR {
        IntEnaRsvdR::new((self.bits >> 1) & 0x7fff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - Interrupt enable bit"]
    #[inline(always)]
    pub fn int_en(&mut self) -> IntEnW<SgiIntEnableSpec> {
        IntEnW::new(self, 0)
    }
}
#[doc = "Interrupt enable\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_int_enable::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_int_enable::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiIntEnableSpec;
impl crate::RegisterSpec for SgiIntEnableSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_int_enable::R`](R) reader structure"]
impl crate::Readable for SgiIntEnableSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_int_enable::W`](W) writer structure"]
impl crate::Writable for SgiIntEnableSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_int_enable to value 0"]
impl crate::Resettable for SgiIntEnableSpec {}
