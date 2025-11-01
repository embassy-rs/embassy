#[doc = "Register `sgi_dummy_ctrl` reader"]
pub type R = crate::R<SgiDummyCtrlSpec>;
#[doc = "Register `sgi_dummy_ctrl` writer"]
pub type W = crate::W<SgiDummyCtrlSpec>;
#[doc = "Field `ddctrl` reader - DES dummy control"]
pub type DdctrlR = crate::FieldReader<u16>;
#[doc = "Field `ddctrl` writer - DES dummy control"]
pub type DdctrlW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `dmyctl_rsvd2` reader - reserved"]
pub type DmyctlRsvd2R = crate::FieldReader;
#[doc = "Field `adctrl` reader - AES dummy control"]
pub type AdctrlR = crate::FieldReader<u16>;
#[doc = "Field `adctrl` writer - AES dummy control"]
pub type AdctrlW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `dmyctl_rsvd1` reader - reserved"]
pub type DmyctlRsvd1R = crate::FieldReader;
impl R {
    #[doc = "Bits 0:9 - DES dummy control"]
    #[inline(always)]
    pub fn ddctrl(&self) -> DdctrlR {
        DdctrlR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 10:15 - reserved"]
    #[inline(always)]
    pub fn dmyctl_rsvd2(&self) -> DmyctlRsvd2R {
        DmyctlRsvd2R::new(((self.bits >> 10) & 0x3f) as u8)
    }
    #[doc = "Bits 16:25 - AES dummy control"]
    #[inline(always)]
    pub fn adctrl(&self) -> AdctrlR {
        AdctrlR::new(((self.bits >> 16) & 0x03ff) as u16)
    }
    #[doc = "Bits 26:31 - reserved"]
    #[inline(always)]
    pub fn dmyctl_rsvd1(&self) -> DmyctlRsvd1R {
        DmyctlRsvd1R::new(((self.bits >> 26) & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:9 - DES dummy control"]
    #[inline(always)]
    pub fn ddctrl(&mut self) -> DdctrlW<SgiDummyCtrlSpec> {
        DdctrlW::new(self, 0)
    }
    #[doc = "Bits 16:25 - AES dummy control"]
    #[inline(always)]
    pub fn adctrl(&mut self) -> AdctrlW<SgiDummyCtrlSpec> {
        AdctrlW::new(self, 16)
    }
}
#[doc = "Configuration of dummy controls\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_dummy_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_dummy_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDummyCtrlSpec;
impl crate::RegisterSpec for SgiDummyCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_dummy_ctrl::R`](R) reader structure"]
impl crate::Readable for SgiDummyCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_dummy_ctrl::W`](W) writer structure"]
impl crate::Writable for SgiDummyCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_dummy_ctrl to value 0"]
impl crate::Resettable for SgiDummyCtrlSpec {}
