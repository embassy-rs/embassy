//! Network interfaces.
//!
//! An interface is a [`Driver`] added to a [`Stack`], together with its
//! configuration: hardware address, IP addresses, and whatever address
//! autoconfiguration is turned on for it.

use heapless::Vec;
use xarxa::Full;
use xarxa::config::IFACE_ADDR_COUNT;
use xarxa::driver::{Capabilities, Driver, LinkState};
#[cfg(feature = "multicast")]
pub use xarxa::iface::MulticastError;
#[cfg(feature = "dhcpv4")]
pub use xarxa::iface::dhcpv4;
#[cfg(feature = "dhcpv4-server")]
pub use xarxa::iface::dhcpv4_server;
#[cfg(feature = "slaac")]
pub use xarxa::iface::slaac;
pub use xarxa::iface::{AddrOrigin, IfaceAddr, IfaceHandle, Medium};
use xarxa::wire::{HardwareAddress, IpAddress, IpCidr};

use crate::{Stack, is_config_up, is_link_up, wait_iface};

/// An interface added to a [`Stack`].
///
/// Returned by [`Stack::add_iface`] and [`Stack::iface`]. It's `Copy`,
/// so you can pass it by value instead of by reference.
#[derive(Copy, Clone)]
pub struct Iface<'d> {
    stack: Stack<'d>,
    handle: IfaceHandle,
}

impl<'d> Iface<'d> {
    pub(crate) fn new(stack: Stack<'d>, handle: IfaceHandle) -> Self {
        Self { stack, handle }
    }

