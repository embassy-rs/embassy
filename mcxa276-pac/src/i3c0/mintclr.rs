#[doc = "Register `MINTCLR` reader"]
pub type R = crate::R<MintclrSpec>;
#[doc = "Register `MINTCLR` writer"]
pub type W = crate::W<MintclrSpec>;
#[doc = "SLVSTART Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slvstart {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Slvstart> for bool {
    #[inline(always)]
    fn from(variant: Slvstart) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLVSTART` reader - SLVSTART Interrupt Enable Clear Flag"]
pub type SlvstartR = crate::BitReader<Slvstart>;
impl SlvstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slvstart {
        match self.bits {
            false => Slvstart::None,
            true => Slvstart::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Slvstart::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Slvstart::Clear
    }
}
#[doc = "Field `SLVSTART` writer - SLVSTART Interrupt Enable Clear Flag"]
pub type SlvstartW<'a, REG> = crate::BitWriter1C<'a, REG, Slvstart>;
impl<'a, REG> SlvstartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Slvstart::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Slvstart::Clear)
    }
}
#[doc = "MCTRLDONE Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mctrldone {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Mctrldone> for bool {
    #[inline(always)]
    fn from(variant: Mctrldone) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MCTRLDONE` reader - MCTRLDONE Interrupt Enable Clear Flag"]
pub type MctrldoneR = crate::BitReader<Mctrldone>;
impl MctrldoneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mctrldone {
        match self.bits {
            false => Mctrldone::None,
            true => Mctrldone::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Mctrldone::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Mctrldone::Clear
    }
}
#[doc = "Field `MCTRLDONE` writer - MCTRLDONE Interrupt Enable Clear Flag"]
pub type MctrldoneW<'a, REG> = crate::BitWriter1C<'a, REG, Mctrldone>;
impl<'a, REG> MctrldoneW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Mctrldone::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Mctrldone::Clear)
    }
}
#[doc = "COMPLETE Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complete {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Complete> for bool {
    #[inline(always)]
    fn from(variant: Complete) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COMPLETE` reader - COMPLETE Interrupt Enable Clear Flag"]
pub type CompleteR = crate::BitReader<Complete>;
impl CompleteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Complete {
        match self.bits {
            false => Complete::None,
            true => Complete::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Complete::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Complete::Clear
    }
}
#[doc = "Field `COMPLETE` writer - COMPLETE Interrupt Enable Clear Flag"]
pub type CompleteW<'a, REG> = crate::BitWriter1C<'a, REG, Complete>;
impl<'a, REG> CompleteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Complete::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Complete::Clear)
    }
}
#[doc = "RXPEND Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxpend {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Rxpend> for bool {
    #[inline(always)]
    fn from(variant: Rxpend) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXPEND` reader - RXPEND Interrupt Enable Clear Flag"]
pub type RxpendR = crate::BitReader<Rxpend>;
impl RxpendR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxpend {
        match self.bits {
            false => Rxpend::None,
            true => Rxpend::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Rxpend::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Rxpend::Clear
    }
}
#[doc = "Field `RXPEND` writer - RXPEND Interrupt Enable Clear Flag"]
pub type RxpendW<'a, REG> = crate::BitWriter1C<'a, REG, Rxpend>;
impl<'a, REG> RxpendW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Rxpend::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Rxpend::Clear)
    }
}
#[doc = "TXNOTFULL Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txnotfull {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Txnotfull> for bool {
    #[inline(always)]
    fn from(variant: Txnotfull) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXNOTFULL` reader - TXNOTFULL Interrupt Enable Clear Flag"]
pub type TxnotfullR = crate::BitReader<Txnotfull>;
impl TxnotfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txnotfull {
        match self.bits {
            false => Txnotfull::None,
            true => Txnotfull::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Txnotfull::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Txnotfull::Clear
    }
}
#[doc = "Field `TXNOTFULL` writer - TXNOTFULL Interrupt Enable Clear Flag"]
pub type TxnotfullW<'a, REG> = crate::BitWriter1C<'a, REG, Txnotfull>;
impl<'a, REG> TxnotfullW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Txnotfull::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Txnotfull::Clear)
    }
}
#[doc = "IBIWON Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibiwon {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Ibiwon> for bool {
    #[inline(always)]
    fn from(variant: Ibiwon) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIWON` reader - IBIWON Interrupt Enable Clear Flag"]
