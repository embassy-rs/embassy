#[doc = "Register `PWM1_EXT_CLK` reader"]
pub type R = crate::R<Pwm1ExtClkSpec>;
#[doc = "Register `PWM1_EXT_CLK` writer"]
pub type W = crate::W<Pwm1ExtClkSpec>;
#[doc = "Trigger input connections for PWM\n\nValue on reset: 15"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trigin {
    #[doc = "1: clk_16k\\[1\\] input is selected"]
    Val1 = 1,
    #[doc = "2: clk_in input is selected"]
    Val2 = 2,
    #[doc = "3: AOI0_OUT0 input is selected"]
    Val3 = 3,
    #[doc = "4: AOI0_OUT1 input is selected"]
    Val4 = 4,
    #[doc = "5: EXTTRIG_IN0 input is selected"]
    Val5 = 5,
    #[doc = "6: EXTTRIG_IN7 input is selected"]
    Val6 = 6,
    #[doc = "7: AOI1_OUT0 input is selected"]
    Val7 = 7,
    #[doc = "8: AOI1_OUT1 input is selected"]
    Val8 = 8,
}
impl From<Trigin> for u8 {
    #[inline(always)]
    fn from(variant: Trigin) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Trigin {
    type Ux = u8;
}
impl crate::IsEnum for Trigin {}
#[doc = "Field `TRIGIN` reader - Trigger input connections for PWM"]
pub type TriginR = crate::FieldReader<Trigin>;
impl TriginR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Trigin> {
        match self.bits {
            1 => Some(Trigin::Val1),
            2 => Some(Trigin::Val2),
            3 => Some(Trigin::Val3),
            4 => Some(Trigin::Val4),
            5 => Some(Trigin::Val5),
            6 => Some(Trigin::Val6),
            7 => Some(Trigin::Val7),
            8 => Some(Trigin::Val8),
            _ => None,
        }
    }
    #[doc = "clk_16k\\[1\\] input is selected"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == Trigin::Val1
    }
    #[doc = "clk_in input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Trigin::Val2
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Trigin::Val3
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val4(&self) -> bool {
        *self == Trigin::Val4
    }
    #[doc = "EXTTRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Trigin::Val5
    }
    #[doc = "EXTTRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Trigin::Val6
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn is_val7(&self) -> bool {
        *self == Trigin::Val7
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn is_val8(&self) -> bool {
        *self == Trigin::Val8
    }
}
#[doc = "Field `TRIGIN` writer - Trigger input connections for PWM"]
pub type TriginW<'a, REG> = crate::FieldWriter<'a, REG, 4, Trigin>;
impl<'a, REG> TriginW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "clk_16k\\[1\\] input is selected"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val1)
    }
    #[doc = "clk_in input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val2)
    }
    #[doc = "AOI0_OUT0 input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val3)
    }
    #[doc = "AOI0_OUT1 input is selected"]
    #[inline(always)]
    pub fn val4(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val4)
    }
    #[doc = "EXTTRIG_IN0 input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val5)
    }
    #[doc = "EXTTRIG_IN7 input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val6)
    }
    #[doc = "AOI1_OUT0 input is selected"]
    #[inline(always)]
    pub fn val7(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val7)
    }
    #[doc = "AOI1_OUT1 input is selected"]
    #[inline(always)]
    pub fn val8(self) -> &'a mut crate::W<REG> {
        self.variant(Trigin::Val8)
    }
}
impl R {
    #[doc = "Bits 0:3 - Trigger input connections for PWM"]
    #[inline(always)]
    pub fn trigin(&self) -> TriginR {
        TriginR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Trigger input connections for PWM"]
    #[inline(always)]
    pub fn trigin(&mut self) -> TriginW<Pwm1ExtClkSpec> {
        TriginW::new(self, 0)
    }
}
#[doc = "PWM1 external clock trigger\n\nYou can [`read`](crate::Reg::read) this register and get [`pwm1_ext_clk::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwm1_ext_clk::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pwm1ExtClkSpec;
impl crate::RegisterSpec for Pwm1ExtClkSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pwm1_ext_clk::R`](R) reader structure"]
impl crate::Readable for Pwm1ExtClkSpec {}
#[doc = "`write(|w| ..)` method takes [`pwm1_ext_clk::W`](W) writer structure"]
impl crate::Writable for Pwm1ExtClkSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PWM1_EXT_CLK to value 0x0f"]
impl crate::Resettable for Pwm1ExtClkSpec {
    const RESET_VALUE: u32 = 0x0f;
}
