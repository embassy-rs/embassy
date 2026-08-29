use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;

use log::info;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};

fn main() {
    pretty_env_logger::init();
    spawn(|| rx_listen());
    spawn(|| rxtx_listen());
    spawn(|| tls_rx_listen());
    spawn(|| tls_rxtx_listen());
    spawn(|| tls_tx_listen());
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
    info!("tx: listening on 0.0.0.0:4321");
    let listener = TcpListener::bind("0.0.0.0:4321").unwrap();
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
    info!("tls tx: listening on 0.0.0.0:4324");
    let listener = TcpListener::bind("0.0.0.0:4324").unwrap();
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
    info!("rx: listening on 0.0.0.0:4322");
    let listener = TcpListener::bind("0.0.0.0:4322").unwrap();
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
    info!("tls rx: listening on 0.0.0.0:4325");
    let listener = TcpListener::bind("0.0.0.0:4325").unwrap();
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
    info!("rxtx: listening on 0.0.0.0:4323");
    let listener = TcpListener::bind("0.0.0.0:4323").unwrap();
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
    info!("tls rxtx: listening on 0.0.0.0:4326");
    let listener = TcpListener::bind("0.0.0.0:4326").unwrap();
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
