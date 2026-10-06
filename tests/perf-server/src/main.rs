use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use log::info;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

/// Every listener binds the IPv6 wildcard rather than the IPv4 one, so that one socket
/// serves clients of both families. With `net.ipv6.bindv6only = 0` (the Linux default) a
/// `[::]` socket also accepts IPv4.
fn wildcard(port: u16) -> String {
    format!("[::]:{port}")
}

fn main() {
    pretty_env_logger::init();
    spawn(rx_listen);
    spawn(rxtx_listen);
    spawn(tls_rx_listen);
    spawn(tls_rxtx_listen);
    spawn(tls_tx_listen);
    spawn(udp_download_listen);
    spawn(udp_upload_listen);
    tx_listen();
}

fn tls_server_config() -> Arc<ServerConfig> {
    let cert = rcgen::generate_simple_self_signed(["localhost".to_string()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert.der().to_owned());
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der()));
    Arc::new(
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .expect("valid self-signed certificate"),
    )
}

fn tx_listen() {
    info!("tx: listening on [::]:4321 (IPv4 and IPv6)");
    let listener = TcpListener::bind(wildcard(4321)).unwrap();
    loop {
        let (socket, addr) = listener.accept().unwrap();
        info!("tx: received connection from: {}", addr);
        spawn(|| tx_conn(socket));
    }
}

fn tx_conn(mut socket: TcpStream) {
    socket.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(30))).unwrap();

    let buf = [0; 1024];
    loop {
        if let Err(e) = socket.write_all(&buf) {
            info!("tx: failed to write to socket; err = {:?}", e);
            return;
        }
    }
}

fn tls_tx_listen() {
    let acceptor = tls_server_config();
    info!("tls tx: listening on [::]:4324 (IPv4 and IPv6)");
    let listener = TcpListener::bind(wildcard(4324)).unwrap();
    loop {
        let (socket, addr) = listener.accept().unwrap();
        info!("tls tx: received connection from: {}", addr);
        let acceptor = Arc::clone(&acceptor);
        spawn(move || tls_tx_conn(socket, acceptor));
    }
}

fn tls_tx_conn(socket: TcpStream, acceptor: Arc<ServerConfig>) {
    let mut socket = StreamOwned::new(ServerConnection::new(acceptor).unwrap(), socket);
    socket
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    socket
        .get_mut()
        .set_write_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let buf = [0; 1024];
    loop {
        if let Err(e) = socket.write_all(&buf) {
            info!("tls tx: failed to write to socket; err = {:?}", e);
            return;
        }
    }
}

fn rx_listen() {
    info!("rx: listening on [::]:4322 (IPv4 and IPv6)");
    let listener = TcpListener::bind(wildcard(4322)).unwrap();
    loop {
        let (socket, addr) = listener.accept().unwrap();
        info!("rx: received connection from: {}", addr);
        spawn(|| rx_conn(socket));
    }
}

fn rx_conn(mut socket: TcpStream) {
    socket.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(30))).unwrap();

    let mut buf = [0; 1024];
    loop {
        if let Err(e) = socket.read_exact(&mut buf) {
            info!("rx: failed to read from socket; err = {:?}", e);
            return;
        }
    }
}

fn tls_rx_listen() {
    let acceptor = tls_server_config();
    info!("tls rx: listening on [::]:4325 (IPv4 and IPv6)");
    let listener = TcpListener::bind(wildcard(4325)).unwrap();
    loop {
        let (socket, addr) = listener.accept().unwrap();
        info!("tls rx: received connection from: {}", addr);
        let acceptor = Arc::clone(&acceptor);
        spawn(move || tls_rx_conn(socket, acceptor));
    }
}

fn tls_rx_conn(socket: TcpStream, acceptor: Arc<ServerConfig>) {
    let mut socket = StreamOwned::new(ServerConnection::new(acceptor).unwrap(), socket);
    socket
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    socket
        .get_mut()
        .set_write_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let mut buf = [0; 1024];
    loop {
        if let Err(e) = socket.read_exact(&mut buf) {
            info!("tls rx: failed to read from socket; err = {:?}", e);
            return;
        }
    }
}

fn rxtx_listen() {
    info!("rxtx: listening on [::]:4323 (IPv4 and IPv6)");
    let listener = TcpListener::bind(wildcard(4323)).unwrap();
    loop {
        let (socket, addr) = listener.accept().unwrap();
        info!("rxtx: received connection from: {}", addr);
        spawn(|| rxtx_conn(socket));
    }
}

fn rxtx_conn(mut socket: TcpStream) {
    socket.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    socket.set_write_timeout(Some(Duration::from_secs(30))).unwrap();

    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf) {
            Ok(n) => {
                if let Err(e) = socket.write_all(&buf[..n]) {
                    info!("rxtx: failed to write to socket; err = {:?}", e);
                    return;
                }
            }
            Err(e) => {
                info!("rxtx: failed to read from socket; err = {:?}", e);
                return;
            }
        }
    }
}

