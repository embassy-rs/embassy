#[doc = "Register `ID4` reader"]
pub type R = crate::R<MbGroupId4Spec>;
#[doc = "Register `ID4` writer"]
pub type W = crate::W<MbGroupId4Spec>;
#[doc = "Field `EXT` reader - Contains extended (LOW word) identifier of message buffer."]
pub type ExtR = crate::FieldReader<u32>;
#[doc = "Field `EXT` writer - Contains extended (LOW word) identifier of message buffer."]
pub type ExtW<'a, REG> = crate::FieldWriter<'a, REG, 18, u32>;
#[doc = "Field `STD` reader - Contains standard/extended (HIGH word) identifier of message buffer."]
pub type StdR = crate::FieldReader<u16>;
#[doc = "Field `STD` writer - Contains standard/extended (HIGH word) identifier of message buffer."]
pub type StdW<'a, REG> = crate::FieldWriter<'a, REG, 11, u16>;
#[doc = "Field `PRIO` reader - Local priority. This 3-bit fieldis only used when LPRIO_EN bit is set in MCR and it only makes sense for Tx buffers. These bits are not transmitted. They are appended to the regular ID to define the transmission priority."]
pub type PrioR = crate::FieldReader;
#[doc = "Field `PRIO` writer - Local priority. This 3-bit fieldis only used when LPRIO_EN bit is set in MCR and it only makes sense for Tx buffers. These bits are not transmitted. They are appended to the regular ID to define the transmission priority."]
pub type PrioW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:17 - Contains extended (LOW word) identifier of message buffer."]
    #[inline(always)]
    pub fn ext(&self) -> ExtR {
        ExtR::new(self.bits & 0x0003_ffff)
    }
    #[doc = "Bits 18:28 - Contains standard/extended (HIGH word) identifier of message buffer."]
    #[inline(always)]
    pub fn std(&self) -> StdR {
        StdR::new(((self.bits >> 18) & 0x07ff) as u16)
    }
    #[doc = "Bits 29:31 - Local priority. This 3-bit fieldis only used when LPRIO_EN bit is set in MCR and it only makes sense for Tx buffers. These bits are not transmitted. They are appended to the regular ID to define the transmission priority."]
    #[inline(always)]
    pub fn prio(&self) -> PrioR {
        PrioR::new(((self.bits >> 29) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:17 - Contains extended (LOW word) identifier of message buffer."]
    #[inline(always)]
    pub fn ext(&mut self) -> ExtW<MbGroupId4Spec> {
        ExtW::new(self, 0)
    }
    #[doc = "Bits 18:28 - Contains standard/extended (HIGH word) identifier of message buffer."]
    #[inline(always)]
    pub fn std(&mut self) -> StdW<MbGroupId4Spec> {
        StdW::new(self, 18)
    }
    #[doc = "Bits 29:31 - Local priority. This 3-bit fieldis only used when LPRIO_EN bit is set in MCR and it only makes sense for Tx buffers. These bits are not transmitted. They are appended to the regular ID to define the transmission priority."]
    #[inline(always)]
    pub fn prio(&mut self) -> PrioW<MbGroupId4Spec> {
        PrioW::new(self, 29)
    }
}
#[doc = "Message Buffer 4 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_group_id4::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_group_id4::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MbGroupId4Spec;
impl crate::RegisterSpec for MbGroupId4Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_group_id4::R`](R) reader structure"]
impl crate::Readable for MbGroupId4Spec {}
#[doc = "`write(|w| ..)` method takes [`mb_group_id4::W`](W) writer structure"]
impl crate::Writable for MbGroupId4Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ID4 to value 0"]
impl crate::Resettable for MbGroupId4Spec {}
