#[doc = "Register `MRCC_GLB_CC2` reader"]
pub type R = crate::R<MrccGlbCc2Spec>;
#[doc = "Register `MRCC_GLB_CC2` writer"]
pub type W = crate::W<MrccGlbCc2Spec>;
#[doc = "RAMA\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rama {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Rama> for bool {
    #[inline(always)]
    fn from(variant: Rama) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMA` reader - RAMA"]
pub type RamaR = crate::BitReader<Rama>;
impl RamaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rama {
        match self.bits {
            false => Rama::Disabled,
            true => Rama::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rama::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rama::Enabled
    }
}
#[doc = "Field `RAMA` writer - RAMA"]
pub type RamaW<'a, REG> = crate::BitWriter<'a, REG, Rama>;
impl<'a, REG> RamaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rama::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rama::Enabled)
    }
}
#[doc = "RAMB\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ramb {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ramb> for bool {
    #[inline(always)]
    fn from(variant: Ramb) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMB` reader - RAMB"]
pub type RambR = crate::BitReader<Ramb>;
impl RambR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ramb {
        match self.bits {
            false => Ramb::Disabled,
            true => Ramb::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ramb::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ramb::Enabled
    }
}
#[doc = "Field `RAMB` writer - RAMB"]
pub type RambW<'a, REG> = crate::BitWriter<'a, REG, Ramb>;
impl<'a, REG> RambW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ramb::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ramb::Enabled)
    }
}
#[doc = "RAMC\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ramc {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Ramc> for bool {
    #[inline(always)]
    fn from(variant: Ramc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMC` reader - RAMC"]
pub type RamcR = crate::BitReader<Ramc>;
impl RamcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ramc {
        match self.bits {
            false => Ramc::Disabled,
            true => Ramc::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ramc::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ramc::Enabled
    }
}
#[doc = "Field `RAMC` writer - RAMC"]
pub type RamcW<'a, REG> = crate::BitWriter<'a, REG, Ramc>;
impl<'a, REG> RamcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ramc::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ramc::Enabled)
    }
}
#[doc = "GPIO0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Gpio0> for bool {
    #[inline(always)]
    fn from(variant: Gpio0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPIO0` reader - GPIO0"]
pub type Gpio0R = crate::BitReader<Gpio0>;
impl Gpio0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpio0 {
        match self.bits {
            false => Gpio0::Disabled,
            true => Gpio0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gpio0::Enabled
    }
}
#[doc = "Field `GPIO0` writer - GPIO0"]
pub type Gpio0W<'a, REG> = crate::BitWriter<'a, REG, Gpio0>;
impl<'a, REG> Gpio0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio0::Enabled)
    }
}
#[doc = "GPIO1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio1 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Gpio1> for bool {
    #[inline(always)]
    fn from(variant: Gpio1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPIO1` reader - GPIO1"]
pub type Gpio1R = crate::BitReader<Gpio1>;
impl Gpio1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpio1 {
        match self.bits {
            false => Gpio1::Disabled,
            true => Gpio1::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio1::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gpio1::Enabled
    }
}
#[doc = "Field `GPIO1` writer - GPIO1"]
pub type Gpio1W<'a, REG> = crate::BitWriter<'a, REG, Gpio1>;
impl<'a, REG> Gpio1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio1::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio1::Enabled)
    }
}
#[doc = "GPIO2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio2 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Gpio2> for bool {
    #[inline(always)]
    fn from(variant: Gpio2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPIO2` reader - GPIO2"]
pub type Gpio2R = crate::BitReader<Gpio2>;
impl Gpio2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpio2 {
        match self.bits {
            false => Gpio2::Disabled,
            true => Gpio2::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio2::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gpio2::Enabled
    }
}
#[doc = "Field `GPIO2` writer - GPIO2"]
pub type Gpio2W<'a, REG> = crate::BitWriter<'a, REG, Gpio2>;
impl<'a, REG> Gpio2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio2::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio2::Enabled)
    }
}
#[doc = "GPIO3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio3 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Gpio3> for bool {
    #[inline(always)]
    fn from(variant: Gpio3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPIO3` reader - GPIO3"]
pub type Gpio3R = crate::BitReader<Gpio3>;
impl Gpio3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpio3 {
        match self.bits {
            false => Gpio3::Disabled,
            true => Gpio3::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio3::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gpio3::Enabled
    }
}
#[doc = "Field `GPIO3` writer - GPIO3"]
pub type Gpio3W<'a, REG> = crate::BitWriter<'a, REG, Gpio3>;
impl<'a, REG> Gpio3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio3::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio3::Enabled)
    }
}
#[doc = "GPIO4\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio4 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Gpio4> for bool {
    #[inline(always)]
    fn from(variant: Gpio4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPIO4` reader - GPIO4"]
pub type Gpio4R = crate::BitReader<Gpio4>;
impl Gpio4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpio4 {
        match self.bits {
            false => Gpio4::Disabled,
            true => Gpio4::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio4::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gpio4::Enabled
    }
}
#[doc = "Field `GPIO4` writer - GPIO4"]
pub type Gpio4W<'a, REG> = crate::BitWriter<'a, REG, Gpio4>;
impl<'a, REG> Gpio4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio4::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio4::Enabled)
    }
}
#[doc = "MAU0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mau0 {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Mau0> for bool {
    #[inline(always)]
    fn from(variant: Mau0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MAU0` reader - MAU0"]
pub type Mau0R = crate::BitReader<Mau0>;
impl Mau0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mau0 {
        match self.bits {
            false => Mau0::Disabled,
            true => Mau0::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Mau0::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Mau0::Enabled
    }
}
#[doc = "Field `MAU0` writer - MAU0"]
pub type Mau0W<'a, REG> = crate::BitWriter<'a, REG, Mau0>;
impl<'a, REG> Mau0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mau0::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mau0::Enabled)
    }
}
#[doc = "ROMC\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Romc {
    #[doc = "0: Peripheral clock is disabled"]
    Disabled = 0,
    #[doc = "1: Peripheral clock is enabled"]
    Enabled = 1,
}
impl From<Romc> for bool {
    #[inline(always)]
    fn from(variant: Romc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROMC` reader - ROMC"]
pub type RomcR = crate::BitReader<Romc>;
impl RomcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Romc {
        match self.bits {
            false => Romc::Disabled,
            true => Romc::Enabled,
        }
    }
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Romc::Disabled
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Romc::Enabled
    }
}
#[doc = "Field `ROMC` writer - ROMC"]
pub type RomcW<'a, REG> = crate::BitWriter<'a, REG, Romc>;
impl<'a, REG> RomcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Peripheral clock is disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Romc::Disabled)
    }
    #[doc = "Peripheral clock is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Romc::Enabled)
    }
}
impl R {
    #[doc = "Bit 1 - RAMA"]
    #[inline(always)]
    pub fn rama(&self) -> RamaR {
        RamaR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RAMB"]
    #[inline(always)]
    pub fn ramb(&self) -> RambR {
        RambR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - RAMC"]
    #[inline(always)]
    pub fn ramc(&self) -> RamcR {
        RamcR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - GPIO0"]
    #[inline(always)]
    pub fn gpio0(&self) -> Gpio0R {
        Gpio0R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - GPIO1"]
    #[inline(always)]
    pub fn gpio1(&self) -> Gpio1R {
        Gpio1R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - GPIO2"]
    #[inline(always)]
    pub fn gpio2(&self) -> Gpio2R {
        Gpio2R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - GPIO3"]
    #[inline(always)]
    pub fn gpio3(&self) -> Gpio3R {
        Gpio3R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - GPIO4"]
    #[inline(always)]
    pub fn gpio4(&self) -> Gpio4R {
        Gpio4R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - MAU0"]
    #[inline(always)]
    pub fn mau0(&self) -> Mau0R {
        Mau0R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - ROMC"]
    #[inline(always)]
    pub fn romc(&self) -> RomcR {
        RomcR::new(((self.bits >> 10) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - RAMA"]
    #[inline(always)]
    pub fn rama(&mut self) -> RamaW<MrccGlbCc2Spec> {
        RamaW::new(self, 1)
    }
    #[doc = "Bit 2 - RAMB"]
    #[inline(always)]
    pub fn ramb(&mut self) -> RambW<MrccGlbCc2Spec> {
        RambW::new(self, 2)
    }
    #[doc = "Bit 3 - RAMC"]
    #[inline(always)]
    pub fn ramc(&mut self) -> RamcW<MrccGlbCc2Spec> {
        RamcW::new(self, 3)
    }
    #[doc = "Bit 4 - GPIO0"]
    #[inline(always)]
    pub fn gpio0(&mut self) -> Gpio0W<MrccGlbCc2Spec> {
        Gpio0W::new(self, 4)
    }
    #[doc = "Bit 5 - GPIO1"]
    #[inline(always)]
    pub fn gpio1(&mut self) -> Gpio1W<MrccGlbCc2Spec> {
        Gpio1W::new(self, 5)
    }
    #[doc = "Bit 6 - GPIO2"]
    #[inline(always)]
    pub fn gpio2(&mut self) -> Gpio2W<MrccGlbCc2Spec> {
        Gpio2W::new(self, 6)
    }
    #[doc = "Bit 7 - GPIO3"]
    #[inline(always)]
    pub fn gpio3(&mut self) -> Gpio3W<MrccGlbCc2Spec> {
        Gpio3W::new(self, 7)
    }
    #[doc = "Bit 8 - GPIO4"]
    #[inline(always)]
    pub fn gpio4(&mut self) -> Gpio4W<MrccGlbCc2Spec> {
        Gpio4W::new(self, 8)
    }
    #[doc = "Bit 9 - MAU0"]
    #[inline(always)]
    pub fn mau0(&mut self) -> Mau0W<MrccGlbCc2Spec> {
        Mau0W::new(self, 9)
    }
    #[doc = "Bit 10 - ROMC"]
    #[inline(always)]
    pub fn romc(&mut self) -> RomcW<MrccGlbCc2Spec> {
        RomcW::new(self, 10)
    }
}
#[doc = "AHB Clock Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_cc2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbCc2Spec;
impl crate::RegisterSpec for MrccGlbCc2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_glb_cc2::R`](R) reader structure"]
impl crate::Readable for MrccGlbCc2Spec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_cc2::W`](W) writer structure"]
impl crate::Writable for MrccGlbCc2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_CC2 to value 0"]
impl crate::Resettable for MrccGlbCc2Spec {}
