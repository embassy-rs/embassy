#[doc = "Register `SHIFTCFG[%s]` reader"]
pub type R = crate::R<ShiftcfgSpec>;
#[doc = "Register `SHIFTCFG[%s]` writer"]
pub type W = crate::W<ShiftcfgSpec>;
#[doc = "Shifter Start\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sstart {
    #[doc = "0: Start bit disabled for Transmitter, Receiver, and Match Store modes; Transmitter mode loads data on enable"]
    Value00 = 0,
    #[doc = "1: Start bit disabled for Transmitter, Receiver, and Match Store modes; Transmitter mode loads data on first shift"]
    Value01 = 1,
    #[doc = "2: Transmitter mode outputs start bit value 0 before loading data on first shift; if start bit is not 0, Receiver and Match Store modes set error flag"]
    Value10 = 2,
    #[doc = "3: Transmitter mode outputs start bit value 1 before loading data on first shift; if start bit is not 1, Receiver and Match Store modes set error flag"]
    Value11 = 3,
}
impl From<Sstart> for u8 {
    #[inline(always)]
    fn from(variant: Sstart) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sstart {
    type Ux = u8;
}
impl crate::IsEnum for Sstart {}
#[doc = "Field `SSTART` reader - Shifter Start"]
pub type SstartR = crate::FieldReader<Sstart>;
impl SstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sstart {
        match self.bits {
            0 => Sstart::Value00,
            1 => Sstart::Value01,
            2 => Sstart::Value10,
            3 => Sstart::Value11,
            _ => unreachable!(),
        }
    }
    #[doc = "Start bit disabled for Transmitter, Receiver, and Match Store modes; Transmitter mode loads data on enable"]
    #[inline(always)]
    pub fn is_value00(&self) -> bool {
        *self == Sstart::Value00
    }
    #[doc = "Start bit disabled for Transmitter, Receiver, and Match Store modes; Transmitter mode loads data on first shift"]
    #[inline(always)]
    pub fn is_value01(&self) -> bool {
        *self == Sstart::Value01
    }
    #[doc = "Transmitter mode outputs start bit value 0 before loading data on first shift; if start bit is not 0, Receiver and Match Store modes set error flag"]
    #[inline(always)]
    pub fn is_value10(&self) -> bool {
        *self == Sstart::Value10
    }
    #[doc = "Transmitter mode outputs start bit value 1 before loading data on first shift; if start bit is not 1, Receiver and Match Store modes set error flag"]
    #[inline(always)]
    pub fn is_value11(&self) -> bool {
        *self == Sstart::Value11
    }
}
#[doc = "Field `SSTART` writer - Shifter Start"]
pub type SstartW<'a, REG> = crate::FieldWriter<'a, REG, 2, Sstart, crate::Safe>;
impl<'a, REG> SstartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Start bit disabled for Transmitter, Receiver, and Match Store modes; Transmitter mode loads data on enable"]
    #[inline(always)]
    pub fn value00(self) -> &'a mut crate::W<REG> {
        self.variant(Sstart::Value00)
    }
    #[doc = "Start bit disabled for Transmitter, Receiver, and Match Store modes; Transmitter mode loads data on first shift"]
    #[inline(always)]
    pub fn value01(self) -> &'a mut crate::W<REG> {
        self.variant(Sstart::Value01)
    }
    #[doc = "Transmitter mode outputs start bit value 0 before loading data on first shift; if start bit is not 0, Receiver and Match Store modes set error flag"]
    #[inline(always)]
    pub fn value10(self) -> &'a mut crate::W<REG> {
        self.variant(Sstart::Value10)
    }
    #[doc = "Transmitter mode outputs start bit value 1 before loading data on first shift; if start bit is not 1, Receiver and Match Store modes set error flag"]
    #[inline(always)]
    pub fn value11(self) -> &'a mut crate::W<REG> {
        self.variant(Sstart::Value11)
    }
}
#[doc = "Shifter Stop\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sstop {
    #[doc = "0: Stop bit disabled for Transmitter, Receiver, and Match Store modes"]
    Value00 = 0,
    #[doc = "1: Stop bit disabled for Transmitter, Receiver, and Match Store modes; when timer is in stop condition, Receiver and Match Store modes store receive data on the configured shift edge"]
    Value01 = 1,
    #[doc = "2: Transmitter mode outputs stop bit value 0 in Match Store mode; if stop bit is not 0, Receiver and Match Store modes set error flag (when timer is in stop condition, these modes also store receive data on the configured shift edge)"]
    Value10 = 2,
    #[doc = "3: Transmitter mode outputs stop bit value 1 in Match Store mode; if stop bit is not 1, Receiver and Match Store modes set error flag (when timer is in stop condition, these modes also store receive data on the configured shift edge)"]
    Value11 = 3,
}
impl From<Sstop> for u8 {
    #[inline(always)]
    fn from(variant: Sstop) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sstop {
    type Ux = u8;
}
impl crate::IsEnum for Sstop {}
#[doc = "Field `SSTOP` reader - Shifter Stop"]
pub type SstopR = crate::FieldReader<Sstop>;
impl SstopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sstop {
        match self.bits {
            0 => Sstop::Value00,
            1 => Sstop::Value01,
            2 => Sstop::Value10,
            3 => Sstop::Value11,
            _ => unreachable!(),
        }
    }
    #[doc = "Stop bit disabled for Transmitter, Receiver, and Match Store modes"]
    #[inline(always)]
    pub fn is_value00(&self) -> bool {
        *self == Sstop::Value00
    }
    #[doc = "Stop bit disabled for Transmitter, Receiver, and Match Store modes; when timer is in stop condition, Receiver and Match Store modes store receive data on the configured shift edge"]
    #[inline(always)]
    pub fn is_value01(&self) -> bool {
        *self == Sstop::Value01
    }
    #[doc = "Transmitter mode outputs stop bit value 0 in Match Store mode; if stop bit is not 0, Receiver and Match Store modes set error flag (when timer is in stop condition, these modes also store receive data on the configured shift edge)"]
    #[inline(always)]
    pub fn is_value10(&self) -> bool {
        *self == Sstop::Value10
    }
    #[doc = "Transmitter mode outputs stop bit value 1 in Match Store mode; if stop bit is not 1, Receiver and Match Store modes set error flag (when timer is in stop condition, these modes also store receive data on the configured shift edge)"]
    #[inline(always)]
    pub fn is_value11(&self) -> bool {
        *self == Sstop::Value11
    }
}
#[doc = "Field `SSTOP` writer - Shifter Stop"]
pub type SstopW<'a, REG> = crate::FieldWriter<'a, REG, 2, Sstop, crate::Safe>;
impl<'a, REG> SstopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Stop bit disabled for Transmitter, Receiver, and Match Store modes"]
    #[inline(always)]
    pub fn value00(self) -> &'a mut crate::W<REG> {
        self.variant(Sstop::Value00)
    }
    #[doc = "Stop bit disabled for Transmitter, Receiver, and Match Store modes; when timer is in stop condition, Receiver and Match Store modes store receive data on the configured shift edge"]
    #[inline(always)]
    pub fn value01(self) -> &'a mut crate::W<REG> {
        self.variant(Sstop::Value01)
    }
    #[doc = "Transmitter mode outputs stop bit value 0 in Match Store mode; if stop bit is not 0, Receiver and Match Store modes set error flag (when timer is in stop condition, these modes also store receive data on the configured shift edge)"]
    #[inline(always)]
    pub fn value10(self) -> &'a mut crate::W<REG> {
        self.variant(Sstop::Value10)
    }
    #[doc = "Transmitter mode outputs stop bit value 1 in Match Store mode; if stop bit is not 1, Receiver and Match Store modes set error flag (when timer is in stop condition, these modes also store receive data on the configured shift edge)"]
    #[inline(always)]
    pub fn value11(self) -> &'a mut crate::W<REG> {
        self.variant(Sstop::Value11)
    }
}
#[doc = "Input Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Insrc {
    #[doc = "0: Pin"]
    Pin = 0,
    #[doc = "1: Shifter n+1 output"]
    ShifterNplus1 = 1,
}
impl From<Insrc> for bool {
    #[inline(always)]
    fn from(variant: Insrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INSRC` reader - Input Source"]
pub type InsrcR = crate::BitReader<Insrc>;
impl InsrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Insrc {
        match self.bits {
            false => Insrc::Pin,
            true => Insrc::ShifterNplus1,
        }
    }
    #[doc = "Pin"]
    #[inline(always)]
    pub fn is_pin(&self) -> bool {
        *self == Insrc::Pin
    }
    #[doc = "Shifter n+1 output"]
    #[inline(always)]
    pub fn is_shifter_nplus1(&self) -> bool {
        *self == Insrc::ShifterNplus1
    }
}
#[doc = "Field `INSRC` writer - Input Source"]
pub type InsrcW<'a, REG> = crate::BitWriter<'a, REG, Insrc>;
impl<'a, REG> InsrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin"]
    #[inline(always)]
    pub fn pin(self) -> &'a mut crate::W<REG> {
        self.variant(Insrc::Pin)
    }
    #[doc = "Shifter n+1 output"]
    #[inline(always)]
    pub fn shifter_nplus1(self) -> &'a mut crate::W<REG> {
        self.variant(Insrc::ShifterNplus1)
    }
}
#[doc = "Late Store\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Latst {
    #[doc = "0: Store the pre-shift register state"]
    Preshift = 0,
    #[doc = "1: Store the post-shift register state"]
    Postshift = 1,
}
impl From<Latst> for bool {
    #[inline(always)]
    fn from(variant: Latst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LATST` reader - Late Store"]
pub type LatstR = crate::BitReader<Latst>;
impl LatstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Latst {
        match self.bits {
            false => Latst::Preshift,
            true => Latst::Postshift,
        }
    }
    #[doc = "Store the pre-shift register state"]
    #[inline(always)]
    pub fn is_preshift(&self) -> bool {
        *self == Latst::Preshift
    }
    #[doc = "Store the post-shift register state"]
    #[inline(always)]
    pub fn is_postshift(&self) -> bool {
        *self == Latst::Postshift
    }
}
#[doc = "Field `LATST` writer - Late Store"]
pub type LatstW<'a, REG> = crate::BitWriter<'a, REG, Latst>;
impl<'a, REG> LatstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Store the pre-shift register state"]
    #[inline(always)]
    pub fn preshift(self) -> &'a mut crate::W<REG> {
        self.variant(Latst::Preshift)
    }
    #[doc = "Store the post-shift register state"]
    #[inline(always)]
    pub fn postshift(self) -> &'a mut crate::W<REG> {
        self.variant(Latst::Postshift)
    }
}
#[doc = "Shifter Size\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ssize {
    #[doc = "0: 32-bit"]
    Width32 = 0,
    #[doc = "1: 24-bit"]
    Width24 = 1,
}
impl From<Ssize> for bool {
    #[inline(always)]
    fn from(variant: Ssize) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SSIZE` reader - Shifter Size"]
pub type SsizeR = crate::BitReader<Ssize>;
impl SsizeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ssize {
        match self.bits {
            false => Ssize::Width32,
            true => Ssize::Width24,
        }
    }
    #[doc = "32-bit"]
    #[inline(always)]
    pub fn is_width32(&self) -> bool {
        *self == Ssize::Width32
    }
    #[doc = "24-bit"]
    #[inline(always)]
    pub fn is_width24(&self) -> bool {
        *self == Ssize::Width24
    }
}
#[doc = "Field `SSIZE` writer - Shifter Size"]
pub type SsizeW<'a, REG> = crate::BitWriter<'a, REG, Ssize>;
impl<'a, REG> SsizeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "32-bit"]
    #[inline(always)]
    pub fn width32(self) -> &'a mut crate::W<REG> {
        self.variant(Ssize::Width32)
    }
    #[doc = "24-bit"]
    #[inline(always)]
    pub fn width24(self) -> &'a mut crate::W<REG> {
        self.variant(Ssize::Width24)
    }
}
#[doc = "Field `PWIDTH` reader - Parallel Width"]
pub type PwidthR = crate::FieldReader;
#[doc = "Field `PWIDTH` writer - Parallel Width"]
pub type PwidthW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
impl R {
    #[doc = "Bits 0:1 - Shifter Start"]
    #[inline(always)]
    pub fn sstart(&self) -> SstartR {
        SstartR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 4:5 - Shifter Stop"]
    #[inline(always)]
    pub fn sstop(&self) -> SstopR {
        SstopR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bit 8 - Input Source"]
    #[inline(always)]
    pub fn insrc(&self) -> InsrcR {
        InsrcR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Late Store"]
    #[inline(always)]
    pub fn latst(&self) -> LatstR {
        LatstR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 12 - Shifter Size"]
    #[inline(always)]
    pub fn ssize(&self) -> SsizeR {
        SsizeR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bits 16:20 - Parallel Width"]
    #[inline(always)]
    pub fn pwidth(&self) -> PwidthR {
        PwidthR::new(((self.bits >> 16) & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Shifter Start"]
    #[inline(always)]
    pub fn sstart(&mut self) -> SstartW<ShiftcfgSpec> {
        SstartW::new(self, 0)
    }
    #[doc = "Bits 4:5 - Shifter Stop"]
    #[inline(always)]
    pub fn sstop(&mut self) -> SstopW<ShiftcfgSpec> {
        SstopW::new(self, 4)
    }
    #[doc = "Bit 8 - Input Source"]
    #[inline(always)]
    pub fn insrc(&mut self) -> InsrcW<ShiftcfgSpec> {
        InsrcW::new(self, 8)
    }
    #[doc = "Bit 9 - Late Store"]
    #[inline(always)]
    pub fn latst(&mut self) -> LatstW<ShiftcfgSpec> {
        LatstW::new(self, 9)
    }
    #[doc = "Bit 12 - Shifter Size"]
    #[inline(always)]
    pub fn ssize(&mut self) -> SsizeW<ShiftcfgSpec> {
        SsizeW::new(self, 12)
    }
    #[doc = "Bits 16:20 - Parallel Width"]
    #[inline(always)]
    pub fn pwidth(&mut self) -> PwidthW<ShiftcfgSpec> {
        PwidthW::new(self, 16)
    }
}
#[doc = "Shifter Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftcfgSpec;
impl crate::RegisterSpec for ShiftcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftcfg::R`](R) reader structure"]
impl crate::Readable for ShiftcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftcfg::W`](W) writer structure"]
impl crate::Writable for ShiftcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTCFG[%s] to value 0"]
impl crate::Resettable for ShiftcfgSpec {}
