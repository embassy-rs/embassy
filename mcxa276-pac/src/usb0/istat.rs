#[doc = "Register `ISTAT` reader"]
pub type R = crate::R<IstatSpec>;
#[doc = "Register `ISTAT` writer"]
pub type W = crate::W<IstatSpec>;
#[doc = "USB Reset Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usbrst {
    #[doc = "0: Not detected"]
    IntNo = 0,
    #[doc = "1: Detected"]
    IntYes = 1,
}
impl From<Usbrst> for bool {
    #[inline(always)]
    fn from(variant: Usbrst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `USBRST` reader - USB Reset Flag"]
pub type UsbrstR = crate::BitReader<Usbrst>;
impl UsbrstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Usbrst {
        match self.bits {
            false => Usbrst::IntNo,
            true => Usbrst::IntYes,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Usbrst::IntNo
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Usbrst::IntYes
    }
}
#[doc = "Field `USBRST` writer - USB Reset Flag"]
pub type UsbrstW<'a, REG> = crate::BitWriter1C<'a, REG, Usbrst>;
impl<'a, REG> UsbrstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Usbrst::IntNo)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Usbrst::IntYes)
    }
}
#[doc = "Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    #[doc = "0: Error did not occur"]
    IntNo = 0,
    #[doc = "1: Error occurred"]
    IntYes = 1,
}
impl From<Error> for bool {
    #[inline(always)]
    fn from(variant: Error) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERROR` reader - Error Flag"]
pub type ErrorR = crate::BitReader<Error>;
impl ErrorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Error {
        match self.bits {
            false => Error::IntNo,
            true => Error::IntYes,
        }
    }
    #[doc = "Error did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Error::IntNo
    }
    #[doc = "Error occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Error::IntYes
    }
}
#[doc = "Field `ERROR` writer - Error Flag"]
pub type ErrorW<'a, REG> = crate::BitWriter1C<'a, REG, Error>;
impl<'a, REG> ErrorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Error::IntNo)
    }
    #[doc = "Error occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Error::IntYes)
    }
}
#[doc = "Start Of Frame (SOF) Token Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Softok {
    #[doc = "0: Did not receive"]
    IntNo = 0,
    #[doc = "1: Received"]
    IntYes = 1,
}
impl From<Softok> for bool {
    #[inline(always)]
    fn from(variant: Softok) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOFTOK` reader - Start Of Frame (SOF) Token Flag"]
pub type SoftokR = crate::BitReader<Softok>;
impl SoftokR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Softok {
        match self.bits {
            false => Softok::IntNo,
            true => Softok::IntYes,
        }
    }
    #[doc = "Did not receive"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Softok::IntNo
    }
    #[doc = "Received"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Softok::IntYes
    }
}
#[doc = "Field `SOFTOK` writer - Start Of Frame (SOF) Token Flag"]
pub type SoftokW<'a, REG> = crate::BitWriter1C<'a, REG, Softok>;
impl<'a, REG> SoftokW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Did not receive"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Softok::IntNo)
    }
    #[doc = "Received"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Softok::IntYes)
    }
}
#[doc = "Current Token Processing Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tokdne {
    #[doc = "0: Not processed"]
    IntNo = 0,
    #[doc = "1: Processed"]
    IntYes = 1,
}
impl From<Tokdne> for bool {
    #[inline(always)]
    fn from(variant: Tokdne) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TOKDNE` reader - Current Token Processing Flag"]
pub type TokdneR = crate::BitReader<Tokdne>;
impl TokdneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tokdne {
        match self.bits {
            false => Tokdne::IntNo,
            true => Tokdne::IntYes,
        }
    }
    #[doc = "Not processed"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Tokdne::IntNo
    }
    #[doc = "Processed"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Tokdne::IntYes
    }
}
#[doc = "Field `TOKDNE` writer - Current Token Processing Flag"]
pub type TokdneW<'a, REG> = crate::BitWriter1C<'a, REG, Tokdne>;
impl<'a, REG> TokdneW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not processed"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Tokdne::IntNo)
    }
    #[doc = "Processed"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Tokdne::IntYes)
    }
}
#[doc = "Sleep Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sleep {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<Sleep> for bool {
    #[inline(always)]
    fn from(variant: Sleep) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLEEP` reader - Sleep Flag"]
pub type SleepR = crate::BitReader<Sleep>;
impl SleepR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sleep {
        match self.bits {
            false => Sleep::IntNo,
            true => Sleep::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Sleep::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Sleep::IntYes
    }
}
#[doc = "Field `SLEEP` writer - Sleep Flag"]
pub type SleepW<'a, REG> = crate::BitWriter1C<'a, REG, Sleep>;
impl<'a, REG> SleepW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Sleep::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Sleep::IntYes)
    }
}
#[doc = "Resume Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resume {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<Resume> for bool {
    #[inline(always)]
    fn from(variant: Resume) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESUME` reader - Resume Flag"]
pub type ResumeR = crate::BitReader<Resume>;
impl ResumeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Resume {
        match self.bits {
            false => Resume::IntNo,
            true => Resume::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Resume::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Resume::IntYes
    }
}
#[doc = "Field `RESUME` writer - Resume Flag"]
pub type ResumeW<'a, REG> = crate::BitWriter1C<'a, REG, Resume>;
impl<'a, REG> ResumeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Resume::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Resume::IntYes)
    }
}
#[doc = "Attach Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attach {
    #[doc = "0: Not detected"]
    IntNo = 0,
    #[doc = "1: Detected"]
    IntYes = 1,
}
impl From<Attach> for bool {
    #[inline(always)]
    fn from(variant: Attach) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ATTACH` reader - Attach Interrupt Flag"]
