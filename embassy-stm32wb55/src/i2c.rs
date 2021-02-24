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
            use async_embedded_traits::i2c::{AsyncI2cTransfer, AsyncI2cWrite, I2cAddress7Bit};
            use core::convert::TryInto;
            use core::future::Future;

            use embassy::interrupt::OwnedInterrupt;
            use embassy::util::InterruptFuture;

            use crate::hal::dma::{ReadDma, WriteDma};
            use crate::hal::pac::Peripherals;
            use crate::hal::{dma::$dmaimpl::$Ci, i2c::I2c, pac::$I2Ci};

            pub struct AsyncI2c<I2C, PINS, I: OwnedInterrupt> {
                buf: &'static [u8],
                i2c: I2c<I2C, PINS>,
                dma_int: I,
                dma_ch: $Ci,
            }

            impl<PINS> AsyncI2c<$I2Ci, PINS, $Cint> {
                pub fn new(
                    buf: &'static [u8],
                    i2c: I2c<$I2Ci, PINS>,
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

                fn handle_dma_int() -> Result<(), ()> {
                    let dma = unsafe { Peripherals::steal().$DMAi };
                    let tc_flag = dma.isr.read().$tcifi().bit();
                    if tc_flag {
                        dma.ifcr.write(|w| w.$ctcifi().set_bit());
                    }

                    let te_flag = dma.isr.read().$teifi().bit();
                    if te_flag {
                        dma.ifcr.write(|w| w.$cteifi().set_bit());
                        return Err(());
                    }

                    Ok(())
                }

                async fn tx_helper(
                    &mut self,
                    dma_ch: $Ci,
                    i2c: I2c<$I2Ci, PINS>,
                    address: u8,
                    data: &[u8],
                    autostop: bool
                ) -> Result<($Ci, I2c<$I2Ci, PINS>), ()> {
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
            }

            impl<PINS> AsyncI2cWrite<I2cAddress7Bit> for AsyncI2c<$I2Ci, PINS, $Cint> {
                type Error = ();
                type WriteFuture<'f> = impl Future<Output = Result<(), ()>>;

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

            impl<PINS> AsyncI2cTransfer<I2cAddress7Bit> for AsyncI2c<$I2Ci, PINS, $Cint> {
                type Error = ();
                type TransferFuture<'f> = impl core::future::Future<Output = Result<(), ()>>;

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
                        let (dma_ch, i2c) = self.tx_helper(dma_ch, i2c, address, tx_data, false).await?;

                        // Make the static buffer mutable as per `embedded-dma` requirements.
                        // It's safe as long as the buffer isn't used until this future completes.
                        #[allow(mutable_transmutes)]
                        let rx_buf = unsafe {
                            core::mem::transmute::<&'static [u8], &'static mut [u8]>(
                                &self.buf[0..rx_data.len()],
                            )
                        };
                        let rx_transfer = i2c.with_rx_dma(dma_ch, address, true);
                        let transfer = rx_transfer.read(rx_buf);
                        InterruptFuture::new(&mut self.dma_int).await;
                        Self::handle_dma_int()?;
                        let (rx_buf, _) = transfer.destroy();
                        rx_data.copy_from_slice(rx_buf);

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
// DMA1's C2 for I2C2

impl_async_i2c_dma!(
    i2c1,
    I2C1,
    dma1impl,
    DMA1,
    C1,
    crate::interrupt::DMA1_CHANNEL1Interrupt,
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
    crate::interrupt::DMA1_CHANNEL2Interrupt,
    tcif2,
    ctcif2,
    teif2,
    cteif2
);
