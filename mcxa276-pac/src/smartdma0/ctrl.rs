#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "Field `START` reader - Start Bit Ignition"]
pub type StartR = crate::BitReader;
#[doc = "Field `START` writer - Start Bit Ignition"]
pub type StartW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EXF` reader - External Flag"]
pub type ExfR = crate::BitReader;
#[doc = "Field `EXF` writer - External Flag"]
pub type ExfW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ERRDIS` reader - Error Disable"]
pub type ErrdisR = crate::BitReader;
#[doc = "Field `ERRDIS` writer - Error Disable"]
pub type ErrdisW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BUFEN` reader - Buffer Enable"]
pub type BufenR = crate::BitReader;
#[doc = "Field `BUFEN` writer - Buffer Enable"]
pub type BufenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SYNCEN` reader - Sync Enable"]
pub type SyncenR = crate::BitReader;
#[doc = "Field `SYNCEN` writer - Sync Enable"]
pub type SyncenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `WKEY` reader - Write Key"]
pub type WkeyR = crate::FieldReader<u16>;
#[doc = "Field `WKEY` writer - Write Key"]
pub type WkeyW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bit 0 - Start Bit Ignition"]
    #[inline(always)]
    pub fn start(&self) -> StartR {
        StartR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - External Flag"]
    #[inline(always)]
    pub fn exf(&self) -> ExfR {
        ExfR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Error Disable"]
    #[inline(always)]
    pub fn errdis(&self) -> ErrdisR {
        ErrdisR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Buffer Enable"]
    #[inline(always)]
    pub fn bufen(&self) -> BufenR {
        BufenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Sync Enable"]
    #[inline(always)]
    pub fn syncen(&self) -> SyncenR {
        SyncenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 16:31 - Write Key"]
    #[inline(always)]
    pub fn wkey(&self) -> WkeyR {
        WkeyR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - Start Bit Ignition"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<CtrlSpec> {
        StartW::new(self, 0)
    }
    #[doc = "Bit 1 - External Flag"]
    #[inline(always)]
    pub fn exf(&mut self) -> ExfW<CtrlSpec> {
        ExfW::new(self, 1)
    }
    #[doc = "Bit 2 - Error Disable"]
    #[inline(always)]
    pub fn errdis(&mut self) -> ErrdisW<CtrlSpec> {
        ErrdisW::new(self, 2)
    }
    #[doc = "Bit 3 - Buffer Enable"]
    #[inline(always)]
    pub fn bufen(&mut self) -> BufenW<CtrlSpec> {
        BufenW::new(self, 3)
    }
    #[doc = "Bit 4 - Sync Enable"]
    #[inline(always)]
    pub fn syncen(&mut self) -> SyncenW<CtrlSpec> {
        SyncenW::new(self, 4)
    }
    #[doc = "Bits 16:31 - Write Key"]
    #[inline(always)]
    pub fn wkey(&mut self) -> WkeyW<CtrlSpec> {
        WkeyW::new(self, 16)
    }
}
#[doc = "Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtrlSpec;
impl crate::RegisterSpec for CtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl::R`](R) reader structure"]
impl crate::Readable for CtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl::W`](W) writer structure"]
impl crate::Writable for CtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL to value 0"]
impl crate::Resettable for CtrlSpec {}
