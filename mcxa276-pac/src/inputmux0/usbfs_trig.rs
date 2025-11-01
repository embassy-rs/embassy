#[doc = "Register `USBFS_TRIG` reader"]
pub type R = crate::R<UsbfsTrigSpec>;
#[doc = "Register `USBFS_TRIG` writer"]
pub type W = crate::W<UsbfsTrigSpec>;
#[doc = "USB-FS trigger input connections.\n\nValue on reset: 7"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Inp {
    #[doc = "1: LPUART0 lpuart_trg_txdata input is selected"]
    Val1 = 1,
    #[doc = "2: LPUART1 lpuart_trg_txdata input is selected"]
    Val2 = 2,
    #[doc = "3: LPUART2 lpuart_trg_txdata input is selected"]
    Val3 = 3,
    #[doc = "4: LPUART3 lpuart_trg_txdata input is selected"]
    Val4 = 4,
    #[doc = "5: LPUART4 lpuart_trg_txdata input is selected"]
    Val5 = 5,
    #[doc = "6: LPUART5 lpuart_trg_txdata input is selected"]
    Val6 = 6,
}
impl From<Inp> for u8 {
    #[inline(always)]
    fn from(variant: Inp) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Inp {
    type Ux = u8;
}
impl crate::IsEnum for Inp {}
#[doc = "Field `INP` reader - USB-FS trigger input connections."]
pub type InpR = crate::FieldReader<Inp>;
impl InpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Inp> {
        match self.bits {
            1 => Some(Inp::Val1),
            2 => Some(Inp::Val2),
            3 => Some(Inp::Val3),
            4 => Some(Inp::Val4),
            5 => Some(Inp::Val5),
            6 => Some(Inp::Val6),
            _ => None,
        }
    }
    #[doc = "LPUART0 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn is_val1(&self) -> bool {
        *self == Inp::Val1
    }
    #[doc = "LPUART1 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn is_val2(&self) -> bool {
        *self == Inp::Val2
    }
    #[doc = "LPUART2 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn is_val3(&self) -> bool {
        *self == Inp::Val3
    }
    #[doc = "LPUART3 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn is_val4(&self) -> bool {
        *self == Inp::Val4
    }
    #[doc = "LPUART4 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn is_val5(&self) -> bool {
        *self == Inp::Val5
    }
    #[doc = "LPUART5 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn is_val6(&self) -> bool {
        *self == Inp::Val6
    }
}
#[doc = "Field `INP` writer - USB-FS trigger input connections."]
pub type InpW<'a, REG> = crate::FieldWriter<'a, REG, 4, Inp>;
impl<'a, REG> InpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "LPUART0 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn val1(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val1)
    }
    #[doc = "LPUART1 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn val2(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val2)
    }
    #[doc = "LPUART2 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn val3(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val3)
    }
    #[doc = "LPUART3 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn val4(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val4)
    }
    #[doc = "LPUART4 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn val5(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val5)
    }
    #[doc = "LPUART5 lpuart_trg_txdata input is selected"]
    #[inline(always)]
    pub fn val6(self) -> &'a mut crate::W<REG> {
        self.variant(Inp::Val6)
    }
}
impl R {
    #[doc = "Bits 0:3 - USB-FS trigger input connections."]
    #[inline(always)]
    pub fn inp(&self) -> InpR {
        InpR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - USB-FS trigger input connections."]
    #[inline(always)]
    pub fn inp(&mut self) -> InpW<UsbfsTrigSpec> {
        InpW::new(self, 0)
    }
}
#[doc = "USB-FS trigger input connections\n\nYou can [`read`](crate::Reg::read) this register and get [`usbfs_trig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbfs_trig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UsbfsTrigSpec;
impl crate::RegisterSpec for UsbfsTrigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`usbfs_trig::R`](R) reader structure"]
impl crate::Readable for UsbfsTrigSpec {}
#[doc = "`write(|w| ..)` method takes [`usbfs_trig::W`](W) writer structure"]
impl crate::Writable for UsbfsTrigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets USBFS_TRIG to value 0x07"]
impl crate::Resettable for UsbfsTrigSpec {
    const RESET_VALUE: u32 = 0x07;
}
