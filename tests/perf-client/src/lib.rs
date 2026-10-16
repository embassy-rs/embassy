#![no_std]

use defmt::{assert, *};
use embassy_futures::join::join;
use embassy_net::Stack;
use embassy_net::iface::Iface;
use embassy_net::tcp::TcpSocket;
use embassy_net::wire::Ipv4Addr;
use embassy_time::{Duration, with_timeout};

pub struct Expected {
    pub down_kbps: usize,
    pub up_kbps: usize,
    pub updown_kbps: usize,
}

pub async fn run(iface: Iface<'_>, expected: Expected) {
    info!("Waiting for DHCP up...");
    iface.wait_config_up().await;
    info!("IP addressing up!");

    let stack = iface.stack();
    let down = test_download(stack).await;
    let up = test_upload(stack).await;
    let updown = test_upload_download(stack).await;

    assert!(down > expected.down_kbps);
    assert!(up > expected.up_kbps);
    assert!(updown > expected.updown_kbps);
}

const TEST_DURATION: usize = 10;
const IO_BUFFER_SIZE: usize = 1024;
const RX_BUFFER_SIZE: usize = 4096;
const TX_BUFFER_SIZE: usize = 4096;
const SERVER_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 2, 2);
const DOWNLOAD_PORT: u16 = 4321;
const UPLOAD_PORT: u16 = 4322;
const UPLOAD_DOWNLOAD_PORT: u16 = 4323;

async fn test_download(stack: Stack<'_>) -> usize {
    info!("Testing download...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = unwrap!(TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer));
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, DOWNLOAD_PORT);
    if let Err(e) = socket.connect((SERVER_ADDRESS, DOWNLOAD_PORT)).await {
        error!("connect error: {:?}", e);
        return 0;
    }
    info!("connected, testing...");

    let mut rx_buf = [0; IO_BUFFER_SIZE];
    let mut total: usize = 0;
    with_timeout(Duration::from_secs(TEST_DURATION as _), async {
        loop {
            match socket.read(&mut rx_buf).await {
                Ok(0) => {
                    error!("read EOF");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    error!("read error: {:?}", e);
                    return 0;
                }
            }
        }
    })
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    info!("download: {} kB/s", kbps);
    kbps
}

async fn test_upload(stack: Stack<'_>) -> usize {
    info!("Testing upload...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = unwrap!(TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer));
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, UPLOAD_PORT);
    if let Err(e) = socket.connect((SERVER_ADDRESS, UPLOAD_PORT)).await {
        error!("connect error: {:?}", e);
        return 0;
    }
    info!("connected, testing...");

    let buf = [0; IO_BUFFER_SIZE];
    let mut total: usize = 0;
    with_timeout(Duration::from_secs(TEST_DURATION as _), async {
        loop {
            match socket.write(&buf).await {
                Ok(0) => {
                    error!("write zero?!??!?!");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    error!("write error: {:?}", e);
                    return 0;
                }
            }
        }
    })
    .await
    .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    info!("upload: {} kB/s", kbps);
    kbps
}

