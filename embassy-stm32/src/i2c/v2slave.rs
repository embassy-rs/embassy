use core::result::Result;

use crate::i2c::Error;
use crate::mode::Mode;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use stm32_metapac::i2c;

use super::{AddressIndex, I2c, Instance};
use crate::i2c::Dir;

/// buffer used for the transactions
pub type I2cBuffer = [u8; SLAVE_BUFFER_SIZE];
/// The max size of the i2c slave buffer
pub const SLAVE_BUFFER_SIZE: usize = 64;
/// The amount of transactions, which can be buffered at the output of the driver
pub const SLAVE_QUEUE_DEPTH: usize = 5;

/// SlaveTransaction contains all the details of the i2c transaction. Created in user contex,
/// Ownership is transferred to the driver.
/// At the moment the transaction is finished the ownership is via a channel returned to the user
pub struct SlaveTransaction {
    buffer: I2cBuffer,
    dir: Dir,
    size: u16,
    index: usize,
    address: u16,
    result: Result<(), Error>,
}
impl SlaveTransaction {
    fn new_write() -> Self {
        SlaveTransaction {
            buffer: [0; SLAVE_BUFFER_SIZE],
            size: 0,
            index: 0,
            address: 0,
            result: Ok(()),
            dir: Dir::Write,
        }
    }
    fn new_read(in_buffer: &[u8]) -> Self {
        let mut buffer = [0; SLAVE_BUFFER_SIZE];
        let size = if in_buffer.len() < SLAVE_BUFFER_SIZE {
            in_buffer.len()
        } else {
            SLAVE_BUFFER_SIZE
        };
        for i in 0..size {
            buffer[i] = in_buffer[i];
        }
        SlaveTransaction {
            buffer,
            size: size as u16,
            index: 0,
            address: 0,
            result: Ok(()),
            dir: Dir::Read,
        }
    }
    /// size of transaction
    pub fn size(&self) -> u16 {
        self.size
    }
    /// current buffer index
    pub fn index(&self) -> usize {
        self.index
    }
    /// return the result of the transaction
    pub fn result(&self) -> Result<(), Error> {
        self.result
    }

    /// return a slice of the buffer with the correct transaction size
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[0..self.size as usize]
    }
    /// return direction of transaction
    pub fn dir(&self) -> Dir {
        self.dir
    }
    /// the actual address used for this transaction (known at the moment the transaction starts)
    pub fn address(&self) -> u16 {
        self.address
    }
    /// set the actual address at the start of the transaction
    pub fn set_address(&mut self, address16: u16) {
        self.address = address16
    }

    /// master read slave write scenario. Master can read until self.size bytes
    /// If no data available (self.size == 0)
    fn master_read(&mut self) -> Result<u8, Error> {
        if self.size == 0 {
            return Err(Error::ZeroLengthTransfer);
        };
        if self.index < self.size as usize {
            let b = self.buffer[self.index];
            self.index += 1;
            Ok(b)
        } else {
            self.index += 1;
            Err(Error::Overrun) // too many bytes asked
        }
    }
    /// master write slave read scenario. Master can write until buffer full
    fn master_write(&mut self, b: u8) -> Result<(), Error> {
        if self.index < SLAVE_BUFFER_SIZE {
            self.buffer[self.index] = b;
            self.index += 1;
            self.size = self.index as u16;
            Ok(())
        } else {
            self.index += 1;
            Err(Error::Overrun)
        }
    }
}

pub(crate) struct SlaveState {
    write_transactions: [Option<SlaveTransaction>; 2],
    read_transactions: [Option<SlaveTransaction>; 2],
    current_transaction: Option<SlaveTransaction>,
    pub(crate) slave_mode: bool,
    pub(crate) address1: u16,
    error_count: usize,
}

