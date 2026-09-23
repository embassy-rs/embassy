//! Network interfaces.
//!
//! An interface is a [`Driver`] added to a [`Stack`], together with its
//! configuration.
//!
//! Interfaces can be configured manually, or automatically with [`dhcpv4`] or [`slaac`].
//!
//! An interface can also hand out addresses itself, as a [`dhcpv4_server`].

#[cfg(feature = "dhcpv4")]
pub mod dhcpv4;
#[cfg(feature = "dhcpv4-server")]
pub mod dhcpv4_server;

use embassy_time::Instant;
use heapless::Vec;
use xarxa::config::IFACE_ADDR_COUNT;
use xarxa::driver::{Capabilities, Driver, LinkState};
#[cfg(feature = "medium-ieee802154")]
use xarxa::error::Full;
#[cfg(feature = "multicast")]
pub use xarxa::iface::MulticastError;
#[cfg(feature = "slaac")]
pub use xarxa::iface::slaac;
pub use xarxa::iface::{AddIfaceError, AddrError, AddrOrigin, IfaceHandle, Medium, MediumMismatch};
use xarxa::wire::{HardwareAddress, IpAddr, IpCidr};
#[cfg(feature = "medium-ieee802154")]
use xarxa::wire::{Ieee802154Pan, SixlowpanAddressContext};

use crate::time::instant_from_xarxa;
use crate::{Stack, is_config_up, is_link_up, wait_iface};

/// An IP address assigned to an interface.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct IfaceAddr {
    /// The address and its prefix.
    pub cidr: IpCidr,
    /// Where the address came from.
    pub origin: AddrOrigin,
    /// When the address stops being preferred and becomes deprecated
    /// (RFC 4862 section 5.5.4). `None` means "forever".
    ///
    /// Only SLAAC sets this: a router advertises a preferred lifetime alongside
    /// the valid one, and shortens it to zero to signal that a prefix is on its
    /// way out while addresses formed from it still work.
    pub preferred_until: Option<Instant>,
}

impl IfaceAddr {
    fn from_xarxa(addr: xarxa::iface::IfaceAddr) -> Self {
        Self {
            cidr: addr.cidr,
            origin: addr.origin,
            preferred_until: addr.preferred_until.map(instant_from_xarxa),
        }
    }

    /// Whether the address is still preferred, i.e. not deprecated.
    ///
    /// A deprecated address keeps working for connections that already use it,
    /// but is avoided when a source address is chosen for a new one.
    pub fn is_preferred(&self, now: Instant) -> bool {
        self.preferred_until.is_none_or(|until| until > now)
    }
}

