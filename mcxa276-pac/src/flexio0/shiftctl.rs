#[doc = "Register `SHIFTCTL[%s]` reader"]
pub type R = crate::R<ShiftctlSpec>;
#[doc = "Register `SHIFTCTL[%s]` writer"]
pub type W = crate::W<ShiftctlSpec>;
#[doc = "Shifter Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Smod {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Receive mode; capture the current shifter content into SHIFTBUF on expiration of the timer"]
    Receive = 1,
    #[doc = "2: Transmit mode; load SHIFTBUF contents into the shifter on expiration of the timer"]
    Transmit = 2,
    #[doc = "4: Match Store mode; shifter data is compared to SHIFTBUF content on expiration of the timer"]
    Matchstore = 4,
    #[doc = "5: Match Continuous mode; shifter data is continuously compared to SHIFTBUF contents"]
    Matchcont = 5,
    #[doc = "6: State mode; SHIFTBUF contents store programmable state attributes"]
    State = 6,
    #[doc = "7: Logic mode; SHIFTBUF contents implement programmable logic lookup table"]
    Logic = 7,
}
impl From<Smod> for u8 {
    #[inline(always)]
    fn from(variant: Smod) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Smod {
    type Ux = u8;
}
impl crate::IsEnum for Smod {}
#[doc = "Field `SMOD` reader - Shifter Mode"]
pub type SmodR = crate::FieldReader<Smod>;
impl SmodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Smod> {
        match self.bits {
            0 => Some(Smod::Disable),
            1 => Some(Smod::Receive),
            2 => Some(Smod::Transmit),
            4 => Some(Smod::Matchstore),
            5 => Some(Smod::Matchcont),
            6 => Some(Smod::State),
            7 => Some(Smod::Logic),
            _ => None,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Smod::Disable
    }
    #[doc = "Receive mode; capture the current shifter content into SHIFTBUF on expiration of the timer"]
    #[inline(always)]
    pub fn is_receive(&self) -> bool {
        *self == Smod::Receive
    }
    #[doc = "Transmit mode; load SHIFTBUF contents into the shifter on expiration of the timer"]
    #[inline(always)]
    pub fn is_transmit(&self) -> bool {
        *self == Smod::Transmit
    }
    #[doc = "Match Store mode; shifter data is compared to SHIFTBUF content on expiration of the timer"]
    #[inline(always)]
    pub fn is_matchstore(&self) -> bool {
        *self == Smod::Matchstore
    }
    #[doc = "Match Continuous mode; shifter data is continuously compared to SHIFTBUF contents"]
    #[inline(always)]
    pub fn is_matchcont(&self) -> bool {
        *self == Smod::Matchcont
    }
    #[doc = "State mode; SHIFTBUF contents store programmable state attributes"]
    #[inline(always)]
    pub fn is_state(&self) -> bool {
        *self == Smod::State
    }
    #[doc = "Logic mode; SHIFTBUF contents implement programmable logic lookup table"]
    #[inline(always)]
    pub fn is_logic(&self) -> bool {
        *self == Smod::Logic
    }
}
#[doc = "Field `SMOD` writer - Shifter Mode"]
pub type SmodW<'a, REG> = crate::FieldWriter<'a, REG, 3, Smod>;
impl<'a, REG> SmodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::Disable)
    }
    #[doc = "Receive mode; capture the current shifter content into SHIFTBUF on expiration of the timer"]
    #[inline(always)]
    pub fn receive(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::Receive)
    }
    #[doc = "Transmit mode; load SHIFTBUF contents into the shifter on expiration of the timer"]
    #[inline(always)]
    pub fn transmit(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::Transmit)
    }
    #[doc = "Match Store mode; shifter data is compared to SHIFTBUF content on expiration of the timer"]
    #[inline(always)]
    pub fn matchstore(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::Matchstore)
    }
    #[doc = "Match Continuous mode; shifter data is continuously compared to SHIFTBUF contents"]
    #[inline(always)]
    pub fn matchcont(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::Matchcont)
    }
    #[doc = "State mode; SHIFTBUF contents store programmable state attributes"]
    #[inline(always)]
    pub fn state(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::State)
    }
    #[doc = "Logic mode; SHIFTBUF contents implement programmable logic lookup table"]
    #[inline(always)]
    pub fn logic(self) -> &'a mut crate::W<REG> {
        self.variant(Smod::Logic)
    }
}
#[doc = "Shifter Pin Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pinpol {
    #[doc = "0: Active high"]
    ActiveHigh = 0,
    #[doc = "1: Active low"]
    ActiveLow = 1,
}
impl From<Pinpol> for bool {
    #[inline(always)]
    fn from(variant: Pinpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PINPOL` reader - Shifter Pin Polarity"]
pub type PinpolR = crate::BitReader<Pinpol>;
impl PinpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pinpol {
        match self.bits {
            false => Pinpol::ActiveHigh,
            true => Pinpol::ActiveLow,
        }
    }
    #[doc = "Active high"]
    #[inline(always)]
    pub fn is_active_high(&self) -> bool {
        *self == Pinpol::ActiveHigh
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn is_active_low(&self) -> bool {
        *self == Pinpol::ActiveLow
    }
}
#[doc = "Field `PINPOL` writer - Shifter Pin Polarity"]
pub type PinpolW<'a, REG> = crate::BitWriter<'a, REG, Pinpol>;
impl<'a, REG> PinpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active high"]
    #[inline(always)]
    pub fn active_high(self) -> &'a mut crate::W<REG> {
        self.variant(Pinpol::ActiveHigh)
    }
    #[doc = "Active low"]
    #[inline(always)]
    pub fn active_low(self) -> &'a mut crate::W<REG> {
        self.variant(Pinpol::ActiveLow)
    }
}
#[doc = "Field `PINSEL` reader - Shifter Pin Select"]
pub type PinselR = crate::FieldReader;
#[doc = "Field `PINSEL` writer - Shifter Pin Select"]
pub type PinselW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Shifter Pin Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pincfg {
    #[doc = "0: Shifter pin output disabled"]
    Disable = 0,
    #[doc = "1: Shifter pin open-drain or bidirectional output enable"]
    OpendBidirouten = 1,
    #[doc = "2: Shifter pin bidirectional output data"]
    BidirOutdata = 2,
    #[doc = "3: Shifter pin output"]
    Output = 3,
}
impl From<Pincfg> for u8 {
    #[inline(always)]
    fn from(variant: Pincfg) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pincfg {
    type Ux = u8;
}
impl crate::IsEnum for Pincfg {}
#[doc = "Field `PINCFG` reader - Shifter Pin Configuration"]
pub type PincfgR = crate::FieldReader<Pincfg>;
impl PincfgR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pincfg {
        match self.bits {
            0 => Pincfg::Disable,
            1 => Pincfg::OpendBidirouten,
            2 => Pincfg::BidirOutdata,
            3 => Pincfg::Output,
            _ => unreachable!(),
        }
    }
    #[doc = "Shifter pin output disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pincfg::Disable
    }
    #[doc = "Shifter pin open-drain or bidirectional output enable"]
    #[inline(always)]
    pub fn is_opend_bidirouten(&self) -> bool {
        *self == Pincfg::OpendBidirouten
    }
    #[doc = "Shifter pin bidirectional output data"]
    #[inline(always)]
    pub fn is_bidir_outdata(&self) -> bool {
        *self == Pincfg::BidirOutdata
    }
    #[doc = "Shifter pin output"]
    #[inline(always)]
    pub fn is_output(&self) -> bool {
        *self == Pincfg::Output
    }
}
#[doc = "Field `PINCFG` writer - Shifter Pin Configuration"]
pub type PincfgW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pincfg, crate::Safe>;
impl<'a, REG> PincfgW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Shifter pin output disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::Disable)
    }
    #[doc = "Shifter pin open-drain or bidirectional output enable"]
    #[inline(always)]
    pub fn opend_bidirouten(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::OpendBidirouten)
    }
    #[doc = "Shifter pin bidirectional output data"]
    #[inline(always)]
    pub fn bidir_outdata(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::BidirOutdata)
    }
    #[doc = "Shifter pin output"]
    #[inline(always)]
    pub fn output(self) -> &'a mut crate::W<REG> {
        self.variant(Pincfg::Output)
    }
}
#[doc = "Timer Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timpol {
    #[doc = "0: Positive edge"]
    Posedge = 0,
    #[doc = "1: Negative edge"]
    Negedge = 1,
}
impl From<Timpol> for bool {
    #[inline(always)]
    fn from(variant: Timpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIMPOL` reader - Timer Polarity"]
pub type TimpolR = crate::BitReader<Timpol>;
impl TimpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timpol {
        match self.bits {
            false => Timpol::Posedge,
            true => Timpol::Negedge,
        }
    }
    #[doc = "Positive edge"]
    #[inline(always)]
    pub fn is_posedge(&self) -> bool {
        *self == Timpol::Posedge
    }
    #[doc = "Negative edge"]
    #[inline(always)]
    pub fn is_negedge(&self) -> bool {
        *self == Timpol::Negedge
    }
}
#[doc = "Field `TIMPOL` writer - Timer Polarity"]
pub type TimpolW<'a, REG> = crate::BitWriter<'a, REG, Timpol>;
impl<'a, REG> TimpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Positive edge"]
    #[inline(always)]
    pub fn posedge(self) -> &'a mut crate::W<REG> {
        self.variant(Timpol::Posedge)
    }
    #[doc = "Negative edge"]
    #[inline(always)]
    pub fn negedge(self) -> &'a mut crate::W<REG> {
        self.variant(Timpol::Negedge)
    }
}
#[doc = "Field `TIMSEL` reader - Timer Select"]
pub type TimselR = crate::FieldReader;
#[doc = "Field `TIMSEL` writer - Timer Select"]
pub type TimselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
impl R {
    #[doc = "Bits 0:2 - Shifter Mode"]
    #[inline(always)]
    pub fn smod(&self) -> SmodR {
        SmodR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 7 - Shifter Pin Polarity"]
    #[inline(always)]
    pub fn pinpol(&self) -> PinpolR {
        PinpolR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:12 - Shifter Pin Select"]
    #[inline(always)]
    pub fn pinsel(&self) -> PinselR {
        PinselR::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bits 16:17 - Shifter Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&self) -> PincfgR {
        PincfgR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 23 - Timer Polarity"]
    #[inline(always)]
    pub fn timpol(&self) -> TimpolR {
        TimpolR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:25 - Timer Select"]
    #[inline(always)]
    pub fn timsel(&self) -> TimselR {
        TimselR::new(((self.bits >> 24) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Shifter Mode"]
    #[inline(always)]
    pub fn smod(&mut self) -> SmodW<ShiftctlSpec> {
        SmodW::new(self, 0)
    }
    #[doc = "Bit 7 - Shifter Pin Polarity"]
    #[inline(always)]
    pub fn pinpol(&mut self) -> PinpolW<ShiftctlSpec> {
        PinpolW::new(self, 7)
    }
    #[doc = "Bits 8:12 - Shifter Pin Select"]
    #[inline(always)]
    pub fn pinsel(&mut self) -> PinselW<ShiftctlSpec> {
        PinselW::new(self, 8)
    }
    #[doc = "Bits 16:17 - Shifter Pin Configuration"]
    #[inline(always)]
    pub fn pincfg(&mut self) -> PincfgW<ShiftctlSpec> {
        PincfgW::new(self, 16)
    }
    #[doc = "Bit 23 - Timer Polarity"]
    #[inline(always)]
    pub fn timpol(&mut self) -> TimpolW<ShiftctlSpec> {
        TimpolW::new(self, 23)
    }
    #[doc = "Bits 24:25 - Timer Select"]
    #[inline(always)]
    pub fn timsel(&mut self) -> TimselW<ShiftctlSpec> {
        TimselW::new(self, 24)
    }
}
#[doc = "Shifter Control\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftctlSpec;
impl crate::RegisterSpec for ShiftctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftctl::R`](R) reader structure"]
impl crate::Readable for ShiftctlSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftctl::W`](W) writer structure"]
impl crate::Writable for ShiftctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTCTL[%s] to value 0"]
impl crate::Resettable for ShiftctlSpec {}
