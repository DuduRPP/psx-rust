use crate::psx::Interconnect;

mod instruction;

use instruction::Instruction;

use self::instruction::RegisterIndex;

/// Emulated PSX CPU state
pub struct Cpu {
    /// Program Counter register value
    pc: u32,
    /// Current instruction pc
    curr_pc: u32,
    /// Because of MIPS "pipelined" architecture we need to always load next instruction
    next_pc: u32,
    /// 32 Core registers array
    regs: [u32; 32],
    /// Interconnect of PSX BIOS and other peripherals
    inter: Interconnect,
    /// Option to handle with load delay instructions
    load: Option<(RegisterIndex, u32)>,

    /// Hi register for div and mul
    hi: u32,
    lo: u32,

    // COP 0
    /// Register 12 -Status Register
    sr: u32,
    /// Register 13 - Cause Register
    cause: u32,
    /// Register 14 - EPC
    epc: u32,

    branch: bool,
    delay_slot: bool,
}

impl Cpu {
    const RESET_STATE_ADDR: u32 = 0xbfc00000;

    pub fn new(inter: Interconnect) -> Cpu {
        let mut regs = [0xdeadbeef; 32];
        regs[0] = 0;
        Cpu {
            pc: Cpu::RESET_STATE_ADDR,
            curr_pc: Cpu::RESET_STATE_ADDR,
            regs,
            inter,
            next_pc: Cpu::RESET_STATE_ADDR + 4,
            load: None,
            hi: 0,
            lo: 0,
            sr: 0,
            cause: 0,
            epc: 0,
            branch: false,
            delay_slot: false,
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
        self.curr_pc = self.pc;
        self.pc = self.next_pc;
        self.next_pc = self.pc.wrapping_add(4);

        let instruction = Instruction(self.load32(self.curr_pc));

        self.delay_slot = self.branch;
        self.branch     = false;

        self.decode_and_execute(instruction);
    }

    fn load8(&self, addr: u32) -> u8 {
        self.inter.load8(addr)
    }

    fn load32(&self, addr: u32) -> u32 {
        self.inter.load32(addr)
    }

    fn store8(&mut self, addr: u32, val: u8) {
        self.inter.store8(addr, val);
    }

    fn store16(&mut self, addr: u32, val: u16) {
        self.inter.store16(addr, val);
    }

    /// Store 32 bit value into memory
    fn store32(&mut self, addr: u32, val: u32) {
        self.inter.store32(addr, val);
    }

    /// Decode instruction and execute them
    fn decode_and_execute(&mut self, instruction: Instruction) {
        // DEBUG
        // println!(
        //     "INS: {:06b} --- SUB:{:06b} S{} T{} D{} --- C: {:08x} --- PC: {:08x} --- NPC {:08x}",
        //     instruction.function(),
        //     instruction.subfunction(),
        //     instruction.s().0,
        //     instruction.t().0,
        //     instruction.d().0,
        //     self.curr_pc,
        //     self.pc,
        //     self.next_pc,
        // );

        match instruction.function() {
            0b000000 => match instruction.subfunction() {
                0b000000 => self.op_sll(instruction),
                0b000010 => self.op_srl(instruction),
                0b000011 => self.op_sra(instruction),
                0b001100 => self.op_syscall(instruction),
                0b001000 => self.op_jr(instruction),
                0b001001 => self.op_jalr(instruction),
                0b010000 => self.op_mfhi(instruction),
                0b010001 => self.op_mthi(instruction),
                0b010010 => self.op_mflo(instruction),
                0b010011 => self.op_mtlo(instruction),
                0b011010 => self.op_div(instruction),
                0b011011 => self.op_divu(instruction),
                0b100000 => self.op_add(instruction),
                0b100001 => self.op_addu(instruction),
                0b100011 => self.op_subu(instruction),
                0b100100 => self.op_and(instruction),
                0b100101 => self.op_or(instruction),
                0b101010 => self.op_slt(instruction),
                0b101011 => self.op_sltu(instruction),
                _ => panic!("Unhandled instruction {:x}", instruction.0),
            },
            0b000001 => self.op_bxx(instruction),
            0b000010 => self.op_j(instruction),
            0b000011 => self.op_jal(instruction),
            0b000100 => self.op_beq(instruction),
            0b000101 => self.op_bne(instruction),
            0b000110 => self.op_blez(instruction),
            0b000111 => self.op_bgtz(instruction),
            0b001101 => self.op_ori(instruction),
            0b001000 => self.op_addi(instruction),
            0b001001 => self.op_addiu(instruction),
            0b001010 => self.op_slti(instruction),
            0b001011 => self.op_sltiu(instruction),
            0b001100 => self.op_andi(instruction),
            0b001111 => self.op_lui(instruction),
            0b010000 => self.op_cop0(instruction),
            0b100000 => self.op_lb(instruction),
            0b100011 => self.op_lw(instruction),
            0b100100 => self.op_lbu(instruction),
            0b101000 => self.op_sb(instruction),
            0b101001 => self.op_sh(instruction),
            0b101011 => self.op_sw(instruction),
            _ => panic!("Unhandled instruction {:x}", instruction.0),
        }
    }

    /// Handles load delay in non chained load instructions
    fn handle_load_delay(&mut self) {
        if let Some((load_reg, v)) = self.load {
            self.set_reg(load_reg, v);
            self.load = None;
        }
    }

    fn handle_load_delay_chain(&mut self, new_load_reg: RegisterIndex, new_v: u32) {
        if let Some((load_reg, v)) = self.load {
            if new_load_reg.0 != load_reg.0 {
                self.set_reg(load_reg, v);
            }
        }

        self.load = Some((new_load_reg, new_v))
    }

    /// Triggers Exceptions
    fn exception(&mut self, cause:Exception){
        let handler: u32 = match self.sr & (1<<22) != 0{
            true => 0xbfc00180,
            false => 0x80000080,
        };

        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= (mode << 2) & 0x3f;

        self.cause = (cause as u32) << 2;

        self.epc = self.curr_pc;

        if self.delay_slot{
            self.epc = self.epc.wrapping_sub(4);
            self.cause |= 1 << 31;
        }

        self.pc      = handler;
        self.next_pc = self.pc.wrapping_add(4);

    }

    fn op_cop0(&mut self, instruction: Instruction) {
        match instruction.cop_opcode() {
            0b00000 => self.op_mfc0(instruction),
            0b00100 => self.op_mtc0(instruction),
            0b10000 => self.op_rfe(instruction),
            _ => panic!("Unhandled instruction {:x}", instruction.0),
        }
    }

    /// Branch with relative immediate offset
    fn branch(&mut self, offset: u32) {
        let offset = offset << 2;

        self.next_pc = self.pc.wrapping_add(offset);
        self.branch  = true;
    }

    /// Decider between bltz, bgez, bltzal, bgezal
    fn op_bxx(&mut self, instruction: Instruction){
        let i = instruction.imm_se();
        let s = instruction.s();

        let is_link = ((instruction.0 >> 17) & 0xf) == 0x8;
        let is_greater = (instruction.0 >> 16) & 1;
        
        let v = self.reg(s) as i32;

        // less than zero test
        let test = (v<0) as u32;

        // xor that allows to verify if it should branch
        let test = test ^ is_greater;

        self.handle_load_delay();

        if is_link{
            let ra = self.next_pc;
            self.set_reg(RegisterIndex(31),ra);
        }

        if test != 0{
            self.branch(i);
        }
    }

    fn op_lui(&mut self, instruction: Instruction) {
        let i = instruction.imm();
        let t = instruction.t();

        let v = i << 16;

        self.handle_load_delay();

        self.set_reg(t, v);
    }

    fn op_ori(&mut self, instruction: Instruction) {
        let i = instruction.imm();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s) | i;

        self.handle_load_delay();

        self.set_reg(t, v)
    }

