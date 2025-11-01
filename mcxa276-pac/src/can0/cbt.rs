#[doc = "Register `CBT` reader"]
pub type R = crate::R<CbtSpec>;
#[doc = "Register `CBT` writer"]
pub type W = crate::W<CbtSpec>;
#[doc = "Field `EPSEG2` reader - Extended Phase Segment 2"]
pub type Epseg2R = crate::FieldReader;
#[doc = "Field `EPSEG2` writer - Extended Phase Segment 2"]
pub type Epseg2W<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `EPSEG1` reader - Extended Phase Segment 1"]
pub type Epseg1R = crate::FieldReader;
#[doc = "Field `EPSEG1` writer - Extended Phase Segment 1"]
pub type Epseg1W<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `EPROPSEG` reader - Extended Propagation Segment"]
pub type EpropsegR = crate::FieldReader;
#[doc = "Field `EPROPSEG` writer - Extended Propagation Segment"]
pub type EpropsegW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `ERJW` reader - Extended Resync Jump Width"]
pub type ErjwR = crate::FieldReader;
#[doc = "Field `ERJW` writer - Extended Resync Jump Width"]
pub type ErjwW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `EPRESDIV` reader - Extended Prescaler Division Factor"]
pub type EpresdivR = crate::FieldReader<u16>;
#[doc = "Field `EPRESDIV` writer - Extended Prescaler Division Factor"]
pub type EpresdivW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Bit Timing Format Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btf {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Btf> for bool {
    #[inline(always)]
    fn from(variant: Btf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BTF` reader - Bit Timing Format Enable"]
pub type BtfR = crate::BitReader<Btf>;
impl BtfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Btf {
        match self.bits {
            false => Btf::Disable,
            true => Btf::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Btf::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Btf::Enable
    }
}
#[doc = "Field `BTF` writer - Bit Timing Format Enable"]
pub type BtfW<'a, REG> = crate::BitWriter<'a, REG, Btf>;
impl<'a, REG> BtfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Btf::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Btf::Enable)
    }
}
impl R {
    #[doc = "Bits 0:4 - Extended Phase Segment 2"]
    #[inline(always)]
    pub fn epseg2(&self) -> Epseg2R {
        Epseg2R::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 5:9 - Extended Phase Segment 1"]
    #[inline(always)]
    pub fn epseg1(&self) -> Epseg1R {
        Epseg1R::new(((self.bits >> 5) & 0x1f) as u8)
    }
    #[doc = "Bits 10:15 - Extended Propagation Segment"]
    #[inline(always)]
    pub fn epropseg(&self) -> EpropsegR {
        EpropsegR::new(((self.bits >> 10) & 0x3f) as u8)
    }
    #[doc = "Bits 16:20 - Extended Resync Jump Width"]
    #[inline(always)]
    pub fn erjw(&self) -> ErjwR {
        ErjwR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bits 21:30 - Extended Prescaler Division Factor"]
    #[inline(always)]
    pub fn epresdiv(&self) -> EpresdivR {
        EpresdivR::new(((self.bits >> 21) & 0x03ff) as u16)
    }
    #[doc = "Bit 31 - Bit Timing Format Enable"]
    #[inline(always)]
    pub fn btf(&self) -> BtfR {
        BtfR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Extended Phase Segment 2"]
    #[inline(always)]
    pub fn epseg2(&mut self) -> Epseg2W<CbtSpec> {
        Epseg2W::new(self, 0)
    }
    #[doc = "Bits 5:9 - Extended Phase Segment 1"]
    #[inline(always)]
    pub fn epseg1(&mut self) -> Epseg1W<CbtSpec> {
        Epseg1W::new(self, 5)
    }
    #[doc = "Bits 10:15 - Extended Propagation Segment"]
    #[inline(always)]
    pub fn epropseg(&mut self) -> EpropsegW<CbtSpec> {
        EpropsegW::new(self, 10)
    }
    #[doc = "Bits 16:20 - Extended Resync Jump Width"]
    #[inline(always)]
    pub fn erjw(&mut self) -> ErjwW<CbtSpec> {
        ErjwW::new(self, 16)
    }
    #[doc = "Bits 21:30 - Extended Prescaler Division Factor"]
    #[inline(always)]
    pub fn epresdiv(&mut self) -> EpresdivW<CbtSpec> {
        EpresdivW::new(self, 21)
    }
    #[doc = "Bit 31 - Bit Timing Format Enable"]
    #[inline(always)]
    pub fn btf(&mut self) -> BtfW<CbtSpec> {
        BtfW::new(self, 31)
    }
}
#[doc = "CAN Bit Timing\n\nYou can [`read`](crate::Reg::read) this register and get [`cbt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cbt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CbtSpec;
impl crate::RegisterSpec for CbtSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cbt::R`](R) reader structure"]
impl crate::Readable for CbtSpec {}
#[doc = "`write(|w| ..)` method takes [`cbt::W`](W) writer structure"]
impl crate::Writable for CbtSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CBT to value 0"]
impl crate::Resettable for CbtSpec {}
