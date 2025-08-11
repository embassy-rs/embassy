//! # I2Cv1
//!
//! This implementation is used for STM32F1, STM32F2, STM32F4, and STM32L1 devices.
//!
//! All other devices (as of 2023-12-28) use [`v2`](super::v2) instead.

use core::future::poll_fn;
use core::task::Poll;

use embassy_embedded_hal::SetConfig;
use embassy_futures::select::{select, Either};
use embassy_hal_internal::drop::OnDrop;
use embedded_hal_1::i2c::Operation;
use mode::Master;

use super::*;
use crate::mode::Mode;
use crate::pac::i2c;

use embassy_sync::waitqueue::AtomicWaker;

/// I2C v2 peripheral state  
pub(crate) struct State {
    pub(crate) waker: AtomicWaker,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum SlaveSendResult {
    Acked,    // Byte sent and ACK received from master
    Nacked,   // Byte sent but NACK received (normal end of transmission)
    Stopped,  // STOP condition detected
    Restart,  // RESTART condition detected
}

#[derive(Debug, PartialEq)]
enum SlaveReceiveResult {
    Byte(u8),     // Data byte received
    Stop,         // STOP condition detected  
    Restart,      // RESTART condition (new ADDR) detected
}

// /!\                      /!\
// /!\ Implementation note! /!\
// /!\                      /!\
//
// It's somewhat unclear whether using interrupts here in a *strictly* one-shot style is actually
// what we want! If you are looking in this file because you are doing async I2C and your code is
// just totally hanging (sometimes), maybe swing by this issue:
// <https://github.com/embassy-rs/embassy/issues/2372>.
//
// There's some more details there, and we might have a fix for you. But please let us know if you
// hit a case like this!
pub unsafe fn on_interrupt<T: Instance>() {
    let regs = T::info().regs;
    trace!("i2c v1 interrupt triggered");
    // i2c v2 only woke the task on transfer complete interrupts. v1 uses interrupts for a bunch of
    // other stuff, so we wake the task on every interrupt.
    T::state().waker.wake();
    critical_section::with(|_| {
        // Clear event interrupt flag.
        regs.cr2().modify(|w| {
            w.set_itevten(false);
            w.set_iterren(false);
        });
    });
}

impl<'d, M: Mode, IM: MasterMode> I2c<'d, M, IM> {
    pub(crate) fn init(&mut self, config: Config) {
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(false);
            //reg.set_anfoff(false);
        });

        // Errata: "Start cannot be generated after a misplaced Stop"
        //
        // > If a master generates a misplaced Stop on the bus (bus error)
        // > while the microcontroller I2C peripheral attempts to switch to
        // > Master mode by setting the START bit, the Start condition is
        // > not properly generated.
        //
        // This also can occur with falsely detected STOP events, for example
        // if the SDA line is shorted to low.
        //
        // The workaround for this is to trigger the SWRST line AFTER power is
        // enabled, AFTER PE is disabled and BEFORE making any other configuration.
        //
        // It COULD be possible to apply this workaround at runtime, instead of
        // only on initialization, however this would require detecting the timeout
        // or BUSY lockup condition, and re-configuring the peripheral after reset.
        //
        // This presents as an ~infinite hang on read or write, as the START condition
        // is never generated, meaning the start event is never generated.
        self.info.regs.cr1().modify(|reg| {
            reg.set_swrst(true);
        });
        self.info.regs.cr1().modify(|reg| {
            reg.set_swrst(false);
        });

        let timings = Timings::new(self.kernel_clock, config.frequency);

