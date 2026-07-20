//! DHCPv4 server implementation.

use crate::Stack;
use crate::udp::{PacketMetadata, UdpSocket};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Instant;
use heapless::Vec;
use rand_core_10::Rng;
use smoltcp::wire::{
    DhcpMessageType, DhcpPacket, DhcpRepr, Ipv4Address,
    DHCP_SERVER_PORT, DHCP_CLIENT_PORT,
};

#[cfg(feature = "dhcpd")]
const DHCPD_DOMAIN_LEN: usize = 32;

/// DHCPd configuration.
#[cfg(feature = "dhcpd")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct DhcpdConfig {
    /// Our own IPv4 address communicated to the client
    pub server_ip: core::net::Ipv4Addr,
    /// Server port. This is almost always 67. Do not change unless you know what you're doing
    pub server_port: u16,
    /// Start of the range we serve IPv4 addresses from
    pub range_start: core::net::Ipv4Addr,
    /// End of the range we serve IPv4 addresses from
    pub range_end: core::net::Ipv4Addr,
    /// Subnet mask we provide to the clients
    pub subnet_mask: core::net::Ipv4Addr,
    /// Time a single lease will be valid for. Defaults to 1h
    pub lease_time: embassy_time::Duration,
    /// The IP of the default router / gateway is provided to the clients
    pub router_ip: Option<core::net::Ipv4Addr>,
    /// The IPs of the DNS servers provided to the clients
    pub dns_ip: Vec<core::net::Ipv4Addr, 3>,
    /// The domain name provided to the clients
    pub domain_name: Option<heapless::String<DHCPD_DOMAIN_LEN>>,
    /// The MTU provided to the clients. Defaults to 1514
    pub mtu: Option<u16>,
}

#[cfg(feature = "dhcpd")]
impl Default for DhcpdConfig {
    fn default() -> Self {
        Self {
            server_ip: core::net::Ipv4Addr::UNSPECIFIED,
            server_port: smoltcp::wire::DHCP_SERVER_PORT,
            range_start: core::net::Ipv4Addr::UNSPECIFIED,
            range_end: core::net::Ipv4Addr::UNSPECIFIED,
            subnet_mask: core::net::Ipv4Addr::UNSPECIFIED,
            lease_time: embassy_time::Duration::from_secs(3600),
            router_ip: None,
            dns_ip: Vec::new(),
            domain_name: None,
            mtu: Some(1514),
        }
    }
}

/// DHCPd lease
#[cfg(feature = "dhcpd")]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct DhcpdLease {
    /// Hardware / MAC address of the client holding the lease
    pub mac: Option<smoltcp::wire::EthernetAddress>,
    /// IPv4 address we assigned to the client
    pub ip: Option<core::net::Ipv4Addr>,
    /// When the lease was last assigned or confirmed
    pub last_refresh: Option<embassy_time::Instant>,
}

#[cfg(feature = "dhcpd")]
impl Default for DhcpdLease {
    fn default() -> Self {
        Self {
            mac: None,
            ip: None,
            last_refresh: None,
        }
    }
}


/// Iterate through all leases, removing the ones that expired
fn expire_leases<const DHCPD_MAX_LEASES: usize>(
    config: &'static DhcpdConfig,
    leases: &mut Vec<DhcpdLease, DHCPD_MAX_LEASES>
) -> () {
    if embassy_time::Instant::now().as_secs() < config.lease_time.as_secs() {
        // We do not yet run long enough to have any expired leases
        return
    }

    let deadline = embassy_time::Instant::now() - config.lease_time;

    debug!("Leases before retain: {}", leases.len());

    // Keep all those where the last_refresh happened after the deadline
    leases.retain(|x| (*x).last_refresh >= Some(deadline));

    debug!("Leases after retain: {}", leases.len());
}

