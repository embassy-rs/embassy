//! An `async`-friendly I2C implementation with DMA support.
//!
//! It uses a single statically allocated buffer for both RX and TX,
//! A DMA for data transfer and a DMA interrupt future for managing the transfers.
//!
//! [async-embedded-traits]'s [`AsyncI2cWrite`] and [`AsyncI2cTransfer`] are implemented
//! for compatibility with device drivers that also implement these traits.
//!
//! [async-embedded-traits]: https://crates.io/crates/async-embedded-traits
//! [`AsyncI2cWrite`]: https://docs.rs/async-embedded-traits/0.1.2/async_embedded_traits/i2c/trait.AsyncI2cWrite.html
//! [`AsyncI2cTransfer`]: https://docs.rs/async-embedded-traits/0.1.2/async_embedded_traits/i2c/trait.AsyncI2cTransfer.html

macro_rules! impl_async_i2c_dma {
    ($i2ci:ident, $I2Ci:ident, $dmaimpl:ident, $DMAi:ident, $Ci:ident, $Cint:path, $tcifi:ident, $ctcifi:ident, $teifi:ident, $cteifi:ident) => {
        pub mod $i2ci {
            use async_embedded_traits::i2c::{
                AsyncI2cRead, AsyncI2cTransfer, AsyncI2cWrite, I2cAddress7Bit,
            };
            use core::convert::TryInto;
            use core::future::Future;

            use embassy::interrupt::Interrupt;
            use embassy::util::InterruptFuture;

            use crate::hal::dma::{ReadDma, WriteDma};
            use crate::hal::pac::Peripherals;
            use crate::hal::{dma::$dmaimpl::$Ci, i2c::{I2c, Error as I2cError}, pac::$I2Ci};

            pub struct AsyncI2c<I2C, SCL, SDA, I: Interrupt> {
                buf: &'static [u8],
                i2c: I2c<I2C, (SCL, SDA)>,
                dma_int: I,
                dma_ch: $Ci,
            }

            impl<SCL, SDA> AsyncI2c<$I2Ci, SCL, SDA, $Cint> {
                pub fn new(
                    buf: &'static [u8],
                    i2c: I2c<$I2Ci, (SCL, SDA)>,
                    dma_int: $Cint,
                    dma_ch: $Ci,
                ) -> Self {
                    Self {
                        buf,
                        i2c,
                        dma_int,
                        dma_ch,
                    }
                }

                pub fn free(self) -> ($I2Ci, (SCL, SDA)) {
                    self.i2c.free()
                }

                fn handle_dma_int() -> Result<(), I2cError> {
                    let dma = unsafe { Peripherals::steal().$DMAi };
                    let tc_flag = dma.isr.read().$tcifi().bit();
                    if tc_flag {
                        dma.ifcr.write(|w| w.$ctcifi().set_bit());
                    }

                    let te_flag = dma.isr.read().$teifi().bit();
                    if te_flag {
                        dma.ifcr.write(|w| w.$cteifi().set_bit());
                        return Err(I2cError::Bus);
                    }

                    Ok(())
                }

                async fn tx_helper(
                    &mut self,
                    dma_ch: $Ci,
                    i2c: I2c<$I2Ci, (SCL, SDA)>,
                    address: u8,
                    data: &[u8],
                    autostop: bool,
                ) -> Result<($Ci, I2c<$I2Ci, (SCL, SDA)>), I2cError> {
                    // Make the static buffer mutable as per `embedded-dma` requirements (`DerefMut`).
                    // It's safe as long as the buffer isn't used until this future completes.
                    #[allow(mutable_transmutes)]
                    let buf = unsafe {
                        core::mem::transmute::<&'static [u8], &'static mut [u8]>(
                            &self.buf[..data.len()],
                        )
                    };
                    buf.copy_from_slice(data);

                    let tx_transfer =
                        i2c.with_tx_dma(dma_ch, address.try_into().unwrap(), autostop);
                    let transfer = tx_transfer.write(buf);
                    InterruptFuture::new(&mut self.dma_int).await;
                    Self::handle_dma_int()?;
                    let (_, tx_dma) = transfer.destroy();

                    Ok(tx_dma.free())
                }

                async fn rx_helper(
                    &mut self,
                    dma_ch: $Ci,
                    i2c: I2c<$I2Ci, (SCL, SDA)>,
                    address: u8,
                    rx_data: &mut [u8],
                    autostop: bool,
                ) -> Result<($Ci, I2c<$I2Ci, (SCL, SDA)>), I2cError> {
                    // Make the static buffer mutable as per `embedded-dma` requirements.
                    // It's safe as long as the buffer isn't used until this future completes.
                    #[allow(mutable_transmutes)]
                    let rx_buf = unsafe {
                        core::mem::transmute::<&'static [u8], &'static mut [u8]>(
                            &self.buf[0..rx_data.len()],
                        )
                    };
                    let rx_transfer = i2c.with_rx_dma(dma_ch, address, autostop);
                    let transfer = rx_transfer.read(rx_buf);
                    InterruptFuture::new(&mut self.dma_int).await;
                    Self::handle_dma_int()?;
                    let (rx_buf, rx_dma) = transfer.destroy();
                    rx_data.copy_from_slice(rx_buf);

                    Ok(rx_dma.free())
                }
            }

            impl<SCL, SDA> AsyncI2cWrite<I2cAddress7Bit> for AsyncI2c<$I2Ci, SCL, SDA, $Cint> {
                type Error = I2cError;
                type WriteFuture<'f> = impl Future<Output = Result<(), Self::Error>>;

                /// Writes `data.len()` bytes into I2C slave device with specific `address`.
                ///
                /// 1. The static buffer (`buf`) is first filled with the data to be transmitted (`len` bytes)
                /// 2. A slice of this buffer of `len` bytes is made (`tx_buf`)
                /// 3. A DMA transfer will copy exactly `len` bytes from said slice into I2C peripheral
                /// 4. A future awaits for a DMA "transfer complete" interrupt to fire
                /// 5. Upon DMA interrupt, the future is woken up and completed
                fn async_write<'a>(
                    &'a mut self,
                    address: I2cAddress7Bit,
                    data: &'a [u8],
                ) -> Self::WriteFuture<'a> {
                    assert!(data.len() <= self.buf.len());
                    let address = address.try_into().unwrap();

                    async move {
                        let dma_ch = unsafe { core::ptr::read(&self.dma_ch) };
                        let i2c = unsafe { core::ptr::read(&self.i2c) };
                        self.tx_helper(dma_ch, i2c, address, data, true).await?;

                        Ok(())
                    }
                }
            }

            impl<SCL, SDA> AsyncI2cRead<I2cAddress7Bit> for AsyncI2c<$I2Ci, SCL, SDA, $Cint> {
                type Error = I2cError;
                type ReadFuture<'f> = impl Future<Output = Result<(), Self::Error>>;

                /// Reads `data.len()` bytes from I2C slave device with specific `address`.
                ///
                /// 1. A slice of `data.len()` is made from the static buffer
                /// 2. A DMA transfer will fill the slice with bytes received from I2C peripheral
                /// 3. Upon DMA "transfer complete" interrupt the future is woken up
                /// 4. The `data` buffer is filled with the received bytes from the static buffer
                fn async_read<'a>(
                    &'a mut self,
                    address: I2cAddress7Bit,
                    data: &'a mut [u8],
                ) -> Self::ReadFuture<'a> {
                    assert!(data.len() <= self.buf.len());
                    let address = address.try_into().unwrap();

                    async move {
                        let dma_ch = unsafe { core::ptr::read(&self.dma_ch) };
                        let i2c = unsafe { core::ptr::read(&self.i2c) };
                        self.rx_helper(dma_ch, i2c, address, data, true).await?;

                        Ok(())
                    }
                }
            }

            impl<SCL, SDA> AsyncI2cTransfer<I2cAddress7Bit> for AsyncI2c<$I2Ci, SCL, SDA, $Cint> {
                type Error = I2cError;
                type TransferFuture<'f> = impl core::future::Future<Output = Result<(), Self::Error>>;

                /// 1. The same steps 1 to 4 from `AsyncI2cWrite` are made, and then
                /// 2. A slice of `rx_data.len()` is made from the static buffer
                /// 3. A DMA transfer will fill the slice with bytes received from I2C peripheral
                /// 4. Upon DMA "transfer complete" interrupt the future is woken up
                /// 5. The `rx_data` slice is filled with the received bytes from the static buffer
                fn async_transfer<'a>(
                    &'a mut self,
                    address: I2cAddress7Bit,
                    tx_data: &'a [u8],
                    rx_data: &'a mut [u8],
                ) -> Self::TransferFuture<'a> {
                    assert!(tx_data.len() <= self.buf.len());
                    assert!(rx_data.len() <= self.buf.len());

                    async move {
                        let address = address.try_into().unwrap();
                        let dma_ch = unsafe { core::ptr::read(&self.dma_ch) };
                        let i2c = unsafe { core::ptr::read(&self.i2c) };

                        // Send the data to I2C bus without asserting STOP condition
                        let (dma_ch, i2c) =
                            self.tx_helper(dma_ch, i2c, address, tx_data, false).await?;
                        self.rx_helper(dma_ch, i2c, address, rx_data, true).await?;

                        Ok(())
                    }
                }
            }
        }
    };
}

// Generate implementations for WB55's I2C1 and I2C3.
// Allocated DMA channels are:
// DMA1's C1 for I2C1
// DMA1's C2 for I2C3

impl_async_i2c_dma!(
    i2c1,
    I2C1,
    dma1impl,
    DMA1,
    C1,
    crate::interrupt::DMA1_CHANNEL1,
    tcif1,
    ctcif1,
    teif1,
    cteif1
);

impl_async_i2c_dma!(
    i2c3,
    I2C3,
    dma1impl,
    DMA1,
    C2,
    crate::interrupt::DMA1_CHANNEL2,
    tcif2,
    ctcif2,
    teif2,
    cteif2
);