        self.info.regs.cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        self.info.regs.ccr().modify(|reg| {
            reg.set_f_s(timings.f_s);
            reg.set_duty(timings.duty);
            reg.set_ccr(timings.ccr);
        });
        self.info.regs.trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(true);
        });
        trace!("i2c v1 init complete");
    }

    fn check_and_clear_error_flags(info: &'static Info) -> Result<i2c::regs::Sr1, Error> {
        // Note that flags should only be cleared once they have been registered. If flags are
        // cleared otherwise, there may be an inherent race condition and flags may be missed.
        let sr1 = info.regs.sr1().read();

        if sr1.timeout() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_timeout(false);
            });
            return Err(Error::Timeout);
        }

        if sr1.pecerr() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_pecerr(false);
            });
            return Err(Error::Crc);
        }

        if sr1.ovr() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_ovr(false);
            });
            return Err(Error::Overrun);
        }

        if sr1.af() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_af(false);
            });
            return Err(Error::Nack);
        }

        if sr1.arlo() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_arlo(false);
            });
            return Err(Error::Arbitration);
        }

        // The errata indicates that BERR may be incorrectly detected. It recommends ignoring and
        // clearing the BERR bit instead.
        if sr1.berr() {
            info.regs.sr1().write(|reg| {
                reg.0 = !0;
                reg.set_berr(false);
            });
        }

        Ok(sr1)
    }

    fn write_bytes(&mut self, address: u8, write_buffer: &[u8], timeout: Timeout, framing: OperationFraming) -> Result<(), Error> {
        if framing.send_start() {
            // Send a START condition

            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
            });

            // Wait until START condition was generated
            while !Self::check_and_clear_error_flags(self.info)?.start() {
                timeout.check()?;
            }

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr(address << 1));

            // Wait until address was sent
            // Wait for the address to be acknowledged
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            while !Self::check_and_clear_error_flags(self.info)?.addr() {
                timeout.check()?;
            }

            // Clear condition by reading SR2
            let _ = self.info.regs.sr2().read();
        }

        // Send bytes
        for c in write_buffer {
            self.send_byte(*c, timeout)?;
        }

        if framing.send_stop() {
            // Send a STOP condition
            self.info.regs.cr1().modify(|reg| reg.set_stop(true));
        }

        // Fallthrough is success
        Ok(())
    }

    fn send_byte(&self, byte: u8, timeout: Timeout) -> Result<(), Error> {
        // Wait until we're ready for sending
        while {
            // Check for any I2C errors. If a NACK occurs, the ADDR bit will never be set.
            !Self::check_and_clear_error_flags(self.info)?.txe()
        } {
            timeout.check()?;
        }

        // Push out a byte of data
        self.info.regs.dr().write(|reg| reg.set_dr(byte));

        // Wait until byte is transferred
        while {
            // Check for any potential error conditions.
            !Self::check_and_clear_error_flags(self.info)?.btf()
        } {
            timeout.check()?;
        }

        Ok(())
    }

    fn recv_byte(&self, timeout: Timeout) -> Result<u8, Error> {
        while {
            // Check for any potential error conditions.
            Self::check_and_clear_error_flags(self.info)?;

            !self.info.regs.sr1().read().rxne()
        } {
            timeout.check()?;
        }

        let value = self.info.regs.dr().read().dr();
        Ok(value)
    }

    fn blocking_read_timeout(
        &mut self,
        address: u8,
        read_buffer: &mut [u8],
        timeout: Timeout,
        framing: OperationFraming,
    ) -> Result<(), Error> {
        let Some((last_byte, read_buffer)) = read_buffer.split_last_mut() else {
            return Err(Error::Overrun);
        };

        if framing.send_start() {
            // Send a START condition and set ACK bit
            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
                reg.set_ack(true);
            });

            // Wait until START condition was generated
            while !Self::check_and_clear_error_flags(self.info)?.start() {
                timeout.check()?;
            }

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr((address << 1) + 1));

            // Wait until address was sent
            // Wait for the address to be acknowledged
            while !Self::check_and_clear_error_flags(self.info)?.addr() {
                timeout.check()?;
            }

            // Clear condition by reading SR2
            let _ = self.info.regs.sr2().read();
        }

        // Receive bytes into buffer
        for c in read_buffer {
            *c = self.recv_byte(timeout)?;
        }

        // Prepare to send NACK then STOP after next byte
        self.info.regs.cr1().modify(|reg| {
            if framing.send_nack() {
                reg.set_ack(false);
            }
            if framing.send_stop() {
                reg.set_stop(true);
            }
        });

        // Receive last byte
        *last_byte = self.recv_byte(timeout)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking read.
    pub fn blocking_read(&mut self, address: u8, read_buffer: &mut [u8]) -> Result<(), Error> {
        self.blocking_read_timeout(address, read_buffer, self.timeout(), OperationFraming::FirstAndLast)
    }

    /// Blocking write.
    pub fn blocking_write(&mut self, address: u8, write_buffer: &[u8]) -> Result<(), Error> {
        self.write_bytes(address, write_buffer, self.timeout(), OperationFraming::FirstAndLast)?;

        // Fallthrough is success
        Ok(())
    }

    /// Blocking write, restart, read.
    pub fn blocking_write_read(&mut self, address: u8, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<(), Error> {
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        let timeout = self.timeout();

        self.write_bytes(address, write_buffer, timeout, OperationFraming::First)?;
        self.blocking_read_timeout(address, read_buffer, timeout, OperationFraming::FirstAndLast)?;

        Ok(())
    }

    /// Blocking transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub fn blocking_transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        let timeout = self.timeout();

        for (op, framing) in assign_operation_framing(operations)? {
            match op {
                Operation::Read(read_buffer) => self.blocking_read_timeout(address, read_buffer, timeout, framing)?,
                Operation::Write(write_buffer) => self.write_bytes(address, write_buffer, timeout, framing)?,
            }
        }

        Ok(())
    }

    // Async

    #[inline] // pretty sure this should always be inlined
    fn enable_interrupts(info: &'static Info) -> () {
        info.regs.cr2().modify(|w| {
            w.set_iterren(true);
            w.set_itevten(true);
        });
    }
}

