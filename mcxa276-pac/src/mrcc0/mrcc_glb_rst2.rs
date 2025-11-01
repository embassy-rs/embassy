#[doc = "Register `MRCC_GLB_RST2` reader"]
pub type R = crate::R<MrccGlbRst2Spec>;
#[doc = "Register `MRCC_GLB_RST2` writer"]
pub type W = crate::W<MrccGlbRst2Spec>;
#[doc = "GPIO0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio0 {
    #[doc = "0: Peripheral is held in reset"]
    Disabled = 0,
    #[doc = "1: Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio0::Disabled
    }
    #[doc = "Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio0::Disabled)
    }
    #[doc = "Peripheral is released from reset"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio0::Enabled)
    }
}
#[doc = "GPIO1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio1 {
    #[doc = "0: Peripheral is held in reset"]
    Disabled = 0,
    #[doc = "1: Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio1::Disabled
    }
    #[doc = "Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio1::Disabled)
    }
    #[doc = "Peripheral is released from reset"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio1::Enabled)
    }
}
#[doc = "GPIO2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio2 {
    #[doc = "0: Peripheral is held in reset"]
    Disabled = 0,
    #[doc = "1: Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio2::Disabled
    }
    #[doc = "Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio2::Disabled)
    }
    #[doc = "Peripheral is released from reset"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio2::Enabled)
    }
}
#[doc = "GPIO3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio3 {
    #[doc = "0: Peripheral is held in reset"]
    Disabled = 0,
    #[doc = "1: Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio3::Disabled
    }
    #[doc = "Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio3::Disabled)
    }
    #[doc = "Peripheral is released from reset"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio3::Enabled)
    }
}
#[doc = "GPIO4\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpio4 {
    #[doc = "0: Peripheral is held in reset"]
    Disabled = 0,
    #[doc = "1: Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gpio4::Disabled
    }
    #[doc = "Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio4::Disabled)
    }
    #[doc = "Peripheral is released from reset"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gpio4::Enabled)
    }
}
#[doc = "MAU0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mau0 {
    #[doc = "0: Peripheral is held in reset"]
    Disabled = 0,
    #[doc = "1: Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Mau0::Disabled
    }
    #[doc = "Peripheral is released from reset"]
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
    #[doc = "Peripheral is held in reset"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mau0::Disabled)
    }
    #[doc = "Peripheral is released from reset"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Mau0::Enabled)
    }
}
impl R {
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
}
impl W {
    #[doc = "Bit 4 - GPIO0"]
    #[inline(always)]
    pub fn gpio0(&mut self) -> Gpio0W<MrccGlbRst2Spec> {
        Gpio0W::new(self, 4)
    }
    #[doc = "Bit 5 - GPIO1"]
    #[inline(always)]
    pub fn gpio1(&mut self) -> Gpio1W<MrccGlbRst2Spec> {
        Gpio1W::new(self, 5)
    }
    #[doc = "Bit 6 - GPIO2"]
    #[inline(always)]
    pub fn gpio2(&mut self) -> Gpio2W<MrccGlbRst2Spec> {
        Gpio2W::new(self, 6)
    }
    #[doc = "Bit 7 - GPIO3"]
    #[inline(always)]
    pub fn gpio3(&mut self) -> Gpio3W<MrccGlbRst2Spec> {
        Gpio3W::new(self, 7)
    }
    #[doc = "Bit 8 - GPIO4"]
    #[inline(always)]
    pub fn gpio4(&mut self) -> Gpio4W<MrccGlbRst2Spec> {
        Gpio4W::new(self, 8)
    }
    #[doc = "Bit 9 - MAU0"]
    #[inline(always)]
    pub fn mau0(&mut self) -> Mau0W<MrccGlbRst2Spec> {
        Mau0W::new(self, 9)
    }
}
#[doc = "Peripheral Reset Control 2\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_glb_rst2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbRst2Spec;
impl crate::RegisterSpec for MrccGlbRst2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_glb_rst2::R`](R) reader structure"]
impl crate::Readable for MrccGlbRst2Spec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_rst2::W`](W) writer structure"]
impl crate::Writable for MrccGlbRst2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_RST2 to value 0"]
impl crate::Resettable for MrccGlbRst2Spec {}
