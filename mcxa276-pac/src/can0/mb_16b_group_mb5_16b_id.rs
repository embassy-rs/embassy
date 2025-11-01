#[doc = "Register `MB5_16B_ID` reader"]
pub type R = crate::R<Mb16bGroupMb5_16bIdSpec>;
#[doc = "Register `MB5_16B_ID` writer"]
pub type W = crate::W<Mb16bGroupMb5_16bIdSpec>;
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
    pub fn ext(&mut self) -> ExtW<Mb16bGroupMb5_16bIdSpec> {
        ExtW::new(self, 0)
    }
    #[doc = "Bits 18:28 - Contains standard/extended (HIGH word) identifier of message buffer."]
    #[inline(always)]
    pub fn std(&mut self) -> StdW<Mb16bGroupMb5_16bIdSpec> {
        StdW::new(self, 18)
    }
    #[doc = "Bits 29:31 - Local priority. This 3-bit fieldis only used when LPRIO_EN bit is set in MCR and it only makes sense for Tx buffers. These bits are not transmitted. They are appended to the regular ID to define the transmission priority."]
    #[inline(always)]
    pub fn prio(&mut self) -> PrioW<Mb16bGroupMb5_16bIdSpec> {
        PrioW::new(self, 29)
    }
}
#[doc = "Message Buffer 5 ID Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mb_16b_group_mb5_16b_id::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mb_16b_group_mb5_16b_id::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mb16bGroupMb5_16bIdSpec;
impl crate::RegisterSpec for Mb16bGroupMb5_16bIdSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mb_16b_group_mb5_16b_id::R`](R) reader structure"]
impl crate::Readable for Mb16bGroupMb5_16bIdSpec {}
#[doc = "`write(|w| ..)` method takes [`mb_16b_group_mb5_16b_id::W`](W) writer structure"]
impl crate::Writable for Mb16bGroupMb5_16bIdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MB5_16B_ID to value 0"]
impl crate::Resettable for Mb16bGroupMb5_16bIdSpec {}
