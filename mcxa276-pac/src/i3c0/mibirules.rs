#[doc = "Register `MIBIRULES` reader"]
pub type R = crate::R<MibirulesSpec>;
#[doc = "Register `MIBIRULES` writer"]
pub type W = crate::W<MibirulesSpec>;
#[doc = "Field `ADDR0` reader - ADDR0"]
pub type Addr0R = crate::FieldReader;
#[doc = "Field `ADDR0` writer - ADDR0"]
pub type Addr0W<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `ADDR1` reader - ADDR1"]
pub type Addr1R = crate::FieldReader;
#[doc = "Field `ADDR1` writer - ADDR1"]
pub type Addr1W<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `ADDR2` reader - ADDR2"]
pub type Addr2R = crate::FieldReader;
#[doc = "Field `ADDR2` writer - ADDR2"]
pub type Addr2W<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `ADDR3` reader - ADDR3"]
pub type Addr3R = crate::FieldReader;
#[doc = "Field `ADDR3` writer - ADDR3"]
pub type Addr3W<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `ADDR4` reader - ADDR4"]
pub type Addr4R = crate::FieldReader;
#[doc = "Field `ADDR4` writer - ADDR4"]
pub type Addr4W<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Most Significant Address Bit is 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msb0 {
    #[doc = "0: MSB is not 0"]
    Disable = 0,
    #[doc = "1: MSB is 0"]
    Enable = 1,
}
impl From<Msb0> for bool {
    #[inline(always)]
    fn from(variant: Msb0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MSB0` reader - Most Significant Address Bit is 0"]
pub type Msb0R = crate::BitReader<Msb0>;
impl Msb0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Msb0 {
        match self.bits {
            false => Msb0::Disable,
            true => Msb0::Enable,
        }
    }
    #[doc = "MSB is not 0"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Msb0::Disable
    }
    #[doc = "MSB is 0"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Msb0::Enable
    }
}
#[doc = "Field `MSB0` writer - Most Significant Address Bit is 0"]
pub type Msb0W<'a, REG> = crate::BitWriter<'a, REG, Msb0>;
impl<'a, REG> Msb0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "MSB is not 0"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Msb0::Disable)
    }
    #[doc = "MSB is 0"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Msb0::Enable)
    }
}
#[doc = "No IBI byte\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nobyte {
    #[doc = "0: With mandatory IBI byte"]
    Ibibyte = 0,
    #[doc = "1: Without mandatory IBI byte"]
    NoIbibyte = 1,
}
impl From<Nobyte> for bool {
    #[inline(always)]
    fn from(variant: Nobyte) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOBYTE` reader - No IBI byte"]
pub type NobyteR = crate::BitReader<Nobyte>;
impl NobyteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nobyte {
        match self.bits {
            false => Nobyte::Ibibyte,
            true => Nobyte::NoIbibyte,
        }
    }
    #[doc = "With mandatory IBI byte"]
    #[inline(always)]
    pub fn is_ibibyte(&self) -> bool {
        *self == Nobyte::Ibibyte
    }
    #[doc = "Without mandatory IBI byte"]
    #[inline(always)]
    pub fn is_no_ibibyte(&self) -> bool {
        *self == Nobyte::NoIbibyte
    }
}
#[doc = "Field `NOBYTE` writer - No IBI byte"]
pub type NobyteW<'a, REG> = crate::BitWriter<'a, REG, Nobyte>;
impl<'a, REG> NobyteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "With mandatory IBI byte"]
    #[inline(always)]
    pub fn ibibyte(self) -> &'a mut crate::W<REG> {
        self.variant(Nobyte::Ibibyte)
    }
    #[doc = "Without mandatory IBI byte"]
    #[inline(always)]
    pub fn no_ibibyte(self) -> &'a mut crate::W<REG> {
        self.variant(Nobyte::NoIbibyte)
    }
}
impl R {
    #[doc = "Bits 0:5 - ADDR0"]
    #[inline(always)]
    pub fn addr0(&self) -> Addr0R {
        Addr0R::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 6:11 - ADDR1"]
    #[inline(always)]
    pub fn addr1(&self) -> Addr1R {
        Addr1R::new(((self.bits >> 6) & 0x3f) as u8)
    }
    #[doc = "Bits 12:17 - ADDR2"]
    #[inline(always)]
    pub fn addr2(&self) -> Addr2R {
        Addr2R::new(((self.bits >> 12) & 0x3f) as u8)
    }
    #[doc = "Bits 18:23 - ADDR3"]
    #[inline(always)]
    pub fn addr3(&self) -> Addr3R {
        Addr3R::new(((self.bits >> 18) & 0x3f) as u8)
    }
    #[doc = "Bits 24:29 - ADDR4"]
    #[inline(always)]
    pub fn addr4(&self) -> Addr4R {
        Addr4R::new(((self.bits >> 24) & 0x3f) as u8)
    }
    #[doc = "Bit 30 - Most Significant Address Bit is 0"]
    #[inline(always)]
    pub fn msb0(&self) -> Msb0R {
        Msb0R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - No IBI byte"]
    #[inline(always)]
    pub fn nobyte(&self) -> NobyteR {
        NobyteR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5 - ADDR0"]
    #[inline(always)]
    pub fn addr0(&mut self) -> Addr0W<MibirulesSpec> {
        Addr0W::new(self, 0)
    }
    #[doc = "Bits 6:11 - ADDR1"]
    #[inline(always)]
    pub fn addr1(&mut self) -> Addr1W<MibirulesSpec> {
        Addr1W::new(self, 6)
    }
    #[doc = "Bits 12:17 - ADDR2"]
    #[inline(always)]
    pub fn addr2(&mut self) -> Addr2W<MibirulesSpec> {
        Addr2W::new(self, 12)
    }
    #[doc = "Bits 18:23 - ADDR3"]
    #[inline(always)]
    pub fn addr3(&mut self) -> Addr3W<MibirulesSpec> {
        Addr3W::new(self, 18)
    }
    #[doc = "Bits 24:29 - ADDR4"]
    #[inline(always)]
    pub fn addr4(&mut self) -> Addr4W<MibirulesSpec> {
        Addr4W::new(self, 24)
    }
    #[doc = "Bit 30 - Most Significant Address Bit is 0"]
    #[inline(always)]
    pub fn msb0(&mut self) -> Msb0W<MibirulesSpec> {
        Msb0W::new(self, 30)
    }
    #[doc = "Bit 31 - No IBI byte"]
    #[inline(always)]
    pub fn nobyte(&mut self) -> NobyteW<MibirulesSpec> {
        NobyteW::new(self, 31)
    }
}
#[doc = "Controller In-band Interrupt Registry and Rules\n\nYou can [`read`](crate::Reg::read) this register and get [`mibirules::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mibirules::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MibirulesSpec;
impl crate::RegisterSpec for MibirulesSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mibirules::R`](R) reader structure"]
impl crate::Readable for MibirulesSpec {}
#[doc = "`write(|w| ..)` method takes [`mibirules::W`](W) writer structure"]
impl crate::Writable for MibirulesSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MIBIRULES to value 0"]
impl crate::Resettable for MibirulesSpec {}
