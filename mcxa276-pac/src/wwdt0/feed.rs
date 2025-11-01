#[doc = "Register `FEED` writer"]
pub type W = crate::W<FeedSpec>;
#[doc = "Field `FEED` writer - Feed Value"]
pub type FeedW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl W {
    #[doc = "Bits 0:7 - Feed Value"]
    #[inline(always)]
    pub fn feed(&mut self) -> FeedW<FeedSpec> {
        FeedW::new(self, 0)
    }
}
#[doc = "Feed Sequence\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`feed::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FeedSpec;
impl crate::RegisterSpec for FeedSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`feed::W`](W) writer structure"]
impl crate::Writable for FeedSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FEED to value 0"]
impl crate::Resettable for FeedSpec {}
