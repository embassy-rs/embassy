use pio::{InSource, InstructionOperands, JmpCondition, OutDestination, SetDestination};

use crate::pio::PioStateMachineInstance;

pub fn set_x<SM: PioStateMachineInstance>(sm: &mut SM, value: u32) {
    const OUT: u16 = InstructionOperands::OUT {
        destination: OutDestination::X,
        bit_count: 32,
    }
    .encode();
    sm.push_tx(value);
    sm.exec_instr(OUT);
}

pub fn get_x<SM: PioStateMachineInstance>(sm: &mut SM) -> u32 {
    const IN: u16 = InstructionOperands::IN {
        source: InSource::X,
        bit_count: 32,
    }
    .encode();
    sm.exec_instr(IN);
    sm.pull_rx()
}

pub fn set_y<SM: PioStateMachineInstance>(sm: &mut SM, value: u32) {
    const OUT: u16 = InstructionOperands::OUT {
        destination: OutDestination::Y,
        bit_count: 32,
    }
    .encode();
    sm.push_tx(value);
    sm.exec_instr(OUT);
}

pub fn get_y<SM: PioStateMachineInstance>(sm: &mut SM) -> u32 {
    const IN: u16 = InstructionOperands::IN {
        source: InSource::Y,
        bit_count: 32,
    }
    .encode();
    sm.exec_instr(IN);

    sm.pull_rx()
}

pub fn set_pindir<SM: PioStateMachineInstance>(sm: &mut SM, data: u8) {
    let set: u16 = InstructionOperands::SET {
        destination: SetDestination::PINDIRS,
        data,
    }
    .encode();
    sm.exec_instr(set);
}

pub fn set_pin<SM: PioStateMachineInstance>(sm: &mut SM, data: u8) {
    let set: u16 = InstructionOperands::SET {
        destination: SetDestination::PINS,
        data,
    }
    .encode();
    sm.exec_instr(set);
}

pub fn set_out_pin<SM: PioStateMachineInstance>(sm: &mut SM, data: u32) {
    const OUT: u16 = InstructionOperands::OUT {
        destination: OutDestination::PINS,
        bit_count: 32,
    }
    .encode();
    sm.push_tx(data);
    sm.exec_instr(OUT);
}
pub fn set_out_pindir<SM: PioStateMachineInstance>(sm: &mut SM, data: u32) {
    const OUT: u16 = InstructionOperands::OUT {
        destination: OutDestination::PINDIRS,
        bit_count: 32,
    }
    .encode();
    sm.push_tx(data);
    sm.exec_instr(OUT);
}

pub fn exec_jmp<SM: PioStateMachineInstance>(sm: &mut SM, to_addr: u8) {
    let jmp: u16 = InstructionOperands::JMP {
        address: to_addr,
        condition: JmpCondition::Always,
    }
    .encode();
    sm.exec_instr(jmp);
}
