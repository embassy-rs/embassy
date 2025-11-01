#[doc = "Register `MATCH_H` reader"]
pub type R = crate::R<MatchHSpec>;
#[doc = "Register `MATCH_H` writer"]
pub type W = crate::W<MatchHSpec>;
#[doc = "Field `MATCH_VALUE` reader - EVTimer Match Value"]
pub type MatchValueR = crate::FieldReader<u16>;
#[doc = "Field `MATCH_VALUE` writer - EVTimer Match Value"]
pub type MatchValueW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - EVTimer Match Value"]
    #[inline(always)]
    pub fn match_value(&self) -> MatchValueR {
        MatchValueR::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - EVTimer Match Value"]
    #[inline(always)]
    pub fn match_value(&mut self) -> MatchValueW<MatchHSpec> {
        MatchValueW::new(self, 0)
    }
}
#[doc = "Local Match High for CPU\n\nYou can [`read`](crate::Reg::read) this register and get [`match_h::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`match_h::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MatchHSpec;
impl crate::RegisterSpec for MatchHSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`match_h::R`](R) reader structure"]
impl crate::Readable for MatchHSpec {}
#[doc = "`write(|w| ..)` method takes [`match_h::W`](W) writer structure"]
impl crate::Writable for MatchHSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MATCH_H to value 0xffff_ffff"]
impl crate::Resettable for MatchHSpec {
    const RESET_VALUE: u32 = 0xffff_ffff;
}
