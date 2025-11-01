#[doc = "Register `GPOLY` reader"]
pub type R = crate::R<GpolySpec>;
#[doc = "Register `GPOLY` writer"]
pub type W = crate::W<GpolySpec>;
#[doc = "Field `LOW` reader - Low Half-Word"]
pub type LowR = crate::FieldReader<u16>;
#[doc = "Field `LOW` writer - Low Half-Word"]
pub type LowW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `HIGH` reader - High Half-Word"]
pub type HighR = crate::FieldReader<u16>;
#[doc = "Field `HIGH` writer - High Half-Word"]
pub type HighW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Low Half-Word"]
    #[inline(always)]
    pub fn low(&self) -> LowR {
        LowR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - High Half-Word"]
    #[inline(always)]
    pub fn high(&self) -> HighR {
        HighR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Low Half-Word"]
    #[inline(always)]
    pub fn low(&mut self) -> LowW<GpolySpec> {
        LowW::new(self, 0)
    }
    #[doc = "Bits 16:31 - High Half-Word"]
    #[inline(always)]
    pub fn high(&mut self) -> HighW<GpolySpec> {
        HighW::new(self, 16)
    }
}
#[doc = "Polynomial\n\nYou can [`read`](crate::Reg::read) this register and get [`gpoly::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpoly::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GpolySpec;
impl crate::RegisterSpec for GpolySpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gpoly::R`](R) reader structure"]
impl crate::Readable for GpolySpec {}
#[doc = "`write(|w| ..)` method takes [`gpoly::W`](W) writer structure"]
impl crate::Writable for GpolySpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GPOLY to value 0x1021"]
impl crate::Resettable for GpolySpec {
    const RESET_VALUE: u32 = 0x1021;
}
