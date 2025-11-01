#[doc = "Register `MCONFIG` reader"]
pub type R = crate::R<MconfigSpec>;
#[doc = "Register `MCONFIG` writer"]
pub type W = crate::W<MconfigSpec>;
#[doc = "Controller Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mstena {
    #[doc = "0: CONTROLLER_OFF"]
    MasterOff = 0,
    #[doc = "1: CONTROLLER_ON"]
    MasterOn = 1,
    #[doc = "2: CONTROLLER_CAPABLE"]
    MasterCapable = 2,
    #[doc = "3: I2C_CONTROLLER_MODE"]
    I2cMasterMode = 3,
}
impl From<Mstena> for u8 {
    #[inline(always)]
    fn from(variant: Mstena) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mstena {
    type Ux = u8;
}
impl crate::IsEnum for Mstena {}
#[doc = "Field `MSTENA` reader - Controller Enable"]
pub type MstenaR = crate::FieldReader<Mstena>;
impl MstenaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mstena {
        match self.bits {
            0 => Mstena::MasterOff,
            1 => Mstena::MasterOn,
            2 => Mstena::MasterCapable,
            3 => Mstena::I2cMasterMode,
            _ => unreachable!(),
        }
    }
    #[doc = "CONTROLLER_OFF"]
    #[inline(always)]
    pub fn is_master_off(&self) -> bool {
        *self == Mstena::MasterOff
    }
    #[doc = "CONTROLLER_ON"]
    #[inline(always)]
    pub fn is_master_on(&self) -> bool {
        *self == Mstena::MasterOn
    }
    #[doc = "CONTROLLER_CAPABLE"]
    #[inline(always)]
    pub fn is_master_capable(&self) -> bool {
        *self == Mstena::MasterCapable
    }
    #[doc = "I2C_CONTROLLER_MODE"]
    #[inline(always)]
    pub fn is_i2c_master_mode(&self) -> bool {
        *self == Mstena::I2cMasterMode
    }
}
#[doc = "Field `MSTENA` writer - Controller Enable"]
pub type MstenaW<'a, REG> = crate::FieldWriter<'a, REG, 2, Mstena, crate::Safe>;
impl<'a, REG> MstenaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "CONTROLLER_OFF"]
    #[inline(always)]
    pub fn master_off(self) -> &'a mut crate::W<REG> {
        self.variant(Mstena::MasterOff)
    }
    #[doc = "CONTROLLER_ON"]
    #[inline(always)]
    pub fn master_on(self) -> &'a mut crate::W<REG> {
        self.variant(Mstena::MasterOn)
    }
    #[doc = "CONTROLLER_CAPABLE"]
    #[inline(always)]
    pub fn master_capable(self) -> &'a mut crate::W<REG> {
        self.variant(Mstena::MasterCapable)
    }
    #[doc = "I2C_CONTROLLER_MODE"]
    #[inline(always)]
    pub fn i2c_master_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Mstena::I2cMasterMode)
    }
}
#[doc = "Disable Timeout\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disto {
    #[doc = "0: Enabled"]
    Enable = 0,
    #[doc = "1: Disabled, if configured"]
    Disable = 1,
}
impl From<Disto> for bool {
    #[inline(always)]
    fn from(variant: Disto) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DISTO` reader - Disable Timeout"]
pub type DistoR = crate::BitReader<Disto>;
impl DistoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Disto {
        match self.bits {
            false => Disto::Enable,
            true => Disto::Disable,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Disto::Enable
    }
    #[doc = "Disabled, if configured"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Disto::Disable
    }
}
#[doc = "Field `DISTO` writer - Disable Timeout"]
pub type DistoW<'a, REG> = crate::BitWriter<'a, REG, Disto>;
impl<'a, REG> DistoW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Disto::Enable)
    }
    #[doc = "Disabled, if configured"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Disto::Disable)
    }
}
#[doc = "High-Keeper\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Hkeep {
    #[doc = "0: None"]
    None = 0,
    #[doc = "1: WIRED_IN"]
    WiredIn = 1,
    #[doc = "2: PASSIVE_SDA (I2C mode, no clock stretches mode)"]
    PassiveSda = 2,
    #[doc = "3: PASSIVE_ON_SDA_SCL"]
    PassiveOnSdaScl = 3,
}
impl From<Hkeep> for u8 {
    #[inline(always)]
    fn from(variant: Hkeep) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Hkeep {
    type Ux = u8;
}
impl crate::IsEnum for Hkeep {}
#[doc = "Field `HKEEP` reader - High-Keeper"]
pub type HkeepR = crate::FieldReader<Hkeep>;
impl HkeepR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hkeep {
        match self.bits {
            0 => Hkeep::None,
            1 => Hkeep::WiredIn,
            2 => Hkeep::PassiveSda,
            3 => Hkeep::PassiveOnSdaScl,
            _ => unreachable!(),
        }
    }
    #[doc = "None"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Hkeep::None
    }
    #[doc = "WIRED_IN"]
    #[inline(always)]
    pub fn is_wired_in(&self) -> bool {
        *self == Hkeep::WiredIn
    }
    #[doc = "PASSIVE_SDA (I2C mode, no clock stretches mode)"]
    #[inline(always)]
    pub fn is_passive_sda(&self) -> bool {
        *self == Hkeep::PassiveSda
    }
    #[doc = "PASSIVE_ON_SDA_SCL"]
    #[inline(always)]
    pub fn is_passive_on_sda_scl(&self) -> bool {
        *self == Hkeep::PassiveOnSdaScl
    }
}
#[doc = "Field `HKEEP` writer - High-Keeper"]
pub type HkeepW<'a, REG> = crate::FieldWriter<'a, REG, 2, Hkeep, crate::Safe>;
impl<'a, REG> HkeepW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "None"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Hkeep::None)
    }
    #[doc = "WIRED_IN"]
    #[inline(always)]
    pub fn wired_in(self) -> &'a mut crate::W<REG> {
        self.variant(Hkeep::WiredIn)
    }
    #[doc = "PASSIVE_SDA (I2C mode, no clock stretches mode)"]
    #[inline(always)]
    pub fn passive_sda(self) -> &'a mut crate::W<REG> {
        self.variant(Hkeep::PassiveSda)
    }
    #[doc = "PASSIVE_ON_SDA_SCL"]
    #[inline(always)]
    pub fn passive_on_sda_scl(self) -> &'a mut crate::W<REG> {
        self.variant(Hkeep::PassiveOnSdaScl)
    }
}
#[doc = "Open-drain Stop\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Odstop {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Odstop> for bool {
    #[inline(always)]
    fn from(variant: Odstop) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ODSTOP` reader - Open-drain Stop"]
pub type OdstopR = crate::BitReader<Odstop>;
impl OdstopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Odstop {
        match self.bits {
            false => Odstop::Disable,
            true => Odstop::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Odstop::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Odstop::Enable
    }
}
#[doc = "Field `ODSTOP` writer - Open-drain Stop"]
pub type OdstopW<'a, REG> = crate::BitWriter<'a, REG, Odstop>;
impl<'a, REG> OdstopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Odstop::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Odstop::Enable)
    }
}
#[doc = "Field `PPBAUD` reader - Push-Pull Baud Rate"]
pub type PpbaudR = crate::FieldReader;
#[doc = "Field `PPBAUD` writer - Push-Pull Baud Rate"]
pub type PpbaudW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `PPLOW` reader - Push-Pull Low"]
pub type PplowR = crate::FieldReader;
#[doc = "Field `PPLOW` writer - Push-Pull Low"]
pub type PplowW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `ODBAUD` reader - Open-drain Baud Rate"]
pub type OdbaudR = crate::FieldReader;
#[doc = "Field `ODBAUD` writer - Open-drain Baud Rate"]
pub type OdbaudW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Open-drain High Push-Pull\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Odhpp {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Odhpp> for bool {
    #[inline(always)]
    fn from(variant: Odhpp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ODHPP` reader - Open-drain High Push-Pull"]
pub type OdhppR = crate::BitReader<Odhpp>;
impl OdhppR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Odhpp {
        match self.bits {
            false => Odhpp::Disable,
            true => Odhpp::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Odhpp::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Odhpp::Enable
    }
}
#[doc = "Field `ODHPP` writer - Open-drain High Push-Pull"]
pub type OdhppW<'a, REG> = crate::BitWriter<'a, REG, Odhpp>;
impl<'a, REG> OdhppW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Odhpp::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Odhpp::Enable)
    }
}
#[doc = "Field `SKEW` reader - Skew"]
pub type SkewR = crate::FieldReader;
#[doc = "Field `SKEW` writer - Skew"]
pub type SkewW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `I2CBAUD` reader - I2C Baud Rate"]
pub type I2cbaudR = crate::FieldReader;
#[doc = "Field `I2CBAUD` writer - I2C Baud Rate"]
pub type I2cbaudW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - Controller Enable"]
    #[inline(always)]
    pub fn mstena(&self) -> MstenaR {
        MstenaR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 3 - Disable Timeout"]
    #[inline(always)]
    pub fn disto(&self) -> DistoR {
        DistoR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:5 - High-Keeper"]
    #[inline(always)]
    pub fn hkeep(&self) -> HkeepR {
        HkeepR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bit 6 - Open-drain Stop"]
    #[inline(always)]
    pub fn odstop(&self) -> OdstopR {
        OdstopR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bits 8:11 - Push-Pull Baud Rate"]
    #[inline(always)]
    pub fn ppbaud(&self) -> PpbaudR {
        PpbaudR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 12:15 - Push-Pull Low"]
    #[inline(always)]
    pub fn pplow(&self) -> PplowR {
        PplowR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:23 - Open-drain Baud Rate"]
    #[inline(always)]
    pub fn odbaud(&self) -> OdbaudR {
        OdbaudR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bit 24 - Open-drain High Push-Pull"]
    #[inline(always)]
    pub fn odhpp(&self) -> OdhppR {
        OdhppR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bits 25:27 - Skew"]
    #[inline(always)]
    pub fn skew(&self) -> SkewR {
        SkewR::new(((self.bits >> 25) & 7) as u8)
    }
    #[doc = "Bits 28:31 - I2C Baud Rate"]
    #[inline(always)]
    pub fn i2cbaud(&self) -> I2cbaudR {
        I2cbaudR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Controller Enable"]
    #[inline(always)]
    pub fn mstena(&mut self) -> MstenaW<MconfigSpec> {
        MstenaW::new(self, 0)
    }
    #[doc = "Bit 3 - Disable Timeout"]
    #[inline(always)]
    pub fn disto(&mut self) -> DistoW<MconfigSpec> {
        DistoW::new(self, 3)
    }
    #[doc = "Bits 4:5 - High-Keeper"]
    #[inline(always)]
    pub fn hkeep(&mut self) -> HkeepW<MconfigSpec> {
        HkeepW::new(self, 4)
    }
    #[doc = "Bit 6 - Open-drain Stop"]
    #[inline(always)]
    pub fn odstop(&mut self) -> OdstopW<MconfigSpec> {
        OdstopW::new(self, 6)
    }
    #[doc = "Bits 8:11 - Push-Pull Baud Rate"]
    #[inline(always)]
    pub fn ppbaud(&mut self) -> PpbaudW<MconfigSpec> {
        PpbaudW::new(self, 8)
    }
    #[doc = "Bits 12:15 - Push-Pull Low"]
    #[inline(always)]
    pub fn pplow(&mut self) -> PplowW<MconfigSpec> {
        PplowW::new(self, 12)
    }
    #[doc = "Bits 16:23 - Open-drain Baud Rate"]
    #[inline(always)]
    pub fn odbaud(&mut self) -> OdbaudW<MconfigSpec> {
        OdbaudW::new(self, 16)
    }
    #[doc = "Bit 24 - Open-drain High Push-Pull"]
    #[inline(always)]
    pub fn odhpp(&mut self) -> OdhppW<MconfigSpec> {
        OdhppW::new(self, 24)
    }
    #[doc = "Bits 25:27 - Skew"]
    #[inline(always)]
    pub fn skew(&mut self) -> SkewW<MconfigSpec> {
        SkewW::new(self, 25)
    }
    #[doc = "Bits 28:31 - I2C Baud Rate"]
    #[inline(always)]
    pub fn i2cbaud(&mut self) -> I2cbaudW<MconfigSpec> {
        I2cbaudW::new(self, 28)
    }
}
#[doc = "Controller Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`mconfig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mconfig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MconfigSpec;
impl crate::RegisterSpec for MconfigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mconfig::R`](R) reader structure"]
impl crate::Readable for MconfigSpec {}
#[doc = "`write(|w| ..)` method takes [`mconfig::W`](W) writer structure"]
impl crate::Writable for MconfigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCONFIG to value 0"]
impl crate::Resettable for MconfigSpec {}
