#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    sm0cnt: Sm0cnt,
    sm0init: Sm0init,
    sm0ctrl2: Sm0ctrl2,
    sm0ctrl: Sm0ctrl,
    _reserved4: [u8; 0x02],
    sm0val0: Sm0val0,
    _reserved5: [u8; 0x02],
    sm0val1: Sm0val1,
    _reserved6: [u8; 0x02],
    sm0val2: Sm0val2,
    _reserved7: [u8; 0x02],
    sm0val3: Sm0val3,
    _reserved8: [u8; 0x02],
    sm0val4: Sm0val4,
    _reserved9: [u8; 0x02],
    sm0val5: Sm0val5,
    _reserved10: [u8; 0x02],
    sm0octrl: Sm0octrl,
    sm0sts: Sm0sts,
    sm0inten: Sm0inten,
    sm0dmaen: Sm0dmaen,
    sm0tctrl: Sm0tctrl,
    sm0dismap0: Sm0dismap0,
    _reserved16: [u8; 0x02],
    sm0dtcnt0: Sm0dtcnt0,
    sm0dtcnt1: Sm0dtcnt1,
    _reserved18: [u8; 0x08],
    sm0captctrlx: Sm0captctrlx,
    sm0captcompx: Sm0captcompx,
    sm0cval0: Sm0cval0,
    sm0cval0cyc: Sm0cval0cyc,
    sm0cval1: Sm0cval1,
    sm0cval1cyc: Sm0cval1cyc,
    _reserved24: [u8; 0x16],
    sm0captfiltx: Sm0captfiltx,
    sm1cnt: Sm1cnt,
    sm1init: Sm1init,
    sm1ctrl2: Sm1ctrl2,
    sm1ctrl: Sm1ctrl,
    _reserved29: [u8; 0x02],
    sm1val0: Sm1val0,
    _reserved30: [u8; 0x02],
    sm1val1: Sm1val1,
    _reserved31: [u8; 0x02],
    sm1val2: Sm1val2,
    _reserved32: [u8; 0x02],
    sm1val3: Sm1val3,
    _reserved33: [u8; 0x02],
    sm1val4: Sm1val4,
    _reserved34: [u8; 0x02],
    sm1val5: Sm1val5,
    _reserved35: [u8; 0x02],
    sm1octrl: Sm1octrl,
    sm1sts: Sm1sts,
    sm1inten: Sm1inten,
    sm1dmaen: Sm1dmaen,
    sm1tctrl: Sm1tctrl,
    sm1dismap0: Sm1dismap0,
    _reserved41: [u8; 0x02],
    sm1dtcnt0: Sm1dtcnt0,
    sm1dtcnt1: Sm1dtcnt1,
    _reserved43: [u8; 0x08],
    sm1captctrlx: Sm1captctrlx,
    sm1captcompx: Sm1captcompx,
    sm1cval0: Sm1cval0,
    sm1cval0cyc: Sm1cval0cyc,
    sm1cval1: Sm1cval1,
    sm1cval1cyc: Sm1cval1cyc,
    _reserved49: [u8; 0x10],
    sm1phasedly: Sm1phasedly,
    _reserved50: [u8; 0x04],
    sm1captfiltx: Sm1captfiltx,
    sm2cnt: Sm2cnt,
    sm2init: Sm2init,
    sm2ctrl2: Sm2ctrl2,
    sm2ctrl: Sm2ctrl,
    _reserved55: [u8; 0x02],
    sm2val0: Sm2val0,
    _reserved56: [u8; 0x02],
    sm2val1: Sm2val1,
    _reserved57: [u8; 0x02],
    sm2val2: Sm2val2,
    _reserved58: [u8; 0x02],
    sm2val3: Sm2val3,
    _reserved59: [u8; 0x02],
    sm2val4: Sm2val4,
    _reserved60: [u8; 0x02],
    sm2val5: Sm2val5,
    _reserved61: [u8; 0x02],
    sm2octrl: Sm2octrl,
    sm2sts: Sm2sts,
    sm2inten: Sm2inten,
    sm2dmaen: Sm2dmaen,
    sm2tctrl: Sm2tctrl,
    sm2dismap0: Sm2dismap0,
    _reserved67: [u8; 0x02],
    sm2dtcnt0: Sm2dtcnt0,
    sm2dtcnt1: Sm2dtcnt1,
    _reserved69: [u8; 0x08],
    sm2captctrlx: Sm2captctrlx,
    sm2captcompx: Sm2captcompx,
    sm2cval0: Sm2cval0,
    sm2cval0cyc: Sm2cval0cyc,
    sm2cval1: Sm2cval1,
    sm2cval1cyc: Sm2cval1cyc,
    _reserved75: [u8; 0x10],
    sm2phasedly: Sm2phasedly,
    _reserved76: [u8; 0x04],
    sm2captfiltx: Sm2captfiltx,
    sm3cnt: Sm3cnt,
    sm3init: Sm3init,
    sm3ctrl2: Sm3ctrl2,
    sm3ctrl: Sm3ctrl,
    _reserved81: [u8; 0x02],
    sm3val0: Sm3val0,
    _reserved82: [u8; 0x02],
    sm3val1: Sm3val1,
    _reserved83: [u8; 0x02],
    sm3val2: Sm3val2,
    _reserved84: [u8; 0x02],
    sm3val3: Sm3val3,
    _reserved85: [u8; 0x02],
    sm3val4: Sm3val4,
    _reserved86: [u8; 0x02],
    sm3val5: Sm3val5,
    _reserved87: [u8; 0x02],
    sm3octrl: Sm3octrl,
    sm3sts: Sm3sts,
    sm3inten: Sm3inten,
    sm3dmaen: Sm3dmaen,
    sm3tctrl: Sm3tctrl,
    sm3dismap0: Sm3dismap0,
    _reserved93: [u8; 0x02],
    sm3dtcnt0: Sm3dtcnt0,
    sm3dtcnt1: Sm3dtcnt1,
    _reserved95: [u8; 0x08],
    sm3captctrlx: Sm3captctrlx,
    sm3captcompx: Sm3captcompx,
    sm3cval0: Sm3cval0,
    sm3cval0cyc: Sm3cval0cyc,
    sm3cval1: Sm3cval1,
    sm3cval1cyc: Sm3cval1cyc,
    _reserved101: [u8; 0x10],
    sm3phasedly: Sm3phasedly,
    _reserved102: [u8; 0x04],
    sm3captfiltx: Sm3captfiltx,
    outen: Outen,
    mask: Mask,
    swcout: Swcout,
    dtsrcsel: Dtsrcsel,
    mctrl: Mctrl,
    mctrl2: Mctrl2,
    fctrl0: Fctrl0,
    fsts0: Fsts0,
    ffilt0: Ffilt0,
    ftst0: Ftst0,
    fctrl20: Fctrl20,
}
impl RegisterBlock {
    #[doc = "0x00 - Counter Register"]
    #[inline(always)]
    pub const fn sm0cnt(&self) -> &Sm0cnt {
        &self.sm0cnt
    }
    #[doc = "0x02 - Initial Count Register"]
    #[inline(always)]
    pub const fn sm0init(&self) -> &Sm0init {
        &self.sm0init
    }
    #[doc = "0x04 - Control 2 Register"]
    #[inline(always)]
    pub const fn sm0ctrl2(&self) -> &Sm0ctrl2 {
        &self.sm0ctrl2
    }
    #[doc = "0x06 - Control Register"]
    #[inline(always)]
    pub const fn sm0ctrl(&self) -> &Sm0ctrl {
        &self.sm0ctrl
    }
    #[doc = "0x0a - Value Register 0"]
    #[inline(always)]
    pub const fn sm0val0(&self) -> &Sm0val0 {
        &self.sm0val0
    }
    #[doc = "0x0e - Value Register 1"]
    #[inline(always)]
    pub const fn sm0val1(&self) -> &Sm0val1 {
        &self.sm0val1
    }
    #[doc = "0x12 - Value Register 2"]
    #[inline(always)]
    pub const fn sm0val2(&self) -> &Sm0val2 {
        &self.sm0val2
    }
    #[doc = "0x16 - Value Register 3"]
    #[inline(always)]
    pub const fn sm0val3(&self) -> &Sm0val3 {
        &self.sm0val3
    }
    #[doc = "0x1a - Value Register 4"]
    #[inline(always)]
    pub const fn sm0val4(&self) -> &Sm0val4 {
        &self.sm0val4
    }
    #[doc = "0x1e - Value Register 5"]
    #[inline(always)]
    pub const fn sm0val5(&self) -> &Sm0val5 {
        &self.sm0val5
    }
    #[doc = "0x22 - Output Control Register"]
    #[inline(always)]
    pub const fn sm0octrl(&self) -> &Sm0octrl {
        &self.sm0octrl
    }
    #[doc = "0x24 - Status Register"]
    #[inline(always)]
    pub const fn sm0sts(&self) -> &Sm0sts {
        &self.sm0sts
    }
    #[doc = "0x26 - Interrupt Enable Register"]
    #[inline(always)]
    pub const fn sm0inten(&self) -> &Sm0inten {
        &self.sm0inten
    }
    #[doc = "0x28 - DMA Enable Register"]
    #[inline(always)]
    pub const fn sm0dmaen(&self) -> &Sm0dmaen {
        &self.sm0dmaen
    }
    #[doc = "0x2a - Output Trigger Control Register"]
    #[inline(always)]
    pub const fn sm0tctrl(&self) -> &Sm0tctrl {
        &self.sm0tctrl
    }
    #[doc = "0x2c - Fault Disable Mapping Register 0"]
    #[inline(always)]
    pub const fn sm0dismap0(&self) -> &Sm0dismap0 {
        &self.sm0dismap0
    }
    #[doc = "0x30 - Deadtime Count Register 0"]
    #[inline(always)]
    pub const fn sm0dtcnt0(&self) -> &Sm0dtcnt0 {
        &self.sm0dtcnt0
    }
    #[doc = "0x32 - Deadtime Count Register 1"]
    #[inline(always)]
    pub const fn sm0dtcnt1(&self) -> &Sm0dtcnt1 {
        &self.sm0dtcnt1
    }
    #[doc = "0x3c - Capture Control X Register"]
    #[inline(always)]
    pub const fn sm0captctrlx(&self) -> &Sm0captctrlx {
        &self.sm0captctrlx
    }
    #[doc = "0x3e - Capture Compare X Register"]
    #[inline(always)]
    pub const fn sm0captcompx(&self) -> &Sm0captcompx {
        &self.sm0captcompx
    }
    #[doc = "0x40 - Capture Value 0 Register"]
    #[inline(always)]
    pub const fn sm0cval0(&self) -> &Sm0cval0 {
        &self.sm0cval0
    }
    #[doc = "0x42 - Capture Value 0 Cycle Register"]
    #[inline(always)]
    pub const fn sm0cval0cyc(&self) -> &Sm0cval0cyc {
        &self.sm0cval0cyc
    }
    #[doc = "0x44 - Capture Value 1 Register"]
    #[inline(always)]
    pub const fn sm0cval1(&self) -> &Sm0cval1 {
        &self.sm0cval1
    }
    #[doc = "0x46 - Capture Value 1 Cycle Register"]
    #[inline(always)]
    pub const fn sm0cval1cyc(&self) -> &Sm0cval1cyc {
        &self.sm0cval1cyc
    }
    #[doc = "0x5e - Capture PWM_X Input Filter Register"]
    #[inline(always)]
    pub const fn sm0captfiltx(&self) -> &Sm0captfiltx {
        &self.sm0captfiltx
    }
    #[doc = "0x60 - Counter Register"]
    #[inline(always)]
    pub const fn sm1cnt(&self) -> &Sm1cnt {
        &self.sm1cnt
    }
    #[doc = "0x62 - Initial Count Register"]
    #[inline(always)]
    pub const fn sm1init(&self) -> &Sm1init {
        &self.sm1init
    }
    #[doc = "0x64 - Control 2 Register"]
    #[inline(always)]
    pub const fn sm1ctrl2(&self) -> &Sm1ctrl2 {
        &self.sm1ctrl2
    }
    #[doc = "0x66 - Control Register"]
    #[inline(always)]
    pub const fn sm1ctrl(&self) -> &Sm1ctrl {
        &self.sm1ctrl
    }
    #[doc = "0x6a - Value Register 0"]
    #[inline(always)]
    pub const fn sm1val0(&self) -> &Sm1val0 {
        &self.sm1val0
    }
    #[doc = "0x6e - Value Register 1"]
    #[inline(always)]
    pub const fn sm1val1(&self) -> &Sm1val1 {
        &self.sm1val1
    }
    #[doc = "0x72 - Value Register 2"]
    #[inline(always)]
    pub const fn sm1val2(&self) -> &Sm1val2 {
        &self.sm1val2
    }
    #[doc = "0x76 - Value Register 3"]
    #[inline(always)]
    pub const fn sm1val3(&self) -> &Sm1val3 {
        &self.sm1val3
    }
    #[doc = "0x7a - Value Register 4"]
    #[inline(always)]
    pub const fn sm1val4(&self) -> &Sm1val4 {
        &self.sm1val4
    }
    #[doc = "0x7e - Value Register 5"]
    #[inline(always)]
    pub const fn sm1val5(&self) -> &Sm1val5 {
        &self.sm1val5
    }
    #[doc = "0x82 - Output Control Register"]
    #[inline(always)]
    pub const fn sm1octrl(&self) -> &Sm1octrl {
        &self.sm1octrl
    }
    #[doc = "0x84 - Status Register"]
    #[inline(always)]
    pub const fn sm1sts(&self) -> &Sm1sts {
        &self.sm1sts
    }
    #[doc = "0x86 - Interrupt Enable Register"]
    #[inline(always)]
    pub const fn sm1inten(&self) -> &Sm1inten {
        &self.sm1inten
    }
    #[doc = "0x88 - DMA Enable Register"]
    #[inline(always)]
    pub const fn sm1dmaen(&self) -> &Sm1dmaen {
        &self.sm1dmaen
    }
    #[doc = "0x8a - Output Trigger Control Register"]
    #[inline(always)]
    pub const fn sm1tctrl(&self) -> &Sm1tctrl {
        &self.sm1tctrl
    }
    #[doc = "0x8c - Fault Disable Mapping Register 0"]
    #[inline(always)]
    pub const fn sm1dismap0(&self) -> &Sm1dismap0 {
        &self.sm1dismap0
    }
    #[doc = "0x90 - Deadtime Count Register 0"]
    #[inline(always)]
    pub const fn sm1dtcnt0(&self) -> &Sm1dtcnt0 {
        &self.sm1dtcnt0
    }
    #[doc = "0x92 - Deadtime Count Register 1"]
    #[inline(always)]
    pub const fn sm1dtcnt1(&self) -> &Sm1dtcnt1 {
        &self.sm1dtcnt1
    }
    #[doc = "0x9c - Capture Control X Register"]
    #[inline(always)]
    pub const fn sm1captctrlx(&self) -> &Sm1captctrlx {
        &self.sm1captctrlx
    }
    #[doc = "0x9e - Capture Compare X Register"]
    #[inline(always)]
    pub const fn sm1captcompx(&self) -> &Sm1captcompx {
        &self.sm1captcompx
    }
    #[doc = "0xa0 - Capture Value 0 Register"]
    #[inline(always)]
    pub const fn sm1cval0(&self) -> &Sm1cval0 {
        &self.sm1cval0
    }
    #[doc = "0xa2 - Capture Value 0 Cycle Register"]
    #[inline(always)]
    pub const fn sm1cval0cyc(&self) -> &Sm1cval0cyc {
        &self.sm1cval0cyc
    }
    #[doc = "0xa4 - Capture Value 1 Register"]
    #[inline(always)]
    pub const fn sm1cval1(&self) -> &Sm1cval1 {
        &self.sm1cval1
    }
    #[doc = "0xa6 - Capture Value 1 Cycle Register"]
    #[inline(always)]
    pub const fn sm1cval1cyc(&self) -> &Sm1cval1cyc {
        &self.sm1cval1cyc
    }
    #[doc = "0xb8 - Phase Delay Register"]
    #[inline(always)]
    pub const fn sm1phasedly(&self) -> &Sm1phasedly {
        &self.sm1phasedly
    }
    #[doc = "0xbe - Capture PWM_X Input Filter Register"]
    #[inline(always)]
    pub const fn sm1captfiltx(&self) -> &Sm1captfiltx {
        &self.sm1captfiltx
    }
    #[doc = "0xc0 - Counter Register"]
    #[inline(always)]
    pub const fn sm2cnt(&self) -> &Sm2cnt {
        &self.sm2cnt
    }
    #[doc = "0xc2 - Initial Count Register"]
    #[inline(always)]
    pub const fn sm2init(&self) -> &Sm2init {
        &self.sm2init
    }
    #[doc = "0xc4 - Control 2 Register"]
    #[inline(always)]
    pub const fn sm2ctrl2(&self) -> &Sm2ctrl2 {
        &self.sm2ctrl2
    }
    #[doc = "0xc6 - Control Register"]
    #[inline(always)]
    pub const fn sm2ctrl(&self) -> &Sm2ctrl {
        &self.sm2ctrl
    }
    #[doc = "0xca - Value Register 0"]
    #[inline(always)]
    pub const fn sm2val0(&self) -> &Sm2val0 {
        &self.sm2val0
    }
    #[doc = "0xce - Value Register 1"]
    #[inline(always)]
    pub const fn sm2val1(&self) -> &Sm2val1 {
        &self.sm2val1
    }
    #[doc = "0xd2 - Value Register 2"]
    #[inline(always)]
    pub const fn sm2val2(&self) -> &Sm2val2 {
        &self.sm2val2
    }
    #[doc = "0xd6 - Value Register 3"]
    #[inline(always)]
    pub const fn sm2val3(&self) -> &Sm2val3 {
        &self.sm2val3
    }
    #[doc = "0xda - Value Register 4"]
    #[inline(always)]
    pub const fn sm2val4(&self) -> &Sm2val4 {
        &self.sm2val4
    }
    #[doc = "0xde - Value Register 5"]
    #[inline(always)]
    pub const fn sm2val5(&self) -> &Sm2val5 {
        &self.sm2val5
    }
    #[doc = "0xe2 - Output Control Register"]
    #[inline(always)]
    pub const fn sm2octrl(&self) -> &Sm2octrl {
        &self.sm2octrl
    }
    #[doc = "0xe4 - Status Register"]
    #[inline(always)]
    pub const fn sm2sts(&self) -> &Sm2sts {
        &self.sm2sts
    }
    #[doc = "0xe6 - Interrupt Enable Register"]
    #[inline(always)]
    pub const fn sm2inten(&self) -> &Sm2inten {
        &self.sm2inten
    }
    #[doc = "0xe8 - DMA Enable Register"]
    #[inline(always)]
    pub const fn sm2dmaen(&self) -> &Sm2dmaen {
        &self.sm2dmaen
    }
    #[doc = "0xea - Output Trigger Control Register"]
    #[inline(always)]
    pub const fn sm2tctrl(&self) -> &Sm2tctrl {
        &self.sm2tctrl
    }
    #[doc = "0xec - Fault Disable Mapping Register 0"]
    #[inline(always)]
    pub const fn sm2dismap0(&self) -> &Sm2dismap0 {
        &self.sm2dismap0
    }
    #[doc = "0xf0 - Deadtime Count Register 0"]
    #[inline(always)]
    pub const fn sm2dtcnt0(&self) -> &Sm2dtcnt0 {
        &self.sm2dtcnt0
    }
    #[doc = "0xf2 - Deadtime Count Register 1"]
    #[inline(always)]
    pub const fn sm2dtcnt1(&self) -> &Sm2dtcnt1 {
        &self.sm2dtcnt1
    }
    #[doc = "0xfc - Capture Control X Register"]
    #[inline(always)]
    pub const fn sm2captctrlx(&self) -> &Sm2captctrlx {
        &self.sm2captctrlx
    }
    #[doc = "0xfe - Capture Compare X Register"]
    #[inline(always)]
    pub const fn sm2captcompx(&self) -> &Sm2captcompx {
        &self.sm2captcompx
    }
    #[doc = "0x100 - Capture Value 0 Register"]
    #[inline(always)]
    pub const fn sm2cval0(&self) -> &Sm2cval0 {
        &self.sm2cval0
    }
    #[doc = "0x102 - Capture Value 0 Cycle Register"]
    #[inline(always)]
    pub const fn sm2cval0cyc(&self) -> &Sm2cval0cyc {
        &self.sm2cval0cyc
    }
    #[doc = "0x104 - Capture Value 1 Register"]
    #[inline(always)]
    pub const fn sm2cval1(&self) -> &Sm2cval1 {
        &self.sm2cval1
    }
    #[doc = "0x106 - Capture Value 1 Cycle Register"]
    #[inline(always)]
    pub const fn sm2cval1cyc(&self) -> &Sm2cval1cyc {
        &self.sm2cval1cyc
    }
    #[doc = "0x118 - Phase Delay Register"]
    #[inline(always)]
    pub const fn sm2phasedly(&self) -> &Sm2phasedly {
        &self.sm2phasedly
    }
    #[doc = "0x11e - Capture PWM_X Input Filter Register"]
    #[inline(always)]
    pub const fn sm2captfiltx(&self) -> &Sm2captfiltx {
        &self.sm2captfiltx
    }
    #[doc = "0x120 - Counter Register"]
    #[inline(always)]
    pub const fn sm3cnt(&self) -> &Sm3cnt {
        &self.sm3cnt
    }
    #[doc = "0x122 - Initial Count Register"]
    #[inline(always)]
    pub const fn sm3init(&self) -> &Sm3init {
        &self.sm3init
    }
    #[doc = "0x124 - Control 2 Register"]
    #[inline(always)]
    pub const fn sm3ctrl2(&self) -> &Sm3ctrl2 {
        &self.sm3ctrl2
    }
    #[doc = "0x126 - Control Register"]
    #[inline(always)]
    pub const fn sm3ctrl(&self) -> &Sm3ctrl {
        &self.sm3ctrl
    }
    #[doc = "0x12a - Value Register 0"]
    #[inline(always)]
    pub const fn sm3val0(&self) -> &Sm3val0 {
        &self.sm3val0
    }
    #[doc = "0x12e - Value Register 1"]
    #[inline(always)]
    pub const fn sm3val1(&self) -> &Sm3val1 {
        &self.sm3val1
    }
    #[doc = "0x132 - Value Register 2"]
    #[inline(always)]
    pub const fn sm3val2(&self) -> &Sm3val2 {
        &self.sm3val2
    }
    #[doc = "0x136 - Value Register 3"]
    #[inline(always)]
    pub const fn sm3val3(&self) -> &Sm3val3 {
        &self.sm3val3
    }
    #[doc = "0x13a - Value Register 4"]
    #[inline(always)]
    pub const fn sm3val4(&self) -> &Sm3val4 {
        &self.sm3val4
    }
    #[doc = "0x13e - Value Register 5"]
    #[inline(always)]
    pub const fn sm3val5(&self) -> &Sm3val5 {
        &self.sm3val5
    }
    #[doc = "0x142 - Output Control Register"]
    #[inline(always)]
    pub const fn sm3octrl(&self) -> &Sm3octrl {
        &self.sm3octrl
    }
    #[doc = "0x144 - Status Register"]
    #[inline(always)]
    pub const fn sm3sts(&self) -> &Sm3sts {
        &self.sm3sts
    }
    #[doc = "0x146 - Interrupt Enable Register"]
    #[inline(always)]
    pub const fn sm3inten(&self) -> &Sm3inten {
        &self.sm3inten
    }
    #[doc = "0x148 - DMA Enable Register"]
    #[inline(always)]
    pub const fn sm3dmaen(&self) -> &Sm3dmaen {
        &self.sm3dmaen
    }
    #[doc = "0x14a - Output Trigger Control Register"]
    #[inline(always)]
    pub const fn sm3tctrl(&self) -> &Sm3tctrl {
        &self.sm3tctrl
    }
    #[doc = "0x14c - Fault Disable Mapping Register 0"]
    #[inline(always)]
    pub const fn sm3dismap0(&self) -> &Sm3dismap0 {
        &self.sm3dismap0
    }
    #[doc = "0x150 - Deadtime Count Register 0"]
    #[inline(always)]
    pub const fn sm3dtcnt0(&self) -> &Sm3dtcnt0 {
        &self.sm3dtcnt0
    }
    #[doc = "0x152 - Deadtime Count Register 1"]
    #[inline(always)]
    pub const fn sm3dtcnt1(&self) -> &Sm3dtcnt1 {
        &self.sm3dtcnt1
    }
    #[doc = "0x15c - Capture Control X Register"]
    #[inline(always)]
    pub const fn sm3captctrlx(&self) -> &Sm3captctrlx {
        &self.sm3captctrlx
    }
    #[doc = "0x15e - Capture Compare X Register"]
    #[inline(always)]
    pub const fn sm3captcompx(&self) -> &Sm3captcompx {
        &self.sm3captcompx
    }
    #[doc = "0x160 - Capture Value 0 Register"]
    #[inline(always)]
    pub const fn sm3cval0(&self) -> &Sm3cval0 {
        &self.sm3cval0
    }
    #[doc = "0x162 - Capture Value 0 Cycle Register"]
    #[inline(always)]
    pub const fn sm3cval0cyc(&self) -> &Sm3cval0cyc {
        &self.sm3cval0cyc
    }
    #[doc = "0x164 - Capture Value 1 Register"]
    #[inline(always)]
    pub const fn sm3cval1(&self) -> &Sm3cval1 {
        &self.sm3cval1
    }
    #[doc = "0x166 - Capture Value 1 Cycle Register"]
    #[inline(always)]
    pub const fn sm3cval1cyc(&self) -> &Sm3cval1cyc {
        &self.sm3cval1cyc
    }
    #[doc = "0x178 - Phase Delay Register"]
    #[inline(always)]
    pub const fn sm3phasedly(&self) -> &Sm3phasedly {
        &self.sm3phasedly
    }
    #[doc = "0x17e - Capture PWM_X Input Filter Register"]
    #[inline(always)]
    pub const fn sm3captfiltx(&self) -> &Sm3captfiltx {
        &self.sm3captfiltx
    }
    #[doc = "0x180 - Output Enable Register"]
    #[inline(always)]
    pub const fn outen(&self) -> &Outen {
        &self.outen
    }
    #[doc = "0x182 - Mask Register"]
    #[inline(always)]
    pub const fn mask(&self) -> &Mask {
        &self.mask
    }
    #[doc = "0x184 - Software Controlled Output Register"]
    #[inline(always)]
    pub const fn swcout(&self) -> &Swcout {
        &self.swcout
    }
    #[doc = "0x186 - PWM Source Select Register"]
    #[inline(always)]
    pub const fn dtsrcsel(&self) -> &Dtsrcsel {
        &self.dtsrcsel
    }
    #[doc = "0x188 - Master Control Register"]
    #[inline(always)]
    pub const fn mctrl(&self) -> &Mctrl {
        &self.mctrl
    }
    #[doc = "0x18a - Master Control 2 Register"]
    #[inline(always)]
    pub const fn mctrl2(&self) -> &Mctrl2 {
        &self.mctrl2
    }
    #[doc = "0x18c - Fault Control Register"]
    #[inline(always)]
    pub const fn fctrl0(&self) -> &Fctrl0 {
        &self.fctrl0
    }
    #[doc = "0x18e - Fault Status Register"]
    #[inline(always)]
    pub const fn fsts0(&self) -> &Fsts0 {
        &self.fsts0
    }
    #[doc = "0x190 - Fault Filter Register"]
    #[inline(always)]
    pub const fn ffilt0(&self) -> &Ffilt0 {
        &self.ffilt0
    }
    #[doc = "0x192 - Fault Test Register"]
    #[inline(always)]
    pub const fn ftst0(&self) -> &Ftst0 {
        &self.ftst0
    }
    #[doc = "0x194 - Fault Control 2 Register"]
    #[inline(always)]
    pub const fn fctrl20(&self) -> &Fctrl20 {
        &self.fctrl20
    }
}
#[doc = "SM0CNT (r) register accessor: Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0cnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0cnt`] module"]
#[doc(alias = "SM0CNT")]
pub type Sm0cnt = crate::Reg<sm0cnt::Sm0cntSpec>;
#[doc = "Counter Register"]
pub mod sm0cnt;
#[doc = "SM0INIT (rw) register accessor: Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0init::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0init::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0init`] module"]
#[doc(alias = "SM0INIT")]
pub type Sm0init = crate::Reg<sm0init::Sm0initSpec>;
#[doc = "Initial Count Register"]
pub mod sm0init;
#[doc = "SM0CTRL2 (rw) register accessor: Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0ctrl2`] module"]
#[doc(alias = "SM0CTRL2")]
pub type Sm0ctrl2 = crate::Reg<sm0ctrl2::Sm0ctrl2Spec>;
#[doc = "Control 2 Register"]
pub mod sm0ctrl2;
#[doc = "SM0CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0ctrl`] module"]
#[doc(alias = "SM0CTRL")]
pub type Sm0ctrl = crate::Reg<sm0ctrl::Sm0ctrlSpec>;
#[doc = "Control Register"]
pub mod sm0ctrl;
#[doc = "SM0VAL0 (rw) register accessor: Value Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0val0`] module"]
#[doc(alias = "SM0VAL0")]
pub type Sm0val0 = crate::Reg<sm0val0::Sm0val0Spec>;
#[doc = "Value Register 0"]
pub mod sm0val0;
#[doc = "SM0VAL1 (rw) register accessor: Value Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0val1`] module"]
#[doc(alias = "SM0VAL1")]
pub type Sm0val1 = crate::Reg<sm0val1::Sm0val1Spec>;
#[doc = "Value Register 1"]
pub mod sm0val1;
#[doc = "SM0VAL2 (rw) register accessor: Value Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0val2`] module"]
#[doc(alias = "SM0VAL2")]
pub type Sm0val2 = crate::Reg<sm0val2::Sm0val2Spec>;
#[doc = "Value Register 2"]
pub mod sm0val2;
#[doc = "SM0VAL3 (rw) register accessor: Value Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0val3`] module"]
#[doc(alias = "SM0VAL3")]
pub type Sm0val3 = crate::Reg<sm0val3::Sm0val3Spec>;
#[doc = "Value Register 3"]
pub mod sm0val3;
#[doc = "SM0VAL4 (rw) register accessor: Value Register 4\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0val4`] module"]
#[doc(alias = "SM0VAL4")]
pub type Sm0val4 = crate::Reg<sm0val4::Sm0val4Spec>;
#[doc = "Value Register 4"]
pub mod sm0val4;
#[doc = "SM0VAL5 (rw) register accessor: Value Register 5\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0val5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0val5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0val5`] module"]
#[doc(alias = "SM0VAL5")]
pub type Sm0val5 = crate::Reg<sm0val5::Sm0val5Spec>;
#[doc = "Value Register 5"]
pub mod sm0val5;
#[doc = "SM0OCTRL (rw) register accessor: Output Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0octrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0octrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0octrl`] module"]
#[doc(alias = "SM0OCTRL")]
pub type Sm0octrl = crate::Reg<sm0octrl::Sm0octrlSpec>;
#[doc = "Output Control Register"]
pub mod sm0octrl;
#[doc = "SM0STS (rw) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0sts::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0sts::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0sts`] module"]
#[doc(alias = "SM0STS")]
pub type Sm0sts = crate::Reg<sm0sts::Sm0stsSpec>;
#[doc = "Status Register"]
pub mod sm0sts;
#[doc = "SM0INTEN (rw) register accessor: Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0inten::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0inten::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0inten`] module"]
#[doc(alias = "SM0INTEN")]
pub type Sm0inten = crate::Reg<sm0inten::Sm0intenSpec>;
#[doc = "Interrupt Enable Register"]
pub mod sm0inten;
#[doc = "SM0DMAEN (rw) register accessor: DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0dmaen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0dmaen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0dmaen`] module"]
#[doc(alias = "SM0DMAEN")]
pub type Sm0dmaen = crate::Reg<sm0dmaen::Sm0dmaenSpec>;
#[doc = "DMA Enable Register"]
pub mod sm0dmaen;
#[doc = "SM0TCTRL (rw) register accessor: Output Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0tctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0tctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0tctrl`] module"]
#[doc(alias = "SM0TCTRL")]
pub type Sm0tctrl = crate::Reg<sm0tctrl::Sm0tctrlSpec>;
#[doc = "Output Trigger Control Register"]
pub mod sm0tctrl;
#[doc = "SM0DISMAP0 (rw) register accessor: Fault Disable Mapping Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0dismap0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0dismap0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0dismap0`] module"]
#[doc(alias = "SM0DISMAP0")]
pub type Sm0dismap0 = crate::Reg<sm0dismap0::Sm0dismap0Spec>;
#[doc = "Fault Disable Mapping Register 0"]
pub mod sm0dismap0;
#[doc = "SM0DTCNT0 (rw) register accessor: Deadtime Count Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0dtcnt0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0dtcnt0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0dtcnt0`] module"]
#[doc(alias = "SM0DTCNT0")]
pub type Sm0dtcnt0 = crate::Reg<sm0dtcnt0::Sm0dtcnt0Spec>;
#[doc = "Deadtime Count Register 0"]
pub mod sm0dtcnt0;
#[doc = "SM0DTCNT1 (rw) register accessor: Deadtime Count Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0dtcnt1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0dtcnt1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0dtcnt1`] module"]
#[doc(alias = "SM0DTCNT1")]
pub type Sm0dtcnt1 = crate::Reg<sm0dtcnt1::Sm0dtcnt1Spec>;
#[doc = "Deadtime Count Register 1"]
pub mod sm0dtcnt1;
#[doc = "SM0CAPTCTRLX (rw) register accessor: Capture Control X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0captctrlx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0captctrlx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0captctrlx`] module"]
#[doc(alias = "SM0CAPTCTRLX")]
pub type Sm0captctrlx = crate::Reg<sm0captctrlx::Sm0captctrlxSpec>;
#[doc = "Capture Control X Register"]
pub mod sm0captctrlx;
#[doc = "SM0CAPTCOMPX (rw) register accessor: Capture Compare X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0captcompx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0captcompx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0captcompx`] module"]
#[doc(alias = "SM0CAPTCOMPX")]
pub type Sm0captcompx = crate::Reg<sm0captcompx::Sm0captcompxSpec>;
#[doc = "Capture Compare X Register"]
pub mod sm0captcompx;
#[doc = "SM0CVAL0 (r) register accessor: Capture Value 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0cval0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0cval0`] module"]
#[doc(alias = "SM0CVAL0")]
pub type Sm0cval0 = crate::Reg<sm0cval0::Sm0cval0Spec>;
#[doc = "Capture Value 0 Register"]
pub mod sm0cval0;
#[doc = "SM0CVAL0CYC (r) register accessor: Capture Value 0 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0cval0cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0cval0cyc`] module"]
#[doc(alias = "SM0CVAL0CYC")]
pub type Sm0cval0cyc = crate::Reg<sm0cval0cyc::Sm0cval0cycSpec>;
#[doc = "Capture Value 0 Cycle Register"]
pub mod sm0cval0cyc;
#[doc = "SM0CVAL1 (r) register accessor: Capture Value 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0cval1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0cval1`] module"]
#[doc(alias = "SM0CVAL1")]
pub type Sm0cval1 = crate::Reg<sm0cval1::Sm0cval1Spec>;
#[doc = "Capture Value 1 Register"]
pub mod sm0cval1;
#[doc = "SM0CVAL1CYC (r) register accessor: Capture Value 1 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0cval1cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0cval1cyc`] module"]
#[doc(alias = "SM0CVAL1CYC")]
pub type Sm0cval1cyc = crate::Reg<sm0cval1cyc::Sm0cval1cycSpec>;
#[doc = "Capture Value 1 Cycle Register"]
pub mod sm0cval1cyc;
#[doc = "SM0CAPTFILTX (rw) register accessor: Capture PWM_X Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm0captfiltx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm0captfiltx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm0captfiltx`] module"]
#[doc(alias = "SM0CAPTFILTX")]
pub type Sm0captfiltx = crate::Reg<sm0captfiltx::Sm0captfiltxSpec>;
#[doc = "Capture PWM_X Input Filter Register"]
pub mod sm0captfiltx;
#[doc = "SM1CNT (r) register accessor: Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1cnt`] module"]
#[doc(alias = "SM1CNT")]
pub type Sm1cnt = crate::Reg<sm1cnt::Sm1cntSpec>;
#[doc = "Counter Register"]
pub mod sm1cnt;
#[doc = "SM1INIT (rw) register accessor: Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1init::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1init::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1init`] module"]
#[doc(alias = "SM1INIT")]
pub type Sm1init = crate::Reg<sm1init::Sm1initSpec>;
#[doc = "Initial Count Register"]
pub mod sm1init;
#[doc = "SM1CTRL2 (rw) register accessor: Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1ctrl2`] module"]
#[doc(alias = "SM1CTRL2")]
pub type Sm1ctrl2 = crate::Reg<sm1ctrl2::Sm1ctrl2Spec>;
#[doc = "Control 2 Register"]
pub mod sm1ctrl2;
#[doc = "SM1CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1ctrl`] module"]
#[doc(alias = "SM1CTRL")]
pub type Sm1ctrl = crate::Reg<sm1ctrl::Sm1ctrlSpec>;
#[doc = "Control Register"]
pub mod sm1ctrl;
#[doc = "SM1VAL0 (rw) register accessor: Value Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1val0`] module"]
#[doc(alias = "SM1VAL0")]
pub type Sm1val0 = crate::Reg<sm1val0::Sm1val0Spec>;
#[doc = "Value Register 0"]
pub mod sm1val0;
#[doc = "SM1VAL1 (rw) register accessor: Value Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1val1`] module"]
#[doc(alias = "SM1VAL1")]
pub type Sm1val1 = crate::Reg<sm1val1::Sm1val1Spec>;
#[doc = "Value Register 1"]
pub mod sm1val1;
#[doc = "SM1VAL2 (rw) register accessor: Value Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1val2`] module"]
#[doc(alias = "SM1VAL2")]
pub type Sm1val2 = crate::Reg<sm1val2::Sm1val2Spec>;
#[doc = "Value Register 2"]
pub mod sm1val2;
#[doc = "SM1VAL3 (rw) register accessor: Value Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1val3`] module"]
#[doc(alias = "SM1VAL3")]
pub type Sm1val3 = crate::Reg<sm1val3::Sm1val3Spec>;
#[doc = "Value Register 3"]
pub mod sm1val3;
#[doc = "SM1VAL4 (rw) register accessor: Value Register 4\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1val4`] module"]
#[doc(alias = "SM1VAL4")]
pub type Sm1val4 = crate::Reg<sm1val4::Sm1val4Spec>;
#[doc = "Value Register 4"]
pub mod sm1val4;
#[doc = "SM1VAL5 (rw) register accessor: Value Register 5\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1val5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1val5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1val5`] module"]
#[doc(alias = "SM1VAL5")]
pub type Sm1val5 = crate::Reg<sm1val5::Sm1val5Spec>;
#[doc = "Value Register 5"]
pub mod sm1val5;
#[doc = "SM1OCTRL (rw) register accessor: Output Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1octrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1octrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1octrl`] module"]
#[doc(alias = "SM1OCTRL")]
pub type Sm1octrl = crate::Reg<sm1octrl::Sm1octrlSpec>;
#[doc = "Output Control Register"]
pub mod sm1octrl;
#[doc = "SM1STS (rw) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1sts::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1sts::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1sts`] module"]
#[doc(alias = "SM1STS")]
pub type Sm1sts = crate::Reg<sm1sts::Sm1stsSpec>;
#[doc = "Status Register"]
pub mod sm1sts;
#[doc = "SM1INTEN (rw) register accessor: Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1inten::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1inten::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1inten`] module"]
#[doc(alias = "SM1INTEN")]
pub type Sm1inten = crate::Reg<sm1inten::Sm1intenSpec>;
#[doc = "Interrupt Enable Register"]
pub mod sm1inten;
#[doc = "SM1DMAEN (rw) register accessor: DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1dmaen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1dmaen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1dmaen`] module"]
#[doc(alias = "SM1DMAEN")]
pub type Sm1dmaen = crate::Reg<sm1dmaen::Sm1dmaenSpec>;
#[doc = "DMA Enable Register"]
pub mod sm1dmaen;
#[doc = "SM1TCTRL (rw) register accessor: Output Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1tctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1tctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1tctrl`] module"]
#[doc(alias = "SM1TCTRL")]
pub type Sm1tctrl = crate::Reg<sm1tctrl::Sm1tctrlSpec>;
#[doc = "Output Trigger Control Register"]
pub mod sm1tctrl;
#[doc = "SM1DISMAP0 (rw) register accessor: Fault Disable Mapping Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1dismap0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1dismap0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1dismap0`] module"]
#[doc(alias = "SM1DISMAP0")]
pub type Sm1dismap0 = crate::Reg<sm1dismap0::Sm1dismap0Spec>;
#[doc = "Fault Disable Mapping Register 0"]
pub mod sm1dismap0;
#[doc = "SM1DTCNT0 (rw) register accessor: Deadtime Count Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1dtcnt0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1dtcnt0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1dtcnt0`] module"]
#[doc(alias = "SM1DTCNT0")]
pub type Sm1dtcnt0 = crate::Reg<sm1dtcnt0::Sm1dtcnt0Spec>;
#[doc = "Deadtime Count Register 0"]
pub mod sm1dtcnt0;
#[doc = "SM1DTCNT1 (rw) register accessor: Deadtime Count Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1dtcnt1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1dtcnt1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1dtcnt1`] module"]
#[doc(alias = "SM1DTCNT1")]
pub type Sm1dtcnt1 = crate::Reg<sm1dtcnt1::Sm1dtcnt1Spec>;
#[doc = "Deadtime Count Register 1"]
pub mod sm1dtcnt1;
#[doc = "SM1CAPTCTRLX (rw) register accessor: Capture Control X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1captctrlx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1captctrlx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1captctrlx`] module"]
#[doc(alias = "SM1CAPTCTRLX")]
pub type Sm1captctrlx = crate::Reg<sm1captctrlx::Sm1captctrlxSpec>;
#[doc = "Capture Control X Register"]
pub mod sm1captctrlx;
#[doc = "SM1CAPTCOMPX (rw) register accessor: Capture Compare X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1captcompx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1captcompx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1captcompx`] module"]
#[doc(alias = "SM1CAPTCOMPX")]
pub type Sm1captcompx = crate::Reg<sm1captcompx::Sm1captcompxSpec>;
#[doc = "Capture Compare X Register"]
pub mod sm1captcompx;
#[doc = "SM1CVAL0 (r) register accessor: Capture Value 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cval0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1cval0`] module"]
#[doc(alias = "SM1CVAL0")]
pub type Sm1cval0 = crate::Reg<sm1cval0::Sm1cval0Spec>;
#[doc = "Capture Value 0 Register"]
pub mod sm1cval0;
#[doc = "SM1CVAL0CYC (r) register accessor: Capture Value 0 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cval0cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1cval0cyc`] module"]
#[doc(alias = "SM1CVAL0CYC")]
pub type Sm1cval0cyc = crate::Reg<sm1cval0cyc::Sm1cval0cycSpec>;
#[doc = "Capture Value 0 Cycle Register"]
pub mod sm1cval0cyc;
#[doc = "SM1CVAL1 (r) register accessor: Capture Value 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cval1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1cval1`] module"]
#[doc(alias = "SM1CVAL1")]
pub type Sm1cval1 = crate::Reg<sm1cval1::Sm1cval1Spec>;
#[doc = "Capture Value 1 Register"]
pub mod sm1cval1;
#[doc = "SM1CVAL1CYC (r) register accessor: Capture Value 1 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1cval1cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1cval1cyc`] module"]
#[doc(alias = "SM1CVAL1CYC")]
pub type Sm1cval1cyc = crate::Reg<sm1cval1cyc::Sm1cval1cycSpec>;
#[doc = "Capture Value 1 Cycle Register"]
pub mod sm1cval1cyc;
#[doc = "SM1PHASEDLY (rw) register accessor: Phase Delay Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1phasedly::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1phasedly::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1phasedly`] module"]
#[doc(alias = "SM1PHASEDLY")]
pub type Sm1phasedly = crate::Reg<sm1phasedly::Sm1phasedlySpec>;
#[doc = "Phase Delay Register"]
pub mod sm1phasedly;
#[doc = "SM1CAPTFILTX (rw) register accessor: Capture PWM_X Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1captfiltx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1captfiltx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm1captfiltx`] module"]
#[doc(alias = "SM1CAPTFILTX")]
pub type Sm1captfiltx = crate::Reg<sm1captfiltx::Sm1captfiltxSpec>;
#[doc = "Capture PWM_X Input Filter Register"]
pub mod sm1captfiltx;
#[doc = "SM2CNT (r) register accessor: Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2cnt`] module"]
#[doc(alias = "SM2CNT")]
pub type Sm2cnt = crate::Reg<sm2cnt::Sm2cntSpec>;
#[doc = "Counter Register"]
pub mod sm2cnt;
#[doc = "SM2INIT (rw) register accessor: Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2init::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2init::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2init`] module"]
#[doc(alias = "SM2INIT")]
pub type Sm2init = crate::Reg<sm2init::Sm2initSpec>;
#[doc = "Initial Count Register"]
pub mod sm2init;
#[doc = "SM2CTRL2 (rw) register accessor: Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2ctrl2`] module"]
#[doc(alias = "SM2CTRL2")]
pub type Sm2ctrl2 = crate::Reg<sm2ctrl2::Sm2ctrl2Spec>;
#[doc = "Control 2 Register"]
pub mod sm2ctrl2;
#[doc = "SM2CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2ctrl`] module"]
#[doc(alias = "SM2CTRL")]
pub type Sm2ctrl = crate::Reg<sm2ctrl::Sm2ctrlSpec>;
#[doc = "Control Register"]
pub mod sm2ctrl;
#[doc = "SM2VAL0 (rw) register accessor: Value Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2val0`] module"]
#[doc(alias = "SM2VAL0")]
pub type Sm2val0 = crate::Reg<sm2val0::Sm2val0Spec>;
#[doc = "Value Register 0"]
pub mod sm2val0;
#[doc = "SM2VAL1 (rw) register accessor: Value Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2val1`] module"]
#[doc(alias = "SM2VAL1")]
pub type Sm2val1 = crate::Reg<sm2val1::Sm2val1Spec>;
#[doc = "Value Register 1"]
pub mod sm2val1;
#[doc = "SM2VAL2 (rw) register accessor: Value Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2val2`] module"]
#[doc(alias = "SM2VAL2")]
pub type Sm2val2 = crate::Reg<sm2val2::Sm2val2Spec>;
#[doc = "Value Register 2"]
pub mod sm2val2;
#[doc = "SM2VAL3 (rw) register accessor: Value Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2val3`] module"]
#[doc(alias = "SM2VAL3")]
pub type Sm2val3 = crate::Reg<sm2val3::Sm2val3Spec>;
#[doc = "Value Register 3"]
pub mod sm2val3;
#[doc = "SM2VAL4 (rw) register accessor: Value Register 4\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2val4`] module"]
#[doc(alias = "SM2VAL4")]
pub type Sm2val4 = crate::Reg<sm2val4::Sm2val4Spec>;
#[doc = "Value Register 4"]
pub mod sm2val4;
#[doc = "SM2VAL5 (rw) register accessor: Value Register 5\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2val5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2val5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2val5`] module"]
#[doc(alias = "SM2VAL5")]
pub type Sm2val5 = crate::Reg<sm2val5::Sm2val5Spec>;
#[doc = "Value Register 5"]
pub mod sm2val5;
#[doc = "SM2OCTRL (rw) register accessor: Output Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2octrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2octrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2octrl`] module"]
#[doc(alias = "SM2OCTRL")]
pub type Sm2octrl = crate::Reg<sm2octrl::Sm2octrlSpec>;
#[doc = "Output Control Register"]
pub mod sm2octrl;
#[doc = "SM2STS (rw) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2sts::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2sts::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2sts`] module"]
#[doc(alias = "SM2STS")]
pub type Sm2sts = crate::Reg<sm2sts::Sm2stsSpec>;
#[doc = "Status Register"]
pub mod sm2sts;
#[doc = "SM2INTEN (rw) register accessor: Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2inten::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2inten::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2inten`] module"]
#[doc(alias = "SM2INTEN")]
pub type Sm2inten = crate::Reg<sm2inten::Sm2intenSpec>;
#[doc = "Interrupt Enable Register"]
pub mod sm2inten;
#[doc = "SM2DMAEN (rw) register accessor: DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2dmaen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2dmaen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2dmaen`] module"]
#[doc(alias = "SM2DMAEN")]
pub type Sm2dmaen = crate::Reg<sm2dmaen::Sm2dmaenSpec>;
#[doc = "DMA Enable Register"]
pub mod sm2dmaen;
#[doc = "SM2TCTRL (rw) register accessor: Output Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2tctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2tctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2tctrl`] module"]
#[doc(alias = "SM2TCTRL")]
pub type Sm2tctrl = crate::Reg<sm2tctrl::Sm2tctrlSpec>;
#[doc = "Output Trigger Control Register"]
pub mod sm2tctrl;
#[doc = "SM2DISMAP0 (rw) register accessor: Fault Disable Mapping Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2dismap0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2dismap0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2dismap0`] module"]
#[doc(alias = "SM2DISMAP0")]
pub type Sm2dismap0 = crate::Reg<sm2dismap0::Sm2dismap0Spec>;
#[doc = "Fault Disable Mapping Register 0"]
pub mod sm2dismap0;
#[doc = "SM2DTCNT0 (rw) register accessor: Deadtime Count Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2dtcnt0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2dtcnt0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2dtcnt0`] module"]
#[doc(alias = "SM2DTCNT0")]
pub type Sm2dtcnt0 = crate::Reg<sm2dtcnt0::Sm2dtcnt0Spec>;
#[doc = "Deadtime Count Register 0"]
pub mod sm2dtcnt0;
#[doc = "SM2DTCNT1 (rw) register accessor: Deadtime Count Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2dtcnt1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2dtcnt1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2dtcnt1`] module"]
#[doc(alias = "SM2DTCNT1")]
pub type Sm2dtcnt1 = crate::Reg<sm2dtcnt1::Sm2dtcnt1Spec>;
#[doc = "Deadtime Count Register 1"]
pub mod sm2dtcnt1;
#[doc = "SM2CAPTCTRLX (rw) register accessor: Capture Control X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2captctrlx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2captctrlx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2captctrlx`] module"]
#[doc(alias = "SM2CAPTCTRLX")]
pub type Sm2captctrlx = crate::Reg<sm2captctrlx::Sm2captctrlxSpec>;
#[doc = "Capture Control X Register"]
pub mod sm2captctrlx;
#[doc = "SM2CAPTCOMPX (rw) register accessor: Capture Compare X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2captcompx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2captcompx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2captcompx`] module"]
#[doc(alias = "SM2CAPTCOMPX")]
pub type Sm2captcompx = crate::Reg<sm2captcompx::Sm2captcompxSpec>;
#[doc = "Capture Compare X Register"]
pub mod sm2captcompx;
#[doc = "SM2CVAL0 (r) register accessor: Capture Value 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cval0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2cval0`] module"]
#[doc(alias = "SM2CVAL0")]
pub type Sm2cval0 = crate::Reg<sm2cval0::Sm2cval0Spec>;
#[doc = "Capture Value 0 Register"]
pub mod sm2cval0;
#[doc = "SM2CVAL0CYC (r) register accessor: Capture Value 0 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cval0cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2cval0cyc`] module"]
#[doc(alias = "SM2CVAL0CYC")]
pub type Sm2cval0cyc = crate::Reg<sm2cval0cyc::Sm2cval0cycSpec>;
#[doc = "Capture Value 0 Cycle Register"]
pub mod sm2cval0cyc;
#[doc = "SM2CVAL1 (r) register accessor: Capture Value 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cval1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2cval1`] module"]
#[doc(alias = "SM2CVAL1")]
pub type Sm2cval1 = crate::Reg<sm2cval1::Sm2cval1Spec>;
#[doc = "Capture Value 1 Register"]
pub mod sm2cval1;
#[doc = "SM2CVAL1CYC (r) register accessor: Capture Value 1 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2cval1cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2cval1cyc`] module"]
#[doc(alias = "SM2CVAL1CYC")]
pub type Sm2cval1cyc = crate::Reg<sm2cval1cyc::Sm2cval1cycSpec>;
#[doc = "Capture Value 1 Cycle Register"]
pub mod sm2cval1cyc;
#[doc = "SM2PHASEDLY (rw) register accessor: Phase Delay Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2phasedly::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2phasedly::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2phasedly`] module"]
#[doc(alias = "SM2PHASEDLY")]
pub type Sm2phasedly = crate::Reg<sm2phasedly::Sm2phasedlySpec>;
#[doc = "Phase Delay Register"]
pub mod sm2phasedly;
#[doc = "SM2CAPTFILTX (rw) register accessor: Capture PWM_X Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2captfiltx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2captfiltx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm2captfiltx`] module"]
#[doc(alias = "SM2CAPTFILTX")]
pub type Sm2captfiltx = crate::Reg<sm2captfiltx::Sm2captfiltxSpec>;
#[doc = "Capture PWM_X Input Filter Register"]
pub mod sm2captfiltx;
#[doc = "SM3CNT (r) register accessor: Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cnt::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3cnt`] module"]
#[doc(alias = "SM3CNT")]
pub type Sm3cnt = crate::Reg<sm3cnt::Sm3cntSpec>;
#[doc = "Counter Register"]
pub mod sm3cnt;
#[doc = "SM3INIT (rw) register accessor: Initial Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3init::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3init::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3init`] module"]
#[doc(alias = "SM3INIT")]
pub type Sm3init = crate::Reg<sm3init::Sm3initSpec>;
#[doc = "Initial Count Register"]
pub mod sm3init;
#[doc = "SM3CTRL2 (rw) register accessor: Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3ctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3ctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3ctrl2`] module"]
#[doc(alias = "SM3CTRL2")]
pub type Sm3ctrl2 = crate::Reg<sm3ctrl2::Sm3ctrl2Spec>;
#[doc = "Control 2 Register"]
pub mod sm3ctrl2;
#[doc = "SM3CTRL (rw) register accessor: Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3ctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3ctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3ctrl`] module"]
#[doc(alias = "SM3CTRL")]
pub type Sm3ctrl = crate::Reg<sm3ctrl::Sm3ctrlSpec>;
#[doc = "Control Register"]
pub mod sm3ctrl;
#[doc = "SM3VAL0 (rw) register accessor: Value Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3val0`] module"]
#[doc(alias = "SM3VAL0")]
pub type Sm3val0 = crate::Reg<sm3val0::Sm3val0Spec>;
#[doc = "Value Register 0"]
pub mod sm3val0;
#[doc = "SM3VAL1 (rw) register accessor: Value Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3val1`] module"]
#[doc(alias = "SM3VAL1")]
pub type Sm3val1 = crate::Reg<sm3val1::Sm3val1Spec>;
#[doc = "Value Register 1"]
pub mod sm3val1;
#[doc = "SM3VAL2 (rw) register accessor: Value Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3val2`] module"]
#[doc(alias = "SM3VAL2")]
pub type Sm3val2 = crate::Reg<sm3val2::Sm3val2Spec>;
#[doc = "Value Register 2"]
pub mod sm3val2;
#[doc = "SM3VAL3 (rw) register accessor: Value Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val3::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val3::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3val3`] module"]
#[doc(alias = "SM3VAL3")]
pub type Sm3val3 = crate::Reg<sm3val3::Sm3val3Spec>;
#[doc = "Value Register 3"]
pub mod sm3val3;
#[doc = "SM3VAL4 (rw) register accessor: Value Register 4\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val4::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val4::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3val4`] module"]
#[doc(alias = "SM3VAL4")]
pub type Sm3val4 = crate::Reg<sm3val4::Sm3val4Spec>;
#[doc = "Value Register 4"]
pub mod sm3val4;
#[doc = "SM3VAL5 (rw) register accessor: Value Register 5\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3val5::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3val5::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3val5`] module"]
#[doc(alias = "SM3VAL5")]
pub type Sm3val5 = crate::Reg<sm3val5::Sm3val5Spec>;
#[doc = "Value Register 5"]
pub mod sm3val5;
#[doc = "SM3OCTRL (rw) register accessor: Output Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3octrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3octrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3octrl`] module"]
#[doc(alias = "SM3OCTRL")]
pub type Sm3octrl = crate::Reg<sm3octrl::Sm3octrlSpec>;
#[doc = "Output Control Register"]
pub mod sm3octrl;
#[doc = "SM3STS (rw) register accessor: Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3sts::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3sts::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3sts`] module"]
#[doc(alias = "SM3STS")]
pub type Sm3sts = crate::Reg<sm3sts::Sm3stsSpec>;
#[doc = "Status Register"]
pub mod sm3sts;
#[doc = "SM3INTEN (rw) register accessor: Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3inten::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3inten::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3inten`] module"]
#[doc(alias = "SM3INTEN")]
pub type Sm3inten = crate::Reg<sm3inten::Sm3intenSpec>;
#[doc = "Interrupt Enable Register"]
pub mod sm3inten;
#[doc = "SM3DMAEN (rw) register accessor: DMA Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3dmaen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3dmaen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3dmaen`] module"]
#[doc(alias = "SM3DMAEN")]
pub type Sm3dmaen = crate::Reg<sm3dmaen::Sm3dmaenSpec>;
#[doc = "DMA Enable Register"]
pub mod sm3dmaen;
#[doc = "SM3TCTRL (rw) register accessor: Output Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3tctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3tctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3tctrl`] module"]
#[doc(alias = "SM3TCTRL")]
pub type Sm3tctrl = crate::Reg<sm3tctrl::Sm3tctrlSpec>;
#[doc = "Output Trigger Control Register"]
pub mod sm3tctrl;
#[doc = "SM3DISMAP0 (rw) register accessor: Fault Disable Mapping Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3dismap0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3dismap0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3dismap0`] module"]
#[doc(alias = "SM3DISMAP0")]
pub type Sm3dismap0 = crate::Reg<sm3dismap0::Sm3dismap0Spec>;
#[doc = "Fault Disable Mapping Register 0"]
pub mod sm3dismap0;
#[doc = "SM3DTCNT0 (rw) register accessor: Deadtime Count Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3dtcnt0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3dtcnt0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3dtcnt0`] module"]
#[doc(alias = "SM3DTCNT0")]
pub type Sm3dtcnt0 = crate::Reg<sm3dtcnt0::Sm3dtcnt0Spec>;
#[doc = "Deadtime Count Register 0"]
pub mod sm3dtcnt0;
#[doc = "SM3DTCNT1 (rw) register accessor: Deadtime Count Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3dtcnt1::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3dtcnt1::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3dtcnt1`] module"]
#[doc(alias = "SM3DTCNT1")]
pub type Sm3dtcnt1 = crate::Reg<sm3dtcnt1::Sm3dtcnt1Spec>;
#[doc = "Deadtime Count Register 1"]
pub mod sm3dtcnt1;
#[doc = "SM3CAPTCTRLX (rw) register accessor: Capture Control X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3captctrlx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3captctrlx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3captctrlx`] module"]
#[doc(alias = "SM3CAPTCTRLX")]
pub type Sm3captctrlx = crate::Reg<sm3captctrlx::Sm3captctrlxSpec>;
#[doc = "Capture Control X Register"]
pub mod sm3captctrlx;
#[doc = "SM3CAPTCOMPX (rw) register accessor: Capture Compare X Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3captcompx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3captcompx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3captcompx`] module"]
#[doc(alias = "SM3CAPTCOMPX")]
pub type Sm3captcompx = crate::Reg<sm3captcompx::Sm3captcompxSpec>;
#[doc = "Capture Compare X Register"]
pub mod sm3captcompx;
#[doc = "SM3CVAL0 (r) register accessor: Capture Value 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cval0::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3cval0`] module"]
#[doc(alias = "SM3CVAL0")]
pub type Sm3cval0 = crate::Reg<sm3cval0::Sm3cval0Spec>;
#[doc = "Capture Value 0 Register"]
pub mod sm3cval0;
#[doc = "SM3CVAL0CYC (r) register accessor: Capture Value 0 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cval0cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3cval0cyc`] module"]
#[doc(alias = "SM3CVAL0CYC")]
pub type Sm3cval0cyc = crate::Reg<sm3cval0cyc::Sm3cval0cycSpec>;
#[doc = "Capture Value 0 Cycle Register"]
pub mod sm3cval0cyc;
#[doc = "SM3CVAL1 (r) register accessor: Capture Value 1 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cval1::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3cval1`] module"]
#[doc(alias = "SM3CVAL1")]
pub type Sm3cval1 = crate::Reg<sm3cval1::Sm3cval1Spec>;
#[doc = "Capture Value 1 Register"]
pub mod sm3cval1;
#[doc = "SM3CVAL1CYC (r) register accessor: Capture Value 1 Cycle Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3cval1cyc::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3cval1cyc`] module"]
#[doc(alias = "SM3CVAL1CYC")]
pub type Sm3cval1cyc = crate::Reg<sm3cval1cyc::Sm3cval1cycSpec>;
#[doc = "Capture Value 1 Cycle Register"]
pub mod sm3cval1cyc;
#[doc = "SM3PHASEDLY (rw) register accessor: Phase Delay Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3phasedly::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3phasedly::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3phasedly`] module"]
#[doc(alias = "SM3PHASEDLY")]
pub type Sm3phasedly = crate::Reg<sm3phasedly::Sm3phasedlySpec>;
#[doc = "Phase Delay Register"]
pub mod sm3phasedly;
#[doc = "SM3CAPTFILTX (rw) register accessor: Capture PWM_X Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3captfiltx::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3captfiltx::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@sm3captfiltx`] module"]
#[doc(alias = "SM3CAPTFILTX")]
pub type Sm3captfiltx = crate::Reg<sm3captfiltx::Sm3captfiltxSpec>;
#[doc = "Capture PWM_X Input Filter Register"]
pub mod sm3captfiltx;
#[doc = "OUTEN (rw) register accessor: Output Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`outen::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`outen::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@outen`] module"]
#[doc(alias = "OUTEN")]
pub type Outen = crate::Reg<outen::OutenSpec>;
#[doc = "Output Enable Register"]
pub mod outen;
#[doc = "MASK (rw) register accessor: Mask Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mask::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mask::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mask`] module"]
#[doc(alias = "MASK")]
pub type Mask = crate::Reg<mask::MaskSpec>;
#[doc = "Mask Register"]
pub mod mask;
#[doc = "SWCOUT (rw) register accessor: Software Controlled Output Register\n\nYou can [`read`](crate::Reg::read) this register and get [`swcout::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swcout::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@swcout`] module"]
#[doc(alias = "SWCOUT")]
pub type Swcout = crate::Reg<swcout::SwcoutSpec>;
#[doc = "Software Controlled Output Register"]
pub mod swcout;
#[doc = "DTSRCSEL (rw) register accessor: PWM Source Select Register\n\nYou can [`read`](crate::Reg::read) this register and get [`dtsrcsel::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dtsrcsel::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@dtsrcsel`] module"]
#[doc(alias = "DTSRCSEL")]
pub type Dtsrcsel = crate::Reg<dtsrcsel::DtsrcselSpec>;
#[doc = "PWM Source Select Register"]
pub mod dtsrcsel;
#[doc = "MCTRL (rw) register accessor: Master Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mctrl::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctrl::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mctrl`] module"]
#[doc(alias = "MCTRL")]
pub type Mctrl = crate::Reg<mctrl::MctrlSpec>;
#[doc = "Master Control Register"]
pub mod mctrl;
#[doc = "MCTRL2 (rw) register accessor: Master Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mctrl2::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctrl2::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@mctrl2`] module"]
#[doc(alias = "MCTRL2")]
pub type Mctrl2 = crate::Reg<mctrl2::Mctrl2Spec>;
#[doc = "Master Control 2 Register"]
pub mod mctrl2;
#[doc = "FCTRL0 (rw) register accessor: Fault Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fctrl0`] module"]
#[doc(alias = "FCTRL0")]
pub type Fctrl0 = crate::Reg<fctrl0::Fctrl0Spec>;
#[doc = "Fault Control Register"]
pub mod fctrl0;
#[doc = "FSTS0 (rw) register accessor: Fault Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fsts0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fsts0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fsts0`] module"]
#[doc(alias = "FSTS0")]
pub type Fsts0 = crate::Reg<fsts0::Fsts0Spec>;
#[doc = "Fault Status Register"]
pub mod fsts0;
#[doc = "FFILT0 (rw) register accessor: Fault Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ffilt0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ffilt0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ffilt0`] module"]
#[doc(alias = "FFILT0")]
pub type Ffilt0 = crate::Reg<ffilt0::Ffilt0Spec>;
#[doc = "Fault Filter Register"]
pub mod ffilt0;
#[doc = "FTST0 (rw) register accessor: Fault Test Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ftst0::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ftst0::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@ftst0`] module"]
#[doc(alias = "FTST0")]
pub type Ftst0 = crate::Reg<ftst0::Ftst0Spec>;
#[doc = "Fault Test Register"]
pub mod ftst0;
#[doc = "FCTRL20 (rw) register accessor: Fault Control 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl20::R`]. You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl20::W`]. You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@fctrl20`] module"]
#[doc(alias = "FCTRL20")]
pub type Fctrl20 = crate::Reg<fctrl20::Fctrl20Spec>;
#[doc = "Fault Control 2 Register"]
pub mod fctrl20;
