//! Instructions controlling the PIO.
use pio::{InSource, InstructionOperands, JmpCondition, OutDestination, SetDestination};

use crate::pio::{Instance, StateMachine};

impl<'d, PIO: Instance, const SM: usize> StateMachine<'d, PIO, SM> {
    /// Set value of scratch register X.
    pub unsafe fn set_x(&mut self, value: u32) {
        const OUT: u16 = InstructionOperands::OUT {
            destination: OutDestination::X,
            bit_count: 32,
        }
        .encode();
        self.tx().push(value);
        self.exec_instr(OUT);
    }

    /// Get value of scratch register X.
    pub unsafe fn get_x(&mut self) -> u32 {
        const IN: u16 = InstructionOperands::IN {
            source: InSource::X,
            bit_count: 32,
        }
        .encode();
        self.exec_instr(IN);
        self.rx().pull()
    }

    /// Set value of scratch register Y.
    pub unsafe fn set_y(&mut self, value: u32) {
        const OUT: u16 = InstructionOperands::OUT {
            destination: OutDestination::Y,
            bit_count: 32,
        }
        .encode();
        self.tx().push(value);
        self.exec_instr(OUT);
    }

    /// Get value of scratch register Y.
    pub unsafe fn get_y(&mut self) -> u32 {
        const IN: u16 = InstructionOperands::IN {
            source: InSource::Y,
            bit_count: 32,
        }
        .encode();
        self.exec_instr(IN);

        self.rx().pull()
    }

    /// Set instruction for pindir destination.
    pub unsafe fn set_pindir(&mut self, data: u8) {
        let set: u16 = InstructionOperands::SET {
            destination: SetDestination::PINDIRS,
            data,
        }
        .encode();
        self.exec_instr(set);
    }

    /// Set instruction for pin destination.
    pub unsafe fn set_pin(&mut self, data: u8) {
        let set: u16 = InstructionOperands::SET {
            destination: SetDestination::PINS,
            data,
        }
        .encode();
        self.exec_instr(set);
    }

    /// Out instruction for pin destination.
    pub unsafe fn set_out_pin(&mut self, data: u32) {
        const OUT: u16 = InstructionOperands::OUT {
            destination: OutDestination::PINS,
            bit_count: 32,
        }
        .encode();
        self.tx().push(data);
        self.exec_instr(OUT);
    }

    /// Out instruction for pindir destination.
    pub unsafe fn set_out_pindir(&mut self, data: u32) {
        const OUT: u16 = InstructionOperands::OUT {
            destination: OutDestination::PINDIRS,
            bit_count: 32,
        }
        .encode();
        self.tx().push(data);
        self.exec_instr(OUT);
    }

    /// Jump instruction to address.
    pub unsafe fn exec_jmp(&mut self, to_addr: u8) {
        let jmp: u16 = InstructionOperands::JMP {
            address: to_addr,
            condition: JmpCondition::Always,
        }
        .encode();
        self.exec_instr(jmp);
    }
}
