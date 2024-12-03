//! Register definitions for Synopsys DesignWare USB OTG core

#![allow(missing_docs)]

use core::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RW;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct R;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct W;

mod sealed {
    use super::*;
    pub trait Access {}
    impl Access for R {}
    impl Access for W {}
    impl Access for RW {}
}

pub trait Access: sealed::Access + Copy {}
impl Access for R {}
impl Access for W {}
impl Access for RW {}

pub trait Read: Access {}
impl Read for RW {}
impl Read for R {}

pub trait Write: Access {}
impl Write for RW {}
impl Write for W {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Reg<T: Copy, A: Access> {
    ptr: *mut u8,
    phantom: PhantomData<*mut (T, A)>,
}
unsafe impl<T: Copy, A: Access> Send for Reg<T, A> {}
unsafe impl<T: Copy, A: Access> Sync for Reg<T, A> {}

impl<T: Copy, A: Access> Reg<T, A> {
    #[allow(clippy::missing_safety_doc)]
    #[inline(always)]
    pub const unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self {
            ptr: ptr as _,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub const fn as_ptr(&self) -> *mut T {
        self.ptr as _
    }
}

impl<T: Copy, A: Read> Reg<T, A> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { (self.ptr as *mut T).read_volatile() }
    }
}

impl<T: Copy, A: Write> Reg<T, A> {
    #[inline(always)]
    pub fn write_value(&self, val: T) {
        unsafe { (self.ptr as *mut T).write_volatile(val) }
    }
}