fn tls_rxtx_listen() {
    let acceptor = tls_server_config();
    info!("tls rxtx: listening on [::]:4326 (IPv4 and IPv6)");
    let listener = TcpListener::bind(wildcard(4326)).unwrap();
    loop {
        let (socket, addr) = listener.accept().unwrap();
        info!("tls rxtx: received connection from: {}", addr);
        let acceptor = Arc::clone(&acceptor);
        spawn(move || tls_rxtx_conn(socket, acceptor));
    }
}

fn tls_rxtx_conn(socket: TcpStream, acceptor: Arc<ServerConfig>) {
    let mut socket = StreamOwned::new(ServerConnection::new(acceptor).unwrap(), socket);
    socket
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    socket
        .get_mut()
        .set_write_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    let mut buf = [0; 1024];
    loop {
        match socket.read(&mut buf) {
            Ok(n) => {
                if let Err(e) = socket.write_all(&buf[..n]) {
                    info!("tls rxtx: failed to write to socket; err = {:?}", e);
                    return;
                }
            }
            Err(e) => {
                info!("tls rxtx: failed to read from socket; err = {:?}", e);
                return;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// UDP
//
// Same two directions as TCP, but connectionless, so each one needs a little
// protocol of its own:
//
//  * **download** (4324, server -> client): the client "subscribes" by sending a
//    datagram. The server then floods that peer with datagrams of the same size as
//    the subscription datagram, paced to `UDP_RATE_BPS`, and keeps doing so as long
//    as the client keeps re-subscribing (every `SUBSCRIPTION` at the latest).
//  * **upload** (4325, client -> server): the server just receives and discards,
//    printing what it got once the flow goes quiet.
//
// The flood is paced because the client is on a 100 Mbit link behind a switch: an
// unpaced `send_to` loop from a gigabit port measures the switch's drop policy, not
// the client's stack. `UDP_RATE_BPS` overrides the default from the environment.
// ---------------------------------------------------------------------------

/// How long a client's subscription to the download flood lasts without renewal.
const SUBSCRIPTION: Duration = Duration::from_secs(2);
/// A flow is considered over after this long without a datagram.
const FLOW_IDLE: Duration = Duration::from_millis(500);
/// Default pacing for the download flood: 100 Mbit/s, i.e. Fast Ethernet line rate.
const DEFAULT_UDP_RATE_BPS: u64 = 100_000_000;

/// Bytes each datagram costs on the wire on top of its payload, over Ethernet + UDP:
/// 8 preamble/SFD, 14 Ethernet header, 20 (IPv4) or 40 (IPv6) IP header, 8 UDP, 4 FCS,
/// 12 interframe gap. The flood is paced against this rather than against payload bytes
/// — pacing on payload alone overdrives a 100 Mbit link by ~4.5% with full-MTU
/// datagrams, and then every client looks like it drops 4.5% no matter how fast it is.
///
/// A client that reached us over IPv4 through the dual-stack socket has a v4-mapped
/// address, so the family has to be read through `to_ipv4_mapped` rather than off the
/// `SocketAddr` variant.
fn wire_overhead(addr: SocketAddr) -> u64 {
    let is_v4 = match addr {
        SocketAddr::V4(_) => true,
        SocketAddr::V6(a) => a.ip().to_ipv4_mapped().is_some(),
    };
    if is_v4 { 66 } else { 86 }
}

fn udp_rate_bps() -> u64 {
    std::env::var("UDP_RATE_BPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_UDP_RATE_BPS)
}

/// The peer currently being flooded, and until when.
struct Subscriber {
    addr: SocketAddr,
    len: usize,
    until: Instant,
}

fn udp_download_listen() {
    let rate = udp_rate_bps();
    info!(
        "udp download: listening on [::]:4324 (IPv4 and IPv6), pacing at {} kbit/s on the wire",
        rate / 1000
    );

    let socket = UdpSocket::bind(wildcard(4324)).unwrap();
    let sender = socket.try_clone().unwrap();
    static SUB: Mutex<Option<Subscriber>> = Mutex::new(None);

    // Subscriptions.
    spawn(move || {
        let mut buf = [0; 2048];
        loop {
            let (n, addr) = match socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(e) => {
                    info!("udp download: recv error; err = {:?}", e);
                    continue;
                }
            };
            let mut sub = SUB.lock().unwrap();
            if sub.as_ref().is_none_or(|s| s.addr != addr) {
                info!("udp download: subscription from {}, {} byte datagrams", addr, n);
            }
            *sub = Some(Subscriber {
                addr,
                len: n.max(1),
                until: Instant::now() + SUBSCRIPTION,
            });
        }
    });

    // The flood itself. `sent` and `since` reset whenever the peer changes, so the
    // pacing schedule is always relative to the start of the current flood.
    let buf = [0u8; 2048];
    let mut current: Option<SocketAddr> = None;
    let mut since = Instant::now();
    let mut sent: u64 = 0; // payload, for reporting
    let mut wire: u64 = 0; // payload + framing, for pacing
    let mut window_start = Instant::now();
    let mut window_bytes: u64 = 0;
    loop {
        let target = {
            let sub = SUB.lock().unwrap();
            match sub.as_ref() {
                Some(s) if Instant::now() < s.until => Some((s.addr, s.len)),
                _ => None,
            }
        };

        let Some((addr, len)) = target else {
            if let Some(addr) = current.take() {
                info!(
                    "udp download: {} idle, sent {} kB in {:.1}s ({} kbit/s)",
                    addr,
                    sent / 1024,
                    since.elapsed().as_secs_f64(),
                    kbits_per_sec(sent, since.elapsed())
                );
            }
            sleep(Duration::from_millis(50));
            continue;
        };

        if current != Some(addr) {
            current = Some(addr);
            since = Instant::now();
            sent = 0;
            wire = 0;
            window_start = since;
            window_bytes = 0;
        }

        // One line a second, so it can be lined up against the client's own.
        let window = window_start.elapsed();
        if window >= Duration::from_secs(1) {
            info!(
                "udp download: {} kbit/s (avg {} kbit/s, {} kB total)",
                kbits_per_sec(window_bytes, window),
                kbits_per_sec(sent, since.elapsed()),
                sent / 1024,
            );
            window_start = Instant::now();
            window_bytes = 0;
        }

        // Pace: if we are ahead of where the target rate says we should be, wait.
        let owed = Duration::from_secs_f64(wire as f64 * 8.0 / rate as f64);
        let elapsed = since.elapsed();
        if let Some(ahead) = owed.checked_sub(elapsed)
            && ahead > Duration::from_millis(1)
        {
            sleep(ahead);
        }

        match sender.send_to(&buf[..len.min(buf.len())], addr) {
            Ok(n) => {
                sent += n as u64;
                wire += n as u64 + wire_overhead(addr);
                window_bytes += n as u64;
            }
            Err(e) => {
                info!("udp download: send error; err = {:?}", e);
                sleep(Duration::from_millis(10));
            }
        }
    }
}

fn udp_upload_listen() {
    info!("udp upload: listening on [::]:4325 (IPv4 and IPv6)");
    let socket = UdpSocket::bind(wildcard(4325)).unwrap();
    socket.set_read_timeout(Some(FLOW_IDLE)).unwrap();

    let mut buf = [0; 2048];
    let mut flow: Option<Flow> = None;

    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, addr)) => {
                match &mut flow {
                    Some(f) if f.peer == addr => f.add(n as u64),
                    _ => {
                        info!("udp upload: flow from {}, {} byte datagrams", addr, n);
                        flow = Some(Flow::new(addr, n as u64));
                    }
                }
                // The client prints a line a second; print one alongside it, so the
                // two can be compared directly.
                flow.as_mut().unwrap().tick("udp upload");
            }
            // Read timeout: whatever flow was in progress is over.
            Err(_) => {
                if let Some(f) = flow.take() {
                    f.report_total("udp upload");
                }
            }
        }
    }
}

