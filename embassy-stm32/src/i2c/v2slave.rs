use core::result::Result;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver};
use stm32_metapac::i2c;

use super::{AddressIndex, I2c, Instance};
use crate::i2c::{Dir, Error};

pub type I2cBuffer = [u8; SLAVE_BUFFER_SIZE];
pub const SLAVE_BUFFER_SIZE: usize = 64;
pub const SLAVE_QUEUE_DEPTH: usize = 5;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(usize)]
pub enum BufferIndex {
    MasterWriteAddress1 = 0,
    MasterReadAddress1,
    MasterWriteAddress2,
    MasterReadAddress2,
}

pub struct SlaveTransaction {
    buffer: I2cBuffer,
    buffer_index: BufferIndex,
    size: u16,
    index: usize,
    address: u16,
    result: Option<Error>,
}
impl SlaveTransaction {
    fn new_write(address: AddressIndex) -> Self {
        SlaveTransaction {
            buffer: [0; SLAVE_BUFFER_SIZE],
            buffer_index: if address == AddressIndex::Address1 {
                BufferIndex::MasterWriteAddress1
            } else {
                BufferIndex::MasterWriteAddress2
            },
            size: 0,
            index: 0,
            address: 0,
            result: None,
        }
    }
    fn new_read(in_buffer: &[u8], address: AddressIndex) -> Self {
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
            buffer_index: if address == AddressIndex::Address1 {
                BufferIndex::MasterReadAddress1
            } else {
                BufferIndex::MasterReadAddress2
            },
            size: size as u16,
            index: 0,
            address: 0,
            result: None,
        }
    }

    pub fn result(&self) -> Option<Error> {
        self.result
    }
    pub fn address(&self) -> u16 {
        self.address
    }
    pub fn size(&self) -> u16 {
        self.size
    }
    pub fn index(&self) -> usize {
        self.index
    }
    // return a slice of the buffer with the correct transaction size
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[0..self.size as usize]
    }
    pub fn dir(&self) -> Dir {
        match self.buffer_index {
            BufferIndex::MasterReadAddress1 => Dir::READ,
            BufferIndex::MasterReadAddress2 => Dir::READ,
            BufferIndex::MasterWriteAddress1 => Dir::WRITE,
            BufferIndex::MasterWriteAddress2 => Dir::WRITE,
        }
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
    transactions: [Option<SlaveTransaction>; 4],
    pub(crate) slave_mode: bool,
    pub(crate) address1: u16,
    transaction_index: BufferIndex,
    error_count: usize,
}

impl SlaveState {
    pub(crate) const fn new() -> Self {
        Self {
            transactions: [None, None, None, None],
            slave_mode: false,
            address1: 0,
            transaction_index: BufferIndex::MasterWriteAddress1,
            error_count: 0,
        }
    }
    fn reset(&mut self) {
        self.error_count = 0;
        self.reset_transactions();
    }
    fn reset_transactions(&mut self) {
        self.reset_transaction(BufferIndex::MasterReadAddress1);
        self.reset_transaction(BufferIndex::MasterReadAddress2);
        self.reset_transaction(BufferIndex::MasterWriteAddress1);
        self.reset_transaction(BufferIndex::MasterWriteAddress2);
        self.prepare_write();
    }
    fn reset_transaction(&mut self, index: BufferIndex) {
        _ = self.transactions[index as usize].take();
    }
    fn take_transaction(&mut self) -> Option<SlaveTransaction> {
        self.transactions[self.transaction_index as usize].take()
    }
    fn prepare_write(&mut self) {
        if self.transactions[0].is_none() {
            _ = self.transactions[0].insert(SlaveTransaction::new_write(AddressIndex::Address1));
        }
        if self.transactions[2].is_none() {
            _ = self.transactions[2].insert(SlaveTransaction::new_write(AddressIndex::Address2));
        }
    }
    // start the transaction. Select the current transaction index based on address and dir
    // Return the size of the current transaction
    fn start_transaction(&mut self, address: u8, dir: Dir) -> u16 {
        let address16 = address as u16;
        let mut address_index = AddressIndex::Address1;
        self.transaction_index = if address16 == self.address1 {
            match dir {
                Dir::WRITE => BufferIndex::MasterWriteAddress1,
                Dir::READ => BufferIndex::MasterReadAddress1,
            }
        } else {
            address_index = AddressIndex::Address2;
            match dir {
                Dir::WRITE => BufferIndex::MasterWriteAddress2,
                Dir::READ => BufferIndex::MasterReadAddress2,
            }
        };
        let transaction = &mut self.transactions[self.transaction_index as usize];
        if transaction.is_none() {
            // transactions should be prepared outside interrupt context.
            // this is fallback code
            match dir {
                Dir::WRITE => {
                    // suboptimal, but not an error. Buffers are created in interrupt context,
                    // where it should be done in user context
                    let t = SlaveTransaction::new_write(address_index);
                    _ = transaction.insert(t);
                }
                Dir::READ => {
                    // this is a real error. Master wants to read but there is no transaction
                    // Create a dummy transaction here to contain the error
                    let buf = [0xff; 1];
                    let mut t = SlaveTransaction::new_read(&buf, address_index);
                    t.result = Some(Error::NoTransaction);
                    _ = transaction.insert(t);
                }
            }
        }
        // return the size of the transaction
        match transaction {
            Some(t) => {
                t.address = address16;
                t.size()
            }
            None => 0,
        }
    }
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
        let transaction = &mut self.transactions[self.transaction_index as usize];
        match transaction {
            Some(t) => t.result = Some(error),
            None => (),
        }
    }
    fn master_read_byte(&mut self) -> Result<u8, Error> {
        let transaction = &mut self.transactions[self.transaction_index as usize];
        match transaction {
            Some(t) => t.master_read(),
            None => {
                self.error_count += 1;
                Err(Error::NoTransaction)
            }
        }
    }
    fn master_write_byte(&mut self, b: u8) -> Result<(), Error> {
        let transaction = &mut self.transactions[self.transaction_index as usize];
        match transaction {
            Some(t) => t.master_write(b),
            None => {
                self.error_count += 1;
                Err(Error::NoTransaction)
            }
        }
    }
}