impl SlaveState {
    pub(crate) const fn new() -> Self {
        Self {
            write_transactions: [None, None],
            read_transactions: [None, None],
            current_transaction: None,
            slave_mode: false,
            address1: 0,
            error_count: 0,
        }
    }
    fn reset(&mut self) {
        self.error_count = 0;
        self.reset_transactions();
    }
    /// reset all internal transactions
    fn reset_transactions(&mut self) {
        self.write_transactions[0] = Some(SlaveTransaction::new_write());
        self.write_transactions[1] = Some(SlaveTransaction::new_write());
        self.reset_read_transactions();
    }
    /// reset read internal transactions
    fn reset_read_transactions(&mut self) {
        self.read_transactions[0].take();
        self.read_transactions[1].take();
    }
    /// fill internal write transaction, in case of empty
    fn prepare_write_transactions(&mut self) {
        for i in 0..2 {
            if self.write_transactions[i].is_none() {
                self.write_transactions[i] = Some(SlaveTransaction::new_write());
            }
        }
    }
    // start the transaction. Select the current transaction index based on address and dir
    // Return the size of the current transaction
    fn start_transaction(&mut self, address: u8, dir: Dir) -> u16 {
        let address16 = address as u16;

        let address_index = if address16 == self.address1() {
            AddressIndex::Address1
        } else {
            AddressIndex::Address2
        };

        self.current_transaction = match dir {
            Dir::Write => self.write_transactions[address_index as usize].take(),
            Dir::Read => self.read_transactions[address_index as usize].take(),
        };
        if self.current_transaction.is_none() {
            // transactions should be prepared outside interrupt context.
            match dir {
                Dir::Write => {
                    // suboptimal, but not an error. Buffers are created in interrupt context,
                    // where it should be done in user context
                    self.current_transaction = Some(SlaveTransaction::new_write());
                }
                Dir::Read => {
                    // this is a real error. Master wants to read but there is no transaction
                    // Create a dummy transaction here to contain the error
                    let buf = [0xff; 1];
                    let mut t = SlaveTransaction::new_read(&buf);
                    t.result = Err(Error::NoTransaction);
                    self.current_transaction = Some(t);
                }
            }
        }
        // return the size of the transaction
        match &mut self.current_transaction {
            Some(t) => {
                t.set_address(address16);
                t.size()
            }
            None => 0,
        }
    }
    // finish the transaction. Take ownership from the internal current transaction, and return it to the user
    fn finish_transaction(&mut self) -> Option<SlaveTransaction> {
        self.current_transaction.take()
    }
    /// return the address1 of the  transaction
    pub fn address1(&self) -> u16 {
        self.address1
    }
    // return the error count, then reset
    fn error_count_reset(&mut self) -> usize {
        let result = self.error_count;
        self.error_count = 0;
        result
    }
    fn set_error(&mut self, error: Error) {
        if let Some(t) = &mut self.current_transaction {
            t.result = Err(error)
        }
    }
    fn master_read_byte(&mut self) -> Result<u8, Error> {
        match &mut self.current_transaction {
            Some(t) => t.master_read(),
            None => {
                self.error_count += 1;
                Err(Error::NoTransaction)
            }
        }
    }
    fn master_write_byte(&mut self, b: u8) -> Result<(), Error> {
        match &mut self.current_transaction {
            Some(t) => t.master_write(b),
            None => {
                self.error_count += 1;
                Err(Error::NoTransaction)
            }
        }
    }
}

impl<'d, M: Mode> I2c<'d, M> {
    /// Starts listening for slave transactions
    pub fn slave_start_listen(&self) -> Result<(), Error> {
        trace!("slave start listen");
        self.info.regs.cr1().modify(|reg| {
            reg.set_addrie(true);
            reg.set_txie(true);
            reg.set_addrie(true);
            reg.set_rxie(true);
            reg.set_nackie(true);
            reg.set_stopie(true);
            reg.set_errie(true);
            reg.set_tcie(true);
            reg.set_sbc(true);
        });
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.reset_transactions();
            state_m.slave_mode = true;
        });
        Ok(())
    }
    /// slave stop listening for slave transactions and switch back to master role
    pub fn slave_stop_listen(&self) -> Result<(), Error> {
        self.info.regs.cr1().modify(|reg| {
            reg.set_addrie(false);
            reg.set_txie(false);
            reg.set_addrie(false);
            reg.set_rxie(false);
            reg.set_nackie(false);
            reg.set_stopie(false);
            reg.set_errie(false);
            reg.set_tcie(false);
        });
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.reset_transactions();
            state_m.slave_mode = false;
        });
        Ok(())
    }

    /// Set the slave address 1 and enable it
    pub fn set_address_1(&self, address7: u8) -> Result<(), Error> {
        self.info.regs.oar1().write(|reg| {
            reg.set_oa1en(false);
        });
        let adress_u16 = address7 as u16;
        self.info.regs.oar1().write(|reg| {
            reg.set_oa1(adress_u16 << 1);
            reg.set_oa1mode(i2c::vals::Addmode::BIT7);
            reg.set_oa1en(true);
        });
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.address1 = adress_u16;
        });
        Ok(())
    }

    /// enable ackniowlidge control
    pub fn slave_sbc(&self, sbc_enabled: bool) {
        // enable acknowlidge control
        self.info.regs.cr1().modify(|w| w.set_sbc(sbc_enabled));
    }
    /// Create a read transaction and activate this in the driver
    /// If a read transaction is pending, return error Error::Overrun
    pub fn slave_prepare_read(&self, buffer: &[u8], address: AddressIndex) -> Result<(), Error> {
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            if state_m.read_transactions[address as usize].is_some() {
                state_m.error_count += 1;
                return Err(Error::Overrun);
            }
            state_m.read_transactions[address as usize] = Some(SlaveTransaction::new_read(buffer));
            Ok(())
        })
    }
    /// Prepare a write transaction and activate this in the driver
    pub fn slave_prepare_write(&self) -> Result<(), Error> {
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.prepare_write_transactions();
            Ok(())
        })
    }
    /// reset all state in the driver
    pub fn slave_reset(&self) {
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.reset();
        });
    }
    /// Return the error count, and reset the error count
    pub fn slave_error_count(&self) -> usize {
        self.state.mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.error_count_reset()
        })
    }

    /// wait async for the next transaction to complete
    /// This is the simple way to wait for the next transaction, but ownership of the driver is needed
    pub async fn next_transaction(&self) -> SlaveTransaction {
        self.state.channel_out.receive().await
    }
    /// more sophisticated way of waiting for the next transaction. No ownership needed of the driver, as the user
    /// gets a copy of the receiver
    pub fn transaction_receiver(&self) -> Receiver<CriticalSectionRawMutex, SlaveTransaction, SLAVE_QUEUE_DEPTH> {
        self.state.channel_out.receiver()
    }
}

