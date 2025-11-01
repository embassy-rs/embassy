#[doc = "Register `MINTSET` reader"]
pub type R = crate::R<MintsetSpec>;
#[doc = "Register `MINTSET` writer"]
pub type W = crate::W<MintsetSpec>;
#[doc = "Target Start Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slvstart {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Slvstart> for bool {
    #[inline(always)]
    fn from(variant: Slvstart) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLVSTART` reader - Target Start Interrupt Enable"]
pub type SlvstartR = crate::BitReader<Slvstart>;
impl SlvstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slvstart {
        match self.bits {
            false => Slvstart::Disable,
            true => Slvstart::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Slvstart::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Slvstart::Enable
    }
}
#[doc = "Field `SLVSTART` writer - Target Start Interrupt Enable"]
pub type SlvstartW<'a, REG> = crate::BitWriter1S<'a, REG, Slvstart>;
impl<'a, REG> SlvstartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Slvstart::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Slvstart::Enable)
    }
}
#[doc = "Controller Control Done Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mctrldone {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Mctrldone> for bool {
    #[inline(always)]
    fn from(variant: Mctrldone) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MCTRLDONE` reader - Controller Control Done Interrupt Enable"]
pub type MctrldoneR = crate::BitReader<Mctrldone>;
impl MctrldoneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mctrldone {
        match self.bits {
            false => Mctrldone::Disable,
            true => Mctrldone::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Mctrldone::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Mctrldone::Enable
    }
}
#[doc = "Field `MCTRLDONE` writer - Controller Control Done Interrupt Enable"]
pub type MctrldoneW<'a, REG> = crate::BitWriter1S<'a, REG, Mctrldone>;
impl<'a, REG> MctrldoneW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Mctrldone::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Mctrldone::Enable)
    }
}
#[doc = "Completed Message Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complete {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Complete> for bool {
    #[inline(always)]
    fn from(variant: Complete) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COMPLETE` reader - Completed Message Interrupt Enable"]
pub type CompleteR = crate::BitReader<Complete>;
impl CompleteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Complete {
        match self.bits {
            false => Complete::Disable,
            true => Complete::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Complete::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Complete::Enable
    }
}
#[doc = "Field `COMPLETE` writer - Completed Message Interrupt Enable"]
pub type CompleteW<'a, REG> = crate::BitWriter1S<'a, REG, Complete>;
impl<'a, REG> CompleteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Complete::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Complete::Enable)
    }
}
#[doc = "Field `RXPEND` reader - Receive Pending Interrupt Enable"]
pub type RxpendR = crate::BitReader;
#[doc = "Field `RXPEND` writer - Receive Pending Interrupt Enable"]
pub type RxpendW<'a, REG> = crate::BitWriter1S<'a, REG>;
#[doc = "Transmit Buffer/FIFO Not Full Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txnotfull {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Txnotfull> for bool {
    #[inline(always)]
    fn from(variant: Txnotfull) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXNOTFULL` reader - Transmit Buffer/FIFO Not Full Interrupt Enable"]
pub type TxnotfullR = crate::BitReader<Txnotfull>;
impl TxnotfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txnotfull {
        match self.bits {
            false => Txnotfull::Disable,
            true => Txnotfull::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Txnotfull::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Txnotfull::Enable
    }
}
#[doc = "Field `TXNOTFULL` writer - Transmit Buffer/FIFO Not Full Interrupt Enable"]
pub type TxnotfullW<'a, REG> = crate::BitWriter1S<'a, REG, Txnotfull>;
impl<'a, REG> TxnotfullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Txnotfull::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Txnotfull::Enable)
    }
}
#[doc = "IBI Won Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibiwon {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Ibiwon> for bool {
    #[inline(always)]
    fn from(variant: Ibiwon) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIWON` reader - IBI Won Interrupt Enable"]
pub type IbiwonR = crate::BitReader<Ibiwon>;
impl IbiwonR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibiwon {
        match self.bits {
            false => Ibiwon::Disable,
            true => Ibiwon::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ibiwon::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ibiwon::Enable
    }
}
#[doc = "Field `IBIWON` writer - IBI Won Interrupt Enable"]
pub type IbiwonW<'a, REG> = crate::BitWriter1S<'a, REG, Ibiwon>;
impl<'a, REG> IbiwonW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiwon::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiwon::Enable)
    }
}
#[doc = "Error or Warning (ERRWARN) Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errwarn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Errwarn> for bool {
    #[inline(always)]
    fn from(variant: Errwarn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRWARN` reader - Error or Warning (ERRWARN) Interrupt Enable"]