    /// The stack this interface belongs to.
    pub fn stack(&self) -> Stack<'d> {
        self.stack
    }

    /// This interface's handle.
    pub fn handle(&self) -> IfaceHandle {
        self.handle
    }

    /// Borrow the interface, without waking the runner.
    fn with<R>(&self, f: impl FnOnce(&mut xarxa::iface::Iface<'_, 'd>) -> R) -> R {
        self.stack.with(|i| f(&mut i.stack.iface(self.handle)))
    }

    /// Borrow the interface, and wake the runner afterwards so it processes what
    /// changed.
    fn with_mut<R>(&self, f: impl FnOnce(&mut xarxa::iface::Iface<'_, 'd>) -> R) -> R {
        self.stack.with_mut(|i| f(&mut i.stack.iface(self.handle)))
    }

    /// The capabilities reported by the device.
    pub fn capabilities(&self) -> Capabilities {
        self.with(|i| i.capabilities())
    }

    /// Call `f` with the interface's device.
    pub fn with_driver<R>(&self, f: impl FnOnce(&mut dyn Driver) -> R) -> R {
        self.with_mut(|i| f(i.driver_mut()))
    }

    /// The link state reported by the device.
    pub fn link_state(&self) -> LinkState {
        self.with(|i| i.link_state())
    }

    /// The interface's IP-layer MTU: the device MTU minus the link-layer header,
    /// clamped to what a [`PacketBuf`](xarxa::driver::PacketBuf) can carry.
    pub fn ip_mtu(&self) -> usize {
        self.with(|i| i.ip_mtu())
    }

    /// Poll the device for the timestamp of an already-transmitted packet, sent with
    /// [`PacketMeta::request_timestamp`](xarxa::driver::PacketMeta::request_timestamp) set.
    ///
    /// Returns `None` if no timestamp is available right now, which is also all a
    /// device without transmit timestamping support ever returns. Timestamps arrive
    /// an arbitrary time after the packet was sent, possibly out of order, and
    /// possibly never, so poll this repeatedly rather than once after sending.
    #[cfg(feature = "packetmeta-timestamp")]
    pub fn poll_tx_timestamp(&self) -> Option<xarxa::driver::TxTimestamp> {
        self.with(|i| i.poll_tx_timestamp())
    }

    /// The hardware address of the interface.
    ///
    /// Initially the address the device reported when the interface was added.
    /// [`set_hardware_addr`](Self::set_hardware_addr) overrides it.
    pub fn hardware_addr(&self) -> HardwareAddress {
        self.with(|i| i.hardware_addr())
    }

    /// Set the hardware address of the interface.
    ///
    /// The stack starts using it for the frames it sends and for ingress filtering
    /// immediately. It does not announce the change on the link, so peers keep the
    /// old address in their neighbor caches until it expires.
    ///
    /// # Panics
    /// Panics if the address is not of the kind the device's medium uses.
    pub fn set_hardware_addr(&self, addr: HardwareAddress) {
        self.with_mut(|i| i.set_hardware_addr(addr))
    }

    /// The IP addresses assigned to the interface.
    pub fn ip_addrs(&self) -> Vec<IfaceAddr, IFACE_ADDR_COUNT> {
        self.with(|i| i.ip_addrs().iter().copied().collect())
    }

    /// Whether the given address is assigned to the interface.
    pub fn has_ip_addr(&self, addr: impl Into<IpAddress>) -> bool {
        self.with(|i| i.has_ip_addr(addr))
    }

    /// Assign an IP address to the interface.
    ///
    /// If the same address is already assigned, its prefix is updated and the
    /// previous CIDR returned. Otherwise the address is appended and `None` is
    /// returned.
    ///
    /// # Panics
    /// Panics if the address is not unicast.
    ///
    /// Errors:
    /// - `Full` if the interface has no room for another address.
    pub fn add_ip_addr(&self, cidr: IpCidr) -> Result<Option<IpCidr>, Full> {
        self.with_mut(|i| i.add_ip_addr(cidr))
    }

    /// Unassign an IP address from the interface, returning the CIDR it was
    /// assigned with, or `None` if it was not assigned.
    pub fn remove_ip_addr(&self, addr: impl Into<IpAddress>) -> Option<IpCidr> {
        self.with_mut(|i| i.remove_ip_addr(addr))
    }

    /// Replace the interface's entire set of IP addresses.
    ///
    /// Equivalent to removing every address and adding the given ones. The
    /// automatic IPv6 link-local address is kept.
    ///
    /// # Panics
    /// Panics if any of the addresses is not unicast.
    ///
    /// Errors:
    /// - `Full` if the addresses do not fit. The interface is left unchanged.
    pub fn set_ip_addrs(&self, addrs: impl IntoIterator<Item = IpCidr>) -> Result<(), Full> {
        self.with_mut(|i| i.set_ip_addrs(addrs))
    }

    /// A counter that goes up every time the interface's configuration changes
    /// for any reason (manual changes, DHCP, SLAAC).
    ///
    /// Compare it with a saved value to find out whether anything changed since.
    pub fn config_generation(&self) -> u32 {
        self.with(|i| i.config_generation())
    }

    /// Turn the DHCPv4 client on, with the given configuration, or off with `None`.
    ///
    /// While on, the client runs from the [`Runner`](crate::Runner). When it gets a
    /// lease the leased address and the default route via the leased router are
    /// installed on the interface, and removed again when the lease is lost or the
    /// client is turned off. Turning it on when it is already on restarts it with
    /// the new configuration.
    ///
    /// # Panics
    /// Panics if the interface is not an Ethernet interface.
    #[cfg(feature = "dhcpv4")]
    pub fn set_dhcpv4(&self, config: Option<dhcpv4::DhcpConfig>) {
        self.with_mut(|i| i.set_dhcpv4(config))
    }

    /// The current DHCPv4 lease, if the client is on and has one.
    #[cfg(feature = "dhcpv4")]
    pub fn dhcpv4_lease(&self) -> Option<dhcpv4::DhcpLease> {
        self.with(|i| i.dhcpv4_lease().cloned())
    }

    /// Restart the DHCPv4 client, dropping the current lease and starting discovery
    /// over.
    #[cfg(feature = "dhcpv4")]
    pub fn restart_dhcpv4(&self) {
        self.with_mut(|i| i.restart_dhcpv4())
    }

    /// Turn the DHCPv4 server on, with the given configuration, or off with `None`.
    ///
    /// While on, the stack answers DHCP requests arriving on this interface,
    /// handing out addresses from the configured pool.
    ///
    /// You must configure at least one IPv4 address on the interface, and the
    /// pool must be inside its subnet.
    ///
    /// Turning the server off, or on again with a new configuration, drops all
    /// leases.
    ///
    /// # Panics
    /// Panics if the interface is not an Ethernet interface, or if the pool is
    /// backwards (`pool_start` above `pool_end`).
    #[cfg(feature = "dhcpv4-server")]
    pub fn set_dhcpv4_server(&self, config: Option<dhcpv4_server::DhcpServerConfig>) {
        self.with_mut(|i| i.set_dhcpv4_server(config))
    }

    /// Call `f` with the DHCP server's lease table. It is empty if the server is off.
    ///
    /// All entries are passed, whether their lease is running or already over.
    /// Check each entry's [`state`](dhcpv4_server::DhcpServerLease::state) and
    /// [`expires_at`](dhcpv4_server::DhcpServerLease::expires_at).
    #[cfg(feature = "dhcpv4-server")]
    pub fn dhcpv4_server_leases<R>(&self, f: impl FnOnce(&[dhcpv4_server::DhcpServerLease]) -> R) -> R {
        self.with(|i| f(i.dhcpv4_server_leases()))
    }

    /// Remove the DHCP server lease of the given address, freeing it for other
    /// clients. Returns whether there was one.
    ///
    /// The client is not told: it keeps using the address until it next renews.
    #[cfg(feature = "dhcpv4-server")]
    pub fn remove_dhcpv4_server_lease(&self, address: xarxa::wire::Ipv4Address) -> bool {
        self.with_mut(|i| i.remove_dhcpv4_server_lease(address))
    }

    /// Turn SLAAC on, with the given configuration, or off with `None`.
    #[cfg(feature = "slaac")]
    pub fn set_slaac(&self, config: Option<slaac::SlaacConfig>) {
        self.with_mut(|i| i.set_slaac(config))
    }

    /// The current SLAAC state, if it is on.
    #[cfg(feature = "slaac")]
    pub fn slaac(&self) -> Option<slaac::SlaacState> {
        self.with(|i| i.slaac().copied())
    }

    /// Restart SLAAC, soliciting routers again.
    #[cfg(feature = "slaac")]
    pub fn restart_slaac(&self) {
        self.with_mut(|i| i.restart_slaac())
    }

    /// Join a multicast group.
    #[cfg(feature = "multicast")]
    pub fn join_multicast_group(&self, addr: impl Into<IpAddress>) -> Result<(), MulticastError> {
        self.with_mut(|i| i.join_multicast_group(addr))
    }

    /// Leave a multicast group.
    #[cfg(feature = "multicast")]
    pub fn leave_multicast_group(&self, addr: impl Into<IpAddress>) -> Result<(), MulticastError> {
        self.with_mut(|i| i.leave_multicast_group(addr))
    }

    /// Whether the interface has joined the given multicast group.
    #[cfg(feature = "multicast")]
    pub fn has_multicast_group(&self, addr: impl Into<IpAddress>) -> bool {
        self.with(|i| i.has_multicast_group(addr))
    }

    /// Whether the link is up.
    pub fn is_link_up(&self) -> bool {
        self.with(is_link_up)
    }

    /// Whether the interface has an address that something other than IPv6
    /// link-local autoconfiguration put there.
    ///
    /// That is: a static address was assigned, or DHCPv4 or SLAAC completed.
    pub fn is_config_up(&self) -> bool {
        self.with(|i| is_config_up(i))
    }

    /// Wait for the network device to obtain a link signal.
    pub async fn wait_link_up(&self) {
        wait_iface(self.stack, self.handle, is_link_up).await
    }

    /// Wait for the network device to lose link signal.
    pub async fn wait_link_down(&self) {
        wait_iface(self.stack, self.handle, |i| !is_link_up(i)).await
    }

    /// Wait for the interface to obtain a valid IP configuration.
    pub async fn wait_config_up(&self) {
        wait_iface(self.stack, self.handle, |i| is_config_up(i)).await
    }

    /// Wait for the interface to lose a valid IP configuration.
    pub async fn wait_config_down(&self) {
        wait_iface(self.stack, self.handle, |i| !is_config_up(i)).await
    }
}
