pub struct Instruction(pub u32);

impl Instruction {
    pub fn function(&self) -> u32 {
        let Instruction(op) = self;

        op >> 26
    }

    pub fn t(&self) -> u32{
        let Instruction(op) = self;
        
        (op>>16) & 0x1f
    }

    pub fn imm(&self) -> u32{
        let Instruction(op) = self;

        op & 0xffff
    }
}
