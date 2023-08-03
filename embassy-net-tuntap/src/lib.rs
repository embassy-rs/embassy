use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::task::Context;

use async_io::Async;
use embassy_net_driver::{self, Capabilities, Driver, HardwareAddress, LinkState};
use log::*;

pub const SIOCGIFMTU: libc::c_ulong = 0x8921;
pub const _SIOCGIFINDEX: libc::c_ulong = 0x8933;
pub const _ETH_P_ALL: libc::c_short = 0x0003;
pub const TUNSETIFF: libc::c_ulong = 0x400454CA;
pub const _IFF_TUN: libc::c_int = 0x0001;
pub const IFF_TAP: libc::c_int = 0x0002;
pub const IFF_NO_PI: libc::c_int = 0x1000;

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

#[derive(Debug)]
pub struct TunTap {
    fd: libc::c_int,
    mtu: usize,
}

impl AsRawFd for TunTap {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl TunTap {
    pub fn new(name: &str) -> io::Result<TunTap> {
        unsafe {
            let fd = libc::open(
                "/dev/net/tun\0".as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_NONBLOCK,
            );
            if fd == -1 {
                return Err(io::Error::last_os_error());
            }

            let mut ifreq = ifreq_for(name);
            ifreq.ifr_data = IFF_TAP | IFF_NO_PI;
            ifreq_ioctl(fd, &mut ifreq, TUNSETIFF)?;

            let socket = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP);
            if socket == -1 {
                return Err(io::Error::last_os_error());
            }

            let ip_mtu = ifreq_ioctl(socket, &mut ifreq, SIOCGIFMTU);
            libc::close(socket);
            let ip_mtu = ip_mtu? as usize;

            // SIOCGIFMTU returns the IP MTU (typically 1500 bytes.)
            // smoltcp counts the entire Ethernet packet in the MTU, so add the Ethernet header size to it.
            let mtu = ip_mtu + ETHERNET_HEADER_LEN;

            Ok(TunTap { fd, mtu })
        }
    }
}

impl Drop for TunTap {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

impl io::Read for TunTap {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if len == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(len as usize)
        }
    }
}

impl io::Write for TunTap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = unsafe { libc::write(self.fd, buf.as_ptr() as *mut libc::c_void, buf.len()) };
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

pub struct TunTapDevice {
    device: Async<TunTap>,
}

impl TunTapDevice {
    pub fn new(name: &str) -> io::Result<TunTapDevice> {
        Ok(Self {
            device: Async::new(TunTap::new(name)?)?,
        })
    }
}

impl Driver for TunTapDevice {
    type RxToken<'a> = RxToken where Self: 'a;
    type TxToken<'a> = TxToken<'a> where Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut buf = vec![0; self.device.get_ref().mtu];
        loop {
            match self.device.get_mut().read(&mut buf) {
                Ok(n) => {
                    buf.truncate(n);
                    return Some((
                        RxToken { buffer: buf },
                        TxToken {
                            device: &mut self.device,
                        },
                    ));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if !self.device.poll_readable(cx).is_ready() {
                        return None;
                    }
                }
                Err(e) => panic!("read error: {:?}", e),
            }
        }
    }

    fn transmit(&mut self, _cx: &mut Context) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            device: &mut self.device,
        })
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = self.device.get_ref().mtu;
        caps
    }

    fn link_state(&mut self, _cx: &mut Context) -> LinkState {
        LinkState::Up
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet([0x02, 0x03, 0x04, 0x05, 0x06, 0x07])
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8>,
}

impl embassy_net_driver::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }
}

#[doc(hidden)]
pub struct TxToken<'a> {
    device: &'a mut Async<TunTap>,
}

impl<'a> embassy_net_driver::TxToken for TxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);

        // todo handle WouldBlock with async
        match self.device.get_mut().write(&buffer) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => info!("transmit WouldBlock"),
            Err(e) => panic!("transmit error: {:?}", e),
        }

        result
    }
}
