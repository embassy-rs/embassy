#[doc = "Register `CTRL2_PN` reader"]
pub type R = crate::R<Ctrl2PnSpec>;
#[doc = "Register `CTRL2_PN` writer"]
pub type W = crate::W<Ctrl2PnSpec>;
#[doc = "Field `MATCHTO` reader - Timeout for No Message Matching the Filtering Criteria"]
pub type MatchtoR = crate::FieldReader<u16>;
#[doc = "Field `MATCHTO` writer - Timeout for No Message Matching the Filtering Criteria"]
pub type MatchtoW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Timeout for No Message Matching the Filtering Criteria"]
    #[inline(always)]
    pub fn matchto(&self) -> MatchtoR {
        MatchtoR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Timeout for No Message Matching the Filtering Criteria"]
    #[inline(always)]
    pub fn matchto(&mut self) -> MatchtoW<Ctrl2PnSpec> {
        MatchtoW::new(self, 0)
    }
}
#[doc = "Pretended Networking Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl2_pn::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl2_pn::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl2PnSpec;
impl crate::RegisterSpec for Ctrl2PnSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl2_pn::R`](R) reader structure"]
impl crate::Readable for Ctrl2PnSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl2_pn::W`](W) writer structure"]
impl crate::Writable for Ctrl2PnSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL2_PN to value 0"]
impl crate::Resettable for Ctrl2PnSpec {}
