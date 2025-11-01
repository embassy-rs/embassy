#[doc = "Register `MATCH_L` reader"]
pub type R = crate::R<MatchLSpec>;
#[doc = "Register `MATCH_L` writer"]
pub type W = crate::W<MatchLSpec>;
#[doc = "Field `MATCH_VALUE` reader - EVTimer Match Value"]
pub type MatchValueR = crate::FieldReader<u32>;
#[doc = "Field `MATCH_VALUE` writer - EVTimer Match Value"]
pub type MatchValueW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - EVTimer Match Value"]
    #[inline(always)]
    pub fn match_value(&self) -> MatchValueR {
        MatchValueR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - EVTimer Match Value"]
    #[inline(always)]
    pub fn match_value(&mut self) -> MatchValueW<MatchLSpec> {
        MatchValueW::new(self, 0)
    }
}
#[doc = "Local Match Low for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`match_l::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`match_l::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MatchLSpec;
impl crate::RegisterSpec for MatchLSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`match_l::R`](R) reader structure"]
impl crate::Readable for MatchLSpec {}
#[doc = "`write(|w| ..)` method takes [`match_l::W`](W) writer structure"]
impl crate::Writable for MatchLSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MATCH_L to value 0xffff_ffff"]
impl crate::Resettable for MatchLSpec {
    const RESET_VALUE: u32 = 0xffff_ffff;
}
