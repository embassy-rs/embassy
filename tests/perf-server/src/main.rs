use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use std::time::Duration;

use log::info;

fn main() {
    pretty_env_logger::init();
    spawn(|| rx_listen());
    spawn(|| rxtx_listen());
    tx_listen();
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