impl<'d, M: Mode> I2c<'d, M, Master> {
    /// Configure the I2C driver for slave operations, allowing for the driver to be used as a slave and a master (multimaster)
    pub fn into_slave_multimaster(mut self, slave_addr_config: SlaveAddrConfig) -> I2c<'d, M, MultiMaster> {
        let mut slave = I2c {
            info: self.info,
            state: self.state,
            kernel_clock: self.kernel_clock,
            tx_dma: self.tx_dma.take(),  // Use take() to move ownership
            rx_dma: self.rx_dma.take(),  // Use take() to move ownership
            #[cfg(feature = "time")]
            timeout: self.timeout,
            _phantom: PhantomData,
            _phantom2: PhantomData,
            _drop_guard: self._drop_guard,  // Move the drop guard
        };
        slave.init_slave(slave_addr_config);
        slave
    }
}

impl<'d, M: Mode, IM: MasterMode> I2c<'d, M, IM> {
    /// Slave configuration with v1 address setup
    pub(crate) fn init_slave(&mut self, config: SlaveAddrConfig) {
        trace!("i2c v1 slave init: config={:?}", config);
        
        // Disable peripheral for configuration
        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(false);
        });
        
        // Configure addresses with proper v1 format
        self.configure_addresses(config);
        
        // Configure general call if requested
        if config.general_call {
            self.info.regs.cr1().modify(|w| w.set_engc(true));
            trace!("i2c v1 slave: General call enabled");
        }
        
        // Log final configuration before enabling
        let cr1 = self.info.regs.cr1().read();
        let oar1 = self.info.regs.oar1().read();
        let oar2 = self.info.regs.oar2().read();
        trace!("i2c v1 slave: Pre-enable state - CR1={:#x}, OAR1={:#x}, OAR2={:#x}", 
               cr1.0, oar1.0, oar2.0);
        trace!("i2c v1 slave: Address details - OAR1.ADD={:#x}, OAR1.ADDMODE={}, bit14={}", 
               oar1.add(), oar1.addmode() as u8, (oar1.0 >> 14) & 1);

        self.info.regs.cr1().modify(|reg| {
            reg.set_pe(true);   // Re-enable peripheral
            reg.set_ack(true);  // Critical for slave to ACK its address
        });
        
        // Verify peripheral is enabled and ready
        let cr1_final = self.info.regs.cr1().read();
        trace!("i2c v1 slave: Final state - CR1={:#x}, PE={}", cr1_final.0, cr1_final.pe());
        
        trace!("i2c v1 slave init complete");
    }

    fn configure_oa1(&mut self, addr: Address) {
        match addr {
            Address::SevenBit(addr) => {
                trace!("i2c v1 slave: Setting OA1 7-bit address: input={:#x}", addr);
                self.info.regs.oar1().write(|reg| {
                    // For I2C v1, the 7-bit address goes in bits [7:1] of the ADD field
                    // The ADD field spans bits [9:0], so we put the address in the correct position
                    let hw_addr = (addr as u16) << 1;  // This puts address in bits [7:1], bit [0] = 0
                    reg.set_add(hw_addr);
                    reg.set_addmode(i2c::vals::Addmode::BIT7);
                });
                
                // CRITICAL: Set bit 14 as required by the reference manual
                // "Bit 14: Should always be kept at 1 by software"
                self.info.regs.oar1().modify(|reg| {
                    reg.0 |= 1 << 14;  // Set bit 14
                });
                
                let oar1_verify = self.info.regs.oar1().read();
                trace!("i2c v1 slave: OA1 configured - OAR1={:#x}, stored_addr={:#x}, bit14={}", 
                       oar1_verify.0, oar1_verify.add(), (oar1_verify.0 >> 14) & 1);
            },
            Address::TenBit(addr) => {
                trace!("i2c v1 slave: Setting OA1 10-bit address: {:#x}", addr);
                self.info.regs.oar1().write(|reg| {
                    reg.set_add(addr);  // For 10-bit, full address goes in ADD field
                    reg.set_addmode(i2c::vals::Addmode::BIT10);
                });
                
                // Set required bit 14 for 10-bit mode too
                self.info.regs.oar1().modify(|reg| {
                    reg.0 |= 1 << 14;  // Set bit 14
                });
                
                let oar1_verify = self.info.regs.oar1().read();
                trace!("i2c v1 slave: OA1 10-bit configured - OAR1={:#x}, bit14={}", 
                       oar1_verify.0, (oar1_verify.0 >> 14) & 1);
            }
        }
    }
    
    fn configure_oa2_simple(&mut self, addr: u8) {
        trace!("i2c v1 slave: Setting OA2 address: {:#x}", addr);
        self.info.regs.oar2().write(|reg| {
            // For OA2, the address goes in bits [7:1] of the ADD2 field
            reg.set_add2(addr);  // ADD2 field automatically handles bits [7:1] placement
            reg.set_endual(i2c::vals::Endual::DUAL);  // Enable dual addressing
        });
        
        let oar2_verify = self.info.regs.oar2().read();
        trace!("i2c v1 slave: OA2 configured - OAR2={:#x}, ADD2={:#x}, ENDUAL={}", 
               oar2_verify.0, oar2_verify.add2(), oar2_verify.endual() as u8);
    }

    fn configure_addresses(&mut self, config: SlaveAddrConfig) {
        match config.addr {
            OwnAddresses::OA1(addr) => {
                self.configure_oa1(addr);
                // Disable OA2 if not needed
                self.info.regs.oar2().write(|reg| {
                    reg.set_endual(i2c::vals::Endual::SINGLE);
                });
            },
            OwnAddresses::OA2(oa2) => {
                // v1 limitation: ignore mask, only support simple OA2
                if !matches!(oa2.mask, AddrMask::NOMASK) {
                    // Could log a warning here that masking is ignored in v1
                    #[cfg(feature = "defmt")]
                    warn!("I2C v1 does not support OA2 address masking, ignoring mask setting");
                }
                
                // Must have a default OA1 when using OA2-only mode
                // Set OA1 to a reserved address that won't conflict
                self.info.regs.oar1().write(|reg| {
                    reg.set_add(0);  // Address 0x00 is reserved, safe to use
                    reg.set_addmode(i2c::vals::Addmode::BIT7);
                });
                
                self.configure_oa2_simple(oa2.addr);
            },
            OwnAddresses::Both { oa1, oa2 } => {
                self.configure_oa1(oa1);
                
                // Same masking limitation applies
                if !matches!(oa2.mask, AddrMask::NOMASK) {
                    #[cfg(feature = "defmt")]
                    defmt::warn!("I2C v1 does not support OA2 address masking, ignoring mask setting");
                }
                
                self.configure_oa2_simple(oa2.addr);
            }
        }
        
        // Configure general call if requested
        if config.general_call {
            self.info.regs.cr1().modify(|w| w.set_engc(true));
        }
    }
}

