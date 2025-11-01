#[doc = "Register `ENCBT` reader"]
pub type R = crate::R<EncbtSpec>;
#[doc = "Register `ENCBT` writer"]
pub type W = crate::W<EncbtSpec>;
#[doc = "Field `NTSEG1` reader - Nominal Time Segment 1"]
pub type Ntseg1R = crate::FieldReader;
#[doc = "Field `NTSEG1` writer - Nominal Time Segment 1"]
pub type Ntseg1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `NTSEG2` reader - Nominal Time Segment 2"]
pub type Ntseg2R = crate::FieldReader;
#[doc = "Field `NTSEG2` writer - Nominal Time Segment 2"]
pub type Ntseg2W<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "Field `NRJW` reader - Nominal Resynchronization Jump Width"]
pub type NrjwR = crate::FieldReader;
#[doc = "Field `NRJW` writer - Nominal Resynchronization Jump Width"]
pub type NrjwW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bits 0:7 - Nominal Time Segment 1"]
    #[inline(always)]
    pub fn ntseg1(&self) -> Ntseg1R {
        Ntseg1R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 12:18 - Nominal Time Segment 2"]
    #[inline(always)]
    pub fn ntseg2(&self) -> Ntseg2R {
        Ntseg2R::new(((self.bits >> 12) & 0x7f) as u8)
    }
    #[doc = "Bits 22:28 - Nominal Resynchronization Jump Width"]
    #[inline(always)]
    pub fn nrjw(&self) -> NrjwR {
        NrjwR::new(((self.bits >> 22) & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Nominal Time Segment 1"]
    #[inline(always)]
    pub fn ntseg1(&mut self) -> Ntseg1W<EncbtSpec> {
        Ntseg1W::new(self, 0)
    }
    #[doc = "Bits 12:18 - Nominal Time Segment 2"]
    #[inline(always)]
    pub fn ntseg2(&mut self) -> Ntseg2W<EncbtSpec> {
        Ntseg2W::new(self, 12)
    }
    #[doc = "Bits 22:28 - Nominal Resynchronization Jump Width"]
    #[inline(always)]
    pub fn nrjw(&mut self) -> NrjwW<EncbtSpec> {
        NrjwW::new(self, 22)
    }
}
#[doc = "Enhanced Nominal CAN Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`encbt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`encbt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EncbtSpec;
impl crate::RegisterSpec for EncbtSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`encbt::R`](R) reader structure"]
impl crate::Readable for EncbtSpec {}
#[doc = "`write(|w| ..)` method takes [`encbt::W`](W) writer structure"]
impl crate::Writable for EncbtSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ENCBT to value 0"]
impl crate::Resettable for EncbtSpec {}
