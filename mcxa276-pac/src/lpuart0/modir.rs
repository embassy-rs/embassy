#[doc = "Register `MODIR` reader"]
pub type R = crate::R<ModirSpec>;
#[doc = "Register `MODIR` writer"]
pub type W = crate::W<ModirSpec>;
#[doc = "Transmitter CTS Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txctse {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Txctse> for bool {
    #[inline(always)]
    fn from(variant: Txctse) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXCTSE` reader - Transmitter CTS Enable"]
pub type TxctseR = crate::BitReader<Txctse>;
impl TxctseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txctse {
        match self.bits {
            false => Txctse::Disabled,
            true => Txctse::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Txctse::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Txctse::Enabled
    }
}
#[doc = "Field `TXCTSE` writer - Transmitter CTS Enable"]
pub type TxctseW<'a, REG> = crate::BitWriter<'a, REG, Txctse>;
impl<'a, REG> TxctseW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txctse::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txctse::Enabled)
    }
}
#[doc = "Transmitter RTS Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txrtse {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Txrtse> for bool {
    #[inline(always)]
    fn from(variant: Txrtse) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXRTSE` reader - Transmitter RTS Enable"]
pub type TxrtseR = crate::BitReader<Txrtse>;
impl TxrtseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txrtse {
        match self.bits {
            false => Txrtse::Disabled,
            true => Txrtse::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Txrtse::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Txrtse::Enabled
    }
}
#[doc = "Field `TXRTSE` writer - Transmitter RTS Enable"]
pub type TxrtseW<'a, REG> = crate::BitWriter<'a, REG, Txrtse>;
impl<'a, REG> TxrtseW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txrtse::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Txrtse::Enabled)
    }
}
#[doc = "Transmitter RTS Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txrtspol {
    #[doc = "0: Active low"]
    Low = 0,
    #[doc = "1: Active high"]
    High = 1,
}
impl From<Txrtspol> for bool {
    #[inline(always)]
    fn from(variant: Txrtspol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXRTSPOL` reader - Transmitter RTS Polarity"]
pub type TxrtspolR = crate::BitReader<Txrtspol>;
impl TxrtspolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txrtspol {
        match self.bits {
            false => Txrtspol::Low,
            true => Txrtspol::High,
        }
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == Txrtspol::Low
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == Txrtspol::High
    }
}
#[doc = "Field `TXRTSPOL` writer - Transmitter RTS Polarity"]
pub type TxrtspolW<'a, REG> = crate::BitWriter<'a, REG, Txrtspol>;
impl<'a, REG> TxrtspolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active low"]
    #[inline(always)]
    pub fn low(self) -> &'a mut crate::W<REG> {
        self.variant(Txrtspol::Low)
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn high(self) -> &'a mut crate::W<REG> {
        self.variant(Txrtspol::High)
    }
}
#[doc = "Receiver RTS Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rxrtse {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rxrtse> for bool {
    #[inline(always)]
    fn from(variant: Rxrtse) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RXRTSE` reader - Receiver RTS Enable"]
pub type RxrtseR = crate::BitReader<Rxrtse>;
impl RxrtseR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rxrtse {
        match self.bits {
            false => Rxrtse::Disabled,
            true => Rxrtse::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rxrtse::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rxrtse::Enabled
    }
}
#[doc = "Field `RXRTSE` writer - Receiver RTS Enable"]
pub type RxrtseW<'a, REG> = crate::BitWriter<'a, REG, Rxrtse>;
impl<'a, REG> RxrtseW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxrtse::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rxrtse::Enabled)
    }
}
#[doc = "Transmit CTS Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txctsc {
    #[doc = "0: Sampled at the start of each character"]
    Start = 0,
    #[doc = "1: Sampled when the transmitter is idle"]
    Idle = 1,
}
impl From<Txctsc> for bool {
    #[inline(always)]
    fn from(variant: Txctsc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXCTSC` reader - Transmit CTS Configuration"]
pub type TxctscR = crate::BitReader<Txctsc>;
impl TxctscR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txctsc {
        match self.bits {
            false => Txctsc::Start,
            true => Txctsc::Idle,
        }
    }
    #[doc = "Sampled at the start of each character"]
    #[inline(always)]
    pub fn is_start(&self) -> bool {
        *self == Txctsc::Start
    }
    #[doc = "Sampled when the transmitter is idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Txctsc::Idle
    }
}
#[doc = "Field `TXCTSC` writer - Transmit CTS Configuration"]
pub type TxctscW<'a, REG> = crate::BitWriter<'a, REG, Txctsc>;
impl<'a, REG> TxctscW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Sampled at the start of each character"]
    #[inline(always)]
    pub fn start(self) -> &'a mut crate::W<REG> {
        self.variant(Txctsc::Start)
    }
    #[doc = "Sampled when the transmitter is idle"]
    #[inline(always)]
    pub fn idle(self) -> &'a mut crate::W<REG> {
        self.variant(Txctsc::Idle)
    }
}
#[doc = "Transmit CTS Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txctssrc {
    #[doc = "0: The CTS_B pin"]
    Cts = 0,
    #[doc = "1: An internal connection to the receiver address match result"]
    Match = 1,
}
impl From<Txctssrc> for bool {
    #[inline(always)]
    fn from(variant: Txctssrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXCTSSRC` reader - Transmit CTS Source"]
pub type TxctssrcR = crate::BitReader<Txctssrc>;
impl TxctssrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txctssrc {
        match self.bits {
            false => Txctssrc::Cts,
            true => Txctssrc::Match,
        }
    }
    #[doc = "The CTS_B pin"]
    #[inline(always)]
    pub fn is_cts(&self) -> bool {
        *self == Txctssrc::Cts
    }
    #[doc = "An internal connection to the receiver address match result"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Txctssrc::Match
    }
}
#[doc = "Field `TXCTSSRC` writer - Transmit CTS Source"]
pub type TxctssrcW<'a, REG> = crate::BitWriter<'a, REG, Txctssrc>;
impl<'a, REG> TxctssrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The CTS_B pin"]
    #[inline(always)]
    pub fn cts(self) -> &'a mut crate::W<REG> {
        self.variant(Txctssrc::Cts)
    }
    #[doc = "An internal connection to the receiver address match result"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Txctssrc::Match)
    }
}
#[doc = "Field `RTSWATER` reader - Receive RTS Configuration"]
pub type RtswaterR = crate::FieldReader;
#[doc = "Field `RTSWATER` writer - Receive RTS Configuration"]
pub type RtswaterW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Transmitter Narrow Pulse\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tnp {
    #[doc = "0: 1 / OSR"]
    OneSample = 0,
    #[doc = "1: 2 / OSR"]
    TwoSample = 1,
    #[doc = "2: 3 / OSR"]
    ThreeSample = 2,
    #[doc = "3: 4 / OSR"]
    FourSample = 3,
}
impl From<Tnp> for u8 {
    #[inline(always)]
    fn from(variant: Tnp) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tnp {
    type Ux = u8;
}
impl crate::IsEnum for Tnp {}
#[doc = "Field `TNP` reader - Transmitter Narrow Pulse"]
pub type TnpR = crate::FieldReader<Tnp>;
impl TnpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tnp {
        match self.bits {
            0 => Tnp::OneSample,
            1 => Tnp::TwoSample,
            2 => Tnp::ThreeSample,
            3 => Tnp::FourSample,
            _ => unreachable!(),
        }
    }
    #[doc = "1 / OSR"]
    #[inline(always)]
    pub fn is_one_sample(&self) -> bool {
        *self == Tnp::OneSample
    }
    #[doc = "2 / OSR"]
    #[inline(always)]
    pub fn is_two_sample(&self) -> bool {
        *self == Tnp::TwoSample
    }
    #[doc = "3 / OSR"]
    #[inline(always)]
    pub fn is_three_sample(&self) -> bool {
        *self == Tnp::ThreeSample
    }
    #[doc = "4 / OSR"]
    #[inline(always)]
    pub fn is_four_sample(&self) -> bool {
        *self == Tnp::FourSample
    }
}
#[doc = "Field `TNP` writer - Transmitter Narrow Pulse"]
pub type TnpW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tnp, crate::Safe>;
impl<'a, REG> TnpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1 / OSR"]
    #[inline(always)]
    pub fn one_sample(self) -> &'a mut crate::W<REG> {
        self.variant(Tnp::OneSample)
    }
    #[doc = "2 / OSR"]
    #[inline(always)]
    pub fn two_sample(self) -> &'a mut crate::W<REG> {
        self.variant(Tnp::TwoSample)
    }
    #[doc = "3 / OSR"]
    #[inline(always)]
    pub fn three_sample(self) -> &'a mut crate::W<REG> {
        self.variant(Tnp::ThreeSample)
    }
    #[doc = "4 / OSR"]
    #[inline(always)]
    pub fn four_sample(self) -> &'a mut crate::W<REG> {
        self.variant(Tnp::FourSample)
    }
}
#[doc = "IR Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Iren {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Iren> for bool {
    #[inline(always)]
    fn from(variant: Iren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IREN` reader - IR Enable"]
pub type IrenR = crate::BitReader<Iren>;
impl IrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Iren {
        match self.bits {
            false => Iren::Disabled,
            true => Iren::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Iren::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Iren::Enabled
    }
}
#[doc = "Field `IREN` writer - IR Enable"]
pub type IrenW<'a, REG> = crate::BitWriter<'a, REG, Iren>;
impl<'a, REG> IrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Iren::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Iren::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Transmitter CTS Enable"]
    #[inline(always)]
    pub fn txctse(&self) -> TxctseR {
        TxctseR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Transmitter RTS Enable"]
    #[inline(always)]
    pub fn txrtse(&self) -> TxrtseR {
        TxrtseR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Transmitter RTS Polarity"]
    #[inline(always)]
    pub fn txrtspol(&self) -> TxrtspolR {
        TxrtspolR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Receiver RTS Enable"]
    #[inline(always)]
    pub fn rxrtse(&self) -> RxrtseR {
        RxrtseR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Transmit CTS Configuration"]
    #[inline(always)]
    pub fn txctsc(&self) -> TxctscR {
        TxctscR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Transmit CTS Source"]
    #[inline(always)]
    pub fn txctssrc(&self) -> TxctssrcR {
        TxctssrcR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Receive RTS Configuration"]
    #[inline(always)]
    pub fn rtswater(&self) -> RtswaterR {
        RtswaterR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 16:17 - Transmitter Narrow Pulse"]
    #[inline(always)]
    pub fn tnp(&self) -> TnpR {
        TnpR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 18 - IR Enable"]
    #[inline(always)]
    pub fn iren(&self) -> IrenR {
        IrenR::new(((self.bits >> 18) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmitter CTS Enable"]
    #[inline(always)]
    pub fn txctse(&mut self) -> TxctseW<ModirSpec> {
        TxctseW::new(self, 0)
    }
    #[doc = "Bit 1 - Transmitter RTS Enable"]
    #[inline(always)]
    pub fn txrtse(&mut self) -> TxrtseW<ModirSpec> {
        TxrtseW::new(self, 1)
    }
    #[doc = "Bit 2 - Transmitter RTS Polarity"]
    #[inline(always)]
    pub fn txrtspol(&mut self) -> TxrtspolW<ModirSpec> {
        TxrtspolW::new(self, 2)
    }
    #[doc = "Bit 3 - Receiver RTS Enable"]
    #[inline(always)]
    pub fn rxrtse(&mut self) -> RxrtseW<ModirSpec> {
        RxrtseW::new(self, 3)
    }
    #[doc = "Bit 4 - Transmit CTS Configuration"]
    #[inline(always)]
    pub fn txctsc(&mut self) -> TxctscW<ModirSpec> {
        TxctscW::new(self, 4)
    }
    #[doc = "Bit 5 - Transmit CTS Source"]
    #[inline(always)]
    pub fn txctssrc(&mut self) -> TxctssrcW<ModirSpec> {
        TxctssrcW::new(self, 5)
    }
    #[doc = "Bits 8:9 - Receive RTS Configuration"]
    #[inline(always)]
    pub fn rtswater(&mut self) -> RtswaterW<ModirSpec> {
        RtswaterW::new(self, 8)
    }
    #[doc = "Bits 16:17 - Transmitter Narrow Pulse"]
    #[inline(always)]
    pub fn tnp(&mut self) -> TnpW<ModirSpec> {
        TnpW::new(self, 16)
    }
    #[doc = "Bit 18 - IR Enable"]
    #[inline(always)]
    pub fn iren(&mut self) -> IrenW<ModirSpec> {
        IrenW::new(self, 18)
    }
}
#[doc = "MODEM IrDA\n\nYou can [`read`](crate::Reg::read) this register and get [`modir::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modir::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModirSpec;
impl crate::RegisterSpec for ModirSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modir::R`](R) reader structure"]
impl crate::Readable for ModirSpec {}
#[doc = "`write(|w| ..)` method takes [`modir::W`](W) writer structure"]
impl crate::Writable for ModirSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODIR to value 0"]
impl crate::Resettable for ModirSpec {}