impl<'d, M: Mode> I2c<'d, M, MultiMaster> {
    /// Listen for incoming I2C address match and return the command type
    pub fn blocking_listen(&mut self) -> Result<SlaveCommand, Error> {
        trace!("i2c v1 slave: blocking_listen start");
        let timeout = self.timeout(); // Get timeout internally
        let result = self.blocking_listen_timeout(timeout);
        trace!("i2c v1 slave: blocking_listen result={:?}", result);
        result
    }
    
    /// Respond to master read request (master wants to read from us)
    pub fn blocking_respond_to_read(&mut self, data: &[u8]) -> Result<usize, Error> {
        trace!("i2c v1 slave: blocking_respond_to_read start, data_len={}", data.len());
        let timeout = self.timeout(); // Get timeout internally  
        let result = self.blocking_respond_to_read_timeout(data, timeout);
        trace!("i2c v1 slave: blocking_respond_to_read result={:?}", result);
        result
    }
    
    /// Respond to master write request (master wants to write to us)
    pub fn blocking_respond_to_write(&mut self, buffer: &mut [u8]) -> Result<usize, Error> {
        trace!("i2c v1 slave: blocking_respond_to_write start, buffer_len={}", buffer.len());
        let timeout = self.timeout(); // Get timeout internally
        let result = self.blocking_respond_to_write_timeout(buffer, timeout);
        trace!("i2c v1 slave: blocking_respond_to_write result={:?}", result);
        result
    }
    
    // Private implementation methods with Timeout parameter
    fn blocking_listen_timeout(&mut self, timeout: Timeout) -> Result<SlaveCommand, Error> {
        trace!("i2c v1 slave: listen_timeout start");
        
        // Disable interrupts for blocking operation
        self.info.regs.cr2().modify(|w| {
            w.set_itevten(false);
            w.set_iterren(false);
        });
        
        // Wait for address match (ADDR flag)
        loop {
            let sr1 = Self::check_slave_error_flags_and_get_sr1(self.info)?;
            
            if sr1.addr() {
                // Address matched! Read SR2 to get direction and clear ADDR
                let sr2 = self.info.regs.sr2().read();
                let direction = if sr2.tra() {
                    trace!("i2c v1 slave: address match - READ direction");
                    SlaveCommandKind::Read
                } else {
                    trace!("i2c v1 slave: address match - WRITE direction");
                    SlaveCommandKind::Write
                };
                
                // Determine which address was matched
                let matched_address = self.determine_matched_address(sr2)?;
                trace!("i2c v1 slave: matched address={:?}", matched_address);
                
                // ADDR is automatically cleared by reading SR1 then SR2
                return Ok(SlaveCommand {
                    kind: direction,
                    address: matched_address,
                });
            }
            
            timeout.check()?;
        }
    }
    
