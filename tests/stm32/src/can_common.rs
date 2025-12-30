use defmt::{assert, *};
use embassy_stm32::can;
use embassy_time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct TestOptions {
    pub max_latency: Duration,
    pub max_buffered: u8,
}

pub async fn run_can_tests<'d>(can: &mut can::Can<'d>, options: &TestOptions) {
    //pub async fn run_can_tests<'d, T: can::Instance>(can: &mut can::Can<'d, T>, options: &TestOptions) {
    let mut i: u8 = 0;
    loop {
        //let tx_frame = can::frame::Frame::new_standard(0x123, &[i, 0x12 as u8, 0x34 as u8, 0x56 as u8, 0x78 as u8, 0x9A as u8, 0xBC as u8 ]).unwrap();
        let tx_frame = can::frame::Frame::new_standard(0x123, &[i; 1]).unwrap();

        //info!("Transmitting frame...");
        let tx_ts = Instant::now();
        can.write(&tx_frame).await;

        let (frame, timestamp) = can.read().await.unwrap().parts();
        //info!("Frame received!");

        // Check data.
        assert!(i == frame.data()[0], "{} == {}", i, frame.data()[0]);

        //info!("loopback time {}", timestamp);
        //info!("loopback frame {=u8}", frame.data()[0]);
        let latency = timestamp.saturating_duration_since(tx_ts);
        info!("loopback latency {} us", latency.as_micros());

        // Theoretical minimum latency is 55us, actual is usually ~80us
        const MIN_LATENCY: Duration = Duration::from_micros(50);
        // Was failing at 150 but we are not getting a real time stamp. I'm not
        // sure if there are other delays
        assert!(
            MIN_LATENCY <= latency && latency <= options.max_latency,
            "{} <= {} <= {}",
            MIN_LATENCY,
            latency,
            options.max_latency
        );

        i += 1;
        if i > 5 {
            break;
        }
    }

    // Below here, check that we can receive from both FIFO0 and FIFO1
    // Above we configured FIFO1 for extended ID packets. There are only 3 slots
    // in each FIFO so make sure we write enough to fill them both up before reading.
    for i in 0..options.max_buffered {
        // Try filling up the RX FIFO0 buffers
        //let tx_frame = if 0 != (i & 0x01) {
        let tx_frame = if i < options.max_buffered / 2 {
            info!("Transmitting standard frame {}", i);
            can::frame::Frame::new_standard(0x123, &[i; 1]).unwrap()
        } else {
            info!("Transmitting extended frame {}", i);
            can::frame::Frame::new_extended(0x1232344, &[i; 1]).unwrap()
        };
        can.write(&tx_frame).await;
    }

    // Try and receive all 6 packets
    for _i in 0..options.max_buffered {
        let (frame, _ts) = can.read().await.unwrap().parts();
        match frame.id() {
            embedded_can::Id::Extended(_id) => {
                info!("Extended received! {}", frame.data()[0]);
                //info!("Extended received! {:x} {} {}", id.as_raw(), frame.data()[0], i);
            }
            embedded_can::Id::Standard(_id) => {
                info!("Standard received! {}", frame.data()[0]);
                //info!("Standard received! {:x} {} {}", id.as_raw(), frame.data()[0], i);
            }
        }
    }
}

pub async fn run_split_can_tests<'d>(tx: &mut can::CanTx<'d>, rx: &mut can::CanRx<'d>, options: &TestOptions) {
    for i in 0..options.max_buffered {
        // Try filling up the RX FIFO0 buffers
        //let tx_frame = if 0 != (i & 0x01) {
        let tx_frame = if i < options.max_buffered / 2 {
            info!("Transmitting standard frame {}", i);
            can::frame::Frame::new_standard(0x123, &[i; 1]).unwrap()
        } else {
            info!("Transmitting extended frame {}", i);
            can::frame::Frame::new_extended(0x1232344, &[i; 1]).unwrap()
        };
        tx.write(&tx_frame).await;
    }

    // Try and receive all 6 packets
    for _i in 0..options.max_buffered {
        let (frame, _ts) = rx.read().await.unwrap().parts();
        match frame.id() {
            embedded_can::Id::Extended(_id) => {
                info!("Extended received! {}", frame.data()[0]);
            }
            embedded_can::Id::Standard(_id) => {
                info!("Standard received! {}", frame.data()[0]);
            }
        }
    }
}
