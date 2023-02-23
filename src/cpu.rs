use crate::{interconnect::Interconnect, instruction::Instruction};

pub struct Cpu{
    pc: u32, // Program Counter
    regs: [u32; 32],
    inter: Interconnect,
}

impl Cpu{
    const RESET_STATE_ADDR: u32 = 0xbfc00000;

    pub fn new(inter: Interconnect) -> Cpu{
        let mut regs = [0xdeadbeef;32];
        regs[0] = 0;
        Cpu{
            pc: Cpu::RESET_STATE_ADDR,
            regs,
            inter,
        }
    }

    fn reg(&self, index: u32) -> u32{
        self.regs[index as usize]
    }
    
    fn set_reg(&mut self, index: u32, val: u32){
        self.regs[index as usize] = val;

        self.regs[0] = 0;
    }

    pub fn run_next_instruction(&mut self){
        let pc = self.pc;
        let instruction = Instruction(self.load32(pc));
        self.pc = pc.wrapping_add(4);
        self.decode_and_execute(instruction);
    }

    fn load32(&self,addr: u32) -> u32{
        self.inter.load32(addr)
    }

    fn decode_and_execute(&mut self, instruction: Instruction) {
        match instruction.function() {
            0b001111 => self.op_lui(instruction),
            _        => panic!("Unhandled instruction {:x}",instruction.0),
        }
    }

    fn op_lui(&mut self, instruction: Instruction){
        let i = instruction.imm();
        let t = instruction.t();

        let v = i << 16;

        self.set_reg(t, v);
    }
}


pub mod map{
    pub struct Range(u32,u32);

    pub const BIOS: Range = Range(0xbfc00000, 512*1024);

    impl Range {
        pub fn contains(self,addr:u32) -> Option<u32> {
            let Range(start, length) = self;

            if addr >= start && start <= start + length{
                Some(addr - start)
            } else {
                None
            }
        }
    }
}
