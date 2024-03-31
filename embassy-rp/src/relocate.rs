use pio::{Program, SideSet, Wrap};

pub struct CodeIterator<'a, I>
where
    I: Iterator<Item = &'a u16>,
{
    iter: I,
    offset: u8,
}

impl<'a, I: Iterator<Item = &'a u16>> CodeIterator<'a, I> {
    pub fn new(iter: I, offset: u8) -> CodeIterator<'a, I> {
        CodeIterator { iter, offset }
    }
}

impl<'a, I> Iterator for CodeIterator<'a, I>
where
    I: Iterator<Item = &'a u16>,
{
    type Item = u16;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&instr| {
            if instr & 0b1110_0000_0000_0000 == 0 {
                // this is a JMP instruction -> add offset to address
                let address = (instr & 0b1_1111) as u8;
                let address = address.wrapping_add(self.offset) % 32;
                instr & (!0b11111) | address as u16
            } else {
                instr
            }
        })
    }
}

pub struct RelocatedProgram<'a, const PROGRAM_SIZE: usize> {
    program: &'a Program<PROGRAM_SIZE>,
    origin: u8,
}

impl<'a, const PROGRAM_SIZE: usize> RelocatedProgram<'a, PROGRAM_SIZE> {
    pub fn new_with_origin(program: &Program<PROGRAM_SIZE>, origin: u8) -> RelocatedProgram<PROGRAM_SIZE> {
        RelocatedProgram { program, origin }
    }

    pub fn code(&'a self) -> CodeIterator<'a, core::slice::Iter<'a, u16>> {
        CodeIterator::new(self.program.code.iter(), self.origin)
    }

    pub fn wrap(&self) -> Wrap {
        let wrap = self.program.wrap;
        let origin = self.origin;
        Wrap {
            source: wrap.source.wrapping_add(origin) % 32,
            target: wrap.target.wrapping_add(origin) % 32,
        }
    }

    pub fn side_set(&self) -> SideSet {
        self.program.side_set
    }

    pub fn origin(&self) -> u8 {
        self.origin
    }
}