pub type IbiwonR = crate::BitReader<Ibiwon>;
impl IbiwonR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibiwon {
        match self.bits {
            false => Ibiwon::None,
            true => Ibiwon::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Ibiwon::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Ibiwon::Clear
    }
}
#[doc = "Field `IBIWON` writer - IBIWON Interrupt Enable Clear Flag"]
pub type IbiwonW<'a, REG> = crate::BitWriter1C<'a, REG, Ibiwon>;
impl<'a, REG> IbiwonW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiwon::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiwon::Clear)
    }
}
#[doc = "ERRWARN Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errwarn {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Errwarn> for bool {
    #[inline(always)]
    fn from(variant: Errwarn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRWARN` reader - ERRWARN Interrupt Enable Clear Flag"]
pub type ErrwarnR = crate::BitReader<Errwarn>;
impl ErrwarnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Errwarn {
        match self.bits {
            false => Errwarn::None,
            true => Errwarn::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Errwarn::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Errwarn::Clear
    }
}
#[doc = "Field `ERRWARN` writer - ERRWARN Interrupt Enable Clear Flag"]
pub type ErrwarnW<'a, REG> = crate::BitWriter1C<'a, REG, Errwarn>;
impl<'a, REG> ErrwarnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Errwarn::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Errwarn::Clear)
    }
}
#[doc = "NOWCONTROLLER Interrupt Enable Clear Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nowmaster {
    #[doc = "0: No effect"]
    None = 0,
    #[doc = "1: Interrupt enable cleared"]
    Clear = 1,
}
impl From<Nowmaster> for bool {
    #[inline(always)]
    fn from(variant: Nowmaster) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOWMASTER` reader - NOWCONTROLLER Interrupt Enable Clear Flag"]
pub type NowmasterR = crate::BitReader<Nowmaster>;
impl NowmasterR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nowmaster {
        match self.bits {
            false => Nowmaster::None,
            true => Nowmaster::Clear,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Nowmaster::None
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Nowmaster::Clear
    }
}
#[doc = "Field `NOWMASTER` writer - NOWCONTROLLER Interrupt Enable Clear Flag"]
pub type NowmasterW<'a, REG> = crate::BitWriter1C<'a, REG, Nowmaster>;
impl<'a, REG> NowmasterW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Nowmaster::None)
    }
    #[doc = "Interrupt enable cleared"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Nowmaster::Clear)
    }
}
impl R {
    #[doc = "Bit 8 - SLVSTART Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn slvstart(&self) -> SlvstartR {
        SlvstartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - MCTRLDONE Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn mctrldone(&self) -> MctrldoneR {
        MctrldoneR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - COMPLETE Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn complete(&self) -> CompleteR {
        CompleteR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - RXPEND Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - TXNOTFULL Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn txnotfull(&self) -> TxnotfullR {
        TxnotfullR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - IBIWON Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn ibiwon(&self) -> IbiwonR {
        IbiwonR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - ERRWARN Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 19 - NOWCONTROLLER Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn nowmaster(&self) -> NowmasterR {
        NowmasterR::new(((self.bits >> 19) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - SLVSTART Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn slvstart(&mut self) -> SlvstartW<MintclrSpec> {
        SlvstartW::new(self, 8)
    }
    #[doc = "Bit 9 - MCTRLDONE Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn mctrldone(&mut self) -> MctrldoneW<MintclrSpec> {
        MctrldoneW::new(self, 9)
    }
    #[doc = "Bit 10 - COMPLETE Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn complete(&mut self) -> CompleteW<MintclrSpec> {
        CompleteW::new(self, 10)
    }
    #[doc = "Bit 11 - RXPEND Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn rxpend(&mut self) -> RxpendW<MintclrSpec> {
        RxpendW::new(self, 11)
    }
    #[doc = "Bit 12 - TXNOTFULL Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn txnotfull(&mut self) -> TxnotfullW<MintclrSpec> {
        TxnotfullW::new(self, 12)
    }
    #[doc = "Bit 13 - IBIWON Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn ibiwon(&mut self) -> IbiwonW<MintclrSpec> {
        IbiwonW::new(self, 13)
    }
    #[doc = "Bit 15 - ERRWARN Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn errwarn(&mut self) -> ErrwarnW<MintclrSpec> {
        ErrwarnW::new(self, 15)
    }
    #[doc = "Bit 19 - NOWCONTROLLER Interrupt Enable Clear Flag"]
    #[inline(always)]
    pub fn nowmaster(&mut self) -> NowmasterW<MintclrSpec> {
        NowmasterW::new(self, 19)
    }
}
#[doc = "Controller Interrupt Clear\n\nYou can [`read`](crate::Reg::read) this register and get [`mintclr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mintclr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MintclrSpec;
impl crate::RegisterSpec for MintclrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mintclr::R`](R) reader structure"]
impl crate::Readable for MintclrSpec {}
#[doc = "`write(|w| ..)` method takes [`mintclr::W`](W) writer structure"]
impl crate::Writable for MintclrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0008_bf00;
}
#[doc = "`reset()` method sets MINTCLR to value 0"]
impl crate::Resettable for MintclrSpec {}