/// Offer an IPv4 to a client. This is not yet a lease and does not modify the list of leases!
fn pick_offer<const DHCPD_MAX_LEASES: usize, T: Rng>(
    rng: &mut T,
    req: &DhcpRepr,
    config: &'static DhcpdConfig,
    leases: &Vec<DhcpdLease, DHCPD_MAX_LEASES>
) -> Option<Ipv4Address> {
    info!("DHCP DISOCVER from {:?}.\nCurrent leases ({}):", req.client_hardware_address, leases.len());

    // Check if we already issued a lease to that client
    for x in leases {
        debug!("{:?}", x);
        if x.mac == Some(req.client_hardware_address) {
            debug!("MATCH: {:?} -> {:?}", x.mac, x.ip);
            return x.ip
        }
    }

    // If there is no lease, check if we have leases available
    if leases.len() >= DHCPD_MAX_LEASES {
        debug!("Too many leases. Given: {}, Max: {}", leases.len(), DHCPD_MAX_LEASES);
        return None
    }

    // Check if the current number of leases is larger than the IP range size
    if leases.len() >= ((config.range_end.to_bits() - config.range_start.to_bits()) as usize) {
        debug!("IP range saturated. Leases given: {}, Range size: {}", leases.len(), (config.range_end.to_bits() - config.range_start.to_bits()));
        return None
    }

    // Find a free IP address in the range. We will eventually find one since
    // the corner cases are handled above. Thus, we can loop{} and return
    loop {
        // Get a random IPv4 between range_start and range_end
        let rand: u32 = rng.next_u32();
        let ip_u32: u32 = config.range_start.to_bits() + (rand % (config.range_end.to_bits() - config.range_start.to_bits() + 1));
        let ip = core::net::Ipv4Addr::from_bits(ip_u32);

        debug!("Scanning the leases for our random IP in range: {:?} - {:?} - {:?}", config.range_start, ip, config.range_end);

        // Find out if it's free or leased
        let mut found: bool = false;
        for x in leases {
            if x.ip == Some(ip) {
                found = true;
                break;
            }
        }
        if !found {
            // Our random IP was not found in the current leases => return it
            debug!("IP {:?} is not leased, taking it!", ip);

            return Some(ip)
        }
    }
}

/// If we ACK a REQUEST, we hand out the lease and need to record that
fn validate_request<const DHCPD_MAX_LEASES: usize>(
    req: &DhcpRepr,
    config: &'static DhcpdConfig,
    leases: &mut Vec<DhcpdLease, DHCPD_MAX_LEASES>
) -> Option<Ipv4Address> {
    if req.requested_ip.is_none() && (req.client_ip == core::net::Ipv4Addr::UNSPECIFIED) {
        return None
    }

    // A REQUEST packet has two use cases:
    //  - acquiring a new lease (req.client_ip == 0.0.0.0 && req.requested_ip.is_some() == true)
    //  - refreshing an existing lease (req.client_ip != 0.0.0.0 && req.requested_ip.is_none() == true)
    // Here we treat both cases "the same"
    let mut requested_ip: Option<core::net::Ipv4Addr> = Some(req.client_ip);

    if req.requested_ip.is_some() {
        requested_ip = req.requested_ip;
    }

    info!("DHCP REQUEST from {:?}: {:?}.\nCurrent leases ({}):", req.client_hardware_address, requested_ip, leases.len());

    // Check if the requested IP is in our range
    if (requested_ip.unwrap().to_bits() < config.range_start.to_bits()) || (requested_ip.unwrap().to_bits() > config.range_end.to_bits()) {
        return None
    }

    // Check if that IP is free (= not leased by someone else)
    let mut found = false;
    for x in leases.iter() {
        if (x.ip == requested_ip) && (x.mac != Some(req.client_hardware_address)) {
            found = true;
        }
    }

    // Leased by someone else => Send NAK
    if found {
        return None
    }

    // The IP is not leased OR leased by the same client

    // Check if we already issued a lease to that client
    for x in leases.iter_mut() {
        debug!("{:?}", x);
        if x.mac == Some(req.client_hardware_address) {
            debug!("MATCH: {:?} -> {:?}", x.mac, x.ip);
            x.last_refresh = Some(Instant::now());
            return x.ip
        }
    }

    // We don't have a lease for that client and need to record a new one

    // Check if we have leases available
    if leases.len() >= DHCPD_MAX_LEASES {
        debug!("Too many leases. Given: {}, Max: {}", leases.len(), DHCPD_MAX_LEASES);
        return None
    }

    let mut lease: DhcpdLease = DhcpdLease::default();

    lease.mac = Some(req.client_hardware_address);
    lease.ip = requested_ip;
    lease.last_refresh = Some(Instant::now());

    let push_result = leases.push(lease);

    if push_result.is_err() {
        return None
    }

    requested_ip
}