    fn op_or(&mut self, instruction: Instruction) {
        let d = instruction.d();
        let s = instruction.s();
        let t = instruction.t();

        let v = self.reg(s) | self.reg(t);

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_andi(&mut self, instruction: Instruction) {
        let i = instruction.imm();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s) & i;

        self.handle_load_delay();

        self.set_reg(t, v)
    }

    fn op_and(&mut self, instruction: Instruction) {
        println!("AND");
        let d = instruction.d();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s) & self.reg(t);

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_sb(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            println!("ignoring store while cache is isolated");
            return;
        }

        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);
        let v = self.reg(t);

        self.handle_load_delay();

        self.store8(addr, v as u8);
    }

    fn op_sh(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            println!("ignoring store while cache is isolated");
            return;
        }

        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);
        let v = self.reg(t);
        if addr % 2 == 0 {
            self.handle_load_delay();

            self.store16(addr, v as u16);
        } else {
            self.exception(Exception::StoreAddressError);
        }

    }

    fn op_sw(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            println!("ignoring store while cache is isolated");
            return;
        }

        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);
        let v = self.reg(t);
        if addr % 4 == 0{
            self.handle_load_delay();

            self.store32(addr, v);
        } else {
            self.exception(Exception::StoreAddressError);
        }
    }

    fn op_lb(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);

        let v = self.load8(addr) as i8;

        self.handle_load_delay_chain(t, v as u32);
    }

    fn op_lbu(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);

        let v = self.load8(addr);

        self.handle_load_delay_chain(t, v as u32);
    }

    fn op_lw(&mut self, instruction: Instruction) {
        if self.sr & 0x10000 != 0 {
            println!("ignoring load while cache is isolated");
            return;
        }

        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let addr = self.reg(s).wrapping_add(i);
        if addr % 4 == 0{
            let v = self.load32(addr);

            self.handle_load_delay_chain(t, v);
        } else {
            self.exception(Exception::LoadAddressError);
        }
    }

    fn op_sra(&mut self, instruction: Instruction) {
        let i = instruction.shift_imm();
        let t = instruction.t();
        let d = instruction.d();

        let v = (self.reg(t) as i32) >> i;

        self.handle_load_delay();

        self.set_reg(d, v as u32)
    }

    fn op_sll(&mut self, instruction: Instruction) {
        let i = instruction.shift_imm();
        let t = instruction.t();
        let d = instruction.d();

        let v = self.reg(t) << i;

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_srl(&mut self, instruction: Instruction) {
        let i = instruction.shift_imm();
        let t = instruction.t();
        let d = instruction.d();

        let v = self.reg(t) >> i;

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_slti(&mut self, instruction: Instruction){
        let i = instruction.imm();
        let s = instruction.s();
        let t = instruction.t();

        let v = (self.reg(s) as i32) << i;

        self.handle_load_delay();

        self.set_reg(t, v as u32);
    }

    fn op_sltiu(&mut self, instruction: Instruction){
        let i = instruction.imm_se();
        let s = instruction.s();
        let t = instruction.t();

        let v = self.reg(s) < i;

        self.handle_load_delay();

        self.set_reg(t, v as u32);
    }

    fn op_slt(&mut self, instruction: Instruction) {
        let s = instruction.s();
        let t = instruction.t();
        let d = instruction.d();

        let s = self.reg(s) as i32;
        let t = self.reg(t) as i32;

        let v = s < t;

        self.handle_load_delay();

        self.set_reg(d, v as u32);
    }

    fn op_sltu(&mut self, instruction: Instruction) {
        let s = instruction.s();
        let t = instruction.t();
        let d = instruction.d();

        let v = self.reg(s) < self.reg(t);

        self.handle_load_delay();

        self.set_reg(d, v as u32);
    }

    fn op_add(&mut self, instruction: Instruction) {
        let d = instruction.d();
        let t = instruction.t();
        let s = instruction.s();

        let s = self.reg(s) as i32;
        let t = self.reg(t) as i32;

        self.handle_load_delay();

        match s.checked_add(t) {
            Some(v) => self.set_reg(d, v as u32),
            None => self.exception(Exception::Overflow),
        };
    }

    fn op_addi(&mut self, instruction: Instruction) {
        let i = instruction.imm_se() as i32;
        let t = instruction.t();
        let s = instruction.s();

        let s = self.reg(s) as i32;

        self.handle_load_delay();
        
        match s.checked_add(i) {
            Some(v) => self.set_reg(t, v as u32),
            None    => self.exception(Exception::Overflow),
        };
    }

    fn op_addiu(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s).wrapping_add(i);

        self.handle_load_delay();

        self.set_reg(t, v)
    }

    fn op_addu(&mut self, instruction: Instruction) {
        let d = instruction.d();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s).wrapping_add(self.reg(t));

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_subu(&mut self, instruction: Instruction) {
        let d = instruction.d();
        let t = instruction.t();
        let s = instruction.s();

        let v = self.reg(s).wrapping_sub(self.reg(t));

        self.handle_load_delay();

        self.set_reg(d, v)
    }

    fn op_div(&mut self, instruction: Instruction){
        let s = instruction.s();
        let t = instruction.t();

        let n = self.reg(s) as i32;
        let d = self.reg(t) as i32;

        self.handle_load_delay();

        if d == 0{
            self.hi = n as u32;
            if n >= 0{
                self.lo = 0xffffffff;
            } else {
                self.lo = 1;
            }
        } else if n as u32 == 0x80000000 && d == -1{
            self.hi = 0;
            self.lo = 0x80000000;
        } else{
            self.hi = (n%d) as u32;
            self.lo = (n/d) as u32;
        }
    }

    fn op_divu(&mut self, instruction: Instruction){
        let s = instruction.s();
        let t = instruction.t();

        let n = self.reg(s);
        let d = self.reg(t);

        self.handle_load_delay();

        if d == 0{
            self.hi = n;
            self.lo = 0xffffffff;
        } else{
            self.hi = n % d;
            self.lo = n / d;
        }
    }

    fn op_mfhi(&mut self, instruction: Instruction){
        let d = instruction.d();

        let hi = self.hi;

        self.handle_load_delay();

        self.set_reg(d, hi);
    }

    fn op_mthi(&mut self, instruction: Instruction){
        let s = instruction.s();

        self.hi = self.reg(s);

        self.handle_load_delay();
    }

    fn op_mflo(&mut self, instruction: Instruction){
        let d = instruction.d();

        self.handle_load_delay();

        self.set_reg(d, self.lo);
    }

    fn op_mtlo(&mut self, instruction: Instruction){
        let s = instruction.s();

        self.lo = self.reg(s);

        self.handle_load_delay();
    }

    fn op_j(&mut self, instruction: Instruction) {
        let i = instruction.imm_jump();

        self.next_pc = (self.pc & 0xf000_0000) | (i << 2);
        self.branch  = true;

        self.handle_load_delay();
    }

    fn op_jal(&mut self, instruction: Instruction) {
        let ra = self.next_pc;

        self.op_j(instruction);

        self.set_reg(RegisterIndex(31), ra);
    }

    /// Jump Register
    fn op_jr(&mut self, instruction: Instruction) {
        let s = instruction.s();

        self.next_pc = self.reg(s);
        self.branch  = true;

        self.handle_load_delay();
    }

    fn op_jalr(&mut self, instruction: Instruction) {
        let d = instruction.d();
        let s = instruction.s();

        let ra       = self.next_pc;
        self.next_pc = self.reg(s);
        self.branch  = true;

        self.set_reg(d, ra);
    }

    fn op_beq(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        if self.reg(s) == self.reg(t) {
            self.branch(i);
        }

        self.handle_load_delay();
    }

    fn op_bne(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let t = instruction.t();
        let s = instruction.s();

        if self.reg(s) != self.reg(t) {
            self.branch(i);
        }

        self.handle_load_delay();
    }

    fn op_blez(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let s = instruction.s();

        let v = self.reg(s) as i32;

        if v <= 0{
            self.branch(i);
        }

        self.handle_load_delay();
    }

    fn op_bgtz(&mut self, instruction: Instruction) {
        let i = instruction.imm_se();
        let s = instruction.s();

        let v = self.reg(s) as i32;

        if v > 0{
            self.branch(i);
        }

        self.handle_load_delay();
    }

    fn op_syscall(&mut self, instruction: Instruction){
        self.exception(Exception::SysCall);
    }

    fn op_mtc0(&mut self, instruction: Instruction) {
        let cpu_r = instruction.t();
        let cop_r = instruction.d();

        let v = self.reg(cpu_r);

        self.handle_load_delay();

        match cop_r.0 {
            3 | 5 | 6 | 7 | 9 | 11 => {
                if v != 0 {
                    panic!("Unhandled write to cop0r{}", cop_r.0)
                }
            }
            12 => self.sr = v,
            13 => {
                if v != 0 {
                    panic!("Unhandled CAUSE register")
                }
            }
            _ => panic!("Unhandled cop0 register {:08x}", cop_r.0),
        }
    }

    fn op_mfc0(&mut self, instruction: Instruction) {
        let cpu_r = instruction.t();
        let cop_r = instruction.d().0;

        let v = match cop_r {
            12 => self.sr,
            13 => self.cause,
            14 => self.epc,
            _ => panic!("Unhandled cop0 register {:08x}", cop_r),
        };

        self.handle_load_delay_chain(cpu_r, v);
    }

    /// Return from exception
    fn op_rfe(&mut self, instruction: Instruction){
        if instruction.0 & 0x3f != 0b010000{
            panic!("Invalid cop0 instruction: {}",instruction.0);
        }

        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= mode >> 2;
    }
}