pub(crate) fn on_interrupt<T: Instance>(state_m: &mut SlaveState) {
    // ============================================ slave interrupt state_m machine
    let regs = T::info().regs;
    let isr = regs.isr().read();

    if isr.berr() {
        regs.icr().modify(|w| {
            w.set_berrcf(true);
        });
        state_m.set_error(Error::Bus);
    } else if isr.arlo() {
        state_m.set_error(Error::Arbitration);
        regs.icr().write(|w| w.set_arlocf(true));
    } else if isr.nackf() {
        regs.icr().write(|w| w.set_nackcf(true));
    } else if isr.txis() {
        // send the next byte to the master, or NACK in case of error, then end transaction
        let b = match state_m.master_read_byte() {
            Ok(b) => b,
            Err(e) => {
                // An extra interrupt after the last byte is sent seems to be generated always
                // Do not generate an error in this (overrun) case
                match e {
                    Error::Overrun => (),
                    _ => {
                        state_m.set_error(e);
                    }
                }
                0xFF
            }
        };
        regs.txdr().write(|w| w.set_txdata(b));
    } else if isr.rxne() {
        let b = regs.rxdr().read().rxdata();
        // byte is received from master. Store in buffer. In case of error send NACK, then end transaction
        match state_m.master_write_byte(b) {
            Ok(()) => (),
            Err(e) => {
                state_m.set_error(e);
            }
        }
    } else if isr.stopf() {
        // take the ownership out of the i2c device driver and transfer it to the queue
        // note that in case the queue is full, the transaction is silently dropped
        // the error count is increased in this case
        let transaction = state_m.finish_transaction();
        match transaction {
            Some(t) => {
                if let Err(_) = T::state().channel_out.try_send(t) {
                    state_m.error_count += 1;
                }
            }
            _ => state_m.error_count += 1,
        };
        // Clear the stop condition flag
        regs.icr().write(|w| w.set_stopcf(true));
    } else if isr.tcr() {
        // This condition Will only happen when reload == 1 and sbr == 1 (slave) and nbytes was written.
        // Send a NACK, set nbytes to clear tcr flag
        regs.cr2().modify(|w| {
            w.set_nack(true);
        });
        // Make one extra loop here to wait on the stop condition
    } else if isr.addr() {
        // handle the slave is addressed case, first step in the transaction
        let taddress = isr.addcode();
        let tdir = if isr.dir() as u8 == 0 { Dir::Write } else { Dir::Read };
        let tsize = state_m.start_transaction(taddress, tdir);

        if tdir == Dir::Read {
            // flush i2c tx register
            regs.isr().write(|w| w.set_txe(true));

            // Set the nbytes START and prepare to receive bytes into `buffer`.
            // Set the actual number of bytes to transfer
            // error case that n = 0  cannot be handled by i2c, we need to send at least 1 byte.
            let (b, size) = match state_m.master_read_byte() {
                Ok(b) => (b, tsize),
                _ => (0xFF, 1),
            };
            regs.cr2().modify(|w| {
                w.set_nbytes(size as u8);
                // during sending nbytes automatically send a ACK, stretch clock after last byte
                w.set_reload(i2c::vals::Reload::COMPLETED);
            });
            regs.txdr().write(|w| w.set_txdata(b));
            // restore sbc after a master_write_read transaction
            regs.cr1().modify(|reg| {
                reg.set_sbc(true);
            });
        } else {
            // Set the nbytes to the maximum buffer size and wait for the bytes from the master
            regs.cr2().modify(|w| {
                w.set_nbytes(SLAVE_BUFFER_SIZE as u8);
                w.set_reload(i2c::vals::Reload::COMPLETED)
            });
            // flush the rx data register
            if regs.isr().read().rxne() {
                _ = regs.rxdr().read().rxdata();
            }
        }
        // end address phase, release clock stretching
        regs.icr().write(|w| w.set_addrcf(true));
    }
}
