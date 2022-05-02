use async_io::Async;
use log::*;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};

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

fn ifreq_ioctl(
    lower: libc::c_int,
    ifreq: &mut ifreq,
    cmd: libc::c_ulong,
) -> io::Result<libc::c_int> {
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
    waker: Option<Waker>,
}

impl TunTapDevice {
    pub fn new(name: &str) -> io::Result<TunTapDevice> {
        Ok(Self {
            device: Async::new(TunTap::new(name)?)?,
            waker: None,
        })
    }
}

use core::task::Waker;
use embassy_net::{
    Device, DeviceCapabilities, LinkState, Packet, PacketBox, PacketBoxExt, PacketBuf,
};
use std::task::Context;

impl Device for TunTapDevice {
    fn is_transmit_ready(&mut self) -> bool {
        true
    }

    fn transmit(&mut self, pkt: PacketBuf) {
        // todo handle WouldBlock
        match self.device.get_mut().write(&pkt) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                info!("transmit WouldBlock");
            }
            Err(e) => panic!("transmit error: {:?}", e),
        }
    }

    fn receive(&mut self) -> Option<PacketBuf> {
        let mut pkt = PacketBox::new(Packet::new()).unwrap();
        loop {
            match self.device.get_mut().read(&mut pkt[..]) {
                Ok(n) => {
                    return Some(pkt.slice(0..n));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    let ready = if let Some(w) = self.waker.as_ref() {
                        let mut cx = Context::from_waker(w);
                        self.device.poll_readable(&mut cx).is_ready()
                    } else {
                        false
                    };
                    if !ready {
                        return None;
                    }
                }
                Err(e) => panic!("read error: {:?}", e),
            }
        }
    }

    fn register_waker(&mut self, w: &Waker) {
        match self.waker {
            // Optimization: If both the old and new Wakers wake the same task, we can simply
            // keep the old waker, skipping the clone. (In most executor implementations,
            // cloning a waker is somewhat expensive, comparable to cloning an Arc).
            Some(ref w2) if (w2.will_wake(w)) => {}
            _ => {
                // clone the new waker and store it
                if let Some(old_waker) = core::mem::replace(&mut self.waker, Some(w.clone())) {
                    // We had a waker registered for another task. Wake it, so the other task can
                    // reregister itself if it's still interested.
                    //
                    // If two tasks are waiting on the same thing concurrently, this will cause them
                    // to wake each other in a loop fighting over this WakerRegistration. This wastes
                    // CPU but things will still work.
                    //
                    // If the user wants to have two tasks waiting on the same thing they should use
                    // a more appropriate primitive that can store multiple wakers.
                    old_waker.wake()
                }
            }
        }
    }

    fn capabilities(&mut self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = self.device.get_ref().mtu;
        caps
    }

    fn link_state(&mut self) -> LinkState {
        LinkState::Up
    }

    fn ethernet_address(&self) -> [u8; 6] {
        [0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
    }
}