impl<'d, T: Instance, TXDMA, RXDMA> I2c<'d, T, TXDMA, RXDMA> {
    /// Starts listening for slave transactions
    pub fn slave_start_listen(&self) -> Result<(), super::Error> {
        T::regs().cr1().modify(|reg| {
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
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.reset_transactions();
            state_m.prepare_write();
            state_m.slave_mode = true;
        });
        Ok(())
    }
    // slave stop listening for slave transactions and switch back to master role
    pub fn slave_stop_listen(&self) -> Result<(), super::Error> {
        T::regs().cr1().modify(|reg| {
            reg.set_addrie(false);
            reg.set_txie(false);
            reg.set_addrie(false);
            reg.set_rxie(false);
            reg.set_nackie(false);
            reg.set_stopie(false);
            reg.set_errie(false);
            reg.set_tcie(false);
        });
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.reset_transactions();
            state_m.slave_mode = false;
        });
        Ok(())
    }

    pub fn set_address_1(&self, address7: u8) -> Result<(), Error> {
        T::regs().oar1().write(|reg| {
            reg.set_oa1en(false);
        });
        let adress_u16 = address7 as u16;
        T::regs().oar1().write(|reg| {
            reg.set_oa1(adress_u16 << 1);
            reg.set_oa1mode(i2c::vals::Addmode::BIT7);
            reg.set_oa1en(true);
        });
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.address1 = adress_u16;
        });
        Ok(())
    }
    pub fn slave_sbc(&self, sbc_enabled: bool) {
        // enable acknowlidge control
        T::regs().cr1().modify(|w| w.set_sbc(sbc_enabled));
    }
    pub fn slave_prepare_read(&self, buffer: &[u8], address: AddressIndex) -> Result<(), Error> {
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.prepare_write();
            if state_m.transactions[address as usize + 1].is_some() {
                state_m.error_count += 1;
                return Err(Error::Overrun);
            }
            _ = state_m.transactions[address as usize + 1].insert(SlaveTransaction::new_read(buffer, address));
            Ok(())
        })
    }
    pub fn slave_prepare_write(&self) -> Result<(), Error> {
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.prepare_write();
            Ok(())
        })
    }

    pub fn slave_reset(&self) {
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.reset();
        });
    }
    // will return the error count, and reset the error count
    pub fn slave_error_count(&self) -> usize {
        T::state().mutex.lock(|f| {
            let mut state_m = f.borrow_mut();
            state_m.error_count_reset()
        })
    }

    /// Get a copy of the receiver for the channel_out. User code can await on this receiver
    pub fn slave_transaction_receiver(
        &self,
    ) -> Receiver<'static, CriticalSectionRawMutex, SlaveTransaction, SLAVE_QUEUE_DEPTH> {
        T::state().channel_out.receiver()
    }
    pub(crate) fn slave_interupt_handler(state_m: &mut SlaveState, regs: &i2c::I2c) {
        // ============================================ slave interrupt state_m machine
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
            let transaction = state_m.take_transaction();
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
            let tdir = if isr.dir() as u8 == 0 { Dir::WRITE } else { Dir::READ };
            let tsize = state_m.start_transaction(taddress, tdir);

            if tdir == Dir::READ {
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
                T::regs().cr1().modify(|reg| {
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
}
