use std::path::Path;
use std::time::Duration;
use std::{env, io, thread};

use rand::random;
use serial::SerialPort;

pub fn main() {
    if let Some(port_name) = env::args().nth(1) {
        let sleep = env::args().position(|x| x == "--sleep").is_some();

        println!("Saturating port {:?} with 115200 8N1", port_name);
        println!("Sleep: {}", sleep);
        let mut port = serial::open(&port_name).unwrap();
        if saturate(&mut port, sleep).is_err() {
            eprintln!("Unable to saturate port");
        }
    } else {
        let path = env::args().next().unwrap();
        let basepath = Path::new(&path).with_extension("");
        let basename = basepath.file_name().unwrap();
        eprintln!("USAGE: {} <port-name>", basename.to_string_lossy());
    }
}

fn saturate<T: SerialPort>(port: &mut T, sleep: bool) -> io::Result<()> {
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        Ok(())
    })?;

    let mut written = 0;
    loop {
        let len = random::<usize>() % 0x1000;
        let buf: Vec<u8> = (written..written + len).map(|x| x as u8).collect();

        port.write_all(&buf)?;

        if sleep {
            let micros = (random::<usize>() % 1000) as u64;
            println!("Sleeping {}us", micros);
            port.flush().unwrap();
            thread::sleep(Duration::from_micros(micros));
        }

        written += len;
        println!("Written: {}", written);
    }
}