pub type ErrwarnR = crate::BitReader<Errwarn>;
impl ErrwarnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Errwarn {
        match self.bits {
            false => Errwarn::Disable,
            true => Errwarn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Errwarn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Errwarn::Enable
    }
}
#[doc = "Field `ERRWARN` writer - Error or Warning (ERRWARN) Interrupt Enable"]
pub type ErrwarnW<'a, REG> = crate::BitWriter1S<'a, REG, Errwarn>;
impl<'a, REG> ErrwarnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Errwarn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Errwarn::Enable)
    }
}
#[doc = "Now Controller Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nowmaster {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Nowmaster> for bool {
    #[inline(always)]
    fn from(variant: Nowmaster) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOWMASTER` reader - Now Controller Interrupt Enable"]
pub type NowmasterR = crate::BitReader<Nowmaster>;
impl NowmasterR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nowmaster {
        match self.bits {
            false => Nowmaster::Disable,
            true => Nowmaster::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Nowmaster::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Nowmaster::Enable
    }
}
#[doc = "Field `NOWMASTER` writer - Now Controller Interrupt Enable"]
pub type NowmasterW<'a, REG> = crate::BitWriter1S<'a, REG, Nowmaster>;
impl<'a, REG> NowmasterW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Nowmaster::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Nowmaster::Enable)
    }
}
impl R {
    #[doc = "Bit 8 - Target Start Interrupt Enable"]
    #[inline(always)]
    pub fn slvstart(&self) -> SlvstartR {
        SlvstartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Controller Control Done Interrupt Enable"]
    #[inline(always)]
    pub fn mctrldone(&self) -> MctrldoneR {
        MctrldoneR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Completed Message Interrupt Enable"]
    #[inline(always)]
    pub fn complete(&self) -> CompleteR {
        CompleteR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Receive Pending Interrupt Enable"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Transmit Buffer/FIFO Not Full Interrupt Enable"]
    #[inline(always)]
    pub fn txnotfull(&self) -> TxnotfullR {
        TxnotfullR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - IBI Won Interrupt Enable"]
    #[inline(always)]
    pub fn ibiwon(&self) -> IbiwonR {
        IbiwonR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - Error or Warning (ERRWARN) Interrupt Enable"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 19 - Now Controller Interrupt Enable"]
    #[inline(always)]
    pub fn nowmaster(&self) -> NowmasterR {
        NowmasterR::new(((self.bits >> 19) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - Target Start Interrupt Enable"]
    #[inline(always)]
    pub fn slvstart(&mut self) -> SlvstartW<MintsetSpec> {
        SlvstartW::new(self, 8)
    }
    #[doc = "Bit 9 - Controller Control Done Interrupt Enable"]
    #[inline(always)]
    pub fn mctrldone(&mut self) -> MctrldoneW<MintsetSpec> {
        MctrldoneW::new(self, 9)
    }
    #[doc = "Bit 10 - Completed Message Interrupt Enable"]
    #[inline(always)]
    pub fn complete(&mut self) -> CompleteW<MintsetSpec> {
        CompleteW::new(self, 10)
    }
    #[doc = "Bit 11 - Receive Pending Interrupt Enable"]
    #[inline(always)]
    pub fn rxpend(&mut self) -> RxpendW<MintsetSpec> {
        RxpendW::new(self, 11)
    }
    #[doc = "Bit 12 - Transmit Buffer/FIFO Not Full Interrupt Enable"]
    #[inline(always)]
    pub fn txnotfull(&mut self) -> TxnotfullW<MintsetSpec> {
        TxnotfullW::new(self, 12)
    }
    #[doc = "Bit 13 - IBI Won Interrupt Enable"]
    #[inline(always)]
    pub fn ibiwon(&mut self) -> IbiwonW<MintsetSpec> {
        IbiwonW::new(self, 13)
    }
    #[doc = "Bit 15 - Error or Warning (ERRWARN) Interrupt Enable"]
    #[inline(always)]
    pub fn errwarn(&mut self) -> ErrwarnW<MintsetSpec> {
        ErrwarnW::new(self, 15)
    }
    #[doc = "Bit 19 - Now Controller Interrupt Enable"]
    #[inline(always)]
    pub fn nowmaster(&mut self) -> NowmasterW<MintsetSpec> {
        NowmasterW::new(self, 19)
    }
}
#[doc = "Controller Interrupt Set\n\nYou can [`read`](crate::Reg::read) this register and get [`mintset::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mintset::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MintsetSpec;
impl crate::RegisterSpec for MintsetSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mintset::R`](R) reader structure"]
impl crate::Readable for MintsetSpec {}
#[doc = "`write(|w| ..)` method takes [`mintset::W`](W) writer structure"]
impl crate::Writable for MintsetSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0008_bf00;
}
#[doc = "`reset()` method sets MINTSET to value 0"]
impl crate::Resettable for MintsetSpec {}
