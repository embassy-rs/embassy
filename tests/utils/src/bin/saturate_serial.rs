use std::path::Path;
use std::{env, io};

use rand::random;
use serial::SerialPort;

pub fn main() {
    if let Some(port_name) = env::args().nth(1) {
        println!("Saturating port {:?} with 115200 8N1", port_name);
        let mut port = serial::open(&port_name).unwrap();
        if saturate(&mut port).is_err() {
            eprintln!("Unable to saturate port");
        }
    } else {
        let path = env::args().next().unwrap();
        let basepath = Path::new(&path).with_extension("");
        let basename = basepath.file_name().unwrap();
        eprintln!("USAGE: {} <port-name>", basename.to_string_lossy());
    }
}

fn saturate<T: SerialPort>(port: &mut T) -> io::Result<()> {
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

        written += len;
        println!("Written: {}", written);
    }
}