impl<T: Default + Copy, A: Write> Reg<T, A> {
    #[inline(always)]
    pub fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

impl<T: Copy, A: Read + Write> Reg<T, A> {
    #[inline(always)]
    pub fn modify<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = self.read();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

#[doc = "USB on the go"]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Otg {
    ptr: *mut u8,
}
unsafe impl Send for Otg {}
unsafe impl Sync for Otg {}
impl Otg {
    #[inline(always)]
    pub const unsafe fn from_ptr(ptr: *mut ()) -> Self {
        Self { ptr: ptr as _ }
    }
    #[inline(always)]
    pub const fn as_ptr(&self) -> *mut () {
        self.ptr as _
    }
    #[doc = "Control and status register"]
    #[inline(always)]
    pub const fn gotgctl(self) -> Reg<regs::Gotgctl, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0usize) as _) }
    }
    #[doc = "Interrupt register"]
    #[inline(always)]
    pub const fn gotgint(self) -> Reg<regs::Gotgint, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x04usize) as _) }
    }
    #[doc = "AHB configuration register"]
    #[inline(always)]
    pub const fn gahbcfg(self) -> Reg<regs::Gahbcfg, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x08usize) as _) }
    }
    #[doc = "USB configuration register"]
    #[inline(always)]
    pub const fn gusbcfg(self) -> Reg<regs::Gusbcfg, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0cusize) as _) }
    }
    #[doc = "Reset register"]
    #[inline(always)]
    pub const fn grstctl(self) -> Reg<regs::Grstctl, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x10usize) as _) }
    }
    #[doc = "Core interrupt register"]
    #[inline(always)]
    pub const fn gintsts(self) -> Reg<regs::Gintsts, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x14usize) as _) }
    }
    #[doc = "Interrupt mask register"]
    #[inline(always)]
    pub const fn gintmsk(self) -> Reg<regs::Gintmsk, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x18usize) as _) }
    }
    #[doc = "Receive status debug read register"]
    #[inline(always)]
    pub const fn grxstsr(self) -> Reg<regs::Grxsts, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x1cusize) as _) }
    }
    #[doc = "Status read and pop register"]
    #[inline(always)]
    pub const fn grxstsp(self) -> Reg<regs::Grxsts, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x20usize) as _) }
    }
    #[doc = "Receive FIFO size register"]
    #[inline(always)]
    pub const fn grxfsiz(self) -> Reg<regs::Grxfsiz, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x24usize) as _) }
    }
    #[doc = "Endpoint 0 transmit FIFO size register (device mode)"]
    #[inline(always)]
    pub const fn dieptxf0(self) -> Reg<regs::Fsiz, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x28usize) as _) }
    }
    #[doc = "Non-periodic transmit FIFO size register (host mode)"]
    #[inline(always)]
    pub const fn hnptxfsiz(self) -> Reg<regs::Fsiz, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x28usize) as _) }
    }
    #[doc = "Non-periodic transmit FIFO/queue status register (host mode)"]
    #[inline(always)]
    pub const fn hnptxsts(self) -> Reg<regs::Hnptxsts, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x2cusize) as _) }
    }
    #[doc = "OTG I2C access register"]
    #[inline(always)]
    pub const fn gi2cctl(self) -> Reg<regs::Gi2cctl, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x30usize) as _) }
    }
    #[doc = "General core configuration register, for core_id 0x0000_1xxx"]
    #[inline(always)]
    pub const fn gccfg_v1(self) -> Reg<regs::GccfgV1, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x38usize) as _) }
    }
    #[doc = "General core configuration register, for core_id 0x0000_\\[23\\]xxx"]
    #[inline(always)]
    pub const fn gccfg_v2(self) -> Reg<regs::GccfgV2, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x38usize) as _) }
    }
    #[doc = "Core ID register"]
    #[inline(always)]
    pub const fn cid(self) -> Reg<regs::Cid, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x3cusize) as _) }
    }
    #[doc = "OTG core LPM configuration register"]
    #[inline(always)]
    pub const fn glpmcfg(self) -> Reg<regs::Glpmcfg, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x54usize) as _) }
    }
    #[doc = "Host periodic transmit FIFO size register"]
    #[inline(always)]
    pub const fn hptxfsiz(self) -> Reg<regs::Fsiz, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0100usize) as _) }
    }
    #[doc = "Device IN endpoint transmit FIFO size register"]
    #[inline(always)]
    pub const fn dieptxf(self, n: usize) -> Reg<regs::Fsiz, RW> {
        assert!(n < 7usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0104usize + n * 4usize) as _) }
    }
    #[doc = "Host configuration register"]
    #[inline(always)]
    pub const fn hcfg(self) -> Reg<regs::Hcfg, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0400usize) as _) }
    }
    #[doc = "Host frame interval register"]
    #[inline(always)]
    pub const fn hfir(self) -> Reg<regs::Hfir, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0404usize) as _) }
    }
    #[doc = "Host frame number/frame time remaining register"]
    #[inline(always)]
    pub const fn hfnum(self) -> Reg<regs::Hfnum, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0408usize) as _) }
    }
    #[doc = "Periodic transmit FIFO/queue status register"]
    #[inline(always)]
    pub const fn hptxsts(self) -> Reg<regs::Hptxsts, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0410usize) as _) }
    }
    #[doc = "Host all channels interrupt register"]
    #[inline(always)]
    pub const fn haint(self) -> Reg<regs::Haint, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0414usize) as _) }
    }
    #[doc = "Host all channels interrupt mask register"]
    #[inline(always)]
    pub const fn haintmsk(self) -> Reg<regs::Haintmsk, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0418usize) as _) }
    }
    #[doc = "Host port control and status register"]
    #[inline(always)]
    pub const fn hprt(self) -> Reg<regs::Hprt, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0440usize) as _) }
    }
    #[doc = "Host channel characteristics register"]
    #[inline(always)]
    pub const fn hcchar(self, n: usize) -> Reg<regs::Hcchar, RW> {
        assert!(n < 12usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0500usize + n * 32usize) as _) }
    }
    #[doc = "Host channel split control register"]
    #[inline(always)]
    pub const fn hcsplt(self, n: usize) -> Reg<u32, RW> {
        assert!(n < 12usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0504usize + n * 32usize) as _) }
    }
    #[doc = "Host channel interrupt register"]
    #[inline(always)]
    pub const fn hcint(self, n: usize) -> Reg<regs::Hcint, RW> {
        assert!(n < 12usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0508usize + n * 32usize) as _) }
    }
    #[doc = "Host channel mask register"]
    #[inline(always)]
    pub const fn hcintmsk(self, n: usize) -> Reg<regs::Hcintmsk, RW> {
        assert!(n < 12usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x050cusize + n * 32usize) as _) }
    }
    #[doc = "Host channel transfer size register"]
    #[inline(always)]
    pub const fn hctsiz(self, n: usize) -> Reg<regs::Hctsiz, RW> {
        assert!(n < 12usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0510usize + n * 32usize) as _) }
    }
    #[doc = "Device configuration register"]
    #[inline(always)]
    pub const fn dcfg(self) -> Reg<regs::Dcfg, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0800usize) as _) }
    }
    #[doc = "Device control register"]
    #[inline(always)]
    pub const fn dctl(self) -> Reg<regs::Dctl, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0804usize) as _) }
    }
    #[doc = "Device status register"]
    #[inline(always)]
    pub const fn dsts(self) -> Reg<regs::Dsts, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0808usize) as _) }
    }
    #[doc = "Device IN endpoint common interrupt mask register"]
    #[inline(always)]
    pub const fn diepmsk(self) -> Reg<regs::Diepmsk, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0810usize) as _) }
    }
    #[doc = "Device OUT endpoint common interrupt mask register"]
    #[inline(always)]
    pub const fn doepmsk(self) -> Reg<regs::Doepmsk, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0814usize) as _) }
    }
    #[doc = "Device all endpoints interrupt register"]
    #[inline(always)]
    pub const fn daint(self) -> Reg<regs::Daint, R> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0818usize) as _) }
    }
    #[doc = "All endpoints interrupt mask register"]
    #[inline(always)]
    pub const fn daintmsk(self) -> Reg<regs::Daintmsk, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x081cusize) as _) }
    }
    #[doc = "Device VBUS discharge time register"]
    #[inline(always)]
    pub const fn dvbusdis(self) -> Reg<regs::Dvbusdis, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0828usize) as _) }
    }
    #[doc = "Device VBUS pulsing time register"]
    #[inline(always)]
    pub const fn dvbuspulse(self) -> Reg<regs::Dvbuspulse, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x082cusize) as _) }
    }
    #[doc = "Device IN endpoint FIFO empty interrupt mask register"]
    #[inline(always)]
    pub const fn diepempmsk(self) -> Reg<regs::Diepempmsk, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0834usize) as _) }
    }
    #[doc = "Device IN endpoint control register"]
    #[inline(always)]
    pub const fn diepctl(self, n: usize) -> Reg<regs::Diepctl, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0900usize + n * 32usize) as _) }
    }
    #[doc = "Device IN endpoint interrupt register"]
    #[inline(always)]
    pub const fn diepint(self, n: usize) -> Reg<regs::Diepint, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0908usize + n * 32usize) as _) }
    }
    #[doc = "Device IN endpoint transfer size register"]
    #[inline(always)]
    pub const fn dieptsiz(self, n: usize) -> Reg<regs::Dieptsiz, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0910usize + n * 32usize) as _) }
    }
    #[doc = "Device IN endpoint transmit FIFO status register"]
    #[inline(always)]
    pub const fn dtxfsts(self, n: usize) -> Reg<regs::Dtxfsts, R> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0918usize + n * 32usize) as _) }
    }
    #[doc = "Device OUT endpoint control register"]
    #[inline(always)]
    pub const fn doepctl(self, n: usize) -> Reg<regs::Doepctl, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0b00usize + n * 32usize) as _) }
    }
    #[doc = "Device OUT endpoint interrupt register"]
    #[inline(always)]
    pub const fn doepint(self, n: usize) -> Reg<regs::Doepint, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0b08usize + n * 32usize) as _) }
    }
    #[doc = "Device OUT endpoint transfer size register"]
    #[inline(always)]
    pub const fn doeptsiz(self, n: usize) -> Reg<regs::Doeptsiz, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x0b10usize + n * 32usize) as _) }
    }
    #[doc = "Power and clock gating control register"]
    #[inline(always)]
    pub const fn pcgcctl(self) -> Reg<regs::Pcgcctl, RW> {
        unsafe { Reg::from_ptr(self.ptr.add(0x0e00usize) as _) }
    }
    #[doc = "Device endpoint / host channel FIFO register"]
    #[inline(always)]
    pub const fn fifo(self, n: usize) -> Reg<regs::Fifo, RW> {
        assert!(n < 16usize);
        unsafe { Reg::from_ptr(self.ptr.add(0x1000usize + n * 4096usize) as _) }
    }
}
pub mod regs {
    #[doc = "Core ID register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Cid(pub u32);
    impl Cid {
        #[doc = "Product ID field"]
        #[inline(always)]
        pub const fn product_id(&self) -> u32 {
            let val = (self.0 >> 0usize) & 0xffff_ffff;
            val as u32
        }
        #[doc = "Product ID field"]
        #[inline(always)]
        pub fn set_product_id(&mut self, val: u32) {
            self.0 = (self.0 & !(0xffff_ffff << 0usize)) | (((val as u32) & 0xffff_ffff) << 0usize);
        }
    }
    impl Default for Cid {
        #[inline(always)]
        fn default() -> Cid {
            Cid(0)
        }
    }
    #[doc = "Device all endpoints interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Daint(pub u32);
    impl Daint {
        #[doc = "IN endpoint interrupt bits"]
        #[inline(always)]
        pub const fn iepint(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "IN endpoint interrupt bits"]
        #[inline(always)]
        pub fn set_iepint(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
        #[doc = "OUT endpoint interrupt bits"]
        #[inline(always)]
        pub const fn oepint(&self) -> u16 {
            let val = (self.0 >> 16usize) & 0xffff;
            val as u16
        }
        #[doc = "OUT endpoint interrupt bits"]
        #[inline(always)]
        pub fn set_oepint(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 16usize)) | (((val as u32) & 0xffff) << 16usize);
        }
    }
    impl Default for Daint {
        #[inline(always)]
        fn default() -> Daint {
            Daint(0)
        }
    }
    #[doc = "All endpoints interrupt mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Daintmsk(pub u32);
    impl Daintmsk {
        #[doc = "IN EP interrupt mask bits"]
        #[inline(always)]
        pub const fn iepm(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "IN EP interrupt mask bits"]
        #[inline(always)]
        pub fn set_iepm(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
        #[doc = "OUT EP interrupt mask bits"]
        #[inline(always)]
        pub const fn oepm(&self) -> u16 {
            let val = (self.0 >> 16usize) & 0xffff;
            val as u16
        }
        #[doc = "OUT EP interrupt mask bits"]
        #[inline(always)]
        pub fn set_oepm(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 16usize)) | (((val as u32) & 0xffff) << 16usize);
        }
    }
    impl Default for Daintmsk {
        #[inline(always)]
        fn default() -> Daintmsk {
            Daintmsk(0)
        }
    }
    #[doc = "Device configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dcfg(pub u32);
    impl Dcfg {
        #[doc = "Device speed"]
        #[inline(always)]
        pub const fn dspd(&self) -> super::vals::Dspd {
            let val = (self.0 >> 0usize) & 0x03;
            super::vals::Dspd::from_bits(val as u8)
        }
        #[doc = "Device speed"]
        #[inline(always)]
        pub fn set_dspd(&mut self, val: super::vals::Dspd) {
            self.0 = (self.0 & !(0x03 << 0usize)) | (((val.to_bits() as u32) & 0x03) << 0usize);
        }
        #[doc = "Non-zero-length status OUT handshake"]
        #[inline(always)]
        pub const fn nzlsohsk(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "Non-zero-length status OUT handshake"]
        #[inline(always)]
        pub fn set_nzlsohsk(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "Device address"]
        #[inline(always)]
        pub const fn dad(&self) -> u8 {
            let val = (self.0 >> 4usize) & 0x7f;
            val as u8
        }
        #[doc = "Device address"]
        #[inline(always)]
        pub fn set_dad(&mut self, val: u8) {
            self.0 = (self.0 & !(0x7f << 4usize)) | (((val as u32) & 0x7f) << 4usize);
        }
        #[doc = "Periodic frame interval"]
        #[inline(always)]
        pub const fn pfivl(&self) -> super::vals::Pfivl {
            let val = (self.0 >> 11usize) & 0x03;
            super::vals::Pfivl::from_bits(val as u8)
        }
        #[doc = "Periodic frame interval"]
        #[inline(always)]
        pub fn set_pfivl(&mut self, val: super::vals::Pfivl) {
            self.0 = (self.0 & !(0x03 << 11usize)) | (((val.to_bits() as u32) & 0x03) << 11usize);
        }
        #[doc = "Transceiver delay"]
        #[inline(always)]
        pub const fn xcvrdly(&self) -> bool {
            let val = (self.0 >> 14usize) & 0x01;
            val != 0
        }
        #[doc = "Transceiver delay"]
        #[inline(always)]
        pub fn set_xcvrdly(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 14usize)) | (((val as u32) & 0x01) << 14usize);
        }
    }
    impl Default for Dcfg {
        #[inline(always)]
        fn default() -> Dcfg {
            Dcfg(0)
        }
    }
    #[doc = "Device control register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dctl(pub u32);
    impl Dctl {
        #[doc = "Remote wakeup signaling"]
        #[inline(always)]
        pub const fn rwusig(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Remote wakeup signaling"]
        #[inline(always)]
        pub fn set_rwusig(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Soft disconnect"]
        #[inline(always)]
        pub const fn sdis(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Soft disconnect"]
        #[inline(always)]
        pub fn set_sdis(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "Global IN NAK status"]
        #[inline(always)]
        pub const fn ginsts(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "Global IN NAK status"]
        #[inline(always)]
        pub fn set_ginsts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "Global OUT NAK status"]
        #[inline(always)]
        pub const fn gonsts(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "Global OUT NAK status"]
        #[inline(always)]
        pub fn set_gonsts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "Test control"]
        #[inline(always)]
        pub const fn tctl(&self) -> u8 {
            let val = (self.0 >> 4usize) & 0x07;
            val as u8
        }
        #[doc = "Test control"]
        #[inline(always)]
        pub fn set_tctl(&mut self, val: u8) {
            self.0 = (self.0 & !(0x07 << 4usize)) | (((val as u32) & 0x07) << 4usize);
        }
        #[doc = "Set global IN NAK"]
        #[inline(always)]
        pub const fn sginak(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "Set global IN NAK"]
        #[inline(always)]
        pub fn set_sginak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Clear global IN NAK"]
        #[inline(always)]
        pub const fn cginak(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Clear global IN NAK"]
        #[inline(always)]
        pub fn set_cginak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "Set global OUT NAK"]
        #[inline(always)]
        pub const fn sgonak(&self) -> bool {
            let val = (self.0 >> 9usize) & 0x01;
            val != 0
        }
        #[doc = "Set global OUT NAK"]
        #[inline(always)]
        pub fn set_sgonak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 9usize)) | (((val as u32) & 0x01) << 9usize);
        }
        #[doc = "Clear global OUT NAK"]
        #[inline(always)]
        pub const fn cgonak(&self) -> bool {
            let val = (self.0 >> 10usize) & 0x01;
            val != 0
        }
        #[doc = "Clear global OUT NAK"]
        #[inline(always)]
        pub fn set_cgonak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 10usize)) | (((val as u32) & 0x01) << 10usize);
        }
        #[doc = "Power-on programming done"]
        #[inline(always)]
        pub const fn poprgdne(&self) -> bool {
            let val = (self.0 >> 11usize) & 0x01;
            val != 0
        }
        #[doc = "Power-on programming done"]
        #[inline(always)]
        pub fn set_poprgdne(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 11usize)) | (((val as u32) & 0x01) << 11usize);
        }
    }
    impl Default for Dctl {
        #[inline(always)]
        fn default() -> Dctl {
            Dctl(0)
        }
    }
    #[doc = "Device endpoint control register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Diepctl(pub u32);
    impl Diepctl {
        #[doc = "MPSIZ"]
        #[inline(always)]
        pub const fn mpsiz(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0x07ff;
            val as u16
        }
        #[doc = "MPSIZ"]
        #[inline(always)]
        pub fn set_mpsiz(&mut self, val: u16) {
            self.0 = (self.0 & !(0x07ff << 0usize)) | (((val as u32) & 0x07ff) << 0usize);
        }
        #[doc = "USBAEP"]
        #[inline(always)]
        pub const fn usbaep(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "USBAEP"]
        #[inline(always)]
        pub fn set_usbaep(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "EONUM/DPID"]
        #[inline(always)]
        pub const fn eonum_dpid(&self) -> bool {
            let val = (self.0 >> 16usize) & 0x01;
            val != 0
        }
        #[doc = "EONUM/DPID"]
        #[inline(always)]
        pub fn set_eonum_dpid(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 16usize)) | (((val as u32) & 0x01) << 16usize);
        }
        #[doc = "NAKSTS"]
        #[inline(always)]
        pub const fn naksts(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "NAKSTS"]
        #[inline(always)]
        pub fn set_naksts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "EPTYP"]
        #[inline(always)]
        pub const fn eptyp(&self) -> super::vals::Eptyp {
            let val = (self.0 >> 18usize) & 0x03;
            super::vals::Eptyp::from_bits(val as u8)
        }
        #[doc = "EPTYP"]
        #[inline(always)]
        pub fn set_eptyp(&mut self, val: super::vals::Eptyp) {
            self.0 = (self.0 & !(0x03 << 18usize)) | (((val.to_bits() as u32) & 0x03) << 18usize);
        }
        #[doc = "SNPM"]
        #[inline(always)]
        pub const fn snpm(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "SNPM"]
        #[inline(always)]
        pub fn set_snpm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "STALL"]
        #[inline(always)]
        pub const fn stall(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "STALL"]
        #[inline(always)]
        pub fn set_stall(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
        #[doc = "TXFNUM"]
        #[inline(always)]
        pub const fn txfnum(&self) -> u8 {
            let val = (self.0 >> 22usize) & 0x0f;
            val as u8
        }
        #[doc = "TXFNUM"]
        #[inline(always)]
        pub fn set_txfnum(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 22usize)) | (((val as u32) & 0x0f) << 22usize);
        }
        #[doc = "CNAK"]
        #[inline(always)]
        pub const fn cnak(&self) -> bool {
            let val = (self.0 >> 26usize) & 0x01;
            val != 0
        }
        #[doc = "CNAK"]
        #[inline(always)]
        pub fn set_cnak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 26usize)) | (((val as u32) & 0x01) << 26usize);
        }
        #[doc = "SNAK"]
        #[inline(always)]
        pub const fn snak(&self) -> bool {
            let val = (self.0 >> 27usize) & 0x01;
            val != 0
        }
        #[doc = "SNAK"]
        #[inline(always)]
        pub fn set_snak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 27usize)) | (((val as u32) & 0x01) << 27usize);
        }
        #[doc = "SD0PID/SEVNFRM"]
        #[inline(always)]
        pub const fn sd0pid_sevnfrm(&self) -> bool {
            let val = (self.0 >> 28usize) & 0x01;
            val != 0
        }
        #[doc = "SD0PID/SEVNFRM"]
        #[inline(always)]
        pub fn set_sd0pid_sevnfrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 28usize)) | (((val as u32) & 0x01) << 28usize);
        }
        #[doc = "SODDFRM/SD1PID"]
        #[inline(always)]
        pub const fn soddfrm_sd1pid(&self) -> bool {
            let val = (self.0 >> 29usize) & 0x01;
            val != 0
        }
        #[doc = "SODDFRM/SD1PID"]
        #[inline(always)]
        pub fn set_soddfrm_sd1pid(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 29usize)) | (((val as u32) & 0x01) << 29usize);
        }
        #[doc = "EPDIS"]
        #[inline(always)]
        pub const fn epdis(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "EPDIS"]
        #[inline(always)]
        pub fn set_epdis(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "EPENA"]
        #[inline(always)]
        pub const fn epena(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "EPENA"]
        #[inline(always)]
        pub fn set_epena(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Diepctl {
        #[inline(always)]
        fn default() -> Diepctl {
            Diepctl(0)
        }
    }
    #[doc = "Device IN endpoint FIFO empty interrupt mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Diepempmsk(pub u32);
    impl Diepempmsk {
        #[doc = "IN EP Tx FIFO empty interrupt mask bits"]
        #[inline(always)]
        pub const fn ineptxfem(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "IN EP Tx FIFO empty interrupt mask bits"]
        #[inline(always)]
        pub fn set_ineptxfem(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Diepempmsk {
        #[inline(always)]
        fn default() -> Diepempmsk {
            Diepempmsk(0)
        }
    }
    #[doc = "Device endpoint interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Diepint(pub u32);
    impl Diepint {
        #[doc = "XFRC"]
        #[inline(always)]
        pub const fn xfrc(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "XFRC"]
        #[inline(always)]
        pub fn set_xfrc(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "EPDISD"]
        #[inline(always)]
        pub const fn epdisd(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "EPDISD"]
        #[inline(always)]
        pub fn set_epdisd(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "TOC"]
        #[inline(always)]
        pub const fn toc(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "TOC"]
        #[inline(always)]
        pub fn set_toc(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "ITTXFE"]
        #[inline(always)]
        pub const fn ittxfe(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "ITTXFE"]
        #[inline(always)]
        pub fn set_ittxfe(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "INEPNE"]
        #[inline(always)]
        pub const fn inepne(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "INEPNE"]
        #[inline(always)]
        pub fn set_inepne(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "TXFE"]
        #[inline(always)]
        pub const fn txfe(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "TXFE"]
        #[inline(always)]
        pub fn set_txfe(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
    }
    impl Default for Diepint {
        #[inline(always)]
        fn default() -> Diepint {
            Diepint(0)
        }
    }
    #[doc = "Device IN endpoint common interrupt mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Diepmsk(pub u32);
    impl Diepmsk {
        #[doc = "Transfer completed interrupt mask"]
        #[inline(always)]
        pub const fn xfrcm(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Transfer completed interrupt mask"]
        #[inline(always)]
        pub fn set_xfrcm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Endpoint disabled interrupt mask"]
        #[inline(always)]
        pub const fn epdm(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Endpoint disabled interrupt mask"]
        #[inline(always)]
        pub fn set_epdm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "Timeout condition mask (Non-isochronous endpoints)"]
        #[inline(always)]
        pub const fn tom(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "Timeout condition mask (Non-isochronous endpoints)"]
        #[inline(always)]
        pub fn set_tom(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "IN token received when TxFIFO empty mask"]
        #[inline(always)]
        pub const fn ittxfemsk(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "IN token received when TxFIFO empty mask"]
        #[inline(always)]
        pub fn set_ittxfemsk(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "IN token received with EP mismatch mask"]
        #[inline(always)]
        pub const fn inepnmm(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "IN token received with EP mismatch mask"]
        #[inline(always)]
        pub fn set_inepnmm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "IN endpoint NAK effective mask"]
        #[inline(always)]
        pub const fn inepnem(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "IN endpoint NAK effective mask"]
        #[inline(always)]
        pub fn set_inepnem(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
    }
    impl Default for Diepmsk {
        #[inline(always)]
        fn default() -> Diepmsk {
            Diepmsk(0)
        }
    }
    #[doc = "Device endpoint transfer size register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dieptsiz(pub u32);
    impl Dieptsiz {
        #[doc = "Transfer size"]
        #[inline(always)]
        pub const fn xfrsiz(&self) -> u32 {
            let val = (self.0 >> 0usize) & 0x0007_ffff;
            val as u32
        }
        #[doc = "Transfer size"]
        #[inline(always)]
        pub fn set_xfrsiz(&mut self, val: u32) {
            self.0 = (self.0 & !(0x0007_ffff << 0usize)) | (((val as u32) & 0x0007_ffff) << 0usize);
        }
        #[doc = "Packet count"]
        #[inline(always)]
        pub const fn pktcnt(&self) -> u16 {
            let val = (self.0 >> 19usize) & 0x03ff;
            val as u16
        }
        #[doc = "Packet count"]
        #[inline(always)]
        pub fn set_pktcnt(&mut self, val: u16) {
            self.0 = (self.0 & !(0x03ff << 19usize)) | (((val as u32) & 0x03ff) << 19usize);
        }
        #[doc = "Multi count"]
        #[inline(always)]
        pub const fn mcnt(&self) -> u8 {
            let val = (self.0 >> 29usize) & 0x03;
            val as u8
        }
        #[doc = "Multi count"]
        #[inline(always)]
        pub fn set_mcnt(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 29usize)) | (((val as u32) & 0x03) << 29usize);
        }
    }
    impl Default for Dieptsiz {
        #[inline(always)]
        fn default() -> Dieptsiz {
            Dieptsiz(0)
        }
    }
    #[doc = "Device endpoint control register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Doepctl(pub u32);
    impl Doepctl {
        #[doc = "MPSIZ"]
        #[inline(always)]
        pub const fn mpsiz(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0x07ff;
            val as u16
        }
        #[doc = "MPSIZ"]
        #[inline(always)]
        pub fn set_mpsiz(&mut self, val: u16) {
            self.0 = (self.0 & !(0x07ff << 0usize)) | (((val as u32) & 0x07ff) << 0usize);
        }
        #[doc = "USBAEP"]
        #[inline(always)]
        pub const fn usbaep(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "USBAEP"]
        #[inline(always)]
        pub fn set_usbaep(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "EONUM/DPID"]
        #[inline(always)]
        pub const fn eonum_dpid(&self) -> bool {
            let val = (self.0 >> 16usize) & 0x01;
            val != 0
        }
        #[doc = "EONUM/DPID"]
        #[inline(always)]
        pub fn set_eonum_dpid(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 16usize)) | (((val as u32) & 0x01) << 16usize);
        }
        #[doc = "NAKSTS"]
        #[inline(always)]
        pub const fn naksts(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "NAKSTS"]
        #[inline(always)]
        pub fn set_naksts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "EPTYP"]
        #[inline(always)]
        pub const fn eptyp(&self) -> super::vals::Eptyp {
            let val = (self.0 >> 18usize) & 0x03;
            super::vals::Eptyp::from_bits(val as u8)
        }
        #[doc = "EPTYP"]
        #[inline(always)]
        pub fn set_eptyp(&mut self, val: super::vals::Eptyp) {
            self.0 = (self.0 & !(0x03 << 18usize)) | (((val.to_bits() as u32) & 0x03) << 18usize);
        }
        #[doc = "SNPM"]
        #[inline(always)]
        pub const fn snpm(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "SNPM"]
        #[inline(always)]
        pub fn set_snpm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "STALL"]
        #[inline(always)]
        pub const fn stall(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "STALL"]
        #[inline(always)]
        pub fn set_stall(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
        #[doc = "CNAK"]
        #[inline(always)]
        pub const fn cnak(&self) -> bool {
            let val = (self.0 >> 26usize) & 0x01;
            val != 0
        }
        #[doc = "CNAK"]
        #[inline(always)]
        pub fn set_cnak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 26usize)) | (((val as u32) & 0x01) << 26usize);
        }
        #[doc = "SNAK"]
        #[inline(always)]
        pub const fn snak(&self) -> bool {
            let val = (self.0 >> 27usize) & 0x01;
            val != 0
        }
        #[doc = "SNAK"]
        #[inline(always)]
        pub fn set_snak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 27usize)) | (((val as u32) & 0x01) << 27usize);
        }
        #[doc = "SD0PID/SEVNFRM"]
        #[inline(always)]
        pub const fn sd0pid_sevnfrm(&self) -> bool {
            let val = (self.0 >> 28usize) & 0x01;
            val != 0
        }
        #[doc = "SD0PID/SEVNFRM"]
        #[inline(always)]
        pub fn set_sd0pid_sevnfrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 28usize)) | (((val as u32) & 0x01) << 28usize);
        }
        #[doc = "SODDFRM"]
        #[inline(always)]
        pub const fn soddfrm(&self) -> bool {
            let val = (self.0 >> 29usize) & 0x01;
            val != 0
        }
        #[doc = "SODDFRM"]
        #[inline(always)]
        pub fn set_soddfrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 29usize)) | (((val as u32) & 0x01) << 29usize);
        }
        #[doc = "EPDIS"]
        #[inline(always)]
        pub const fn epdis(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "EPDIS"]
        #[inline(always)]
        pub fn set_epdis(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "EPENA"]
        #[inline(always)]
        pub const fn epena(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "EPENA"]
        #[inline(always)]
        pub fn set_epena(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Doepctl {
        #[inline(always)]
        fn default() -> Doepctl {
            Doepctl(0)
        }
    }
    #[doc = "Device endpoint interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Doepint(pub u32);
    impl Doepint {
        #[doc = "XFRC"]
        #[inline(always)]
        pub const fn xfrc(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "XFRC"]
        #[inline(always)]
        pub fn set_xfrc(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "EPDISD"]
        #[inline(always)]
        pub const fn epdisd(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "EPDISD"]
        #[inline(always)]
        pub fn set_epdisd(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "STUP"]
        #[inline(always)]
        pub const fn stup(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "STUP"]
        #[inline(always)]
        pub fn set_stup(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "OTEPDIS"]
        #[inline(always)]
        pub const fn otepdis(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "OTEPDIS"]
        #[inline(always)]
        pub fn set_otepdis(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "B2BSTUP"]
        #[inline(always)]
        pub const fn b2bstup(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "B2BSTUP"]
        #[inline(always)]
        pub fn set_b2bstup(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
    }
    impl Default for Doepint {
        #[inline(always)]
        fn default() -> Doepint {
            Doepint(0)
        }
    }
    #[doc = "Device OUT endpoint common interrupt mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Doepmsk(pub u32);
    impl Doepmsk {
        #[doc = "Transfer completed interrupt mask"]
        #[inline(always)]
        pub const fn xfrcm(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Transfer completed interrupt mask"]
        #[inline(always)]
        pub fn set_xfrcm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Endpoint disabled interrupt mask"]
        #[inline(always)]
        pub const fn epdm(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Endpoint disabled interrupt mask"]
        #[inline(always)]
        pub fn set_epdm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "SETUP phase done mask"]
        #[inline(always)]
        pub const fn stupm(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "SETUP phase done mask"]
        #[inline(always)]
        pub fn set_stupm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "OUT token received when endpoint disabled mask"]
        #[inline(always)]
        pub const fn otepdm(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "OUT token received when endpoint disabled mask"]
        #[inline(always)]
        pub fn set_otepdm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
    }
    impl Default for Doepmsk {
        #[inline(always)]
        fn default() -> Doepmsk {
            Doepmsk(0)
        }
    }
    #[doc = "Device OUT endpoint transfer size register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Doeptsiz(pub u32);
    impl Doeptsiz {
        #[doc = "Transfer size"]
        #[inline(always)]
        pub const fn xfrsiz(&self) -> u32 {
            let val = (self.0 >> 0usize) & 0x0007_ffff;
            val as u32
        }
        #[doc = "Transfer size"]
        #[inline(always)]
        pub fn set_xfrsiz(&mut self, val: u32) {
            self.0 = (self.0 & !(0x0007_ffff << 0usize)) | (((val as u32) & 0x0007_ffff) << 0usize);
        }
        #[doc = "Packet count"]
        #[inline(always)]
        pub const fn pktcnt(&self) -> u16 {
            let val = (self.0 >> 19usize) & 0x03ff;
            val as u16
        }
        #[doc = "Packet count"]
        #[inline(always)]
        pub fn set_pktcnt(&mut self, val: u16) {
            self.0 = (self.0 & !(0x03ff << 19usize)) | (((val as u32) & 0x03ff) << 19usize);
        }
        #[doc = "Received data PID/SETUP packet count"]
        #[inline(always)]
        pub const fn rxdpid_stupcnt(&self) -> u8 {
            let val = (self.0 >> 29usize) & 0x03;
            val as u8
        }
        #[doc = "Received data PID/SETUP packet count"]
        #[inline(always)]
        pub fn set_rxdpid_stupcnt(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 29usize)) | (((val as u32) & 0x03) << 29usize);
        }
    }
    impl Default for Doeptsiz {
        #[inline(always)]
        fn default() -> Doeptsiz {
            Doeptsiz(0)
        }
    }
    #[doc = "Device status register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dsts(pub u32);
    impl Dsts {
        #[doc = "Suspend status"]
        #[inline(always)]
        pub const fn suspsts(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Suspend status"]
        #[inline(always)]
        pub fn set_suspsts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Enumerated speed"]
        #[inline(always)]
        pub const fn enumspd(&self) -> super::vals::Dspd {
            let val = (self.0 >> 1usize) & 0x03;
            super::vals::Dspd::from_bits(val as u8)
        }
        #[doc = "Enumerated speed"]
        #[inline(always)]
        pub fn set_enumspd(&mut self, val: super::vals::Dspd) {
            self.0 = (self.0 & !(0x03 << 1usize)) | (((val.to_bits() as u32) & 0x03) << 1usize);
        }
        #[doc = "Erratic error"]
        #[inline(always)]
        pub const fn eerr(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "Erratic error"]
        #[inline(always)]
        pub fn set_eerr(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "Frame number of the received SOF"]
        #[inline(always)]
        pub const fn fnsof(&self) -> u16 {
            let val = (self.0 >> 8usize) & 0x3fff;
            val as u16
        }
        #[doc = "Frame number of the received SOF"]
        #[inline(always)]
        pub fn set_fnsof(&mut self, val: u16) {
            self.0 = (self.0 & !(0x3fff << 8usize)) | (((val as u32) & 0x3fff) << 8usize);
        }
    }
    impl Default for Dsts {
        #[inline(always)]
        fn default() -> Dsts {
            Dsts(0)
        }
    }
    #[doc = "Device IN endpoint transmit FIFO status register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dtxfsts(pub u32);
    impl Dtxfsts {
        #[doc = "IN endpoint TxFIFO space available"]
        #[inline(always)]
        pub const fn ineptfsav(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "IN endpoint TxFIFO space available"]
        #[inline(always)]
        pub fn set_ineptfsav(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Dtxfsts {
        #[inline(always)]
        fn default() -> Dtxfsts {
            Dtxfsts(0)
        }
    }
    #[doc = "Device VBUS discharge time register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dvbusdis(pub u32);
    impl Dvbusdis {
        #[doc = "Device VBUS discharge time"]
        #[inline(always)]
        pub const fn vbusdt(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Device VBUS discharge time"]
        #[inline(always)]
        pub fn set_vbusdt(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Dvbusdis {
        #[inline(always)]
        fn default() -> Dvbusdis {
            Dvbusdis(0)
        }
    }
    #[doc = "Device VBUS pulsing time register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Dvbuspulse(pub u32);
    impl Dvbuspulse {
        #[doc = "Device VBUS pulsing time"]
        #[inline(always)]
        pub const fn dvbusp(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0x0fff;
            val as u16
        }
        #[doc = "Device VBUS pulsing time"]
        #[inline(always)]
        pub fn set_dvbusp(&mut self, val: u16) {
            self.0 = (self.0 & !(0x0fff << 0usize)) | (((val as u32) & 0x0fff) << 0usize);
        }
    }
    impl Default for Dvbuspulse {
        #[inline(always)]
        fn default() -> Dvbuspulse {
            Dvbuspulse(0)
        }
    }
    #[doc = "FIFO register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Fifo(pub u32);
    impl Fifo {
        #[doc = "Data"]
        #[inline(always)]
        pub const fn data(&self) -> u32 {
            let val = (self.0 >> 0usize) & 0xffff_ffff;
            val as u32
        }
        #[doc = "Data"]
        #[inline(always)]
        pub fn set_data(&mut self, val: u32) {
            self.0 = (self.0 & !(0xffff_ffff << 0usize)) | (((val as u32) & 0xffff_ffff) << 0usize);
        }
    }
    impl Default for Fifo {
        #[inline(always)]
        fn default() -> Fifo {
            Fifo(0)
        }
    }
    #[doc = "FIFO size register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Fsiz(pub u32);
    impl Fsiz {
        #[doc = "RAM start address"]
        #[inline(always)]
        pub const fn sa(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "RAM start address"]
        #[inline(always)]
        pub fn set_sa(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
        #[doc = "FIFO depth"]
        #[inline(always)]
        pub const fn fd(&self) -> u16 {
            let val = (self.0 >> 16usize) & 0xffff;
            val as u16
        }
        #[doc = "FIFO depth"]
        #[inline(always)]
        pub fn set_fd(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 16usize)) | (((val as u32) & 0xffff) << 16usize);
        }
    }
    impl Default for Fsiz {
        #[inline(always)]
        fn default() -> Fsiz {
            Fsiz(0)
        }
    }
    #[doc = "AHB configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gahbcfg(pub u32);
    impl Gahbcfg {
        #[doc = "Global interrupt mask"]
        #[inline(always)]
        pub const fn gint(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Global interrupt mask"]
        #[inline(always)]
        pub fn set_gint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Burst length/type"]
        #[inline(always)]
        pub const fn hbstlen(&self) -> u8 {
            let val = (self.0 >> 1usize) & 0x0f;
            val as u8
        }
        #[doc = "Burst length/type"]
        #[inline(always)]
        pub fn set_hbstlen(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 1usize)) | (((val as u32) & 0x0f) << 1usize);
        }
        #[doc = "DMA enable"]
        #[inline(always)]
        pub const fn dmaen(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "DMA enable"]
        #[inline(always)]
        pub fn set_dmaen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "TxFIFO empty level"]
        #[inline(always)]
        pub const fn txfelvl(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "TxFIFO empty level"]
        #[inline(always)]
        pub fn set_txfelvl(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Periodic TxFIFO empty level"]
        #[inline(always)]
        pub const fn ptxfelvl(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Periodic TxFIFO empty level"]
        #[inline(always)]
        pub fn set_ptxfelvl(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
    }
    impl Default for Gahbcfg {
        #[inline(always)]
        fn default() -> Gahbcfg {
            Gahbcfg(0)
        }
    }
    #[doc = "General core configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct GccfgV1(pub u32);
    impl GccfgV1 {
        #[doc = "Power down"]
        #[inline(always)]
        pub const fn pwrdwn(&self) -> bool {
            let val = (self.0 >> 16usize) & 0x01;
            val != 0
        }
        #[doc = "Power down"]
        #[inline(always)]
        pub fn set_pwrdwn(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 16usize)) | (((val as u32) & 0x01) << 16usize);
        }
        #[doc = "Enable the VBUS \"A\" sensing device"]
        #[inline(always)]
        pub const fn vbusasen(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "Enable the VBUS \"A\" sensing device"]
        #[inline(always)]
        pub fn set_vbusasen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "Enable the VBUS \"B\" sensing device"]
        #[inline(always)]
        pub const fn vbusbsen(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "Enable the VBUS \"B\" sensing device"]
        #[inline(always)]
        pub fn set_vbusbsen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
        #[doc = "SOF output enable"]
        #[inline(always)]
        pub const fn sofouten(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "SOF output enable"]
        #[inline(always)]
        pub fn set_sofouten(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "VBUS sensing disable"]
        #[inline(always)]
        pub const fn novbussens(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "VBUS sensing disable"]
        #[inline(always)]
        pub fn set_novbussens(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
    }
    impl Default for GccfgV1 {
        #[inline(always)]
        fn default() -> GccfgV1 {
            GccfgV1(0)
        }
    }
    #[doc = "General core configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct GccfgV2(pub u32);
    impl GccfgV2 {
        #[doc = "Data contact detection (DCD) status"]
        #[inline(always)]
        pub const fn dcdet(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Data contact detection (DCD) status"]
        #[inline(always)]
        pub fn set_dcdet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Primary detection (PD) status"]
        #[inline(always)]
        pub const fn pdet(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Primary detection (PD) status"]
        #[inline(always)]
        pub fn set_pdet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "Secondary detection (SD) status"]
        #[inline(always)]
        pub const fn sdet(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "Secondary detection (SD) status"]
        #[inline(always)]
        pub fn set_sdet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "DM pull-up detection status"]
        #[inline(always)]
        pub const fn ps2det(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "DM pull-up detection status"]
        #[inline(always)]
        pub fn set_ps2det(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "Power down"]
        #[inline(always)]
        pub const fn pwrdwn(&self) -> bool {
            let val = (self.0 >> 16usize) & 0x01;
            val != 0
        }
        #[doc = "Power down"]
        #[inline(always)]
        pub fn set_pwrdwn(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 16usize)) | (((val as u32) & 0x01) << 16usize);
        }
        #[doc = "Battery charging detector (BCD) enable"]
        #[inline(always)]
        pub const fn bcden(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "Battery charging detector (BCD) enable"]
        #[inline(always)]
        pub fn set_bcden(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "Data contact detection (DCD) mode enable"]
        #[inline(always)]
        pub const fn dcden(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "Data contact detection (DCD) mode enable"]
        #[inline(always)]
        pub fn set_dcden(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "Primary detection (PD) mode enable"]
        #[inline(always)]
        pub const fn pden(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "Primary detection (PD) mode enable"]
        #[inline(always)]
        pub fn set_pden(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
        #[doc = "Secondary detection (SD) mode enable"]
        #[inline(always)]
        pub const fn sden(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "Secondary detection (SD) mode enable"]
        #[inline(always)]
        pub fn set_sden(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "USB VBUS detection enable"]
        #[inline(always)]
        pub const fn vbden(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "USB VBUS detection enable"]
        #[inline(always)]
        pub fn set_vbden(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
        #[doc = "Internal high-speed PHY enable."]
        #[inline(always)]
        pub const fn phyhsen(&self) -> bool {
            let val = (self.0 >> 23usize) & 0x01;
            val != 0
        }
        #[doc = "Internal high-speed PHY enable."]
        #[inline(always)]
        pub fn set_phyhsen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 23usize)) | (((val as u32) & 0x01) << 23usize);
        }
    }
    impl Default for GccfgV2 {
        #[inline(always)]
        fn default() -> GccfgV2 {
            GccfgV2(0)
        }
    }
    #[doc = "I2C access register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gi2cctl(pub u32);
    impl Gi2cctl {
        #[doc = "I2C Read/Write Data"]
        #[inline(always)]
        pub const fn rwdata(&self) -> u8 {
            let val = (self.0 >> 0usize) & 0xff;
            val as u8
        }
        #[doc = "I2C Read/Write Data"]
        #[inline(always)]
        pub fn set_rwdata(&mut self, val: u8) {
            self.0 = (self.0 & !(0xff << 0usize)) | (((val as u32) & 0xff) << 0usize);
        }
        #[doc = "I2C Register Address"]
        #[inline(always)]
        pub const fn regaddr(&self) -> u8 {
            let val = (self.0 >> 8usize) & 0xff;
            val as u8
        }
        #[doc = "I2C Register Address"]
        #[inline(always)]
        pub fn set_regaddr(&mut self, val: u8) {
            self.0 = (self.0 & !(0xff << 8usize)) | (((val as u32) & 0xff) << 8usize);
        }
        #[doc = "I2C Address"]
        #[inline(always)]
        pub const fn addr(&self) -> u8 {
            let val = (self.0 >> 16usize) & 0x7f;
            val as u8
        }
        #[doc = "I2C Address"]
        #[inline(always)]
        pub fn set_addr(&mut self, val: u8) {
            self.0 = (self.0 & !(0x7f << 16usize)) | (((val as u32) & 0x7f) << 16usize);
        }
        #[doc = "I2C Enable"]
        #[inline(always)]
        pub const fn i2cen(&self) -> bool {
            let val = (self.0 >> 23usize) & 0x01;
            val != 0
        }
        #[doc = "I2C Enable"]
        #[inline(always)]
        pub fn set_i2cen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 23usize)) | (((val as u32) & 0x01) << 23usize);
        }
        #[doc = "I2C ACK"]
        #[inline(always)]
        pub const fn ack(&self) -> bool {
            let val = (self.0 >> 24usize) & 0x01;
            val != 0
        }
        #[doc = "I2C ACK"]
        #[inline(always)]
        pub fn set_ack(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 24usize)) | (((val as u32) & 0x01) << 24usize);
        }
        #[doc = "I2C Device Address"]
        #[inline(always)]
        pub const fn i2cdevadr(&self) -> u8 {
            let val = (self.0 >> 26usize) & 0x03;
            val as u8
        }
        #[doc = "I2C Device Address"]
        #[inline(always)]
        pub fn set_i2cdevadr(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 26usize)) | (((val as u32) & 0x03) << 26usize);
        }
        #[doc = "I2C DatSe0 USB mode"]
        #[inline(always)]
        pub const fn i2cdatse0(&self) -> bool {
            let val = (self.0 >> 28usize) & 0x01;
            val != 0
        }
        #[doc = "I2C DatSe0 USB mode"]
        #[inline(always)]
        pub fn set_i2cdatse0(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 28usize)) | (((val as u32) & 0x01) << 28usize);
        }
        #[doc = "Read/Write Indicator"]
        #[inline(always)]
        pub const fn rw(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "Read/Write Indicator"]
        #[inline(always)]
        pub fn set_rw(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "I2C Busy/Done"]
        #[inline(always)]
        pub const fn bsydne(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "I2C Busy/Done"]
        #[inline(always)]
        pub fn set_bsydne(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Gi2cctl {
        #[inline(always)]
        fn default() -> Gi2cctl {
            Gi2cctl(0)
        }
    }
    #[doc = "Interrupt mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gintmsk(pub u32);
    impl Gintmsk {
        #[doc = "Mode mismatch interrupt mask"]
        #[inline(always)]
        pub const fn mmism(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Mode mismatch interrupt mask"]
        #[inline(always)]
        pub fn set_mmism(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "OTG interrupt mask"]
        #[inline(always)]
        pub const fn otgint(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "OTG interrupt mask"]
        #[inline(always)]
        pub fn set_otgint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "Start of frame mask"]
        #[inline(always)]
        pub const fn sofm(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "Start of frame mask"]
        #[inline(always)]
        pub fn set_sofm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "Receive FIFO non-empty mask"]
        #[inline(always)]
        pub const fn rxflvlm(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "Receive FIFO non-empty mask"]
        #[inline(always)]
        pub fn set_rxflvlm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "Non-periodic TxFIFO empty mask"]
        #[inline(always)]
        pub const fn nptxfem(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "Non-periodic TxFIFO empty mask"]
        #[inline(always)]
        pub fn set_nptxfem(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "Global non-periodic IN NAK effective mask"]
        #[inline(always)]
        pub const fn ginakeffm(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "Global non-periodic IN NAK effective mask"]
        #[inline(always)]
        pub fn set_ginakeffm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "Global OUT NAK effective mask"]
        #[inline(always)]
        pub const fn gonakeffm(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "Global OUT NAK effective mask"]
        #[inline(always)]
        pub fn set_gonakeffm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Early suspend mask"]
        #[inline(always)]
        pub const fn esuspm(&self) -> bool {
            let val = (self.0 >> 10usize) & 0x01;
            val != 0
        }
        #[doc = "Early suspend mask"]
        #[inline(always)]
        pub fn set_esuspm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 10usize)) | (((val as u32) & 0x01) << 10usize);
        }
        #[doc = "USB suspend mask"]
        #[inline(always)]
        pub const fn usbsuspm(&self) -> bool {
            let val = (self.0 >> 11usize) & 0x01;
            val != 0
        }
        #[doc = "USB suspend mask"]
        #[inline(always)]
        pub fn set_usbsuspm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 11usize)) | (((val as u32) & 0x01) << 11usize);
        }
        #[doc = "USB reset mask"]
        #[inline(always)]
        pub const fn usbrst(&self) -> bool {
            let val = (self.0 >> 12usize) & 0x01;
            val != 0
        }
        #[doc = "USB reset mask"]
        #[inline(always)]
        pub fn set_usbrst(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 12usize)) | (((val as u32) & 0x01) << 12usize);
        }
        #[doc = "Enumeration done mask"]
        #[inline(always)]
        pub const fn enumdnem(&self) -> bool {
            let val = (self.0 >> 13usize) & 0x01;
            val != 0
        }
        #[doc = "Enumeration done mask"]
        #[inline(always)]
        pub fn set_enumdnem(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 13usize)) | (((val as u32) & 0x01) << 13usize);
        }
        #[doc = "Isochronous OUT packet dropped interrupt mask"]
        #[inline(always)]
        pub const fn isoodrpm(&self) -> bool {
            let val = (self.0 >> 14usize) & 0x01;
            val != 0
        }
        #[doc = "Isochronous OUT packet dropped interrupt mask"]
        #[inline(always)]
        pub fn set_isoodrpm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 14usize)) | (((val as u32) & 0x01) << 14usize);
        }
        #[doc = "End of periodic frame interrupt mask"]
        #[inline(always)]
        pub const fn eopfm(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "End of periodic frame interrupt mask"]
        #[inline(always)]
        pub fn set_eopfm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "Endpoint mismatch interrupt mask"]
        #[inline(always)]
        pub const fn epmism(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "Endpoint mismatch interrupt mask"]
        #[inline(always)]
        pub fn set_epmism(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "IN endpoints interrupt mask"]
        #[inline(always)]
        pub const fn iepint(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "IN endpoints interrupt mask"]
        #[inline(always)]
        pub fn set_iepint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "OUT endpoints interrupt mask"]
        #[inline(always)]
        pub const fn oepint(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "OUT endpoints interrupt mask"]
        #[inline(always)]
        pub fn set_oepint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
        #[doc = "Incomplete isochronous IN transfer mask"]
        #[inline(always)]
        pub const fn iisoixfrm(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "Incomplete isochronous IN transfer mask"]
        #[inline(always)]
        pub fn set_iisoixfrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "Incomplete periodic transfer mask (host mode) / Incomplete isochronous OUT transfer mask (device mode)"]
        #[inline(always)]
        pub const fn ipxfrm_iisooxfrm(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "Incomplete periodic transfer mask (host mode) / Incomplete isochronous OUT transfer mask (device mode)"]
        #[inline(always)]
        pub fn set_ipxfrm_iisooxfrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
        #[doc = "Data fetch suspended mask"]
        #[inline(always)]
        pub const fn fsuspm(&self) -> bool {
            let val = (self.0 >> 22usize) & 0x01;
            val != 0
        }
        #[doc = "Data fetch suspended mask"]
        #[inline(always)]
        pub fn set_fsuspm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 22usize)) | (((val as u32) & 0x01) << 22usize);
        }
        #[doc = "Reset detected interrupt mask"]
        #[inline(always)]
        pub const fn rstde(&self) -> bool {
            let val = (self.0 >> 23usize) & 0x01;
            val != 0
        }
        #[doc = "Reset detected interrupt mask"]
        #[inline(always)]
        pub fn set_rstde(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 23usize)) | (((val as u32) & 0x01) << 23usize);
        }
        #[doc = "Host port interrupt mask"]
        #[inline(always)]
        pub const fn prtim(&self) -> bool {
            let val = (self.0 >> 24usize) & 0x01;
            val != 0
        }
        #[doc = "Host port interrupt mask"]
        #[inline(always)]
        pub fn set_prtim(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 24usize)) | (((val as u32) & 0x01) << 24usize);
        }
        #[doc = "Host channels interrupt mask"]
        #[inline(always)]
        pub const fn hcim(&self) -> bool {
            let val = (self.0 >> 25usize) & 0x01;
            val != 0
        }
        #[doc = "Host channels interrupt mask"]
        #[inline(always)]
        pub fn set_hcim(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 25usize)) | (((val as u32) & 0x01) << 25usize);
        }
        #[doc = "Periodic TxFIFO empty mask"]
        #[inline(always)]
        pub const fn ptxfem(&self) -> bool {
            let val = (self.0 >> 26usize) & 0x01;
            val != 0
        }
        #[doc = "Periodic TxFIFO empty mask"]
        #[inline(always)]
        pub fn set_ptxfem(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 26usize)) | (((val as u32) & 0x01) << 26usize);
        }
        #[doc = "LPM interrupt mask"]
        #[inline(always)]
        pub const fn lpmintm(&self) -> bool {
            let val = (self.0 >> 27usize) & 0x01;
            val != 0
        }
        #[doc = "LPM interrupt mask"]
        #[inline(always)]
        pub fn set_lpmintm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 27usize)) | (((val as u32) & 0x01) << 27usize);
        }
        #[doc = "Connector ID status change mask"]
        #[inline(always)]
        pub const fn cidschgm(&self) -> bool {
            let val = (self.0 >> 28usize) & 0x01;
            val != 0
        }
        #[doc = "Connector ID status change mask"]
        #[inline(always)]
        pub fn set_cidschgm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 28usize)) | (((val as u32) & 0x01) << 28usize);
        }
        #[doc = "Disconnect detected interrupt mask"]
        #[inline(always)]
        pub const fn discint(&self) -> bool {
            let val = (self.0 >> 29usize) & 0x01;
            val != 0
        }
        #[doc = "Disconnect detected interrupt mask"]
        #[inline(always)]
        pub fn set_discint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 29usize)) | (((val as u32) & 0x01) << 29usize);
        }
        #[doc = "Session request/new session detected interrupt mask"]
        #[inline(always)]
        pub const fn srqim(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "Session request/new session detected interrupt mask"]
        #[inline(always)]
        pub fn set_srqim(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "Resume/remote wakeup detected interrupt mask"]
        #[inline(always)]
        pub const fn wuim(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "Resume/remote wakeup detected interrupt mask"]
        #[inline(always)]
        pub fn set_wuim(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Gintmsk {
        #[inline(always)]
        fn default() -> Gintmsk {
            Gintmsk(0)
        }
    }
    #[doc = "Core interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gintsts(pub u32);
    impl Gintsts {
        #[doc = "Current mode of operation"]
        #[inline(always)]
        pub const fn cmod(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Current mode of operation"]
        #[inline(always)]
        pub fn set_cmod(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Mode mismatch interrupt"]
        #[inline(always)]
        pub const fn mmis(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Mode mismatch interrupt"]
        #[inline(always)]
        pub fn set_mmis(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "OTG interrupt"]
        #[inline(always)]
        pub const fn otgint(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "OTG interrupt"]
        #[inline(always)]
        pub fn set_otgint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "Start of frame"]
        #[inline(always)]
        pub const fn sof(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "Start of frame"]
        #[inline(always)]
        pub fn set_sof(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "RxFIFO non-empty"]
        #[inline(always)]
        pub const fn rxflvl(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "RxFIFO non-empty"]
        #[inline(always)]
        pub fn set_rxflvl(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "Non-periodic TxFIFO empty"]
        #[inline(always)]
        pub const fn nptxfe(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "Non-periodic TxFIFO empty"]
        #[inline(always)]
        pub fn set_nptxfe(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "Global IN non-periodic NAK effective"]
        #[inline(always)]
        pub const fn ginakeff(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "Global IN non-periodic NAK effective"]
        #[inline(always)]
        pub fn set_ginakeff(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "Global OUT NAK effective"]
        #[inline(always)]
        pub const fn goutnakeff(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "Global OUT NAK effective"]
        #[inline(always)]
        pub fn set_goutnakeff(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Early suspend"]
        #[inline(always)]
        pub const fn esusp(&self) -> bool {
            let val = (self.0 >> 10usize) & 0x01;
            val != 0
        }
        #[doc = "Early suspend"]
        #[inline(always)]
        pub fn set_esusp(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 10usize)) | (((val as u32) & 0x01) << 10usize);
        }
        #[doc = "USB suspend"]
        #[inline(always)]
        pub const fn usbsusp(&self) -> bool {
            let val = (self.0 >> 11usize) & 0x01;
            val != 0
        }
        #[doc = "USB suspend"]
        #[inline(always)]
        pub fn set_usbsusp(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 11usize)) | (((val as u32) & 0x01) << 11usize);
        }
        #[doc = "USB reset"]
        #[inline(always)]
        pub const fn usbrst(&self) -> bool {
            let val = (self.0 >> 12usize) & 0x01;
            val != 0
        }
        #[doc = "USB reset"]
        #[inline(always)]
        pub fn set_usbrst(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 12usize)) | (((val as u32) & 0x01) << 12usize);
        }
        #[doc = "Enumeration done"]
        #[inline(always)]
        pub const fn enumdne(&self) -> bool {
            let val = (self.0 >> 13usize) & 0x01;
            val != 0
        }
        #[doc = "Enumeration done"]
        #[inline(always)]
        pub fn set_enumdne(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 13usize)) | (((val as u32) & 0x01) << 13usize);
        }
        #[doc = "Isochronous OUT packet dropped interrupt"]
        #[inline(always)]
        pub const fn isoodrp(&self) -> bool {
            let val = (self.0 >> 14usize) & 0x01;
            val != 0
        }
        #[doc = "Isochronous OUT packet dropped interrupt"]
        #[inline(always)]
        pub fn set_isoodrp(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 14usize)) | (((val as u32) & 0x01) << 14usize);
        }
        #[doc = "End of periodic frame interrupt"]
        #[inline(always)]
        pub const fn eopf(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "End of periodic frame interrupt"]
        #[inline(always)]
        pub fn set_eopf(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "IN endpoint interrupt"]
        #[inline(always)]
        pub const fn iepint(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "IN endpoint interrupt"]
        #[inline(always)]
        pub fn set_iepint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "OUT endpoint interrupt"]
        #[inline(always)]
        pub const fn oepint(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "OUT endpoint interrupt"]
        #[inline(always)]
        pub fn set_oepint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
        #[doc = "Incomplete isochronous IN transfer"]
        #[inline(always)]
        pub const fn iisoixfr(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "Incomplete isochronous IN transfer"]
        #[inline(always)]
        pub fn set_iisoixfr(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "Incomplete periodic transfer (host mode) / Incomplete isochronous OUT transfer (device mode)"]
        #[inline(always)]
        pub const fn ipxfr_incompisoout(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "Incomplete periodic transfer (host mode) / Incomplete isochronous OUT transfer (device mode)"]
        #[inline(always)]
        pub fn set_ipxfr_incompisoout(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
        #[doc = "Data fetch suspended"]
        #[inline(always)]
        pub const fn datafsusp(&self) -> bool {
            let val = (self.0 >> 22usize) & 0x01;
            val != 0
        }
        #[doc = "Data fetch suspended"]
        #[inline(always)]
        pub fn set_datafsusp(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 22usize)) | (((val as u32) & 0x01) << 22usize);
        }
        #[doc = "Host port interrupt"]
        #[inline(always)]
        pub const fn hprtint(&self) -> bool {
            let val = (self.0 >> 24usize) & 0x01;
            val != 0
        }
        #[doc = "Host port interrupt"]
        #[inline(always)]
        pub fn set_hprtint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 24usize)) | (((val as u32) & 0x01) << 24usize);
        }
        #[doc = "Host channels interrupt"]
        #[inline(always)]
        pub const fn hcint(&self) -> bool {
            let val = (self.0 >> 25usize) & 0x01;
            val != 0
        }
        #[doc = "Host channels interrupt"]
        #[inline(always)]
        pub fn set_hcint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 25usize)) | (((val as u32) & 0x01) << 25usize);
        }
        #[doc = "Periodic TxFIFO empty"]
        #[inline(always)]
        pub const fn ptxfe(&self) -> bool {
            let val = (self.0 >> 26usize) & 0x01;
            val != 0
        }
        #[doc = "Periodic TxFIFO empty"]
        #[inline(always)]
        pub fn set_ptxfe(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 26usize)) | (((val as u32) & 0x01) << 26usize);
        }
        #[doc = "Connector ID status change"]
        #[inline(always)]
        pub const fn cidschg(&self) -> bool {
            let val = (self.0 >> 28usize) & 0x01;
            val != 0
        }
        #[doc = "Connector ID status change"]
        #[inline(always)]
        pub fn set_cidschg(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 28usize)) | (((val as u32) & 0x01) << 28usize);
        }
        #[doc = "Disconnect detected interrupt"]
        #[inline(always)]
        pub const fn discint(&self) -> bool {
            let val = (self.0 >> 29usize) & 0x01;
            val != 0
        }
        #[doc = "Disconnect detected interrupt"]
        #[inline(always)]
        pub fn set_discint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 29usize)) | (((val as u32) & 0x01) << 29usize);
        }
        #[doc = "Session request/new session detected interrupt"]
        #[inline(always)]
        pub const fn srqint(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "Session request/new session detected interrupt"]
        #[inline(always)]
        pub fn set_srqint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "Resume/remote wakeup detected interrupt"]
        #[inline(always)]
        pub const fn wkupint(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "Resume/remote wakeup detected interrupt"]
        #[inline(always)]
        pub fn set_wkupint(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Gintsts {
        #[inline(always)]
        fn default() -> Gintsts {
            Gintsts(0)
        }
    }
    #[doc = "Core LPM configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Glpmcfg(pub u32);
    impl Glpmcfg {
        #[doc = "LPM support enable"]
        #[inline(always)]
        pub const fn lpmen(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "LPM support enable"]
        #[inline(always)]
        pub fn set_lpmen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "LPM token acknowledge enable"]
        #[inline(always)]
        pub const fn lpmack(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "LPM token acknowledge enable"]
        #[inline(always)]
        pub fn set_lpmack(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "Best effort service latency"]
        #[inline(always)]
        pub const fn besl(&self) -> u8 {
            let val = (self.0 >> 2usize) & 0x0f;
            val as u8
        }
        #[doc = "Best effort service latency"]
        #[inline(always)]
        pub fn set_besl(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 2usize)) | (((val as u32) & 0x0f) << 2usize);
        }
        #[doc = "bRemoteWake value"]
        #[inline(always)]
        pub const fn remwake(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "bRemoteWake value"]
        #[inline(always)]
        pub fn set_remwake(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "L1 Shallow Sleep enable"]
        #[inline(always)]
        pub const fn l1ssen(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "L1 Shallow Sleep enable"]
        #[inline(always)]
        pub fn set_l1ssen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "BESL threshold"]
        #[inline(always)]
        pub const fn beslthrs(&self) -> u8 {
            let val = (self.0 >> 8usize) & 0x0f;
            val as u8
        }
        #[doc = "BESL threshold"]
        #[inline(always)]
        pub fn set_beslthrs(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 8usize)) | (((val as u32) & 0x0f) << 8usize);
        }
        #[doc = "L1 deep sleep enable"]
        #[inline(always)]
        pub const fn l1dsen(&self) -> bool {
            let val = (self.0 >> 12usize) & 0x01;
            val != 0
        }
        #[doc = "L1 deep sleep enable"]
        #[inline(always)]
        pub fn set_l1dsen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 12usize)) | (((val as u32) & 0x01) << 12usize);
        }
        #[doc = "LPM response"]
        #[inline(always)]
        pub const fn lpmrst(&self) -> u8 {
            let val = (self.0 >> 13usize) & 0x03;
            val as u8
        }
        #[doc = "LPM response"]
        #[inline(always)]
        pub fn set_lpmrst(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 13usize)) | (((val as u32) & 0x03) << 13usize);
        }
        #[doc = "Port sleep status"]
        #[inline(always)]
        pub const fn slpsts(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "Port sleep status"]
        #[inline(always)]
        pub fn set_slpsts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "Sleep State Resume OK"]
        #[inline(always)]
        pub const fn l1rsmok(&self) -> bool {
            let val = (self.0 >> 16usize) & 0x01;
            val != 0
        }
        #[doc = "Sleep State Resume OK"]
        #[inline(always)]
        pub fn set_l1rsmok(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 16usize)) | (((val as u32) & 0x01) << 16usize);
        }
        #[doc = "LPM Channel Index"]
        #[inline(always)]
        pub const fn lpmchidx(&self) -> u8 {
            let val = (self.0 >> 17usize) & 0x0f;
            val as u8
        }
        #[doc = "LPM Channel Index"]
        #[inline(always)]
        pub fn set_lpmchidx(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 17usize)) | (((val as u32) & 0x0f) << 17usize);
        }
        #[doc = "LPM retry count"]
        #[inline(always)]
        pub const fn lpmrcnt(&self) -> u8 {
            let val = (self.0 >> 21usize) & 0x07;
            val as u8
        }
        #[doc = "LPM retry count"]
        #[inline(always)]
        pub fn set_lpmrcnt(&mut self, val: u8) {
            self.0 = (self.0 & !(0x07 << 21usize)) | (((val as u32) & 0x07) << 21usize);
        }
        #[doc = "Send LPM transaction"]
        #[inline(always)]
        pub const fn sndlpm(&self) -> bool {
            let val = (self.0 >> 24usize) & 0x01;
            val != 0
        }
        #[doc = "Send LPM transaction"]
        #[inline(always)]
        pub fn set_sndlpm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 24usize)) | (((val as u32) & 0x01) << 24usize);
        }
        #[doc = "LPM retry count status"]
        #[inline(always)]
        pub const fn lpmrcntsts(&self) -> u8 {
            let val = (self.0 >> 25usize) & 0x07;
            val as u8
        }
        #[doc = "LPM retry count status"]
        #[inline(always)]
        pub fn set_lpmrcntsts(&mut self, val: u8) {
            self.0 = (self.0 & !(0x07 << 25usize)) | (((val as u32) & 0x07) << 25usize);
        }
        #[doc = "Enable best effort service latency"]
        #[inline(always)]
        pub const fn enbesl(&self) -> bool {
            let val = (self.0 >> 28usize) & 0x01;
            val != 0
        }
        #[doc = "Enable best effort service latency"]
        #[inline(always)]
        pub fn set_enbesl(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 28usize)) | (((val as u32) & 0x01) << 28usize);
        }
    }
    impl Default for Glpmcfg {
        #[inline(always)]
        fn default() -> Glpmcfg {
            Glpmcfg(0)
        }
    }
    #[doc = "Control and status register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gotgctl(pub u32);
    impl Gotgctl {
        #[doc = "Session request success"]
        #[inline(always)]
        pub const fn srqscs(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Session request success"]
        #[inline(always)]
        pub fn set_srqscs(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Session request"]
        #[inline(always)]
        pub const fn srq(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Session request"]
        #[inline(always)]
        pub fn set_srq(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "VBUS valid override enable"]
        #[inline(always)]
        pub const fn vbvaloen(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "VBUS valid override enable"]
        #[inline(always)]
        pub fn set_vbvaloen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "VBUS valid override value"]
        #[inline(always)]
        pub const fn vbvaloval(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "VBUS valid override value"]
        #[inline(always)]
        pub fn set_vbvaloval(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "A-peripheral session valid override enable"]
        #[inline(always)]
        pub const fn avaloen(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "A-peripheral session valid override enable"]
        #[inline(always)]
        pub fn set_avaloen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "A-peripheral session valid override value"]
        #[inline(always)]
        pub const fn avaloval(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "A-peripheral session valid override value"]
        #[inline(always)]
        pub fn set_avaloval(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "B-peripheral session valid override enable"]
        #[inline(always)]
        pub const fn bvaloen(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "B-peripheral session valid override enable"]
        #[inline(always)]
        pub fn set_bvaloen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "B-peripheral session valid override value"]
        #[inline(always)]
        pub const fn bvaloval(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "B-peripheral session valid override value"]
        #[inline(always)]
        pub fn set_bvaloval(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Host negotiation success"]
        #[inline(always)]
        pub const fn hngscs(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Host negotiation success"]
        #[inline(always)]
        pub fn set_hngscs(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "HNP request"]
        #[inline(always)]
        pub const fn hnprq(&self) -> bool {
            let val = (self.0 >> 9usize) & 0x01;
            val != 0
        }
        #[doc = "HNP request"]
        #[inline(always)]
        pub fn set_hnprq(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 9usize)) | (((val as u32) & 0x01) << 9usize);
        }
        #[doc = "Host set HNP enable"]
        #[inline(always)]
        pub const fn hshnpen(&self) -> bool {
            let val = (self.0 >> 10usize) & 0x01;
            val != 0
        }
        #[doc = "Host set HNP enable"]
        #[inline(always)]
        pub fn set_hshnpen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 10usize)) | (((val as u32) & 0x01) << 10usize);
        }
        #[doc = "Device HNP enabled"]
        #[inline(always)]
        pub const fn dhnpen(&self) -> bool {
            let val = (self.0 >> 11usize) & 0x01;
            val != 0
        }
        #[doc = "Device HNP enabled"]
        #[inline(always)]
        pub fn set_dhnpen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 11usize)) | (((val as u32) & 0x01) << 11usize);
        }
        #[doc = "Embedded host enable"]
        #[inline(always)]
        pub const fn ehen(&self) -> bool {
            let val = (self.0 >> 12usize) & 0x01;
            val != 0
        }
        #[doc = "Embedded host enable"]
        #[inline(always)]
        pub fn set_ehen(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 12usize)) | (((val as u32) & 0x01) << 12usize);
        }
        #[doc = "Connector ID status"]
        #[inline(always)]
        pub const fn cidsts(&self) -> bool {
            let val = (self.0 >> 16usize) & 0x01;
            val != 0
        }
        #[doc = "Connector ID status"]
        #[inline(always)]
        pub fn set_cidsts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 16usize)) | (((val as u32) & 0x01) << 16usize);
        }
        #[doc = "Long/short debounce time"]
        #[inline(always)]
        pub const fn dbct(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "Long/short debounce time"]
        #[inline(always)]
        pub fn set_dbct(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "A-session valid"]
        #[inline(always)]
        pub const fn asvld(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "A-session valid"]
        #[inline(always)]
        pub fn set_asvld(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "B-session valid"]
        #[inline(always)]
        pub const fn bsvld(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "B-session valid"]
        #[inline(always)]
        pub fn set_bsvld(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
    }
    impl Default for Gotgctl {
        #[inline(always)]
        fn default() -> Gotgctl {
            Gotgctl(0)
        }
    }
    #[doc = "Interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gotgint(pub u32);
    impl Gotgint {
        #[doc = "Session end detected"]
        #[inline(always)]
        pub const fn sedet(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "Session end detected"]
        #[inline(always)]
        pub fn set_sedet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "Session request success status change"]
        #[inline(always)]
        pub const fn srsschg(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Session request success status change"]
        #[inline(always)]
        pub fn set_srsschg(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "Host negotiation success status change"]
        #[inline(always)]
        pub const fn hnsschg(&self) -> bool {
            let val = (self.0 >> 9usize) & 0x01;
            val != 0
        }
        #[doc = "Host negotiation success status change"]
        #[inline(always)]
        pub fn set_hnsschg(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 9usize)) | (((val as u32) & 0x01) << 9usize);
        }
        #[doc = "Host negotiation detected"]
        #[inline(always)]
        pub const fn hngdet(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "Host negotiation detected"]
        #[inline(always)]
        pub fn set_hngdet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "A-device timeout change"]
        #[inline(always)]
        pub const fn adtochg(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "A-device timeout change"]
        #[inline(always)]
        pub fn set_adtochg(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "Debounce done"]
        #[inline(always)]
        pub const fn dbcdne(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "Debounce done"]
        #[inline(always)]
        pub fn set_dbcdne(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
        #[doc = "ID input pin changed"]
        #[inline(always)]
        pub const fn idchng(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "ID input pin changed"]
        #[inline(always)]
        pub fn set_idchng(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
    }
    impl Default for Gotgint {
        #[inline(always)]
        fn default() -> Gotgint {
            Gotgint(0)
        }
    }
    #[doc = "Reset register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Grstctl(pub u32);
    impl Grstctl {
        #[doc = "Core soft reset"]
        #[inline(always)]
        pub const fn csrst(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Core soft reset"]
        #[inline(always)]
        pub fn set_csrst(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "HCLK soft reset"]
        #[inline(always)]
        pub const fn hsrst(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "HCLK soft reset"]
        #[inline(always)]
        pub fn set_hsrst(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "Host frame counter reset"]
        #[inline(always)]
        pub const fn fcrst(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "Host frame counter reset"]
        #[inline(always)]
        pub fn set_fcrst(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "RxFIFO flush"]
        #[inline(always)]
        pub const fn rxfflsh(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "RxFIFO flush"]
        #[inline(always)]
        pub fn set_rxfflsh(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "TxFIFO flush"]
        #[inline(always)]
        pub const fn txfflsh(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "TxFIFO flush"]
        #[inline(always)]
        pub fn set_txfflsh(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "TxFIFO number"]
        #[inline(always)]
        pub const fn txfnum(&self) -> u8 {
            let val = (self.0 >> 6usize) & 0x1f;
            val as u8
        }
        #[doc = "TxFIFO number"]
        #[inline(always)]
        pub fn set_txfnum(&mut self, val: u8) {
            self.0 = (self.0 & !(0x1f << 6usize)) | (((val as u32) & 0x1f) << 6usize);
        }
        #[doc = "DMA request signal enabled for USB OTG HS"]
        #[inline(always)]
        pub const fn dmareq(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "DMA request signal enabled for USB OTG HS"]
        #[inline(always)]
        pub fn set_dmareq(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "AHB master idle"]
        #[inline(always)]
        pub const fn ahbidl(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "AHB master idle"]
        #[inline(always)]
        pub fn set_ahbidl(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Grstctl {
        #[inline(always)]
        fn default() -> Grstctl {
            Grstctl(0)
        }
    }
    #[doc = "Receive FIFO size register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Grxfsiz(pub u32);
    impl Grxfsiz {
        #[doc = "RxFIFO depth"]
        #[inline(always)]
        pub const fn rxfd(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "RxFIFO depth"]
        #[inline(always)]
        pub fn set_rxfd(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Grxfsiz {
        #[inline(always)]
        fn default() -> Grxfsiz {
            Grxfsiz(0)
        }
    }
    #[doc = "Status read and pop register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Grxsts(pub u32);
    impl Grxsts {
        #[doc = "Endpoint number (device mode) / Channel number (host mode)"]
        #[inline(always)]
        pub const fn epnum(&self) -> u8 {
            let val = (self.0 >> 0usize) & 0x0f;
            val as u8
        }
        #[doc = "Endpoint number (device mode) / Channel number (host mode)"]
        #[inline(always)]
        pub fn set_epnum(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u32) & 0x0f) << 0usize);
        }
        #[doc = "Byte count"]
        #[inline(always)]
        pub const fn bcnt(&self) -> u16 {
            let val = (self.0 >> 4usize) & 0x07ff;
            val as u16
        }
        #[doc = "Byte count"]
        #[inline(always)]
        pub fn set_bcnt(&mut self, val: u16) {
            self.0 = (self.0 & !(0x07ff << 4usize)) | (((val as u32) & 0x07ff) << 4usize);
        }
        #[doc = "Data PID"]
        #[inline(always)]
        pub const fn dpid(&self) -> super::vals::Dpid {
            let val = (self.0 >> 15usize) & 0x03;
            super::vals::Dpid::from_bits(val as u8)
        }
        #[doc = "Data PID"]
        #[inline(always)]
        pub fn set_dpid(&mut self, val: super::vals::Dpid) {
            self.0 = (self.0 & !(0x03 << 15usize)) | (((val.to_bits() as u32) & 0x03) << 15usize);
        }
        #[doc = "Packet status (device mode)"]
        #[inline(always)]
        pub const fn pktstsd(&self) -> super::vals::Pktstsd {
            let val = (self.0 >> 17usize) & 0x0f;
            super::vals::Pktstsd::from_bits(val as u8)
        }
        #[doc = "Packet status (device mode)"]
        #[inline(always)]
        pub fn set_pktstsd(&mut self, val: super::vals::Pktstsd) {
            self.0 = (self.0 & !(0x0f << 17usize)) | (((val.to_bits() as u32) & 0x0f) << 17usize);
        }
        #[doc = "Packet status (host mode)"]
        #[inline(always)]
        pub const fn pktstsh(&self) -> super::vals::Pktstsh {
            let val = (self.0 >> 17usize) & 0x0f;
            super::vals::Pktstsh::from_bits(val as u8)
        }
        #[doc = "Packet status (host mode)"]
        #[inline(always)]
        pub fn set_pktstsh(&mut self, val: super::vals::Pktstsh) {
            self.0 = (self.0 & !(0x0f << 17usize)) | (((val.to_bits() as u32) & 0x0f) << 17usize);
        }
        #[doc = "Frame number (device mode)"]
        #[inline(always)]
        pub const fn frmnum(&self) -> u8 {
            let val = (self.0 >> 21usize) & 0x0f;
            val as u8
        }
        #[doc = "Frame number (device mode)"]
        #[inline(always)]
        pub fn set_frmnum(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 21usize)) | (((val as u32) & 0x0f) << 21usize);
        }
    }
    impl Default for Grxsts {
        #[inline(always)]
        fn default() -> Grxsts {
            Grxsts(0)
        }
    }
    #[doc = "USB configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Gusbcfg(pub u32);
    impl Gusbcfg {
        #[doc = "FS timeout calibration"]
        #[inline(always)]
        pub const fn tocal(&self) -> u8 {
            let val = (self.0 >> 0usize) & 0x07;
            val as u8
        }
        #[doc = "FS timeout calibration"]
        #[inline(always)]
        pub fn set_tocal(&mut self, val: u8) {
            self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u32) & 0x07) << 0usize);
        }
        #[doc = "Full-speed internal serial transceiver enable"]
        #[inline(always)]
        pub const fn physel(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "Full-speed internal serial transceiver enable"]
        #[inline(always)]
        pub fn set_physel(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "SRP-capable"]
        #[inline(always)]
        pub const fn srpcap(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "SRP-capable"]
        #[inline(always)]
        pub fn set_srpcap(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "HNP-capable"]
        #[inline(always)]
        pub const fn hnpcap(&self) -> bool {
            let val = (self.0 >> 9usize) & 0x01;
            val != 0
        }
        #[doc = "HNP-capable"]
        #[inline(always)]
        pub fn set_hnpcap(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 9usize)) | (((val as u32) & 0x01) << 9usize);
        }
        #[doc = "USB turnaround time"]
        #[inline(always)]
        pub const fn trdt(&self) -> u8 {
            let val = (self.0 >> 10usize) & 0x0f;
            val as u8
        }
        #[doc = "USB turnaround time"]
        #[inline(always)]
        pub fn set_trdt(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 10usize)) | (((val as u32) & 0x0f) << 10usize);
        }
        #[doc = "PHY Low-power clock select"]
        #[inline(always)]
        pub const fn phylpcs(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "PHY Low-power clock select"]
        #[inline(always)]
        pub fn set_phylpcs(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "ULPI FS/LS select"]
        #[inline(always)]
        pub const fn ulpifsls(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "ULPI FS/LS select"]
        #[inline(always)]
        pub fn set_ulpifsls(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "ULPI Auto-resume"]
        #[inline(always)]
        pub const fn ulpiar(&self) -> bool {
            let val = (self.0 >> 18usize) & 0x01;
            val != 0
        }
        #[doc = "ULPI Auto-resume"]
        #[inline(always)]
        pub fn set_ulpiar(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 18usize)) | (((val as u32) & 0x01) << 18usize);
        }
        #[doc = "ULPI Clock SuspendM"]
        #[inline(always)]
        pub const fn ulpicsm(&self) -> bool {
            let val = (self.0 >> 19usize) & 0x01;
            val != 0
        }
        #[doc = "ULPI Clock SuspendM"]
        #[inline(always)]
        pub fn set_ulpicsm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 19usize)) | (((val as u32) & 0x01) << 19usize);
        }
        #[doc = "ULPI External VBUS Drive"]
        #[inline(always)]
        pub const fn ulpievbusd(&self) -> bool {
            let val = (self.0 >> 20usize) & 0x01;
            val != 0
        }
        #[doc = "ULPI External VBUS Drive"]
        #[inline(always)]
        pub fn set_ulpievbusd(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 20usize)) | (((val as u32) & 0x01) << 20usize);
        }
        #[doc = "ULPI external VBUS indicator"]
        #[inline(always)]
        pub const fn ulpievbusi(&self) -> bool {
            let val = (self.0 >> 21usize) & 0x01;
            val != 0
        }
        #[doc = "ULPI external VBUS indicator"]
        #[inline(always)]
        pub fn set_ulpievbusi(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 21usize)) | (((val as u32) & 0x01) << 21usize);
        }
        #[doc = "TermSel DLine pulsing selection"]
        #[inline(always)]
        pub const fn tsdps(&self) -> bool {
            let val = (self.0 >> 22usize) & 0x01;
            val != 0
        }
        #[doc = "TermSel DLine pulsing selection"]
        #[inline(always)]
        pub fn set_tsdps(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 22usize)) | (((val as u32) & 0x01) << 22usize);
        }
        #[doc = "Indicator complement"]
        #[inline(always)]
        pub const fn pcci(&self) -> bool {
            let val = (self.0 >> 23usize) & 0x01;
            val != 0
        }
        #[doc = "Indicator complement"]
        #[inline(always)]
        pub fn set_pcci(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 23usize)) | (((val as u32) & 0x01) << 23usize);
        }
        #[doc = "Indicator pass through"]
        #[inline(always)]
        pub const fn ptci(&self) -> bool {
            let val = (self.0 >> 24usize) & 0x01;
            val != 0
        }
        #[doc = "Indicator pass through"]
        #[inline(always)]
        pub fn set_ptci(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 24usize)) | (((val as u32) & 0x01) << 24usize);
        }
        #[doc = "ULPI interface protect disable"]
        #[inline(always)]
        pub const fn ulpiipd(&self) -> bool {
            let val = (self.0 >> 25usize) & 0x01;
            val != 0
        }
        #[doc = "ULPI interface protect disable"]
        #[inline(always)]
        pub fn set_ulpiipd(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 25usize)) | (((val as u32) & 0x01) << 25usize);
        }
        #[doc = "Force host mode"]
        #[inline(always)]
        pub const fn fhmod(&self) -> bool {
            let val = (self.0 >> 29usize) & 0x01;
            val != 0
        }
        #[doc = "Force host mode"]
        #[inline(always)]
        pub fn set_fhmod(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 29usize)) | (((val as u32) & 0x01) << 29usize);
        }
        #[doc = "Force device mode"]
        #[inline(always)]
        pub const fn fdmod(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "Force device mode"]
        #[inline(always)]
        pub fn set_fdmod(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "Corrupt Tx packet"]
        #[inline(always)]
        pub const fn ctxpkt(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "Corrupt Tx packet"]
        #[inline(always)]
        pub fn set_ctxpkt(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Gusbcfg {
        #[inline(always)]
        fn default() -> Gusbcfg {
            Gusbcfg(0)
        }
    }
    #[doc = "Host all channels interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Haint(pub u32);
    impl Haint {
        #[doc = "Channel interrupts"]
        #[inline(always)]
        pub const fn haint(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Channel interrupts"]
        #[inline(always)]
        pub fn set_haint(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Haint {
        #[inline(always)]
        fn default() -> Haint {
            Haint(0)
        }
    }
    #[doc = "Host all channels interrupt mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Haintmsk(pub u32);
    impl Haintmsk {
        #[doc = "Channel interrupt mask"]
        #[inline(always)]
        pub const fn haintm(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Channel interrupt mask"]
        #[inline(always)]
        pub fn set_haintm(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Haintmsk {
        #[inline(always)]
        fn default() -> Haintmsk {
            Haintmsk(0)
        }
    }
    #[doc = "Host channel characteristics register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hcchar(pub u32);
    impl Hcchar {
        #[doc = "Maximum packet size"]
        #[inline(always)]
        pub const fn mpsiz(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0x07ff;
            val as u16
        }
        #[doc = "Maximum packet size"]
        #[inline(always)]
        pub fn set_mpsiz(&mut self, val: u16) {
            self.0 = (self.0 & !(0x07ff << 0usize)) | (((val as u32) & 0x07ff) << 0usize);
        }
        #[doc = "Endpoint number"]
        #[inline(always)]
        pub const fn epnum(&self) -> u8 {
            let val = (self.0 >> 11usize) & 0x0f;
            val as u8
        }
        #[doc = "Endpoint number"]
        #[inline(always)]
        pub fn set_epnum(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 11usize)) | (((val as u32) & 0x0f) << 11usize);
        }
        #[doc = "Endpoint direction"]
        #[inline(always)]
        pub const fn epdir(&self) -> bool {
            let val = (self.0 >> 15usize) & 0x01;
            val != 0
        }
        #[doc = "Endpoint direction"]
        #[inline(always)]
        pub fn set_epdir(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 15usize)) | (((val as u32) & 0x01) << 15usize);
        }
        #[doc = "Low-speed device"]
        #[inline(always)]
        pub const fn lsdev(&self) -> bool {
            let val = (self.0 >> 17usize) & 0x01;
            val != 0
        }
        #[doc = "Low-speed device"]
        #[inline(always)]
        pub fn set_lsdev(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 17usize)) | (((val as u32) & 0x01) << 17usize);
        }
        #[doc = "Endpoint type"]
        #[inline(always)]
        pub const fn eptyp(&self) -> super::vals::Eptyp {
            let val = (self.0 >> 18usize) & 0x03;
            super::vals::Eptyp::from_bits(val as u8)
        }
        #[doc = "Endpoint type"]
        #[inline(always)]
        pub fn set_eptyp(&mut self, val: super::vals::Eptyp) {
            self.0 = (self.0 & !(0x03 << 18usize)) | (((val.to_bits() as u32) & 0x03) << 18usize);
        }
        #[doc = "Multicount"]
        #[inline(always)]
        pub const fn mcnt(&self) -> u8 {
            let val = (self.0 >> 20usize) & 0x03;
            val as u8
        }
        #[doc = "Multicount"]
        #[inline(always)]
        pub fn set_mcnt(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 20usize)) | (((val as u32) & 0x03) << 20usize);
        }
        #[doc = "Device address"]
        #[inline(always)]
        pub const fn dad(&self) -> u8 {
            let val = (self.0 >> 22usize) & 0x7f;
            val as u8
        }
        #[doc = "Device address"]
        #[inline(always)]
        pub fn set_dad(&mut self, val: u8) {
            self.0 = (self.0 & !(0x7f << 22usize)) | (((val as u32) & 0x7f) << 22usize);
        }
        #[doc = "Odd frame"]
        #[inline(always)]
        pub const fn oddfrm(&self) -> bool {
            let val = (self.0 >> 29usize) & 0x01;
            val != 0
        }
        #[doc = "Odd frame"]
        #[inline(always)]
        pub fn set_oddfrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 29usize)) | (((val as u32) & 0x01) << 29usize);
        }
        #[doc = "Channel disable"]
        #[inline(always)]
        pub const fn chdis(&self) -> bool {
            let val = (self.0 >> 30usize) & 0x01;
            val != 0
        }
        #[doc = "Channel disable"]
        #[inline(always)]
        pub fn set_chdis(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 30usize)) | (((val as u32) & 0x01) << 30usize);
        }
        #[doc = "Channel enable"]
        #[inline(always)]
        pub const fn chena(&self) -> bool {
            let val = (self.0 >> 31usize) & 0x01;
            val != 0
        }
        #[doc = "Channel enable"]
        #[inline(always)]
        pub fn set_chena(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 31usize)) | (((val as u32) & 0x01) << 31usize);
        }
    }
    impl Default for Hcchar {
        #[inline(always)]
        fn default() -> Hcchar {
            Hcchar(0)
        }
    }
    #[doc = "Host configuration register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hcfg(pub u32);
    impl Hcfg {
        #[doc = "FS/LS PHY clock select"]
        #[inline(always)]
        pub const fn fslspcs(&self) -> u8 {
            let val = (self.0 >> 0usize) & 0x03;
            val as u8
        }
        #[doc = "FS/LS PHY clock select"]
        #[inline(always)]
        pub fn set_fslspcs(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 0usize)) | (((val as u32) & 0x03) << 0usize);
        }
        #[doc = "FS- and LS-only support"]
        #[inline(always)]
        pub const fn fslss(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "FS- and LS-only support"]
        #[inline(always)]
        pub fn set_fslss(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
    }
    impl Default for Hcfg {
        #[inline(always)]
        fn default() -> Hcfg {
            Hcfg(0)
        }
    }
    #[doc = "Host channel interrupt register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hcint(pub u32);
    impl Hcint {
        #[doc = "Transfer completed"]
        #[inline(always)]
        pub const fn xfrc(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Transfer completed"]
        #[inline(always)]
        pub fn set_xfrc(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Channel halted"]
        #[inline(always)]
        pub const fn chh(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Channel halted"]
        #[inline(always)]
        pub fn set_chh(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "STALL response received interrupt"]
        #[inline(always)]
        pub const fn stall(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "STALL response received interrupt"]
        #[inline(always)]
        pub fn set_stall(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "NAK response received interrupt"]
        #[inline(always)]
        pub const fn nak(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "NAK response received interrupt"]
        #[inline(always)]
        pub fn set_nak(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "ACK response received/transmitted interrupt"]
        #[inline(always)]
        pub const fn ack(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "ACK response received/transmitted interrupt"]
        #[inline(always)]
        pub fn set_ack(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "Transaction error"]
        #[inline(always)]
        pub const fn txerr(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "Transaction error"]
        #[inline(always)]
        pub fn set_txerr(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Babble error"]
        #[inline(always)]
        pub const fn bberr(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Babble error"]
        #[inline(always)]
        pub fn set_bberr(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "Frame overrun"]
        #[inline(always)]
        pub const fn frmor(&self) -> bool {
            let val = (self.0 >> 9usize) & 0x01;
            val != 0
        }
        #[doc = "Frame overrun"]
        #[inline(always)]
        pub fn set_frmor(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 9usize)) | (((val as u32) & 0x01) << 9usize);
        }
        #[doc = "Data toggle error"]
        #[inline(always)]
        pub const fn dterr(&self) -> bool {
            let val = (self.0 >> 10usize) & 0x01;
            val != 0
        }
        #[doc = "Data toggle error"]
        #[inline(always)]
        pub fn set_dterr(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 10usize)) | (((val as u32) & 0x01) << 10usize);
        }
    }
    impl Default for Hcint {
        #[inline(always)]
        fn default() -> Hcint {
            Hcint(0)
        }
    }
    #[doc = "Host channel mask register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hcintmsk(pub u32);
    impl Hcintmsk {
        #[doc = "Transfer completed mask"]
        #[inline(always)]
        pub const fn xfrcm(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Transfer completed mask"]
        #[inline(always)]
        pub fn set_xfrcm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Channel halted mask"]
        #[inline(always)]
        pub const fn chhm(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Channel halted mask"]
        #[inline(always)]
        pub fn set_chhm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "STALL response received interrupt mask"]
        #[inline(always)]
        pub const fn stallm(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "STALL response received interrupt mask"]
        #[inline(always)]
        pub fn set_stallm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "NAK response received interrupt mask"]
        #[inline(always)]
        pub const fn nakm(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "NAK response received interrupt mask"]
        #[inline(always)]
        pub fn set_nakm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "ACK response received/transmitted interrupt mask"]
        #[inline(always)]
        pub const fn ackm(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "ACK response received/transmitted interrupt mask"]
        #[inline(always)]
        pub fn set_ackm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "Response received interrupt mask"]
        #[inline(always)]
        pub const fn nyet(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "Response received interrupt mask"]
        #[inline(always)]
        pub fn set_nyet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "Transaction error mask"]
        #[inline(always)]
        pub const fn txerrm(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "Transaction error mask"]
        #[inline(always)]
        pub fn set_txerrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Babble error mask"]
        #[inline(always)]
        pub const fn bberrm(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Babble error mask"]
        #[inline(always)]
        pub fn set_bberrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "Frame overrun mask"]
        #[inline(always)]
        pub const fn frmorm(&self) -> bool {
            let val = (self.0 >> 9usize) & 0x01;
            val != 0
        }
        #[doc = "Frame overrun mask"]
        #[inline(always)]
        pub fn set_frmorm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 9usize)) | (((val as u32) & 0x01) << 9usize);
        }
        #[doc = "Data toggle error mask"]
        #[inline(always)]
        pub const fn dterrm(&self) -> bool {
            let val = (self.0 >> 10usize) & 0x01;
            val != 0
        }
        #[doc = "Data toggle error mask"]
        #[inline(always)]
        pub fn set_dterrm(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 10usize)) | (((val as u32) & 0x01) << 10usize);
        }
    }
    impl Default for Hcintmsk {
        #[inline(always)]
        fn default() -> Hcintmsk {
            Hcintmsk(0)
        }
    }
    #[doc = "Host channel transfer size register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hctsiz(pub u32);
    impl Hctsiz {
        #[doc = "Transfer size"]
        #[inline(always)]
        pub const fn xfrsiz(&self) -> u32 {
            let val = (self.0 >> 0usize) & 0x0007_ffff;
            val as u32
        }
        #[doc = "Transfer size"]
        #[inline(always)]
        pub fn set_xfrsiz(&mut self, val: u32) {
            self.0 = (self.0 & !(0x0007_ffff << 0usize)) | (((val as u32) & 0x0007_ffff) << 0usize);
        }
        #[doc = "Packet count"]
        #[inline(always)]
        pub const fn pktcnt(&self) -> u16 {
            let val = (self.0 >> 19usize) & 0x03ff;
            val as u16
        }
        #[doc = "Packet count"]
        #[inline(always)]
        pub fn set_pktcnt(&mut self, val: u16) {
            self.0 = (self.0 & !(0x03ff << 19usize)) | (((val as u32) & 0x03ff) << 19usize);
        }
        #[doc = "Data PID"]
        #[inline(always)]
        pub const fn dpid(&self) -> u8 {
            let val = (self.0 >> 29usize) & 0x03;
            val as u8
        }
        #[doc = "Data PID"]
        #[inline(always)]
        pub fn set_dpid(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 29usize)) | (((val as u32) & 0x03) << 29usize);
        }
    }
    impl Default for Hctsiz {
        #[inline(always)]
        fn default() -> Hctsiz {
            Hctsiz(0)
        }
    }
    #[doc = "Host frame interval register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hfir(pub u32);
    impl Hfir {
        #[doc = "Frame interval"]
        #[inline(always)]
        pub const fn frivl(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Frame interval"]
        #[inline(always)]
        pub fn set_frivl(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
    }
    impl Default for Hfir {
        #[inline(always)]
        fn default() -> Hfir {
            Hfir(0)
        }
    }
    #[doc = "Host frame number/frame time remaining register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hfnum(pub u32);
    impl Hfnum {
        #[doc = "Frame number"]
        #[inline(always)]
        pub const fn frnum(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Frame number"]
        #[inline(always)]
        pub fn set_frnum(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
        #[doc = "Frame time remaining"]
        #[inline(always)]
        pub const fn ftrem(&self) -> u16 {
            let val = (self.0 >> 16usize) & 0xffff;
            val as u16
        }
        #[doc = "Frame time remaining"]
        #[inline(always)]
        pub fn set_ftrem(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 16usize)) | (((val as u32) & 0xffff) << 16usize);
        }
    }
    impl Default for Hfnum {
        #[inline(always)]
        fn default() -> Hfnum {
            Hfnum(0)
        }
    }
    #[doc = "Non-periodic transmit FIFO/queue status register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hnptxsts(pub u32);
    impl Hnptxsts {
        #[doc = "Non-periodic TxFIFO space available"]
        #[inline(always)]
        pub const fn nptxfsav(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Non-periodic TxFIFO space available"]
        #[inline(always)]
        pub fn set_nptxfsav(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
        #[doc = "Non-periodic transmit request queue space available"]
        #[inline(always)]
        pub const fn nptqxsav(&self) -> u8 {
            let val = (self.0 >> 16usize) & 0xff;
            val as u8
        }
        #[doc = "Non-periodic transmit request queue space available"]
        #[inline(always)]
        pub fn set_nptqxsav(&mut self, val: u8) {
            self.0 = (self.0 & !(0xff << 16usize)) | (((val as u32) & 0xff) << 16usize);
        }
        #[doc = "Top of the non-periodic transmit request queue"]
        #[inline(always)]
        pub const fn nptxqtop(&self) -> u8 {
            let val = (self.0 >> 24usize) & 0x7f;
            val as u8
        }
        #[doc = "Top of the non-periodic transmit request queue"]
        #[inline(always)]
        pub fn set_nptxqtop(&mut self, val: u8) {
            self.0 = (self.0 & !(0x7f << 24usize)) | (((val as u32) & 0x7f) << 24usize);
        }
    }
    impl Default for Hnptxsts {
        #[inline(always)]
        fn default() -> Hnptxsts {
            Hnptxsts(0)
        }
    }
    #[doc = "Host port control and status register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hprt(pub u32);
    impl Hprt {
        #[doc = "Port connect status"]
        #[inline(always)]
        pub const fn pcsts(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Port connect status"]
        #[inline(always)]
        pub fn set_pcsts(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Port connect detected"]
        #[inline(always)]
        pub const fn pcdet(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Port connect detected"]
        #[inline(always)]
        pub fn set_pcdet(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "Port enable"]
        #[inline(always)]
        pub const fn pena(&self) -> bool {
            let val = (self.0 >> 2usize) & 0x01;
            val != 0
        }
        #[doc = "Port enable"]
        #[inline(always)]
        pub fn set_pena(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u32) & 0x01) << 2usize);
        }
        #[doc = "Port enable/disable change"]
        #[inline(always)]
        pub const fn penchng(&self) -> bool {
            let val = (self.0 >> 3usize) & 0x01;
            val != 0
        }
        #[doc = "Port enable/disable change"]
        #[inline(always)]
        pub fn set_penchng(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u32) & 0x01) << 3usize);
        }
        #[doc = "Port overcurrent active"]
        #[inline(always)]
        pub const fn poca(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "Port overcurrent active"]
        #[inline(always)]
        pub fn set_poca(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
        #[doc = "Port overcurrent change"]
        #[inline(always)]
        pub const fn pocchng(&self) -> bool {
            let val = (self.0 >> 5usize) & 0x01;
            val != 0
        }
        #[doc = "Port overcurrent change"]
        #[inline(always)]
        pub fn set_pocchng(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u32) & 0x01) << 5usize);
        }
        #[doc = "Port resume"]
        #[inline(always)]
        pub const fn pres(&self) -> bool {
            let val = (self.0 >> 6usize) & 0x01;
            val != 0
        }
        #[doc = "Port resume"]
        #[inline(always)]
        pub fn set_pres(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u32) & 0x01) << 6usize);
        }
        #[doc = "Port suspend"]
        #[inline(always)]
        pub const fn psusp(&self) -> bool {
            let val = (self.0 >> 7usize) & 0x01;
            val != 0
        }
        #[doc = "Port suspend"]
        #[inline(always)]
        pub fn set_psusp(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u32) & 0x01) << 7usize);
        }
        #[doc = "Port reset"]
        #[inline(always)]
        pub const fn prst(&self) -> bool {
            let val = (self.0 >> 8usize) & 0x01;
            val != 0
        }
        #[doc = "Port reset"]
        #[inline(always)]
        pub fn set_prst(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 8usize)) | (((val as u32) & 0x01) << 8usize);
        }
        #[doc = "Port line status"]
        #[inline(always)]
        pub const fn plsts(&self) -> u8 {
            let val = (self.0 >> 10usize) & 0x03;
            val as u8
        }
        #[doc = "Port line status"]
        #[inline(always)]
        pub fn set_plsts(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 10usize)) | (((val as u32) & 0x03) << 10usize);
        }
        #[doc = "Port power"]
        #[inline(always)]
        pub const fn ppwr(&self) -> bool {
            let val = (self.0 >> 12usize) & 0x01;
            val != 0
        }
        #[doc = "Port power"]
        #[inline(always)]
        pub fn set_ppwr(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 12usize)) | (((val as u32) & 0x01) << 12usize);
        }
        #[doc = "Port test control"]
        #[inline(always)]
        pub const fn ptctl(&self) -> u8 {
            let val = (self.0 >> 13usize) & 0x0f;
            val as u8
        }
        #[doc = "Port test control"]
        #[inline(always)]
        pub fn set_ptctl(&mut self, val: u8) {
            self.0 = (self.0 & !(0x0f << 13usize)) | (((val as u32) & 0x0f) << 13usize);
        }
        #[doc = "Port speed"]
        #[inline(always)]
        pub const fn pspd(&self) -> u8 {
            let val = (self.0 >> 17usize) & 0x03;
            val as u8
        }
        #[doc = "Port speed"]
        #[inline(always)]
        pub fn set_pspd(&mut self, val: u8) {
            self.0 = (self.0 & !(0x03 << 17usize)) | (((val as u32) & 0x03) << 17usize);
        }
    }
    impl Default for Hprt {
        #[inline(always)]
        fn default() -> Hprt {
            Hprt(0)
        }
    }
    #[doc = "Periodic transmit FIFO/queue status register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Hptxsts(pub u32);
    impl Hptxsts {
        #[doc = "Periodic transmit data FIFO space available"]
        #[inline(always)]
        pub const fn ptxfsavl(&self) -> u16 {
            let val = (self.0 >> 0usize) & 0xffff;
            val as u16
        }
        #[doc = "Periodic transmit data FIFO space available"]
        #[inline(always)]
        pub fn set_ptxfsavl(&mut self, val: u16) {
            self.0 = (self.0 & !(0xffff << 0usize)) | (((val as u32) & 0xffff) << 0usize);
        }
        #[doc = "Periodic transmit request queue space available"]
        #[inline(always)]
        pub const fn ptxqsav(&self) -> u8 {
            let val = (self.0 >> 16usize) & 0xff;
            val as u8
        }
        #[doc = "Periodic transmit request queue space available"]
        #[inline(always)]
        pub fn set_ptxqsav(&mut self, val: u8) {
            self.0 = (self.0 & !(0xff << 16usize)) | (((val as u32) & 0xff) << 16usize);
        }
        #[doc = "Top of the periodic transmit request queue"]
        #[inline(always)]
        pub const fn ptxqtop(&self) -> u8 {
            let val = (self.0 >> 24usize) & 0xff;
            val as u8
        }
        #[doc = "Top of the periodic transmit request queue"]
        #[inline(always)]
        pub fn set_ptxqtop(&mut self, val: u8) {
            self.0 = (self.0 & !(0xff << 24usize)) | (((val as u32) & 0xff) << 24usize);
        }
    }
    impl Default for Hptxsts {
        #[inline(always)]
        fn default() -> Hptxsts {
            Hptxsts(0)
        }
    }
    #[doc = "Power and clock gating control register"]
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Pcgcctl(pub u32);
    impl Pcgcctl {
        #[doc = "Stop PHY clock"]
        #[inline(always)]
        pub const fn stppclk(&self) -> bool {
            let val = (self.0 >> 0usize) & 0x01;
            val != 0
        }
        #[doc = "Stop PHY clock"]
        #[inline(always)]
        pub fn set_stppclk(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u32) & 0x01) << 0usize);
        }
        #[doc = "Gate HCLK"]
        #[inline(always)]
        pub const fn gatehclk(&self) -> bool {
            let val = (self.0 >> 1usize) & 0x01;
            val != 0
        }
        #[doc = "Gate HCLK"]
        #[inline(always)]
        pub fn set_gatehclk(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u32) & 0x01) << 1usize);
        }
        #[doc = "PHY Suspended"]
        #[inline(always)]
        pub const fn physusp(&self) -> bool {
            let val = (self.0 >> 4usize) & 0x01;
            val != 0
        }
        #[doc = "PHY Suspended"]
        #[inline(always)]
        pub fn set_physusp(&mut self, val: bool) {
            self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u32) & 0x01) << 4usize);
        }
    }
    impl Default for Pcgcctl {
        #[inline(always)]
        fn default() -> Pcgcctl {
            Pcgcctl(0)
        }
    }
}
pub mod vals {
    #[repr(u8)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[allow(non_camel_case_types)]
    pub enum Dpid {
        DATA0 = 0x0,
        DATA2 = 0x01,
        DATA1 = 0x02,
        MDATA = 0x03,
    }
    impl Dpid {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Dpid {
            unsafe { core::mem::transmute(val & 0x03) }
        }
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }
    impl From<u8> for Dpid {
        #[inline(always)]
        fn from(val: u8) -> Dpid {
            Dpid::from_bits(val)
        }
    }
    impl From<Dpid> for u8 {
        #[inline(always)]
        fn from(val: Dpid) -> u8 {
            Dpid::to_bits(val)
        }
    }
    #[repr(u8)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[allow(non_camel_case_types)]
    pub enum Dspd {
        #[doc = "High speed"]
        HIGH_SPEED = 0x0,
        #[doc = "Full speed using external ULPI PHY"]
        FULL_SPEED_EXTERNAL = 0x01,
        _RESERVED_2 = 0x02,
        #[doc = "Full speed using internal embedded PHY"]
        FULL_SPEED_INTERNAL = 0x03,
    }
    impl Dspd {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Dspd {
            unsafe { core::mem::transmute(val & 0x03) }
        }
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }
    impl From<u8> for Dspd {
        #[inline(always)]
        fn from(val: u8) -> Dspd {
            Dspd::from_bits(val)
        }
    }
    impl From<Dspd> for u8 {
        #[inline(always)]
        fn from(val: Dspd) -> u8 {
            Dspd::to_bits(val)
        }
    }
    #[repr(u8)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[allow(non_camel_case_types)]
    pub enum Eptyp {
        CONTROL = 0x0,
        ISOCHRONOUS = 0x01,
        BULK = 0x02,
        INTERRUPT = 0x03,
    }
    impl Eptyp {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Eptyp {
            unsafe { core::mem::transmute(val & 0x03) }
        }
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }
    impl From<u8> for Eptyp {
        #[inline(always)]
        fn from(val: u8) -> Eptyp {
            Eptyp::from_bits(val)
        }
    }
    impl From<Eptyp> for u8 {
        #[inline(always)]
        fn from(val: Eptyp) -> u8 {
            Eptyp::to_bits(val)
        }
    }
    #[repr(u8)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[allow(non_camel_case_types)]
    pub enum Pfivl {
        #[doc = "80% of the frame interval"]
        FRAME_INTERVAL_80 = 0x0,
        #[doc = "85% of the frame interval"]
        FRAME_INTERVAL_85 = 0x01,
        #[doc = "90% of the frame interval"]
        FRAME_INTERVAL_90 = 0x02,
        #[doc = "95% of the frame interval"]
        FRAME_INTERVAL_95 = 0x03,
    }
    impl Pfivl {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Pfivl {
            unsafe { core::mem::transmute(val & 0x03) }
        }
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }
    impl From<u8> for Pfivl {
        #[inline(always)]
        fn from(val: u8) -> Pfivl {
            Pfivl::from_bits(val)
        }
    }
    impl From<Pfivl> for u8 {
        #[inline(always)]
        fn from(val: Pfivl) -> u8 {
            Pfivl::to_bits(val)
        }
    }
    #[repr(u8)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[allow(non_camel_case_types)]
    pub enum Pktstsd {
        _RESERVED_0 = 0x0,
        #[doc = "Global OUT NAK (triggers an interrupt)"]
        OUT_NAK = 0x01,
        #[doc = "OUT data packet received"]
        OUT_DATA_RX = 0x02,
        #[doc = "OUT transfer completed (triggers an interrupt)"]
        OUT_DATA_DONE = 0x03,
        #[doc = "SETUP transaction completed (triggers an interrupt)"]
        SETUP_DATA_DONE = 0x04,
        _RESERVED_5 = 0x05,
        #[doc = "SETUP data packet received"]
        SETUP_DATA_RX = 0x06,
        _RESERVED_7 = 0x07,
        _RESERVED_8 = 0x08,
        _RESERVED_9 = 0x09,
        _RESERVED_a = 0x0a,
        _RESERVED_b = 0x0b,
        _RESERVED_c = 0x0c,
        _RESERVED_d = 0x0d,
        _RESERVED_e = 0x0e,
        _RESERVED_f = 0x0f,
    }
    impl Pktstsd {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Pktstsd {
            unsafe { core::mem::transmute(val & 0x0f) }
        }
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }
    impl From<u8> for Pktstsd {
        #[inline(always)]
        fn from(val: u8) -> Pktstsd {
            Pktstsd::from_bits(val)
        }
    }
    impl From<Pktstsd> for u8 {
        #[inline(always)]
        fn from(val: Pktstsd) -> u8 {
            Pktstsd::to_bits(val)
        }
    }
    #[repr(u8)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[allow(non_camel_case_types)]
    pub enum Pktstsh {
        _RESERVED_0 = 0x0,
        _RESERVED_1 = 0x01,
        #[doc = "IN data packet received"]
        IN_DATA_RX = 0x02,
        #[doc = "IN transfer completed (triggers an interrupt)"]
        IN_DATA_DONE = 0x03,
        _RESERVED_4 = 0x04,
        #[doc = "Data toggle error (triggers an interrupt)"]
        DATA_TOGGLE_ERR = 0x05,
        _RESERVED_6 = 0x06,
        #[doc = "Channel halted (triggers an interrupt)"]
        CHANNEL_HALTED = 0x07,
        _RESERVED_8 = 0x08,
        _RESERVED_9 = 0x09,
        _RESERVED_a = 0x0a,
        _RESERVED_b = 0x0b,
        _RESERVED_c = 0x0c,
        _RESERVED_d = 0x0d,
        _RESERVED_e = 0x0e,
        _RESERVED_f = 0x0f,
    }
    impl Pktstsh {
        #[inline(always)]
        pub const fn from_bits(val: u8) -> Pktstsh {
            unsafe { core::mem::transmute(val & 0x0f) }
        }
        #[inline(always)]
        pub const fn to_bits(self) -> u8 {
            unsafe { core::mem::transmute(self) }
        }
    }
    impl From<u8> for Pktstsh {
        #[inline(always)]
        fn from(val: u8) -> Pktstsh {
            Pktstsh::from_bits(val)
        }
    }
    impl From<Pktstsh> for u8 {
        #[inline(always)]
        fn from(val: Pktstsh) -> u8 {
            Pktstsh::to_bits(val)
        }
    }
}
