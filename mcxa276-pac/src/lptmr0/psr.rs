#[doc = "Register `PSR` reader"]
pub type R = crate::R<PsrSpec>;
#[doc = "Register `PSR` writer"]
pub type W = crate::W<PsrSpec>;
#[doc = "Prescaler and Glitch Filter Clock Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pcs {
    #[doc = "0: Clock 0"]
    Pcs00 = 0,
    #[doc = "1: Clock 1"]
    Pcs01 = 1,
    #[doc = "2: Clock 2"]
    Pcs10 = 2,
    #[doc = "3: Clock 3"]
    Pcs11 = 3,
}
impl From<Pcs> for u8 {
    #[inline(always)]
    fn from(variant: Pcs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pcs {
    type Ux = u8;
}
impl crate::IsEnum for Pcs {}
#[doc = "Field `PCS` reader - Prescaler and Glitch Filter Clock Select"]
pub type PcsR = crate::FieldReader<Pcs>;
impl PcsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pcs {
        match self.bits {
            0 => Pcs::Pcs00,
            1 => Pcs::Pcs01,
            2 => Pcs::Pcs10,
            3 => Pcs::Pcs11,
            _ => unreachable!(),
        }
    }
    #[doc = "Clock 0"]
    #[inline(always)]
    pub fn is_pcs00(&self) -> bool {
        *self == Pcs::Pcs00
    }
    #[doc = "Clock 1"]
    #[inline(always)]
    pub fn is_pcs01(&self) -> bool {
        *self == Pcs::Pcs01
    }
    #[doc = "Clock 2"]
    #[inline(always)]
    pub fn is_pcs10(&self) -> bool {
        *self == Pcs::Pcs10
    }
    #[doc = "Clock 3"]
    #[inline(always)]
    pub fn is_pcs11(&self) -> bool {
        *self == Pcs::Pcs11
    }
}
#[doc = "Field `PCS` writer - Prescaler and Glitch Filter Clock Select"]
pub type PcsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pcs, crate::Safe>;
impl<'a, REG> PcsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Clock 0"]
    #[inline(always)]
    pub fn pcs00(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::Pcs00)
    }
    #[doc = "Clock 1"]
    #[inline(always)]
    pub fn pcs01(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::Pcs01)
    }
    #[doc = "Clock 2"]
    #[inline(always)]
    pub fn pcs10(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::Pcs10)
    }
    #[doc = "Clock 3"]
    #[inline(always)]
    pub fn pcs11(self) -> &'a mut crate::W<REG> {
        self.variant(Pcs::Pcs11)
    }
}
#[doc = "Prescaler and Glitch Filter Bypass\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pbyp {
    #[doc = "0: Prescaler and glitch filter enable"]
    Pbyp0 = 0,
    #[doc = "1: Prescaler and glitch filter bypass"]
    Pbyp1 = 1,
}
impl From<Pbyp> for bool {
    #[inline(always)]
    fn from(variant: Pbyp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PBYP` reader - Prescaler and Glitch Filter Bypass"]
pub type PbypR = crate::BitReader<Pbyp>;
impl PbypR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pbyp {
        match self.bits {
            false => Pbyp::Pbyp0,
            true => Pbyp::Pbyp1,
        }
    }
    #[doc = "Prescaler and glitch filter enable"]
    #[inline(always)]
    pub fn is_pbyp0(&self) -> bool {
        *self == Pbyp::Pbyp0
    }
    #[doc = "Prescaler and glitch filter bypass"]
    #[inline(always)]
    pub fn is_pbyp1(&self) -> bool {
        *self == Pbyp::Pbyp1
    }
}
#[doc = "Field `PBYP` writer - Prescaler and Glitch Filter Bypass"]
pub type PbypW<'a, REG> = crate::BitWriter<'a, REG, Pbyp>;
impl<'a, REG> PbypW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Prescaler and glitch filter enable"]
    #[inline(always)]
    pub fn pbyp0(self) -> &'a mut crate::W<REG> {
        self.variant(Pbyp::Pbyp0)
    }
    #[doc = "Prescaler and glitch filter bypass"]
    #[inline(always)]
    pub fn pbyp1(self) -> &'a mut crate::W<REG> {
        self.variant(Pbyp::Pbyp1)
    }
}
#[doc = "Prescaler and Glitch Filter Value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Prescale {
    #[doc = "0: Prescaler divides the prescaler clock by 2; glitch filter does not support this configuration"]
    Prescale0000 = 0,
    #[doc = "1: Prescaler divides the prescaler clock by 4; glitch filter recognizes change on input pin after two rising clock edges"]
    Prescale0001 = 1,
    #[doc = "2: Prescaler divides the prescaler clock by 8; glitch filter recognizes change on input pin after four rising clock edges"]
    Prescale0010 = 2,
    #[doc = "3: Prescaler divides the prescaler clock by 16; glitch filter recognizes change on input pin after eight rising clock edges"]
    Prescale0011 = 3,
    #[doc = "4: Prescaler divides the prescaler clock by 32; glitch filter recognizes change on input pin after 16 rising clock edges"]
    Prescale0100 = 4,
    #[doc = "5: Prescaler divides the prescaler clock by 64; glitch filter recognizes change on input pin after 32 rising clock edges"]
    Prescale0101 = 5,
    #[doc = "6: Prescaler divides the prescaler clock by 128; glitch filter recognizes change on input pin after 64 rising clock edges"]
    Prescale0110 = 6,
    #[doc = "7: Prescaler divides the prescaler clock by 256; glitch filter recognizes change on input pin after 128 rising clock edges"]
    Prescale0111 = 7,
    #[doc = "8: Prescaler divides the prescaler clock by 512; glitch filter recognizes change on input pin after 256 rising clock edges"]
    Prescale1000 = 8,
    #[doc = "9: Prescaler divides the prescaler clock by 1024; glitch filter recognizes change on input pin after 512 rising clock edges"]
    Prescale1001 = 9,
    #[doc = "10: Prescaler divides the prescaler clock by 2048; glitch filter recognizes change on input pin after 1024 rising clock edges"]
    Prescale1010 = 10,
    #[doc = "11: Prescaler divides the prescaler clock by 4096; glitch filter recognizes change on input pin after 2048 rising clock edges"]
    Prescale1011 = 11,
    #[doc = "12: Prescaler divides the prescaler clock by 8192; glitch filter recognizes change on input pin after 4096 rising clock edges"]
    Prescale1100 = 12,
    #[doc = "13: Prescaler divides the prescaler clock by 16,384; glitch filter recognizes change on input pin after 8192 rising clock edges"]
    Prescale1101 = 13,
    #[doc = "14: Prescaler divides the prescaler clock by 32,768; glitch filter recognizes change on input pin after 16,384 rising clock edges"]
    Prescale1110 = 14,
    #[doc = "15: Prescaler divides the prescaler clock by 65,536; glitch filter recognizes change on input pin after 32,768 rising clock edges"]
    Prescale1111 = 15,
}
impl From<Prescale> for u8 {
    #[inline(always)]
    fn from(variant: Prescale) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Prescale {
    type Ux = u8;
}
impl crate::IsEnum for Prescale {}
#[doc = "Field `PRESCALE` reader - Prescaler and Glitch Filter Value"]
pub type PrescaleR = crate::FieldReader<Prescale>;
impl PrescaleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Prescale {
        match self.bits {
            0 => Prescale::Prescale0000,
            1 => Prescale::Prescale0001,
            2 => Prescale::Prescale0010,
            3 => Prescale::Prescale0011,
            4 => Prescale::Prescale0100,
            5 => Prescale::Prescale0101,
            6 => Prescale::Prescale0110,
            7 => Prescale::Prescale0111,
            8 => Prescale::Prescale1000,
            9 => Prescale::Prescale1001,
            10 => Prescale::Prescale1010,
            11 => Prescale::Prescale1011,
            12 => Prescale::Prescale1100,
            13 => Prescale::Prescale1101,
            14 => Prescale::Prescale1110,
            15 => Prescale::Prescale1111,
            _ => unreachable!(),
        }
    }
    #[doc = "Prescaler divides the prescaler clock by 2; glitch filter does not support this configuration"]
    #[inline(always)]
    pub fn is_prescale0000(&self) -> bool {
        *self == Prescale::Prescale0000
    }
    #[doc = "Prescaler divides the prescaler clock by 4; glitch filter recognizes change on input pin after two rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0001(&self) -> bool {
        *self == Prescale::Prescale0001
    }
    #[doc = "Prescaler divides the prescaler clock by 8; glitch filter recognizes change on input pin after four rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0010(&self) -> bool {
        *self == Prescale::Prescale0010
    }
    #[doc = "Prescaler divides the prescaler clock by 16; glitch filter recognizes change on input pin after eight rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0011(&self) -> bool {
        *self == Prescale::Prescale0011
    }
    #[doc = "Prescaler divides the prescaler clock by 32; glitch filter recognizes change on input pin after 16 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0100(&self) -> bool {
        *self == Prescale::Prescale0100
    }
    #[doc = "Prescaler divides the prescaler clock by 64; glitch filter recognizes change on input pin after 32 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0101(&self) -> bool {
        *self == Prescale::Prescale0101
    }
    #[doc = "Prescaler divides the prescaler clock by 128; glitch filter recognizes change on input pin after 64 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0110(&self) -> bool {
        *self == Prescale::Prescale0110
    }
    #[doc = "Prescaler divides the prescaler clock by 256; glitch filter recognizes change on input pin after 128 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale0111(&self) -> bool {
        *self == Prescale::Prescale0111
    }
    #[doc = "Prescaler divides the prescaler clock by 512; glitch filter recognizes change on input pin after 256 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1000(&self) -> bool {
        *self == Prescale::Prescale1000
    }
    #[doc = "Prescaler divides the prescaler clock by 1024; glitch filter recognizes change on input pin after 512 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1001(&self) -> bool {
        *self == Prescale::Prescale1001
    }
    #[doc = "Prescaler divides the prescaler clock by 2048; glitch filter recognizes change on input pin after 1024 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1010(&self) -> bool {
        *self == Prescale::Prescale1010
    }
    #[doc = "Prescaler divides the prescaler clock by 4096; glitch filter recognizes change on input pin after 2048 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1011(&self) -> bool {
        *self == Prescale::Prescale1011
    }
    #[doc = "Prescaler divides the prescaler clock by 8192; glitch filter recognizes change on input pin after 4096 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1100(&self) -> bool {
        *self == Prescale::Prescale1100
    }
    #[doc = "Prescaler divides the prescaler clock by 16,384; glitch filter recognizes change on input pin after 8192 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1101(&self) -> bool {
        *self == Prescale::Prescale1101
    }
    #[doc = "Prescaler divides the prescaler clock by 32,768; glitch filter recognizes change on input pin after 16,384 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1110(&self) -> bool {
        *self == Prescale::Prescale1110
    }
    #[doc = "Prescaler divides the prescaler clock by 65,536; glitch filter recognizes change on input pin after 32,768 rising clock edges"]
    #[inline(always)]
    pub fn is_prescale1111(&self) -> bool {
        *self == Prescale::Prescale1111
    }
}
#[doc = "Field `PRESCALE` writer - Prescaler and Glitch Filter Value"]
pub type PrescaleW<'a, REG> = crate::FieldWriter<'a, REG, 4, Prescale, crate::Safe>;
impl<'a, REG> PrescaleW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Prescaler divides the prescaler clock by 2; glitch filter does not support this configuration"]
    #[inline(always)]
    pub fn prescale0000(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0000)
    }
    #[doc = "Prescaler divides the prescaler clock by 4; glitch filter recognizes change on input pin after two rising clock edges"]
    #[inline(always)]
    pub fn prescale0001(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0001)
    }
    #[doc = "Prescaler divides the prescaler clock by 8; glitch filter recognizes change on input pin after four rising clock edges"]
    #[inline(always)]
    pub fn prescale0010(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0010)
    }
    #[doc = "Prescaler divides the prescaler clock by 16; glitch filter recognizes change on input pin after eight rising clock edges"]
    #[inline(always)]
    pub fn prescale0011(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0011)
    }
    #[doc = "Prescaler divides the prescaler clock by 32; glitch filter recognizes change on input pin after 16 rising clock edges"]
    #[inline(always)]
    pub fn prescale0100(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0100)
    }
    #[doc = "Prescaler divides the prescaler clock by 64; glitch filter recognizes change on input pin after 32 rising clock edges"]
    #[inline(always)]
    pub fn prescale0101(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0101)
    }
    #[doc = "Prescaler divides the prescaler clock by 128; glitch filter recognizes change on input pin after 64 rising clock edges"]
    #[inline(always)]
    pub fn prescale0110(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0110)
    }
    #[doc = "Prescaler divides the prescaler clock by 256; glitch filter recognizes change on input pin after 128 rising clock edges"]
    #[inline(always)]
    pub fn prescale0111(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale0111)
    }
    #[doc = "Prescaler divides the prescaler clock by 512; glitch filter recognizes change on input pin after 256 rising clock edges"]
    #[inline(always)]
    pub fn prescale1000(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1000)
    }
    #[doc = "Prescaler divides the prescaler clock by 1024; glitch filter recognizes change on input pin after 512 rising clock edges"]
    #[inline(always)]
    pub fn prescale1001(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1001)
    }
    #[doc = "Prescaler divides the prescaler clock by 2048; glitch filter recognizes change on input pin after 1024 rising clock edges"]
    #[inline(always)]
    pub fn prescale1010(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1010)
    }
    #[doc = "Prescaler divides the prescaler clock by 4096; glitch filter recognizes change on input pin after 2048 rising clock edges"]
    #[inline(always)]
    pub fn prescale1011(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1011)
    }
    #[doc = "Prescaler divides the prescaler clock by 8192; glitch filter recognizes change on input pin after 4096 rising clock edges"]
    #[inline(always)]
    pub fn prescale1100(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1100)
    }
    #[doc = "Prescaler divides the prescaler clock by 16,384; glitch filter recognizes change on input pin after 8192 rising clock edges"]
    #[inline(always)]
    pub fn prescale1101(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1101)
    }
    #[doc = "Prescaler divides the prescaler clock by 32,768; glitch filter recognizes change on input pin after 16,384 rising clock edges"]
    #[inline(always)]
    pub fn prescale1110(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1110)
    }
    #[doc = "Prescaler divides the prescaler clock by 65,536; glitch filter recognizes change on input pin after 32,768 rising clock edges"]
    #[inline(always)]
    pub fn prescale1111(self) -> &'a mut crate::W<REG> {
        self.variant(Prescale::Prescale1111)
    }
}
impl R {
    #[doc = "Bits 0:1 - Prescaler and Glitch Filter Clock Select"]
    #[inline(always)]
    pub fn pcs(&self) -> PcsR {
        PcsR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 2 - Prescaler and Glitch Filter Bypass"]
    #[inline(always)]
    pub fn pbyp(&self) -> PbypR {
        PbypR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bits 3:6 - Prescaler and Glitch Filter Value"]
    #[inline(always)]
    pub fn prescale(&self) -> PrescaleR {
        PrescaleR::new(((self.bits >> 3) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Prescaler and Glitch Filter Clock Select"]
    #[inline(always)]
    pub fn pcs(&mut self) -> PcsW<PsrSpec> {
        PcsW::new(self, 0)
    }
    #[doc = "Bit 2 - Prescaler and Glitch Filter Bypass"]
    #[inline(always)]
    pub fn pbyp(&mut self) -> PbypW<PsrSpec> {
        PbypW::new(self, 2)
    }
    #[doc = "Bits 3:6 - Prescaler and Glitch Filter Value"]
    #[inline(always)]
    pub fn prescale(&mut self) -> PrescaleW<PsrSpec> {
        PrescaleW::new(self, 3)
    }
}
#[doc = "Prescaler and Glitch Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`psr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`psr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PsrSpec;
impl crate::RegisterSpec for PsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`psr::R`](R) reader structure"]
impl crate::Readable for PsrSpec {}
#[doc = "`write(|w| ..)` method takes [`psr::W`](W) writer structure"]
impl crate::Writable for PsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PSR to value 0"]
impl crate::Resettable for PsrSpec {}
