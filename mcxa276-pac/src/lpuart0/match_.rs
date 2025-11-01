#[doc = "Register `MATCH` reader"]
pub type R = crate::R<MatchSpec>;
#[doc = "Register `MATCH` writer"]
pub type W = crate::W<MatchSpec>;
#[doc = "Field `MA1` reader - Match Address 1"]
pub type Ma1R = crate::FieldReader<u16>;
#[doc = "Field `MA1` writer - Match Address 1"]
pub type Ma1W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `MA2` reader - Match Address 2"]
pub type Ma2R = crate::FieldReader<u16>;
#[doc = "Field `MA2` writer - Match Address 2"]
pub type Ma2W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Match Address 1"]
    #[inline(always)]
    pub fn ma1(&self) -> Ma1R {
        Ma1R::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 16:25 - Match Address 2"]
    #[inline(always)]
    pub fn ma2(&self) -> Ma2R {
        Ma2R::new(((self.bits >> 16) & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Match Address 1"]
    #[inline(always)]
    pub fn ma1(&mut self) -> Ma1W<MatchSpec> {
        Ma1W::new(self, 0)
    }
    #[doc = "Bits 16:25 - Match Address 2"]
    #[inline(always)]
    pub fn ma2(&mut self) -> Ma2W<MatchSpec> {
        Ma2W::new(self, 16)
    }
}
#[doc = "Match Address\n\nYou can [`read`](crate::Reg::read) this register and get [`match_::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`match_::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MatchSpec;
impl crate::RegisterSpec for MatchSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`match_::R`](R) reader structure"]
impl crate::Readable for MatchSpec {}
#[doc = "`write(|w| ..)` method takes [`match_::W`](W) writer structure"]
impl crate::Writable for MatchSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MATCH to value 0"]
impl crate::Resettable for MatchSpec {}
