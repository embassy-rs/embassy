#[doc = "Register `SDYNADDR` reader"]
pub type R = crate::R<SdynaddrSpec>;
#[doc = "Register `SDYNADDR` writer"]
pub type W = crate::W<SdynaddrSpec>;
#[doc = "Dynamic Address Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Davalid {
    #[doc = "0: DANOTASSIGNED: a dynamic address is not assigned"]
    Danotassigned = 0,
    #[doc = "1: DAASSIGNED: a dynamic address is assigned"]
    Daassigned = 1,
}
impl From<Davalid> for bool {
    #[inline(always)]
    fn from(variant: Davalid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DAVALID` reader - Dynamic Address Valid"]
pub type DavalidR = crate::BitReader<Davalid>;
impl DavalidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Davalid {
        match self.bits {
            false => Davalid::Danotassigned,
            true => Davalid::Daassigned,
        }
    }
    #[doc = "DANOTASSIGNED: a dynamic address is not assigned"]
    #[inline(always)]
    pub fn is_danotassigned(&self) -> bool {
        *self == Davalid::Danotassigned
    }
    #[doc = "DAASSIGNED: a dynamic address is assigned"]
    #[inline(always)]
    pub fn is_daassigned(&self) -> bool {
        *self == Davalid::Daassigned
    }
}
#[doc = "Field `DAVALID` writer - Dynamic Address Valid"]
pub type DavalidW<'a, REG> = crate::BitWriter<'a, REG, Davalid>;
impl<'a, REG> DavalidW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "DANOTASSIGNED: a dynamic address is not assigned"]
    #[inline(always)]
    pub fn danotassigned(self) -> &'a mut crate::W<REG> {
        self.variant(Davalid::Danotassigned)
    }
    #[doc = "DAASSIGNED: a dynamic address is assigned"]
    #[inline(always)]
    pub fn daassigned(self) -> &'a mut crate::W<REG> {
        self.variant(Davalid::Daassigned)
    }
}
#[doc = "Field `DADDR` reader - Dynamic Address"]
pub type DaddrR = crate::FieldReader;
#[doc = "Field `DADDR` writer - Dynamic Address"]
pub type DaddrW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "Field `MAPSA` writer - Map a Static Address"]
pub type MapsaW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SA10B` writer - 10-Bit Static Address"]
pub type Sa10bW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `KEY` reader - Key"]
pub type KeyR = crate::FieldReader<u16>;
#[doc = "Field `KEY` writer - Key"]
pub type KeyW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bit 0 - Dynamic Address Valid"]
    #[inline(always)]
    pub fn davalid(&self) -> DavalidR {
        DavalidR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:7 - Dynamic Address"]
    #[inline(always)]
    pub fn daddr(&self) -> DaddrR {
        DaddrR::new(((self.bits >> 1) & 0x7f) as u8)
    }
    #[doc = "Bits 16:31 - Key"]
    #[inline(always)]
    pub fn key(&self) -> KeyR {
        KeyR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - Dynamic Address Valid"]
    #[inline(always)]
    pub fn davalid(&mut self) -> DavalidW<SdynaddrSpec> {
        DavalidW::new(self, 0)
    }
    #[doc = "Bits 1:7 - Dynamic Address"]
    #[inline(always)]
    pub fn daddr(&mut self) -> DaddrW<SdynaddrSpec> {
        DaddrW::new(self, 1)
    }
    #[doc = "Bit 12 - Map a Static Address"]
    #[inline(always)]
    pub fn mapsa(&mut self) -> MapsaW<SdynaddrSpec> {
        MapsaW::new(self, 12)
    }
    #[doc = "Bits 13:15 - 10-Bit Static Address"]
    #[inline(always)]
    pub fn sa10b(&mut self) -> Sa10bW<SdynaddrSpec> {
        Sa10bW::new(self, 13)
    }
    #[doc = "Bits 16:31 - Key"]
    #[inline(always)]
    pub fn key(&mut self) -> KeyW<SdynaddrSpec> {
        KeyW::new(self, 16)
    }
}
#[doc = "Target Dynamic Address\n\nYou can [`read`](crate::Reg::read) this register and get [`sdynaddr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdynaddr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SdynaddrSpec;
impl crate::RegisterSpec for SdynaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sdynaddr::R`](R) reader structure"]
impl crate::Readable for SdynaddrSpec {}
#[doc = "`write(|w| ..)` method takes [`sdynaddr::W`](W) writer structure"]
impl crate::Writable for SdynaddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SDYNADDR to value 0"]
impl crate::Resettable for SdynaddrSpec {}
