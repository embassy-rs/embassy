#[doc = "Register `MDYNADDR` reader"]
pub type R = crate::R<MdynaddrSpec>;
#[doc = "Register `MDYNADDR` writer"]
pub type W = crate::W<MdynaddrSpec>;
#[doc = "Dynamic Address Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Davalid {
    #[doc = "0: No valid DA assigned"]
    NoValid = 0,
    #[doc = "1: Valid DA assigned"]
    Valid = 1,
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
            false => Davalid::NoValid,
            true => Davalid::Valid,
        }
    }
    #[doc = "No valid DA assigned"]
    #[inline(always)]
    pub fn is_no_valid(&self) -> bool {
        *self == Davalid::NoValid
    }
    #[doc = "Valid DA assigned"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Davalid::Valid
    }
}
#[doc = "Field `DAVALID` writer - Dynamic Address Valid"]
pub type DavalidW<'a, REG> = crate::BitWriter<'a, REG, Davalid>;
impl<'a, REG> DavalidW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No valid DA assigned"]
    #[inline(always)]
    pub fn no_valid(self) -> &'a mut crate::W<REG> {
        self.variant(Davalid::NoValid)
    }
    #[doc = "Valid DA assigned"]
    #[inline(always)]
    pub fn valid(self) -> &'a mut crate::W<REG> {
        self.variant(Davalid::Valid)
    }
}
#[doc = "Field `DADDR` reader - Dynamic Address"]
pub type DaddrR = crate::FieldReader;
#[doc = "Field `DADDR` writer - Dynamic Address"]
pub type DaddrW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
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
}
impl W {
    #[doc = "Bit 0 - Dynamic Address Valid"]
    #[inline(always)]
    pub fn davalid(&mut self) -> DavalidW<MdynaddrSpec> {
        DavalidW::new(self, 0)
    }
    #[doc = "Bits 1:7 - Dynamic Address"]
    #[inline(always)]
    pub fn daddr(&mut self) -> DaddrW<MdynaddrSpec> {
        DaddrW::new(self, 1)
    }
}
#[doc = "Controller Dynamic Address\n\nYou can [`read`](crate::Reg::read) this register and get [`mdynaddr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mdynaddr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MdynaddrSpec;
impl crate::RegisterSpec for MdynaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mdynaddr::R`](R) reader structure"]
impl crate::Readable for MdynaddrSpec {}
#[doc = "`write(|w| ..)` method takes [`mdynaddr::W`](W) writer structure"]
impl crate::Writable for MdynaddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MDYNADDR to value 0"]
impl crate::Resettable for MdynaddrSpec {}
