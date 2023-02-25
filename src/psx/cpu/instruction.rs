/// PSX MIPS 32bits Instruction
#[derive(Clone, Copy)]
pub struct Instruction(pub u32);
#[derive(PartialEq, Clone, Copy)]
pub struct RegisterIndex(pub u32);

impl Instruction {
    pub fn function(&self) -> u32 {
        let Instruction(op) = self;

        op >> 26
    }

    pub fn subfunction(&self) -> u32 {
        let Instruction(op) = self;

        op & 0x3f
    }

    pub fn cop_opcode(&self) -> u32 {
        let Instruction(op) = self;

        (op >> 21) & 0x1f
    }

    pub fn d(&self) -> RegisterIndex {
        let Instruction(op) = self;

        RegisterIndex((op >> 11) & 0x1f)
    }

    pub fn t(&self) -> RegisterIndex {
        let Instruction(op) = self;

        RegisterIndex((op >> 16) & 0x1f)
    }

    pub fn s(&self) -> RegisterIndex {
        let Instruction(op) = self;

        RegisterIndex((op >> 21) & 0x1f)
    }

    pub fn shift_imm(&self) -> u32 {
        let Instruction(op) = self;

        (op >> 6) & 0x1f
    }

    pub fn imm(&self) -> u32 {
        let Instruction(op) = self;

        op & 0xffff
    }

    pub fn imm_se(&self) -> u32 {
        let Instruction(op) = self;

        let v = (op & 0xffff) as i16;

        v as u32
    }

    pub fn imm_jump(&self) -> u32 {
        let Instruction(op) = self;

        op & 0x3ff_ffff
    }
}