/// A one-peer receive flow, reported once a second and once at the end.
struct Flow {
    peer: SocketAddr,
    started: Instant,
    window_start: Instant,
    window_bytes: u64,
    total_bytes: u64,
    total_datagrams: u64,
}

impl Flow {
    fn new(peer: SocketAddr, first: u64) -> Self {
        let now = Instant::now();
        Self {
            peer,
            started: now,
            window_start: now,
            window_bytes: first,
            total_bytes: first,
            total_datagrams: 1,
        }
    }

    fn add(&mut self, n: u64) {
        self.window_bytes += n;
        self.total_bytes += n;
        self.total_datagrams += 1;
    }

    fn tick(&mut self, label: &str) {
        let window = self.window_start.elapsed();
        if window < Duration::from_secs(1) {
            return;
        }
        info!(
            "{}: {} kbit/s (avg {} kbit/s, {} kB total)",
            label,
            kbits_per_sec(self.window_bytes, window),
            kbits_per_sec(self.total_bytes, self.started.elapsed()),
            self.total_bytes / 1024,
        );
        self.window_start = Instant::now();
        self.window_bytes = 0;
    }

    fn report_total(&self, label: &str) {
        // The last `FLOW_IDLE` of the measured window is the timeout that ended it.
        let elapsed = self.started.elapsed().saturating_sub(FLOW_IDLE);
        info!(
            "{}: {} done, {} kB in {} datagrams over {:.1}s ({} kbit/s)",
            label,
            self.peer,
            self.total_bytes / 1024,
            self.total_datagrams,
            elapsed.as_secs_f64(),
            kbits_per_sec(self.total_bytes, elapsed)
        );
    }
}

fn kbits_per_sec(bytes: u64, elapsed: Duration) -> u64 {
    let secs = elapsed.as_secs_f64();
    if secs <= 0.0 {
        return 0;
    }
    (bytes as f64 * 8.0 / secs / 1000.0) as u64
}