    fn blocking_respond_to_read_timeout(&mut self, data: &[u8], timeout: Timeout) -> Result<usize, Error> {
        trace!("i2c v1 slave: respond_to_read_timeout start, data_len={}", data.len());
        let mut bytes_sent = 0;
        
        for &byte in data {
            trace!("i2c v1 slave: sending byte={:#x} ({})", byte, bytes_sent);
            match self.send_byte_or_nack(byte, timeout)? {
                SlaveSendResult::Acked => {
                    bytes_sent += 1;
                    trace!("i2c v1 slave: byte acked, total_sent={}", bytes_sent);
                    // Continue sending
                },
                SlaveSendResult::Nacked => {
                    bytes_sent += 1; // Count the NACKed byte as sent
                    trace!("i2c v1 slave: byte nacked by master (normal completion), total_sent={}", bytes_sent);
                    break; // Normal end of transmission
                },
                SlaveSendResult::Stopped => {
                    trace!("i2c v1 slave: stop condition detected, stopping transmission");
                    break; // Master sent STOP
                }
                SlaveSendResult::Restart => {
                    trace!("i2c v1 slave: restart detected, stopping transmission");
                    break; // Master sent RESTART
                }
            }
        }
        
        trace!("i2c v1 slave: respond_to_read_timeout complete, bytes_sent={}", bytes_sent);
        Ok(bytes_sent) // Always return success with byte count
    }
    
    fn blocking_respond_to_write_timeout(&mut self, buffer: &mut [u8], timeout: Timeout) -> Result<usize, Error> {
        trace!("i2c v1 slave: respond_to_write_timeout start, buffer_len={}", buffer.len());
        let mut bytes_received = 0;
        while bytes_received < buffer.len() {
            match self.recv_byte_or_stop(timeout)? {
                SlaveReceiveResult::Byte(b) => {
                    trace!("i2c v1 slave: received byte={:#x} ({})", b, bytes_received);
                    buffer[bytes_received] = b;
                    bytes_received += 1;
                },
                SlaveReceiveResult::Stop => {
                    trace!("i2c v1 slave: stop condition detected, stopping reception");
                    break;
                },
                SlaveReceiveResult::Restart => {
                    trace!("i2c v1 slave: restart detected, stopping reception");
                    break;
                },
            }
        }
        trace!("i2c v1 slave: respond_to_write_timeout complete, bytes_received={}", bytes_received);
        Ok(bytes_received)
    }
    
    fn determine_matched_address(&self, sr2: stm32_metapac::i2c::regs::Sr2) -> Result<Address, Error> {
        trace!("i2c v1 slave: determine_matched_address, sr2={:#x}", sr2.0);
        // Check for general call first
        if sr2.gencall() {
            trace!("i2c v1 slave: general call address matched");
            Ok(Address::SevenBit(0x00))
        } else if sr2.dualf() {
            // OA2 was matched - verify it's actually enabled
            let oar2 = self.info.regs.oar2().read();
            if oar2.endual() != i2c::vals::Endual::DUAL {
                error!("i2c v1 slave: OA2 matched but not enabled - hardware inconsistency");
                return Err(Error::Bus); // Hardware inconsistency
            }
            trace!("i2c v1 slave: OA2 address matched: {:#x}", oar2.add2());
            Ok(Address::SevenBit(oar2.add2()))
        } else {
            // OA1 was matched
            let oar1 = self.info.regs.oar1().read();
            match oar1.addmode() {
                i2c::vals::Addmode::BIT7 => {
                    let addr = (oar1.add() >> 1) as u8;
                    trace!("i2c v1 slave: OA1 7-bit address matched: {:#x}", addr);
                    Ok(Address::SevenBit(addr))
                },
                i2c::vals::Addmode::BIT10 => {
                    trace!("i2c v1 slave: OA1 10-bit address matched: {:#x}", oar1.add());
                    Ok(Address::TenBit(oar1.add()))
                },
            }
        }
    }

