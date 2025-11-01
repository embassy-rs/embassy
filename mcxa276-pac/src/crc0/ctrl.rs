#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "TCRC\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tcrc {
    #[doc = "0: 16 bits"]
    B16 = 0,
    #[doc = "1: 32 bits"]
    B32 = 1,
}
impl From<Tcrc> for bool {
    #[inline(always)]
    fn from(variant: Tcrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCRC` reader - TCRC"]
pub type TcrcR = crate::BitReader<Tcrc>;
impl TcrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcrc {
        match self.bits {
            false => Tcrc::B16,
            true => Tcrc::B32,
        }
    }
    #[doc = "16 bits"]
    #[inline(always)]
    pub fn is_b16(&self) -> bool {
        *self == Tcrc::B16
    }
    #[doc = "32 bits"]
    #[inline(always)]
    pub fn is_b32(&self) -> bool {
        *self == Tcrc::B32
    }
}
#[doc = "Field `TCRC` writer - TCRC"]
pub type TcrcW<'a, REG> = crate::BitWriter<'a, REG, Tcrc>;
impl<'a, REG> TcrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "16 bits"]
    #[inline(always)]
    pub fn b16(self) -> &'a mut crate::W<REG> {
        self.variant(Tcrc::B16)
    }
    #[doc = "32 bits"]
    #[inline(always)]
    pub fn b32(self) -> &'a mut crate::W<REG> {
        self.variant(Tcrc::B32)
    }
}
#[doc = "Write as Seed\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Was {
    #[doc = "0: Data values"]
    Data = 0,
    #[doc = "1: Seed values"]
    Seed = 1,
}
impl From<Was> for bool {
    #[inline(always)]
    fn from(variant: Was) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WAS` reader - Write as Seed"]
pub type WasR = crate::BitReader<Was>;
impl WasR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Was {
        match self.bits {
            false => Was::Data,
            true => Was::Seed,
        }
    }
    #[doc = "Data values"]
    #[inline(always)]
    pub fn is_data(&self) -> bool {
        *self == Was::Data
    }
    #[doc = "Seed values"]
    #[inline(always)]
    pub fn is_seed(&self) -> bool {
        *self == Was::Seed
    }
}
#[doc = "Field `WAS` writer - Write as Seed"]
pub type WasW<'a, REG> = crate::BitWriter<'a, REG, Was>;
impl<'a, REG> WasW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Data values"]
    #[inline(always)]
    pub fn data(self) -> &'a mut crate::W<REG> {
        self.variant(Was::Data)
    }
    #[doc = "Seed values"]
    #[inline(always)]
    pub fn seed(self) -> &'a mut crate::W<REG> {
        self.variant(Was::Seed)
    }
}
#[doc = "Complement Read of CRC Data Register\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fxor {
    #[doc = "0: Disables XOR on reading data."]
    Noxor = 0,
    #[doc = "1: Inverts or complements the read value of the CRC Data."]
    Invert = 1,
}
impl From<Fxor> for bool {
    #[inline(always)]
    fn from(variant: Fxor) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FXOR` reader - Complement Read of CRC Data Register"]
pub type FxorR = crate::BitReader<Fxor>;
impl FxorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fxor {
        match self.bits {
            false => Fxor::Noxor,
            true => Fxor::Invert,
        }
    }
    #[doc = "Disables XOR on reading data."]
    #[inline(always)]
    pub fn is_noxor(&self) -> bool {
        *self == Fxor::Noxor
    }
    #[doc = "Inverts or complements the read value of the CRC Data."]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Fxor::Invert
    }
}
#[doc = "Field `FXOR` writer - Complement Read of CRC Data Register"]
pub type FxorW<'a, REG> = crate::BitWriter<'a, REG, Fxor>;
impl<'a, REG> FxorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables XOR on reading data."]
    #[inline(always)]
    pub fn noxor(self) -> &'a mut crate::W<REG> {
        self.variant(Fxor::Noxor)
    }
    #[doc = "Inverts or complements the read value of the CRC Data."]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Fxor::Invert)
    }
}
#[doc = "Transpose Type for Read\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Totr {
    #[doc = "0: No transposition"]
    Notrnps = 0,
    #[doc = "1: Bits in bytes are transposed, but bytes are not transposed."]
    BtsTrnps = 1,
    #[doc = "2: Both bits in bytes and bytes are transposed."]
    BytsBtsTrnps = 2,
    #[doc = "3: Only bytes are transposed, no bits in a byte are transposed."]
    BytsTrnps = 3,
}
impl From<Totr> for u8 {
    #[inline(always)]
    fn from(variant: Totr) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Totr {
    type Ux = u8;
}
impl crate::IsEnum for Totr {}
#[doc = "Field `TOTR` reader - Transpose Type for Read"]
pub type TotrR = crate::FieldReader<Totr>;
impl TotrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Totr {
        match self.bits {
            0 => Totr::Notrnps,
            1 => Totr::BtsTrnps,
            2 => Totr::BytsBtsTrnps,
            3 => Totr::BytsTrnps,
            _ => unreachable!(),
        }
    }
    #[doc = "No transposition"]
    #[inline(always)]
    pub fn is_notrnps(&self) -> bool {
        *self == Totr::Notrnps
    }
    #[doc = "Bits in bytes are transposed, but bytes are not transposed."]
    #[inline(always)]
    pub fn is_bts_trnps(&self) -> bool {
        *self == Totr::BtsTrnps
    }
    #[doc = "Both bits in bytes and bytes are transposed."]
    #[inline(always)]
    pub fn is_byts_bts_trnps(&self) -> bool {
        *self == Totr::BytsBtsTrnps
    }
    #[doc = "Only bytes are transposed, no bits in a byte are transposed."]
    #[inline(always)]
    pub fn is_byts_trnps(&self) -> bool {
        *self == Totr::BytsTrnps
    }
}
#[doc = "Field `TOTR` writer - Transpose Type for Read"]
pub type TotrW<'a, REG> = crate::FieldWriter<'a, REG, 2, Totr, crate::Safe>;
impl<'a, REG> TotrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No transposition"]
    #[inline(always)]
    pub fn notrnps(self) -> &'a mut crate::W<REG> {
        self.variant(Totr::Notrnps)
    }
    #[doc = "Bits in bytes are transposed, but bytes are not transposed."]
    #[inline(always)]
    pub fn bts_trnps(self) -> &'a mut crate::W<REG> {
        self.variant(Totr::BtsTrnps)
    }
    #[doc = "Both bits in bytes and bytes are transposed."]
    #[inline(always)]
    pub fn byts_bts_trnps(self) -> &'a mut crate::W<REG> {
        self.variant(Totr::BytsBtsTrnps)
    }
    #[doc = "Only bytes are transposed, no bits in a byte are transposed."]
    #[inline(always)]
    pub fn byts_trnps(self) -> &'a mut crate::W<REG> {
        self.variant(Totr::BytsTrnps)
    }
}
#[doc = "Transpose Type for Write\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tot {
    #[doc = "0: No transposition"]
    Notrnps = 0,
    #[doc = "1: Bits in bytes are transposed, but bytes are not transposed."]
    BtsTrnps = 1,
    #[doc = "2: Both bits in bytes and bytes are transposed."]
    BytsBtsTrnps = 2,
    #[doc = "3: Only bytes are transposed, no bits in a byte are transposed."]
    BytsTrnps = 3,
}
impl From<Tot> for u8 {
    #[inline(always)]
    fn from(variant: Tot) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tot {
    type Ux = u8;
}
impl crate::IsEnum for Tot {}
#[doc = "Field `TOT` reader - Transpose Type for Write"]
pub type TotR = crate::FieldReader<Tot>;
impl TotR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tot {
        match self.bits {
            0 => Tot::Notrnps,
            1 => Tot::BtsTrnps,
            2 => Tot::BytsBtsTrnps,
            3 => Tot::BytsTrnps,
            _ => unreachable!(),
        }
    }
    #[doc = "No transposition"]
    #[inline(always)]
    pub fn is_notrnps(&self) -> bool {
        *self == Tot::Notrnps
    }
    #[doc = "Bits in bytes are transposed, but bytes are not transposed."]
    #[inline(always)]
    pub fn is_bts_trnps(&self) -> bool {
        *self == Tot::BtsTrnps
    }
    #[doc = "Both bits in bytes and bytes are transposed."]
    #[inline(always)]
    pub fn is_byts_bts_trnps(&self) -> bool {
        *self == Tot::BytsBtsTrnps
    }
    #[doc = "Only bytes are transposed, no bits in a byte are transposed."]
    #[inline(always)]
    pub fn is_byts_trnps(&self) -> bool {
        *self == Tot::BytsTrnps
    }
}
#[doc = "Field `TOT` writer - Transpose Type for Write"]
pub type TotW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tot, crate::Safe>;
impl<'a, REG> TotW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No transposition"]
    #[inline(always)]
    pub fn notrnps(self) -> &'a mut crate::W<REG> {
        self.variant(Tot::Notrnps)
    }
    #[doc = "Bits in bytes are transposed, but bytes are not transposed."]
    #[inline(always)]
    pub fn bts_trnps(self) -> &'a mut crate::W<REG> {
        self.variant(Tot::BtsTrnps)
    }
    #[doc = "Both bits in bytes and bytes are transposed."]
    #[inline(always)]
    pub fn byts_bts_trnps(self) -> &'a mut crate::W<REG> {
        self.variant(Tot::BytsBtsTrnps)
    }
    #[doc = "Only bytes are transposed, no bits in a byte are transposed."]
    #[inline(always)]
    pub fn byts_trnps(self) -> &'a mut crate::W<REG> {
        self.variant(Tot::BytsTrnps)
    }
}
impl R {
    #[doc = "Bit 24 - TCRC"]
    #[inline(always)]
    pub fn tcrc(&self) -> TcrcR {
        TcrcR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Write as Seed"]
    #[inline(always)]
    pub fn was(&self) -> WasR {
        WasR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Complement Read of CRC Data Register"]
    #[inline(always)]
    pub fn fxor(&self) -> FxorR {
        FxorR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bits 28:29 - Transpose Type for Read"]
    #[inline(always)]
    pub fn totr(&self) -> TotrR {
        TotrR::new(((self.bits >> 28) & 3) as u8)
    }
    #[doc = "Bits 30:31 - Transpose Type for Write"]
    #[inline(always)]
    pub fn tot(&self) -> TotR {
        TotR::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 24 - TCRC"]
    #[inline(always)]
    pub fn tcrc(&mut self) -> TcrcW<CtrlSpec> {
        TcrcW::new(self, 24)
    }
    #[doc = "Bit 25 - Write as Seed"]
    #[inline(always)]
    pub fn was(&mut self) -> WasW<CtrlSpec> {
        WasW::new(self, 25)
    }
    #[doc = "Bit 26 - Complement Read of CRC Data Register"]
    #[inline(always)]
    pub fn fxor(&mut self) -> FxorW<CtrlSpec> {
        FxorW::new(self, 26)
    }
    #[doc = "Bits 28:29 - Transpose Type for Read"]
    #[inline(always)]
    pub fn totr(&mut self) -> TotrW<CtrlSpec> {
        TotrW::new(self, 28)
    }
    #[doc = "Bits 30:31 - Transpose Type for Write"]
    #[inline(always)]
    pub fn tot(&mut self) -> TotW<CtrlSpec> {
        TotW::new(self, 30)
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
