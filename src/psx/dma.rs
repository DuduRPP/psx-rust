pub struct Dma{
    /// Control Register - offset 0x70
    control: u32,
    // Interrupt Register - offset 0x74
    /// Master enable irq - Bit 23
    irq_en: bool,
    /// IRQ enable for each channel - Bits[22:16]
    channel_irq_en: u8,
    /// Flags for each channel IRQ - Bits [24:30]
    channel_irq_flags: u8,
    /// Force Interrupt - Bit 15
    force_irq: bool,
    /// no apparent use - [0:5] bits
    irq_dummy: u8,

}

impl Dma{
    pub fn new() -> Self{
        Dma { control: 0x07654321,
              irq_en: false,
              channel_irq_en: 0,
              channel_irq_flags: 0,
              force_irq: false,
              irq_dummy: 0
        }
    }

    pub fn control(&self) -> u32{
        self.control
    }
    pub fn set_control(&mut self, val:u32){
        self.control = val;
    }

    /// Bit 31 sees if irq is active
    pub fn irq_status(&self) -> bool{
        let channel_irq = self.channel_irq_flags & self.channel_irq_en;

        self.force_irq || (self.irq_en && channel_irq != 0)
    }

    pub fn interrupt(&self) -> u32{
        let mut r: u32 = 0;

        r |= self.irq_dummy as u32;
        r |= (self.force_irq as u32) << 15;
        r |= (self.channel_irq_en as u32) << 16;
        r |= (self.irq_en as u32) << 23;
        r |= (self.channel_irq_flags as u32) << 24;
        r |= (self.irq_status() as u32) << 31;

        r
    }
    pub fn set_interrupt(&mut self, val:u32){
        self.irq_dummy = (val & 0x3f) as u8;
        self.force_irq = (val >> 15) & 1 != 0;
        self.channel_irq_en = ((val >> 16) & 0x7f) as u8;
        self.irq_en = (val >> 23) & 1 != 0;

        let ack = ((val >> 24) & 0x3f) as u8;
        self.channel_irq_flags &= !ack;
    }
}