    /// Send a byte in slave transmitter mode and check for ACK/NACK/STOP
    fn send_byte_or_nack(&mut self, byte: u8, timeout: Timeout) -> Result<SlaveSendResult, Error> {
        trace!("i2c v1 slave: send_byte_or_nack start, byte={:#x}", byte);
        
        // Wait until we're ready for sending (TXE flag set)
        loop {
            let sr1 = Self::check_slave_error_flags_and_get_sr1(self.info)?;
            
            // Check for STOP condition first
            if sr1.stopf() {
                trace!("i2c v1 slave: STOP detected before send");
                self.info.regs.cr1().modify(|_w| {});
                return Ok(SlaveSendResult::Stopped);
            }
            
            // Check for RESTART (new ADDR)
            if sr1.addr() {
                trace!("i2c v1 slave: RESTART detected before send");
                return Ok(SlaveSendResult::Restart);
            }
            
            // Check for NACK (AF flag) before writing
            let sr1_current = self.info.regs.sr1().read();
            if sr1_current.af() {
                trace!("i2c v1 slave: NACK detected before send");
                self.info.regs.sr1().write(|reg| {
                    reg.0 = !0;
                    reg.set_af(false);
                });
                return Ok(SlaveSendResult::Nacked);
            }
            
            // Check if we can send data
            if sr1.txe() {
                trace!("i2c v1 slave: TXE ready, sending byte");
                break; // Ready to send
            }
            
            timeout.check()?;
        }
        
        // Send the byte
        self.info.regs.dr().write(|w| w.set_dr(byte));
        trace!("i2c v1 slave: byte written to DR, waiting for completion");
        
        // Wait for completion - but be more flexible about what constitutes "completion"
        // In slave transmitter mode, we need to detect:
        // 1. BTF - byte transfer finished (normal case)
        // 2. AF (NACK) - master signals end of transaction
        // 3. STOP - master terminates transaction
        // 4. ADDR - master starts new transaction (restart)
        loop {
            // Get current flags without error handling that clears AF
            let sr1 = self.info.regs.sr1().read();
            
            // Check for NACK FIRST - this is the most likely end condition
            if sr1.af() {
                trace!("i2c v1 slave: NACK detected after send");
                // Clear the AF flag
                self.info.regs.sr1().write(|reg| {
                    reg.0 = !0;
                    reg.set_af(false);
                });
                return Ok(SlaveSendResult::Nacked);
            }
            
            // Check for STOP condition
            if sr1.stopf() {
                trace!("i2c v1 slave: STOP detected after send");
                self.info.regs.cr1().modify(|_w| {});
                return Ok(SlaveSendResult::Stopped);
            }
            
            // Check for RESTART (new ADDR)  
            if sr1.addr() {
                trace!("i2c v1 slave: RESTART detected after send");
                return Ok(SlaveSendResult::Restart);
            }
            
            // Check for byte transfer finished (normal ACK case)
            if sr1.btf() {
                trace!("i2c v1 slave: BTF set, byte transfer complete (ACK)");
                return Ok(SlaveSendResult::Acked);
            }
            
            // Check for other error conditions that should be propagated
            if sr1.timeout() || sr1.ovr() || sr1.arlo() || sr1.berr() {
                // Use the error handling function for these
                match Self::check_and_clear_error_flags(self.info) {
                    Ok(_) => {}, // Shouldn't happen given the flags we checked
                    Err(e) => return Err(e),
                }
            }
            
            timeout.check()?;
        }
    }
    
    /// Receive a byte in slave receiver mode or detect STOP condition
    fn recv_byte_or_stop(&mut self, timeout: Timeout) -> Result<SlaveReceiveResult, Error> {
        trace!("i2c v1 slave: recv_byte_or_stop start");
        loop {
            let sr1 = Self::check_slave_error_flags_and_get_sr1(self.info)?;
            
            // Check for received data FIRST (handles race condition)
            if sr1.rxne() {
                let byte = self.info.regs.dr().read().dr();
                trace!("i2c v1 slave: received byte={:#x}", byte);
                return Ok(SlaveReceiveResult::Byte(byte));
            }
            
            // Check for RESTART (new ADDR) before STOP
            if sr1.addr() {
                trace!("i2c v1 slave: RESTART detected during receive");
                return Ok(SlaveReceiveResult::Restart);
            }
            
            // Check for STOP condition LAST
            if sr1.stopf() {
                trace!("i2c v1 slave: STOP detected during receive");
                self.info.regs.cr1().modify(|_w| {});
                return Ok(SlaveReceiveResult::Stop);
            }
            
            timeout.check()?;
        }
    }

    /// Wrapper that treats AF (NACK) as normal protocol behavior in slave mode
    fn check_slave_error_flags_and_get_sr1(info: &'static Info) -> Result<i2c::regs::Sr1, Error> {
        match Self::check_and_clear_error_flags(info) {
            Ok(sr1) => Ok(sr1),
            Err(Error::Nack) => {
                // AF flag was set and cleared by check_and_clear_error_flags
                // In slave mode, this is normal protocol behavior, not an error
                // Read SR1 again to get current state (AF should now be cleared)
                Ok(info.regs.sr1().read())
            },
            Err(other_error) => Err(other_error), // Propagate real errors
        }
    }
}