fn build_offer<'a>(
    req: &'a DhcpRepr<'a>,
    config: &'static DhcpdConfig,
    yiaddr: Ipv4Address
) -> DhcpRepr<'a> {
    DhcpRepr {
        message_type: DhcpMessageType::Offer,
        transaction_id: req.transaction_id,
        secs: 0,
        client_hardware_address: req.client_hardware_address,
        client_ip: Ipv4Address::UNSPECIFIED,
        your_ip: yiaddr,
        server_ip: config.server_ip,
        router: config.router_ip,
        subnet_mask: Some(config.subnet_mask),
        relay_agent_ip: Ipv4Address::UNSPECIFIED,
        broadcast: true,
        requested_ip: None,
        client_identifier: None,
        server_identifier: Some(config.server_ip),
        parameter_request_list: None,
        max_size: None,
        lease_duration: Some(config.lease_time.as_secs() as u32),
        renew_duration: None,
        rebind_duration: None,
        dns_servers: Some(config.dns_ip.clone()),
        additional_options: &[],
    }
}

fn build_ack<'a>(
    req: &'a DhcpRepr<'a>,
    config: &'static DhcpdConfig,
    yiaddr: Ipv4Address
) -> DhcpRepr<'a> {
    DhcpRepr {
        message_type: DhcpMessageType::Ack,
        transaction_id: req.transaction_id,
        secs: 0,
        client_hardware_address: req.client_hardware_address,
        client_ip: Ipv4Address::UNSPECIFIED,
        your_ip: yiaddr,
        server_ip: config.server_ip,
        router: config.router_ip,
        subnet_mask: Some(config.subnet_mask),
        relay_agent_ip: Ipv4Address::UNSPECIFIED,
        broadcast: true,
        requested_ip: None,
        client_identifier: None,
        server_identifier: Some(config.server_ip),
        parameter_request_list: None,
        max_size: None,
        lease_duration: Some(config.lease_time.as_secs() as u32),
        renew_duration: None,
        rebind_duration: None,
        dns_servers: Some(config.dns_ip.clone()),
        additional_options: &[],
    }
}

fn build_nak<'a>(
    req: &'a DhcpRepr<'a>,
    config: &'static DhcpdConfig,
    yiaddr: Ipv4Address
) -> DhcpRepr<'a> {
    DhcpRepr {
        message_type: DhcpMessageType::Nak,
        transaction_id: req.transaction_id,
        secs: 0,
        client_hardware_address: req.client_hardware_address,
        client_ip: Ipv4Address::UNSPECIFIED,
        your_ip: yiaddr,
        server_ip: config.server_ip,
        router: config.router_ip,
        subnet_mask: Some(config.subnet_mask),
        relay_agent_ip: Ipv4Address::UNSPECIFIED,
        broadcast: true,
        requested_ip: None,
        client_identifier: None,
        server_identifier: Some(config.server_ip),
        parameter_request_list: None,
        max_size: None,
        lease_duration: Some(config.lease_time.as_secs() as u32),
        renew_duration: None,
        rebind_duration: None,
        dns_servers: Some(config.dns_ip.clone()),
        additional_options: &[],
    }
}