async fn test_upload_download(stack: Stack<'_>) -> usize {
    info!("Testing upload+download...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = unwrap!(TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer));
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("connecting to {:?}:{}...", SERVER_ADDRESS, UPLOAD_DOWNLOAD_PORT);
    if let Err(e) = socket.connect((SERVER_ADDRESS, UPLOAD_DOWNLOAD_PORT)).await {
        error!("connect error: {:?}", e);
        return 0;
    }
    info!("connected, testing...");

    let (mut reader, mut writer) = socket.split();

    let tx_buf = [0; IO_BUFFER_SIZE];
    let mut rx_buf = [0; IO_BUFFER_SIZE];
    let mut total: usize = 0;
    let tx_fut = async {
        loop {
            match writer.write(&tx_buf).await {
                Ok(0) => {
                    error!("write zero?!??!?!");
                    return 0;
                }
                Ok(_) => {}
                Err(e) => {
                    error!("write error: {:?}", e);
                    return 0;
                }
            }
        }
    };

    let rx_fut = async {
        loop {
            match reader.read(&mut rx_buf).await {
                Ok(0) => {
                    error!("read EOF");
                    return 0;
                }
                Ok(n) => total += n,
                Err(e) => {
                    error!("read error: {:?}", e);
                    return 0;
                }
            }
        }
    };

    with_timeout(Duration::from_secs(TEST_DURATION as _), join(tx_fut, rx_fut))
        .await
        .ok();

    let kbps = (total + 512) / 1024 / TEST_DURATION;
    info!("upload+download: {} kB/s", kbps);
    kbps
}

// ===== TLS perf tests (stm32h5, enabled via the `tls` feature) =====
#[cfg(feature = "tls")]
mod tls_impl {
    use embedded_tls::{Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext};
    use static_cell::StaticCell;

    use super::*;

    // perf-server's TLS endpoints (see tests/perf-server/src/main.rs).
    const TLS_DOWNLOAD_PORT: u16 = 4324;
    const TLS_UPLOAD_PORT: u16 = 4325;
    const TLS_UPLOAD_DOWNLOAD_PORT: u16 = 4326;

    // The read buffer must hold one full encrypted TLS record. perf-server
    // (rustls) sends ~1KB plaintext records; 4KB is comfortable.
    const TLS_READ_RECORD_BUFFER_SIZE: usize = 4096;
    const TLS_WRITE_RECORD_BUFFER_SIZE: usize = 4096;

    // Static so the record buffers don't sit on the task stack.
    static TLS_READ_RECORD_BUF: StaticCell<[u8; TLS_READ_RECORD_BUFFER_SIZE]> = StaticCell::new();
    static TLS_WRITE_RECORD_BUF: StaticCell<[u8; TLS_WRITE_RECORD_BUFFER_SIZE]> = StaticCell::new();

    pub async fn run_tls(iface: Iface<'_>, expected: Expected) {
        info!("Waiting for DHCP up...");
        iface.wait_config_up().await;
        info!("IP addressing up!");

        let stack = iface.stack();

        let read_record_buf = TLS_READ_RECORD_BUF.init([0; TLS_READ_RECORD_BUFFER_SIZE]);
        let write_record_buf = TLS_WRITE_RECORD_BUF.init([0; TLS_WRITE_RECORD_BUFFER_SIZE]);

        let down = test_download(stack, &mut read_record_buf[..], &mut write_record_buf[..]).await;
        let up = test_upload(stack, &mut read_record_buf[..], &mut write_record_buf[..]).await;
        let updown = test_upload_download(stack, &mut read_record_buf[..], &mut write_record_buf[..]).await;

        super::assert!(down > expected.down_kbps);
        super::assert!(up > expected.up_kbps);
        super::assert!(updown > expected.updown_kbps);
    }

    // Connect a TLS 1.3 session to perf-server. All cryptography is served by
    // `embassy-crypto` drivers resolved at link time; NoVerify skips server
    // certificate verification (fine for testing). Cipher suite
    // TLS13_AES_128_GCM_SHA256, first in rustls' default list, so suite
    // overlap with perf-server is guaranteed.
    macro_rules! tls_connect {
        ($socket:expr, $port:expr, $read_buf:expr, $write_buf:expr) => {{
            let mut socket = $socket;
            socket.set_timeout(Some(Duration::from_secs(30)));
            info!("connecting to {:?}:{}...", SERVER_ADDRESS, $port);
            if let Err(e) = socket.connect((SERVER_ADDRESS, $port)).await {
                error!("connect error: {:?}", e);
                return 0;
            }
            info!("socket connected");
            let config = TlsConfig::new();
            let mut tls: TlsConnection<'_, _, Aes128GcmSha256> = TlsConnection::new(socket, $read_buf, $write_buf);
            match tls.open(TlsContext::new(&config, NoVerify)).await {
                Ok(()) => {
                    info!("TLS connected, testing...");
                    tls.set_flush_policy(embedded_tls::flush_policy::FlushPolicy::Relaxed);
                    tls
                }
                Err(e) => {
                    error!("TLS handshake error: {:?}", e);
                    return 0;
                }
            }
        }};
    }

    async fn test_download(stack: Stack<'_>, read_record_buf: &mut [u8], write_record_buf: &mut [u8]) -> usize {
        info!("Testing TLS download...");

        let mut rx_buffer = [0; RX_BUFFER_SIZE];
        let mut tx_buffer = [0; TX_BUFFER_SIZE];
        let socket = unwrap!(TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer));
        let mut tls = tls_connect!(socket, TLS_DOWNLOAD_PORT, read_record_buf, write_record_buf);

        let mut rx_buf = [0; IO_BUFFER_SIZE];
        let mut total: usize = 0;
        with_timeout(Duration::from_secs(TEST_DURATION as _), async {
            loop {
                match tls.read(&mut rx_buf).await {
                    Ok(0) => {
                        error!("read EOF");
                        return 0;
                    }
                    Ok(n) => total += n,
                    Err(e) => {
                        error!("read error: {:?}", e);
                        return 0;
                    }
                }
            }
        })
        .await
        .ok();

        let kbps = (total + 512) / 1024 / TEST_DURATION;
        info!("tls download: {} kB/s", kbps);
        kbps
    }

    async fn test_upload(stack: Stack<'_>, read_record_buf: &mut [u8], write_record_buf: &mut [u8]) -> usize {
        info!("Testing TLS upload...");

        let mut rx_buffer = [0; RX_BUFFER_SIZE];
        let mut tx_buffer = [0; TX_BUFFER_SIZE];
        let socket = unwrap!(TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer));
        let mut tls = tls_connect!(socket, TLS_UPLOAD_PORT, read_record_buf, write_record_buf);

        let buf = [0; IO_BUFFER_SIZE];
        let mut total: usize = 0;
        with_timeout(Duration::from_secs(TEST_DURATION as _), async {
            loop {
                match tls.write(&buf).await {
                    Ok(0) => {
                        error!("write zero?!??!?!");
                        return 0;
                    }
                    Ok(n) => total += n,
                    Err(e) => {
                        error!("write error: {:?}", e);
                        return 0;
                    }
                }
            }
        })
        .await
        .ok();

        let kbps = (total + 512) / 1024 / TEST_DURATION;
        info!("tls upload: {} kB/s", kbps);
        kbps
    }

    async fn test_upload_download(stack: Stack<'_>, read_record_buf: &mut [u8], write_record_buf: &mut [u8]) -> usize {
        info!("Testing TLS upload+download...");

        let mut rx_buffer = [0; RX_BUFFER_SIZE];
        let mut tx_buffer = [0; TX_BUFFER_SIZE];
        let socket = unwrap!(TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer));
        let mut tls = tls_connect!(socket, TLS_UPLOAD_DOWNLOAD_PORT, read_record_buf, write_record_buf);

        // TlsConnection has no split(), so this is a write/read ping-pong
        // (matching the server's read-then-echo loop), not full duplex.
        let tx_buf = [0; IO_BUFFER_SIZE];
        let mut rx_buf = [0; IO_BUFFER_SIZE];
        let mut total: usize = 0;
        with_timeout(Duration::from_secs(TEST_DURATION as _), async {
            loop {
                match tls.write(&tx_buf).await {
                    Ok(0) => {
                        error!("write zero?!??!?!");
                        return 0;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("write error: {:?}", e);
                        return 0;
                    }
                }
                if let Err(e) = tls.flush().await {
                    error!("flush error: {:?}", e);
                    return 0;
                }
                match tls.read(&mut rx_buf).await {
                    Ok(0) => {
                        error!("read EOF");
                        return 0;
                    }
                    Ok(n) => total += n,
                    Err(e) => {
                        error!("read error: {:?}", e);
                        return 0;
                    }
                }
            }
        })
        .await
        .ok();

        let kbps = (total + 512) / 1024 / TEST_DURATION;
        info!("tls upload+download: {} kB/s", kbps);
        kbps
    }
}

#[cfg(feature = "tls")]
pub use tls_impl::run_tls;