impl<'d, IM: MasterMode> I2c<'d, Async, IM> {
    async fn write_with_framing(&mut self, address: u8, write_buffer: &[u8], framing: OperationFraming) -> Result<(), Error> {
        self.info.regs.cr2().modify(|w| {
            // Note: Do not enable the ITBUFEN bit in the I2C_CR2 register if DMA is used for
            // reception.
            w.set_itbufen(false);
            // DMA mode can be enabled for transmission by setting the DMAEN bit in the I2C_CR2
            // register.
            w.set_dmaen(true);
            // Sending NACK is not necessary (nor possible) for write transfer.
            w.set_last(false);
        });

        // Sentinel to disable transfer when an error occurs or future is canceled.
        // TODO: Generate STOP condition on cancel?
        let on_drop = OnDrop::new(|| {
            self.info.regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            })
        });

        if framing.send_start() {
            // Send a START condition
            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
            });

            // Wait until START condition was generated
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.start() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr(address << 1));

            // Wait for the address to be acknowledged
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.addr() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // Clear condition by reading SR2
            self.info.regs.sr2().read();
        }

        let dma_transfer = unsafe {
            // Set the I2C_DR register address in the DMA_SxPAR register. The data will be moved to
            // this address from the memory after each TxE event.
            let dst = self.info.regs.dr().as_ptr() as *mut u8;

            self.tx_dma.as_mut().unwrap().write(write_buffer, dst, Default::default())
        };

        // Wait for bytes to be sent, or an error to occur.
        let poll_error = poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(self.info) {
                Err(e) => Poll::Ready(Err::<(), Error>(e)),
                Ok(_) => {
                    // When pending, (re-)enable interrupts to wake us up.
                    Self::enable_interrupts(self.info);
                    Poll::Pending
                }
            }
        });

        // Wait for either the DMA transfer to successfully finish, or an I2C error to occur.
        match select(dma_transfer, poll_error).await {
            Either::Second(Err(e)) => Err(e),
            _ => Ok(()),
        }?;

        self.info.regs.cr2().modify(|w| {
            w.set_dmaen(false);
        });

        if framing.send_stop() {
            // The I2C transfer itself will take longer than the DMA transfer, so wait for that to finish too.

            // 18.3.8 “Master transmitter: In the interrupt routine after the EOT interrupt, disable DMA
            // requests then wait for a BTF event before programming the Stop condition.”
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.btf() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            self.info.regs.cr1().modify(|w| {
                w.set_stop(true);
            });
        }

        drop(on_drop);

        // Fallthrough is success
        Ok(())
    }

    /// Write.
    pub async fn write(&mut self, address: u8, write_buffer: &[u8]) -> Result<(), Error> {
        self.write_with_framing(address, write_buffer, OperationFraming::FirstAndLast)
            .await?;

        Ok(())
    }

    /// Read.
    pub async fn read(&mut self, address: u8, read_buffer: &mut [u8]) -> Result<(), Error> {
        self.read_with_framing(address, read_buffer, OperationFraming::FirstAndLast)
            .await?;

        Ok(())
    }

    async fn read_with_framing(&mut self, address: u8, read_buffer: &mut [u8], framing: OperationFraming) -> Result<(), Error> {
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        // Some branches below depend on whether the buffer contains only a single byte.
        let single_byte = read_buffer.len() == 1;

        self.info.regs.cr2().modify(|w| {
            // Note: Do not enable the ITBUFEN bit in the I2C_CR2 register if DMA is used for
            // reception.
            w.set_itbufen(false);
            // DMA mode can be enabled for transmission by setting the DMAEN bit in the I2C_CR2
            // register.
            w.set_dmaen(true);
            // If, in the I2C_CR2 register, the LAST bit is set, I2C automatically sends a NACK
            // after the next byte following EOT_1. The user can generate a Stop condition in
            // the DMA Transfer Complete interrupt routine if enabled.
            w.set_last(framing.send_nack() && !single_byte);
        });

        // Sentinel to disable transfer when an error occurs or future is canceled.
        // TODO: Generate STOP condition on cancel?
        let on_drop = OnDrop::new(|| {
            self.info.regs.cr2().modify(|w| {
                w.set_dmaen(false);
                w.set_iterren(false);
                w.set_itevten(false);
            })
        });

        if framing.send_start() {
            // Send a START condition and set ACK bit
            self.info.regs.cr1().modify(|reg| {
                reg.set_start(true);
                reg.set_ack(true);
            });

            // Wait until START condition was generated
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.start() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // Check if we were the ones to generate START
            if self.info.regs.cr1().read().start() || !self.info.regs.sr2().read().msl() {
                return Err(Error::Arbitration);
            }

            // Set up current address we're trying to talk to
            self.info.regs.dr().write(|reg| reg.set_dr((address << 1) + 1));

            // Wait for the address to be acknowledged
            poll_fn(|cx| {
                self.state.waker.register(cx.waker());

                match Self::check_and_clear_error_flags(self.info) {
                    Err(e) => Poll::Ready(Err(e)),
                    Ok(sr1) => {
                        if sr1.addr() {
                            Poll::Ready(Ok(()))
                        } else {
                            // When pending, (re-)enable interrupts to wake us up.
                            Self::enable_interrupts(self.info);
                            Poll::Pending
                        }
                    }
                }
            })
            .await?;

            // 18.3.8: When a single byte must be received: the NACK must be programmed during EV6
            // event, i.e. program ACK=0 when ADDR=1, before clearing ADDR flag.
            if framing.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }

            // Clear condition by reading SR2
            self.info.regs.sr2().read();
        } else {
            // Before starting reception of single byte (but without START condition, i.e. in case
            // of merged operations), program NACK to emit at end of this byte.
            if framing.send_nack() && single_byte {
                self.info.regs.cr1().modify(|w| {
                    w.set_ack(false);
                });
            }
        }

        // 18.3.8: When a single byte must be received: [snip] Then the user can program the STOP
        // condition either after clearing ADDR flag, or in the DMA Transfer Complete interrupt
        // routine.
        if framing.send_stop() && single_byte {
            self.info.regs.cr1().modify(|w| {
                w.set_stop(true);
            });
        }

        let dma_transfer = unsafe {
            // Set the I2C_DR register address in the DMA_SxPAR register. The data will be moved
            // from this address from the memory after each RxE event.
            let src = self.info.regs.dr().as_ptr() as *mut u8;

            self.rx_dma.as_mut().unwrap().read(src, read_buffer, Default::default())
        };

        // Wait for bytes to be received, or an error to occur.
        let poll_error = poll_fn(|cx| {
            self.state.waker.register(cx.waker());

            match Self::check_and_clear_error_flags(self.info) {
                Err(e) => Poll::Ready(Err::<(), Error>(e)),
                _ => {
                    // When pending, (re-)enable interrupts to wake us up.
                    Self::enable_interrupts(self.info);
                    Poll::Pending
                }
            }
        });

        match select(dma_transfer, poll_error).await {
            Either::Second(Err(e)) => Err(e),
            _ => Ok(()),
        }?;

        self.info.regs.cr2().modify(|w| {
            w.set_dmaen(false);
        });

        if framing.send_stop() && !single_byte {
            self.info.regs.cr1().modify(|w| {
                w.set_stop(true);
            });
        }

        drop(on_drop);

        // Fallthrough is success
        Ok(())
    }

    /// Write, restart, read.
    pub async fn write_read(&mut self, address: u8, write_buffer: &[u8], read_buffer: &mut [u8]) -> Result<(), Error> {
        // Check empty read buffer before starting transaction. Otherwise, we would not generate the
        // stop condition below.
        if read_buffer.is_empty() {
            return Err(Error::Overrun);
        }

        self.write_with_framing(address, write_buffer, OperationFraming::First).await?;
        self.read_with_framing(address, read_buffer, OperationFraming::FirstAndLast).await
    }

    /// Transaction with operations.
    ///
    /// Consecutive operations of same type are merged. See [transaction contract] for details.
    ///
    /// [transaction contract]: embedded_hal_1::i2c::I2c::transaction
    pub async fn transaction(&mut self, address: u8, operations: &mut [Operation<'_>]) -> Result<(), Error> {
        for (op, framing) in assign_operation_framing(operations)? {
            match op {
                Operation::Read(read_buffer) => self.read_with_framing(address, read_buffer, framing).await?,
                Operation::Write(write_buffer) => self.write_with_framing(address, write_buffer, framing).await?,
            }
        }

        Ok(())
    }
}


