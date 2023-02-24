use crate::psx::Interconnect;

mod instruction;

use instruction::Instruction;

use self::instruction::RegisterIndex;

/// Emulated PSX CPU state
pub struct Cpu {
    /// Program Counter register value
    pc: u32, 
    /// 32 Core registers array
    regs: [u32; 32],
    /// Interconnect of PSX BIOS and other peripherals
    inter: Interconnect,
    /// Because of MIPS "pipelined" architecture we need to always load next instruction 
    next_pc: u32,
    /// Option to handle with load delay instructions
    load: Option<(RegisterIndex,u32)>,

    sr: u32,
}

impl Cpu {
    const RESET_STATE_ADDR: u32 = 0xbfc00000;

    pub fn new(inter: Interconnect) -> Cpu {
        let mut regs = [0xdeadbeef; 32];
        regs[0] = 0;
        Cpu {
            pc: Cpu::RESET_STATE_ADDR + 4,
            regs,
            inter,
            next_pc: Cpu::RESET_STATE_ADDR,
                load: None,
            sr: 0,
        }
    }

    fn reg(&self, index: RegisterIndex) -> u32 {
        self.regs[index.0 as usize]
    }

    fn set_reg(&mut self, index: RegisterIndex, val: u32) {
        self.regs[index.0 as usize] = val;

        self.regs[0] = 0;
    }

    pub fn run_next_instruction(&mut self) {
        let pc = self.pc;

        let instruction = Instruction(self.load32(self.next_pc));

        self.next_pc = pc;

        self.pc = pc.wrapping_add(4);

        self.decode_and_execute(instruction);
    }

    fn load32(&self, addr: u32) -> u32 {
        self.inter.load32(addr)
    }

    /// Store 32 bit value into memory
    fn store32(&mut self, addr: u32, val: u32){
        self.inter.store32(addr,val);
    }

    /// Decode instruction and execute them 
    fn decode_and_execute(&mut self, instruction: Instruction) {
        match instruction.function() {
            0b000000 => match instruction.subfunction() {
                0b000000 => self.op_sll(instruction),
                0b100101 => self.op_or(instruction),
                _ => panic!("Unhandled instruction {:x}", instruction.0),
            },
            0b000010 => self.op_j(instruction),
            0b000101 => self.op_bne(instruction),
            0b001101 => self.op_ori(instruction),
            0b001000 => self.op_addi(instruction),
            0b001001 => self.op_addiu(instruction),
            0b001111 => self.op_lui(instruction),
            0b010000 => self.op_cop0(instruction),
            0b100011 => self.op_lw(instruction),
            0b101011 => self.op_sw(instruction),
            _ => panic!("Unhandled instruction {:x}", instruction.0),
        }
    }

    /// Handles load delay in non chained load instructions
    fn handle_load_delay(&mut self){
        if let Some((load_reg,v)) = self.load{
            self.set_reg(load_reg, v);
            self.load = None;
        }
    }

    fn handle_load_delay_chain(&mut self, new_load_reg: RegisterIndex, new_v: u32){
        if let Some((load_reg, v)) = self.load{
            if new_load_reg != load_reg{
                self.set_reg(load_reg, v);
            }
        }

        self.load = Some((new_load_reg,new_v))
    }

    fn op_cop0(&mut self, instruction: Instruction){
        match instruction.cop_opcode() {
            0b00100 => self.op_mtc0(instruction),
            _       => panic!("Unhandled instruction {:x}",instruction.0),
        }
    }

    /// Branch with relative immediate offset
    fn branch(&mut self, offset: u32){
        let offset = offset << 2;

        let mut pc = self.pc;

        pc = pc.wrapping_add(offset);
        pc = pc.wrapping_sub(4);

        self.pc = pc
    }

    fn op_lui(&mut self, instruction: Instruction) {
        let i = instruction.imm();
        let t = instruction.t();

        let v = i << 16;

        self.handle_load_delay();

        self.set_reg(t, v);
    }

    fn op_ori(&mut self, instruction: Instruction){
        let i = instruction.imm();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s) | i;

        self.handle_load_delay();

        self.set_reg(t, v)
    }

    fn op_or(&mut self, instruction: Instruction){
        let d = instruction.d();
        let s = instruction.s();
        let t = instruction.t();

        let v = self.reg(s) | self.reg(t);

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_sw(&mut self, instruction: Instruction){
        if self.sr & 0x10000 != 0{
            println!("ignoring store while cache is isolated");
            return ;
        }

        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);
        let v    = self.reg(t);

        self.handle_load_delay();

        self.store32(addr, v);

    }

    fn op_lw(&mut self, instruction: Instruction){
        if self.sr & 0x10000 != 0{
            println!("ignoring load while cache is isolated");
            return ;
        }

        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);
        
        let v = self.load32(addr);

        self.handle_load_delay_chain(t, v);
    }

    fn op_sll(&mut self, instruction: Instruction){
        let i = instruction.shift_imm();
        let t = instruction.t();
        let d = instruction.d();

        let v = self.reg(t) << i;
        
        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_addi(&mut self, instruction:Instruction){
        let i = instruction.imm_se() as i32;
        let t = instruction.t();
        let s = instruction.s();

        let s = self.reg(s) as i32;

        let v = match s.checked_add(i) {
            Some(v) => v as u32,
            None    => panic!("ADDI overflow"),
        };

        self.handle_load_delay();

        self.set_reg(t, v)
    }

    fn op_addiu(&mut self, instruction:Instruction){
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s).wrapping_add(i);

        self.handle_load_delay();

        self.set_reg(t, v)
    }

    fn op_j(&mut self, instruction: Instruction) {
        let i = instruction.imm_jump();

        self.pc = (self.pc & 0xf0000000) | (i << 2);

        self.handle_load_delay();
    }

    fn op_bne(&mut self, instruction:Instruction){
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        if self.reg(s) != self.reg(t){
            self.branch(i);
        }

        self.handle_load_delay();
    }

    fn op_mtc0(&mut self, instruction: Instruction){
        let cpu_r = instruction.t();
        let cop_r = instruction.d();

        let v = self.reg(cpu_r);

        self.handle_load_delay();

        match cop_r.0 {
           12 => self.sr = v,
           _  => panic!("Unhandled cop0 register {:08x}",cop_r.0)
        }
    }
}

pub mod map {
    /// Range contains starting value and length
    pub struct Range(u32, u32);

    /// RAM range
    pub const RAM: Range = Range(0xa0000000, 2 * 1024 * 1024);

    /// BIOS range 
    pub const BIOS: Range = Range(0xbfc00000, 512 * 1024);

    pub const MEMLCONTROL: Range = Range(0x1f801000,36);

    /// Register related to RAM configuration
    pub const RAM_SIZE: Range = Range(0x1f801060,4);
    pub const CACHE_CONTROL: Range = Range(0xfffe0130,4);

    impl Range {
        pub fn contains(self, addr: u32) -> Option<u32> {
            let Range(start, length) = self;

            if addr >= start && addr <= start + length {
                Some(addr - start)
            } else {
                None
            }
        }
    }
}
