#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
use std::io;
use std::io::{Read, Write};
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::prelude::BorrowedFd;
use std::task::{Context, Waker};

use async_io::Async;
use log::*;
use xarxa_driver::{Capabilities, Driver, HardwareAddress, LinkState, NotSupported, PacketBuf};

const ETHERNET_HEADER_LEN: usize = 14;

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
struct ifreq {
    ifr_name: [libc::c_char; libc::IF_NAMESIZE],
    ifr_data: libc::c_int, /* ifr_ifindex or ifr_mtu */
}

fn ifreq_for(name: &str) -> ifreq {
    let mut ifreq = ifreq {
        ifr_name: [0; libc::IF_NAMESIZE],
        ifr_data: 0,
    };
    for (i, byte) in name.as_bytes().iter().enumerate() {
        ifreq.ifr_name[i] = *byte as libc::c_char
    }
    ifreq
}

fn ifreq_ioctl(lower: libc::c_int, ifreq: &mut ifreq, cmd: libc::c_ulong) -> io::Result<libc::c_int> {
    unsafe {
        let res = libc::ioctl(lower, cmd as _, ifreq as *mut ifreq);
        if res == -1 {
            return Err(io::Error::last_os_error());
        }
    }

    Ok(ifreq.ifr_data)
}

/// A TUN/TAP device.
#[derive(Debug)]
pub struct TunTap {
    fd: OwnedFd,
    mtu: usize,
}

impl AsRawFd for TunTap {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl AsFd for TunTap {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl TunTap {
    /// Create a new TUN/TAP device.
    pub fn new(name: &str) -> io::Result<TunTap> {
        use std::os::fd::FromRawFd as _;

        let fd = unsafe {
            let fd = libc::open(c"/dev/net/tun".as_ptr(), libc::O_RDWR | libc::O_NONBLOCK);
            if fd == -1 {
                return Err(io::Error::last_os_error());
            }

            // SAFETY: the file descriptor is open.
            OwnedFd::from_raw_fd(fd)
        };

        let mtu = unsafe {
            let mut ifreq = ifreq_for(name);
            ifreq.ifr_data = libc::IFF_TAP | libc::IFF_NO_PI;
            ifreq_ioctl(fd.as_raw_fd(), &mut ifreq, libc::TUNSETIFF)?;

            let socket = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP);
            if socket == -1 {
                return Err(io::Error::last_os_error());
            }

            let ip_mtu = ifreq_ioctl(socket, &mut ifreq, libc::SIOCGIFMTU);
            libc::close(socket);
            let ip_mtu = ip_mtu? as usize;

            // SIOCGIFMTU returns the IP MTU (typically 1500 bytes.)
            // xarxa counts the entire Ethernet packet in the MTU, so add the Ethernet header size to it.
            ip_mtu + ETHERNET_HEADER_LEN
        };

        Ok(TunTap { fd, mtu })
    }
}

impl io::Read for TunTap {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = unsafe { libc::read(self.fd.as_raw_fd(), buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if len == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(len as usize)
        }
    }
}

impl io::Write for TunTap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = unsafe { libc::write(self.fd.as_raw_fd(), buf.as_ptr() as *mut libc::c_void, buf.len()) };
        if len == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(len as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A TUN/TAP device, wrapped in an async interface.
pub struct TunTapDevice {
    device: Async<TunTap>,
    hardware_address: [u8; 6],
}

impl TunTapDevice {
    /// Create a new TUN/TAP device.
    pub fn new(name: &str) -> io::Result<TunTapDevice> {
        Ok(Self {
            device: Async::new(TunTap::new(name)?)?,
            hardware_address: [0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
        })
    }

    /// Sets the MAC address of the TAP device.
    ///
    /// Note that this can not be completely random; for example, choosing a multicast address
    /// (least significant bit of the first octet is 1) would cause xarxa to crash.
    pub fn set_hardware_address(&mut self, hardware_address: [u8; 6]) {
        self.hardware_address = hardware_address;
    }
}

impl Driver for TunTapDevice {
    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = self.device.get_ref().mtu;
        caps
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet(self.hardware_address)
    }

    fn link_state(&mut self) -> LinkState {
        LinkState::Up
    }

    fn register_waker(&mut self, waker: &Waker) -> Result<(), NotSupported> {
        let mut cx = Context::from_waker(waker);
        let _ = self.device.poll_readable(&mut cx);
        Ok(())
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        let mut buf = PacketBuf::try_new()?;
        let mtu = self.device.get_ref().mtu.min(buf.capacity());
        buf.set_len(mtu);
        match unsafe { self.device.get_mut() }.read(&mut buf) {
            Ok(n) => {
                buf.set_len(n);
                Some(buf)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => None,
            Err(e) => panic!("read error: {:?}", e),
        }
    }

    fn can_transmit(&mut self) -> bool {
        true
    }

    fn transmit(&mut self, buf: PacketBuf) -> Result<(), PacketBuf> {
        // todo handle WouldBlock with async
        match unsafe { self.device.get_mut() }.write(&buf) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => info!("transmit WouldBlock"),
            Err(e) => panic!("transmit error: {:?}", e),
        }
        Ok(())
    }
}