/// Exception types stored in CAUSE register (cop0 - $13)
enum Exception {
    /// Address error on load
    LoadAddressError = 0x4,
    /// Address error on store
    StoreAddressError = 0x5,
    /// System Call Exception
    SysCall = 0x8,
    /// Overflow Exception
    Overflow = 0xc,

}

pub mod map {

    /// Array mask that's used to get only KUSEG region from KSEG0 and KSEG1,
    /// KSEG2 is not touched since it doesnt share anything with other regions
    const REGION_MASK: [u32; 8] = [
        // KUSEG
        0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, // KSEG0
        0x7fffffff, // KSEG1
        0x1fffffff, // KSEG2
        0xffffffff, 0xffffffff,
    ];

    /// Range contains starting value and length
    pub struct Range(u32, u32);

    /// RAM range
    pub const RAM: Range = Range(0x00000000, 2 * 1024 * 1024);

    /// BIOS range
    pub const BIOS: Range = Range(0x1fc00000, 512 * 1024);

    pub const MEMLCONTROL: Range = Range(0x1f801000, 36);

    /// Register related to RAM configuration
    pub const RAM_SIZE: Range = Range(0x1f801060, 4);
    pub const CACHE_CONTROL: Range = Range(0xfffe0130, 4);

    // I/O
    /// Expansion 1
    pub const EXPANSION_1: Range = Range(0x1f000000, 8 * 1024);

    /// Interrupt Control registers
    pub const IRQ_CONTROL: Range = Range(0x1f801070, 8);

    /// Timers registers
    pub const TIMERS: Range = Range(0x1f801100, 52);

    /// Sound registers
    pub const SPU: Range = Range(0x1f801c00, 640);

    /// Expansion 2
    pub const EXPANSION_2: Range = Range(0x1f802000, 8 * 1024);

    pub fn mask_region(addr: u32) -> u32 {
        let index = (addr >> 29) as usize;

        addr & REGION_MASK[index]
    }

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
