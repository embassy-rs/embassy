use defmt::*;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::{Read, Write};

use crate::UsbIoError;

pub struct ApiHandler<'a, S>
where
    S: Read + Write + Unpin,
{
    pub serial: S,
    cmd_buf: &'a mut [u8; 128],
    rec_len: usize,
    rec_stale: Instant,
}

impl<'a, S> ApiHandler<'a, S>
where
    S: Read + Write + Unpin + embedded_io_async::ErrorType<Error = UsbIoError>,
    <S as embedded_io_async::ErrorType>::Error: defmt::Format,
{
    pub fn new(serial: S, cmd_buf: &'a mut [u8; 128]) -> Self {
        ApiHandler {
            serial,
            cmd_buf,
            rec_len: 0,
            rec_stale: Instant::now(),
        }
    }

    async fn read_data(&mut self) -> Result<usize, UsbIoError> {
        let read_slice = self.cmd_buf[self.rec_len..].as_mut();
        match Read::read(&mut self.serial, read_slice).await {
            Ok(r) => {
                self.rec_stale = Instant::now() + Duration::from_millis(800);
                debug!("Added {} bytes for {} total", r, self.rec_len + 1);
                Ok(r)
            }
            Err(e) => {
                error!("Error reading from serial port `{}`", e);
                self.flush_cmd_buf();
                Timer::after(Duration::from_millis(500)).await;
                Err(e)
            }
        }
    }

    pub async fn receive(&mut self) -> Result<bool, UsbIoError> {
        let stale_timeout = self.rec_stale;

        let rec_len = if let Either::First(future) = select(self.read_data(), Timer::at(stale_timeout)).await {
            match future {
                Ok(r) => r,
                Err(e) => return Err(e),
            }
        } else if self.rec_len > 0 {
            error!("API command is stale");
            self.flush_cmd_buf();
            return Ok(false);
        } else {
            self.flush_cmd_buf();
            0
        };

        self.rec_len += rec_len;
        Ok(true)
    }

    pub fn get_rec_len(&mut self) -> usize {
        self.rec_len
    }

    fn flush_cmd_buf(&mut self) {
        info!("Flush cmd buffer");
        self.rec_len = 0;
        self.rec_stale = Instant::MAX;
    }
}
