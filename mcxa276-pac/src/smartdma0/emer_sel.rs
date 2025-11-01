#[doc = "Register `EMER_SEL` reader"]
pub type R = crate::R<EmerSelSpec>;
#[doc = "Register `EMER_SEL` writer"]
pub type W = crate::W<EmerSelSpec>;
#[doc = "Field `EN` reader - Emergency code routine"]
pub type EnR = crate::BitReader;
#[doc = "Field `EN` writer - Emergency code routine"]
pub type EnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RQ` reader - Software emergency request"]
pub type RqR = crate::BitReader;
#[doc = "Field `RQ` writer - Software emergency request"]
pub type RqW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 8 - Emergency code routine"]
    #[inline(always)]
    pub fn en(&self) -> EnR {
        EnR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Software emergency request"]
    #[inline(always)]
    pub fn rq(&self) -> RqR {
        RqR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - Emergency code routine"]
    #[inline(always)]
    pub fn en(&mut self) -> EnW<EmerSelSpec> {
        EnW::new(self, 8)
    }
    #[doc = "Bit 9 - Software emergency request"]
    #[inline(always)]
    pub fn rq(&mut self) -> RqW<EmerSelSpec> {
        RqW::new(self, 9)
    }
}
#[doc = "Emergency Select\n\nYou can [`read`](crate::Reg::read) this register and get [`emer_sel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`emer_sel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EmerSelSpec;
impl crate::RegisterSpec for EmerSelSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`emer_sel::R`](R) reader structure"]
impl crate::Readable for EmerSelSpec {}
#[doc = "`write(|w| ..)` method takes [`emer_sel::W`](W) writer structure"]
impl crate::Writable for EmerSelSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets EMER_SEL to value 0"]
impl crate::Resettable for EmerSelSpec {}