pub type AttachR = crate::BitReader<Attach>;
impl AttachR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Attach {
        match self.bits {
            false => Attach::IntNo,
            true => Attach::IntYes,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Attach::IntNo
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Attach::IntYes
    }
}
#[doc = "Field `ATTACH` writer - Attach Interrupt Flag"]
pub type AttachW<'a, REG> = crate::BitWriter1C<'a, REG, Attach>;
impl<'a, REG> AttachW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Attach::IntNo)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Attach::IntYes)
    }
}
#[doc = "Stall Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stall {
    #[doc = "0: Interrupt did not occur"]
    IntNo = 0,
    #[doc = "1: Interrupt occurred"]
    IntYes = 1,
}
impl From<Stall> for bool {
    #[inline(always)]
    fn from(variant: Stall) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STALL` reader - Stall Interrupt Flag"]
pub type StallR = crate::BitReader<Stall>;
impl StallR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stall {
        match self.bits {
            false => Stall::IntNo,
            true => Stall::IntYes,
        }
    }
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn is_int_no(&self) -> bool {
        *self == Stall::IntNo
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn is_int_yes(&self) -> bool {
        *self == Stall::IntYes
    }
}
#[doc = "Field `STALL` writer - Stall Interrupt Flag"]
pub type StallW<'a, REG> = crate::BitWriter1C<'a, REG, Stall>;
impl<'a, REG> StallW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt did not occur"]
    #[inline(always)]
    pub fn int_no(self) -> &'a mut crate::W<REG> {
        self.variant(Stall::IntNo)
    }
    #[doc = "Interrupt occurred"]
    #[inline(always)]
    pub fn int_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Stall::IntYes)
    }
}
impl R {
    #[doc = "Bit 0 - USB Reset Flag"]
    #[inline(always)]
    pub fn usbrst(&self) -> UsbrstR {
        UsbrstR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Error Flag"]
    #[inline(always)]
    pub fn error(&self) -> ErrorR {
        ErrorR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Start Of Frame (SOF) Token Flag"]
    #[inline(always)]
    pub fn softok(&self) -> SoftokR {
        SoftokR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Current Token Processing Flag"]
    #[inline(always)]
    pub fn tokdne(&self) -> TokdneR {
        TokdneR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Sleep Flag"]
    #[inline(always)]
    pub fn sleep(&self) -> SleepR {
        SleepR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Resume Flag"]
    #[inline(always)]
    pub fn resume(&self) -> ResumeR {
        ResumeR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Attach Interrupt Flag"]
    #[inline(always)]
    pub fn attach(&self) -> AttachR {
        AttachR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Stall Interrupt Flag"]
    #[inline(always)]
    pub fn stall(&self) -> StallR {
        StallR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - USB Reset Flag"]
    #[inline(always)]
    pub fn usbrst(&mut self) -> UsbrstW<IstatSpec> {
        UsbrstW::new(self, 0)
    }
    #[doc = "Bit 1 - Error Flag"]
    #[inline(always)]
    pub fn error(&mut self) -> ErrorW<IstatSpec> {
        ErrorW::new(self, 1)
    }
    #[doc = "Bit 2 - Start Of Frame (SOF) Token Flag"]
    #[inline(always)]
    pub fn softok(&mut self) -> SoftokW<IstatSpec> {
        SoftokW::new(self, 2)
    }
    #[doc = "Bit 3 - Current Token Processing Flag"]
    #[inline(always)]
    pub fn tokdne(&mut self) -> TokdneW<IstatSpec> {
        TokdneW::new(self, 3)
    }
    #[doc = "Bit 4 - Sleep Flag"]
    #[inline(always)]
    pub fn sleep(&mut self) -> SleepW<IstatSpec> {
        SleepW::new(self, 4)
    }
    #[doc = "Bit 5 - Resume Flag"]
    #[inline(always)]
    pub fn resume(&mut self) -> ResumeW<IstatSpec> {
        ResumeW::new(self, 5)
    }
    #[doc = "Bit 6 - Attach Interrupt Flag"]
    #[inline(always)]
    pub fn attach(&mut self) -> AttachW<IstatSpec> {
        AttachW::new(self, 6)
    }
    #[doc = "Bit 7 - Stall Interrupt Flag"]
    #[inline(always)]
    pub fn stall(&mut self) -> StallW<IstatSpec> {
        StallW::new(self, 7)
    }
}
#[doc = "Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`istat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`istat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IstatSpec;
impl crate::RegisterSpec for IstatSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`istat::R`](R) reader structure"]
impl crate::Readable for IstatSpec {}
#[doc = "`write(|w| ..)` method takes [`istat::W`](W) writer structure"]
impl crate::Writable for IstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u8 = 0xff;
}
#[doc = "`reset()` method sets ISTAT to value 0"]
impl crate::Resettable for IstatSpec {}