/// Serialize the DHCP representation into a buffer
fn emit_dhcp(repr: &DhcpRepr, out: &mut [u8]) -> Option<usize> {
    let len = repr.buffer_len();
    let pkt_buf = &mut out[..len];
    let mut pkt = DhcpPacket::new_unchecked(pkt_buf);
    let repr_result = repr.emit(&mut pkt);
    if repr_result.is_err() {
        return None
    }
    Some(len)
}



/// DHCPd runner.
///
/// You must call [`Runner::run()`] in a background task for the network stack to work.
pub struct Runner<'d, const DHCPD_MAX_LEASES: usize, T: Rng> {
    stack: Stack<'d>,
    rng: T,
    config: &'static DhcpdConfig,
    leases: &'static Mutex<NoopRawMutex, Vec<DhcpdLease, DHCPD_MAX_LEASES>>,
}

impl<'d, const DHCPD_MAX_LEASES: usize, T: Rng> Runner<'d, DHCPD_MAX_LEASES, T> {
    /// Run the network stack.
    ///
    /// You must call this in a background task, to process network events.
    pub async fn run(&mut self) -> ! {
        let mut rx_meta = [PacketMetadata::EMPTY; 4];
        let mut tx_meta = [PacketMetadata::EMPTY; 4];
        let mut rx_buf = [0u8; 1536];
        let mut tx_buf = [0u8; 1536];

        let mut sock = UdpSocket::new(self.stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
        sock.bind(DHCP_SERVER_PORT).unwrap();

        loop {
            let mut buf = [0u8; 1500];
            let Ok((n, meta)) = sock.recv_from(&mut buf).await else { continue; };

            info!("DHCPd received {} bytes from {:?}", n, meta);

            let pkt = match DhcpPacket::new_checked(&buf[..n]) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let req = match DhcpRepr::parse(&pkt) {
                Ok(r) => r,
                Err(_) => continue,
            };

            info!("Parsed {:?}", req);

            let mut g_leases = self.leases.lock().await;

            // Check all leases for validity and remove the expired ones
            expire_leases(self.config, &mut *g_leases);

            match req.message_type {
                DhcpMessageType::Discover => {
                    let yiaddr = pick_offer(&mut self.rng, &req, self.config, &mut *g_leases);
                    if yiaddr.is_some() {
                        let resp = build_offer(&req, self.config, yiaddr.unwrap());
                        debug!("Sending back DHCP OFFER: {:?}", resp);
                        let mut out = [0u8; 512];
                        let len = emit_dhcp(&resp, &mut out);
                        let _ = sock.send_to(&out[..len.unwrap()], (Ipv4Address::BROADCAST, DHCP_CLIENT_PORT)).await;
                    }
                }
                DhcpMessageType::Request => {
                    if let Some(yiaddr) = validate_request(&req, self.config, &mut *g_leases) {
                        let resp = build_ack(&req, self.config, yiaddr);
                        debug!("Sending back DHCP ACK: {:?}", resp);
                        let mut out = [0u8; 512];
                        let len = emit_dhcp(&resp, &mut out);
                        let _ = sock.send_to(&out[..len.unwrap()], (Ipv4Address::BROADCAST, DHCP_CLIENT_PORT)).await;
                    } else {
                        let resp = build_nak(&req, self.config, req.requested_ip.unwrap());
                        debug!("Sending back DHCP NAK: {:?}", resp);
                        let mut out = [0u8; 512];
                        let len = emit_dhcp(&resp, &mut out);
                        let _ = sock.send_to(&out[..len.unwrap()], (Ipv4Address::BROADCAST, DHCP_CLIENT_PORT)).await;
                    }
                }
                _ => {}
            }
        }
    }
}


/// Create a new runner

pub fn new<'d, const DHCPD_MAX_LEASES: usize, T: Rng>(
    stack: Stack<'static>,
    rng: T,
    config: &'static DhcpdConfig,
    leases: &'static Mutex<NoopRawMutex, Vec<DhcpdLease, DHCPD_MAX_LEASES>>,
) -> Runner<'d, DHCPD_MAX_LEASES, T> {
    Runner {
        stack,
        rng,
        config,
        leases,
    }
}