/// An interface added to a [`Stack`].
///
/// Returned by [`Stack::add_iface_borrowed`] and [`Stack::iface`]. It's `Copy`,
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
    /// clamped to what a [`PacketBuf`](crate::driver::PacketBuf) can carry.
    pub fn ip_mtu(&self) -> usize {
        self.with(|i| i.ip_mtu())
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
    /// old address in their neighbor caches until it expires. Send a gratuitous ARP
    /// or unsolicited neighbor advertisement from a raw socket if that matters.
    ///
    /// An IEEE 802.15.4 interface must use an extended address. A short address
    /// is accepted, but the stack can not put it in NDISC link-layer address
    /// options, so neighbor discovery does not work with one.
    ///
    /// # Errors
    /// - `MediumMismatch`: if the address is not of the kind the interface's
    ///   medium uses. The interface is left unchanged.
    pub fn set_hardware_addr(&self, addr: HardwareAddress) -> Result<(), MediumMismatch> {
        self.with_mut(|i| i.set_hardware_addr(addr))
    }

    /// The PAN identifier of an IEEE 802.15.4 interface, `None` for any PAN.
    #[cfg(feature = "medium-ieee802154")]
    pub fn pan_id(&self) -> Option<Ieee802154Pan> {
        self.with(|i| i.pan_id())
    }

    /// Set the PAN identifier of an IEEE 802.15.4 interface.
    ///
    /// With a PAN set, frames for another PAN are dropped, except broadcast
    /// ones. With `None`, the default, frames for every PAN are accepted.
    /// Sent frames carry the PAN, or a zero PAN with `None`.
    ///
    /// Does nothing on other media.
    #[cfg(feature = "medium-ieee802154")]
    pub fn set_pan_id(&self, pan_id: Option<Ieee802154Pan>) {
        self.with_mut(|i| i.set_pan_id(pan_id))
    }

    /// The 6LoWPAN address contexts, by context identifier, passed to `f`.
    #[cfg(feature = "medium-ieee802154")]
    pub fn sixlowpan_address_context<R>(&self, f: impl FnOnce(&[SixlowpanAddressContext]) -> R) -> R {
        self.with(|i| f(i.sixlowpan_address_context()))
    }

    /// Replace the 6LoWPAN address contexts.
    ///
    /// Received packets whose addresses are compressed against a context
    /// identifier are resolved with the context at that index. Sent packets
    /// never use contexts.
    ///
    /// # Errors
    /// - `Full`: if the contexts do not fit. Only possible without the `alloc`
    ///   feature, where the limit is
    ///   [`SIXLOWPAN_ADDRESS_CONTEXT_COUNT`](crate::config::SIXLOWPAN_ADDRESS_CONTEXT_COUNT).
    ///   The interface is left unchanged.
    #[cfg(feature = "medium-ieee802154")]
    pub fn set_sixlowpan_address_context(
        &self,
        contexts: impl IntoIterator<Item = SixlowpanAddressContext>,
    ) -> Result<(), Full> {
        self.with_mut(|i| i.set_sixlowpan_address_context(contexts))
    }

    /// The IP addresses assigned to the interface, with their origin.
    pub fn ip_addrs(&self) -> Vec<IfaceAddr, IFACE_ADDR_COUNT> {
        self.with(|i| i.ip_addrs().iter().copied().map(IfaceAddr::from_xarxa).collect())
    }

    /// Check whether the given address is assigned to the interface.
    pub fn has_ip_addr(&self, addr: impl Into<IpAddr>) -> bool {
        self.with(|i| i.has_ip_addr(addr))
    }

    /// Assign an IP address to the interface.
    ///
    /// If the same address is already assigned, its prefix is updated and the
    /// previous CIDR returned. Otherwise the address is appended and `None` is
    /// returned. Source address selection prefers the first address matching the
    /// destination's subnet, so ordering only matters between addresses of the same
    /// subnet.
    ///
    /// # Errors
    /// - `NotUnicast`: if the address is not unicast.
    /// - `Full`: if the interface has no room for another address. Only possible
    ///   without the `alloc` feature, where the limit is
    ///   [`IFACE_ADDR_COUNT`].
    pub fn add_ip_addr(&self, cidr: IpCidr) -> Result<Option<IpCidr>, AddrError> {
        self.with_mut(|i| i.add_ip_addr(cidr))
    }

    /// Unassign an IP address from the interface, returning the CIDR it was
    /// assigned with, or `None` if it was not assigned.
    pub fn remove_ip_addr(&self, addr: impl Into<IpAddr>) -> Option<IpCidr> {
        self.with_mut(|i| i.remove_ip_addr(addr))
    }

    /// Replace the interface's entire set of IP addresses.
    ///
    /// Equivalent to removing every address and adding the given ones. The
    /// automatic IPv6 link-local address is kept.
    ///
    /// On error the interface is left unchanged.
    ///
    /// # Errors
    /// - `NotUnicast`: if any of the addresses is not unicast.
    /// - `Full`: if the addresses do not fit. Only possible without the `alloc`
    ///   feature, where the limit is [`IFACE_ADDR_COUNT`].
    pub fn set_ip_addrs(&self, addrs: impl IntoIterator<Item = IpCidr>) -> Result<(), AddrError> {
        self.with_mut(|i| i.set_ip_addrs(addrs))
    }

    /// A counter that goes up every time the interface's configuration changes
    /// for any reason (manual changes, DHCP, SLAAC)
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
    /// # Errors
    /// - `MediumMismatch`: if the interface is not an Ethernet interface.
    #[cfg(feature = "dhcpv4")]
    pub fn set_dhcpv4(&self, config: Option<dhcpv4::DhcpConfig>) -> Result<(), MediumMismatch> {
        self.with_mut(|i| i.set_dhcpv4(config.map(|c| c.to_xarxa())))
    }

    /// The lease the DHCPv4 client currently holds, if any.
    #[cfg(feature = "dhcpv4")]
    pub fn dhcpv4_lease(&self) -> Option<dhcpv4::DhcpLease> {
        self.with(|i| i.dhcpv4_lease().cloned())
    }

    /// Drop the DHCPv4 lease, if any, and look for a server again.
    ///
    /// The [`Runner`](crate::Runner) does this when the link comes back up; call it
    /// directly for a driver that cannot report link state. Does nothing if the client
    /// is off.
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
    /// On error the server is left as it was.
    ///
    /// # Errors
    /// - `MediumMismatch`: if the interface is not an Ethernet interface.
    /// - `InvalidPool`: if `pool_end` is below `pool_start`.
    #[cfg(feature = "dhcpv4-server")]
    pub fn set_dhcpv4_server(
        &self,
        config: Option<dhcpv4_server::DhcpServerConfig>,
    ) -> Result<(), dhcpv4_server::DhcpServerError> {
        self.with_mut(|i| i.set_dhcpv4_server(config.map(|c| c.to_xarxa())))
    }

    /// Call `f` with an iterator over the DHCP server's lease table. It is empty
    /// if the server is off.
    ///
    /// All entries are passed, whether their lease is running or already over.
    /// Check each entry's [`state`](self::dhcpv4_server::DhcpServerLease::state)
    /// and [`expires_at`](self::dhcpv4_server::DhcpServerLease::expires_at).
    #[cfg(feature = "dhcpv4-server")]
    pub fn dhcpv4_server_leases<R>(
        &self,
        f: impl FnOnce(&mut dyn Iterator<Item = dhcpv4_server::DhcpServerLease>) -> R,
    ) -> R {
        self.with(|i| {
            let mut leases = i
                .dhcpv4_server_leases()
                .iter()
                .map(|l| dhcpv4_server::DhcpServerLease::from_xarxa(l.clone()));
            f(&mut leases)
        })
    }

    /// Remove the DHCP server lease of the given address, freeing it for other
    /// clients. Returns whether there was one.
    ///
    /// The client is not told: it keeps using the address until it next renews.
    #[cfg(feature = "dhcpv4-server")]
    pub fn remove_dhcpv4_server_lease(&self, address: xarxa::wire::Ipv4Addr) -> bool {
        self.with_mut(|i| i.remove_dhcpv4_server_lease(address))
    }

    /// Turn IPv6 stateless address autoconfiguration on, with the given
    /// configuration, or off with `None`.
    ///
    /// While on, the stack sends router solicitations from the [`Runner`](crate::Runner).
    /// Every prefix a router advertises for autoconfiguration becomes an address on the
    /// interface (the prefix plus the EUI-64 of the hardware address), and every
    /// advertising router becomes a default route. Both are removed when their
    /// lifetime runs out or when SLAAC is turned off. Turning it on when it is
    /// already on restarts it.
    ///
    /// # Errors
    /// - `MediumMismatch`: if the interface is not an Ethernet or IEEE 802.15.4
    ///   interface.
    #[cfg(feature = "slaac")]
    pub fn set_slaac(&self, config: Option<slaac::SlaacConfig>) -> Result<(), MediumMismatch> {
        self.with_mut(|i| i.set_slaac(config))
    }

    /// What SLAAC has learned from the routers on the link, or `None` if SLAAC is off.
    #[cfg(feature = "slaac")]
    pub fn slaac(&self) -> Option<slaac::SlaacState> {
        self.with(|i| i.slaac().copied())
    }

    /// Solicit routers again, keeping the addresses and routes already configured.
    ///
    /// The [`Runner`](crate::Runner) does this when the link comes back up; call it
    /// directly for a driver that cannot report link state. Does nothing if SLAAC is off.
    #[cfg(feature = "slaac")]
    pub fn restart_slaac(&self) {
        self.with_mut(|i| i.restart_slaac())
    }

    /// Join a multicast group.
    ///
    /// The stack accepts packets sent to the group right away, and reports the
    /// membership to the routers on the link from the [`Runner`](crate::Runner).
    ///
    /// # Errors
    /// - `Unaddressable`: if the address is not a multicast address.
    #[cfg(feature = "multicast")]
    pub fn join_multicast_group(&self, addr: impl Into<IpAddr>) -> Result<(), MulticastError> {
        self.with_mut(|i| i.join_multicast_group(addr))
    }

    /// Leave a multicast group.
    ///
    /// The stack stops accepting packets sent to the group right away, and
    /// reports the leave to the routers on the link from the
    /// [`Runner`](crate::Runner). Leaving a group that was not joined
    /// does nothing.
    ///
    /// # Errors
    /// - `Unaddressable`: if the address is not a multicast address.
    #[cfg(feature = "multicast")]
    pub fn leave_multicast_group(&self, addr: impl Into<IpAddr>) -> Result<(), MulticastError> {
        self.with_mut(|i| i.leave_multicast_group(addr))
    }

    /// Check whether the interface listens to the given multicast address.
    ///
    /// Besides the joined groups, this is true for the groups every host is a
    /// member of: the IPv4 all systems group, the IPv6 all nodes group, and the
    /// IPv6 solicited node group of each address assigned to the interface.
    #[cfg(feature = "multicast")]
    pub fn has_multicast_group(&self, addr: impl Into<IpAddr>) -> bool {
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

    /// Check whether the network stack has a valid IPv4 configuration.
    #[cfg(feature = "ipv4")]
    pub fn is_config_v4_up(&self) -> bool {
        self.with(|i| crate::is_config_v4_up(i))
    }

    /// Check whether the network stack has a valid non link-local IPv6 configuration.
    #[cfg(feature = "ipv6")]
    pub fn is_config_v6_up(&self) -> bool {
        self.with(|i| crate::is_config_v6_up(i))
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

    /// Wait for the interface to obtain a valid IPv4 configuration.
    #[cfg(feature = "ipv4")]
    pub async fn wait_config_v4_up(&self) {
        wait_iface(self.stack, self.handle, |i| crate::is_config_v4_up(i)).await
    }

    /// Wait for the interface to lose a valid IPv4 configuration.
    #[cfg(feature = "ipv4")]
    pub async fn wait_config_v4_down(&self) {
        wait_iface(self.stack, self.handle, |i| !crate::is_config_v4_up(i)).await
    }

    /// Wait for the interface to obtain a valid IPv6 configuration.
    ///
    /// This does not include link-local addresses.
    #[cfg(feature = "ipv6")]
    pub async fn wait_config_v6_up(&self) {
        wait_iface(self.stack, self.handle, |i| crate::is_config_v6_up(i)).await
    }

    /// Wait for the interface to lose a valid IPv6 configuration.
    ///
    /// This does not include link-local addresses.
    #[cfg(feature = "ipv6")]
    pub async fn wait_config_v6_down(&self) {
        wait_iface(self.stack, self.handle, |i| !crate::is_config_v6_up(i)).await
    }
}