/// Timing configuration for I2C v1 hardware
/// 
/// This struct encapsulates the complex timing calculations required for STM32 I2C v1 
/// peripherals, which use three separate registers (CR2.FREQ, CCR, TRISE) instead of 
/// the unified TIMINGR register found in v2 hardware.
struct Timings {
    freq: u8,                   // APB frequency in MHz for CR2.FREQ register
    f_s: i2c::vals::FS,         // Standard or Fast mode selection  
    trise: u8,                  // Rise time compensation value
    ccr: u16,                   // Clock control register value
    duty: i2c::vals::Duty,      // Fast mode duty cycle selection
}

impl Timings {
    fn new(i2cclk: Hertz, frequency: Hertz) -> Self {
        // Calculate settings for I2C speed modes
        let frequency = frequency.0;
        let clock = i2cclk.0;
        let freq = clock / 1_000_000;
        assert!((2..=50).contains(&freq));

        // Configure bus frequency into I2C peripheral
        let trise = if frequency <= 100_000 {
            freq + 1
        } else {
            (freq * 300) / 1000 + 1
        };

        let mut ccr;
        let duty;
        let f_s;

        // I2C clock control calculation
        if frequency <= 100_000 {
            duty = i2c::vals::Duty::DUTY2_1;
            f_s = i2c::vals::FS::STANDARD;
            ccr = {
                let ccr = clock / (frequency * 2);
                if ccr < 4 {
                    4
                } else {
                    ccr
                }
            };
        } else {
            const DUTYCYCLE: u8 = 0;
            f_s = i2c::vals::FS::FAST;
            if DUTYCYCLE == 0 {
                duty = i2c::vals::Duty::DUTY2_1;
                ccr = clock / (frequency * 3);
                ccr = if ccr < 1 { 1 } else { ccr };
            } else {
                duty = i2c::vals::Duty::DUTY16_9;
                ccr = clock / (frequency * 25);
                ccr = if ccr < 1 { 1 } else { ccr };
            }
        }

        Self {
            freq: freq as u8,
            f_s,
            trise: trise as u8,
            ccr: ccr as u16,
            duty,
        }
    }
}

impl<'d, M: Mode> SetConfig for I2c<'d, M, Master> {
    type Config = Hertz;
    type ConfigError = ();
    fn set_config(&mut self, config: &Self::Config) -> Result<(), ()> {
        let timings = Timings::new(self.kernel_clock, *config);
        self.info.regs.cr2().modify(|reg| {
            reg.set_freq(timings.freq);
        });
        self.info.regs.ccr().modify(|reg| {
            reg.set_f_s(timings.f_s);
            reg.set_duty(timings.duty);
            reg.set_ccr(timings.ccr);
        });
        self.info.regs.trise().modify(|reg| {
            reg.set_trise(timings.trise);
        });

        Ok(())
    }
}
