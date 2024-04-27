use std::usize;

struct CPU {
    pub acc_reg: u8,
    pub pc: u16,
    pub status: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    memory: [u8; 0xffff]
}

struct OpCode {
    instruction: Vec<char>,
    addressing_mode: AddressingMode,
    cycle_count: u8,
    size: u8,
    opcode: u8
}

static CPU_OPS_CODE: Vec<OpCode> = [
    OpCode::new("BRK", 7, 2, 0x00, AddressingMode::Implicit),
    OpCode::new("TAX", 2, 2, 0xAA, AddressingMode::Implicit),

    OpCode::new("LDA", 2, 2, 0XA9, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 3, 0XA5, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 4, 0XB5, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 4, 0XAD, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 4 /* +1 if page crossed */, 0XBD, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 4 /* +1 if page crossed */, 0XB9, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 6, 0XA1, AddressingMode::Implicit),
    OpCode::new("LDA", 2, 5 /* +1 if page crossed */, 0XB1, AddressingMode::Implicit),
];

enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute, 
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedDirect,
    IndirectedIndex,
    Noneaddressing
}

impl CPU {
    pub fn new() -> Self {
        Self {
            acc_reg: 0,
            pc: 0,
            status: 0,
            reg_x: 0,
            reg_y : 0,
            memory: [0; 0xffff]
        }
    }

    fn get_memory_address() {

    }

    fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xfffc, 0x8000);
    }

    fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn mem_read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    
    fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // Since the 6502 uses little endian addressing, we first read the 
        // least significant byte and then we read the most siginificant byte,
        // which is the next byte in memory
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.pc = self.mem_read_u16(0xfffc);

        self.reg_x = 0;
        self.reg_y = 0;
        self.acc_reg = 0;
        self.status = 0;
    }
    
    fn lda(&mut self, value: u8) {
        self.acc_reg = value;
        self.update_negative_zero_flags(self.acc_reg);
    }

    fn tax(&mut self) {
        self.reg_x = self.acc_reg;
        self.update_negative_zero_flags(self.reg_x);
    }

    fn update_negative_zero_flags(&mut self, result: u8) {
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if (result & 0b1000_0000) != 0 {
            self.status = self.status | 0b1000_0000;
        } else  {
            self.status = self.status & 0b0111_1111;
        }

    }
    
    pub fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_negative_zero_flags(self.reg_x);
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.pc);
        }
    }
}

impl OpCode {
    fn new(instruction: Vec<char>, opcode: u8, cycle_count: u8, size: u8,
            addressing_mode: AddressingMode) -> Self {
        Self {
            instruction,
            cycle_count,
            opcode, 
            addressing_mode,
            size
        }
    }
}

#[cfg(test)]
mod test{ 
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
        assert!(cpu.acc_reg == 0x05);
        
        // Check if negative flag is set, which it shouldn't
        assert!(cpu.status & 0b1000_0000 == 0);
        
        // Check if result zero flag is set, which it shouldn't
        assert!(cpu.status & 0b0000_0010 == 0)
    }

    #[test]
    fn test_0xa_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
        // Check if result zero flag is set, which it should
        assert!(cpu.status & 0b0000_0010 != 0)
    }

    #[test]
    fn test_0xa_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0xFF, 0x00]);
        // Check if result zero flag is set, which it should
        assert!(cpu.status & 0b1000_0000 != 0)
    }

    #[test]
    fn test_0xaa_tax_moves_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xAA, 0x00]);
        cpu.reset();

        cpu.acc_reg = 0x15;
        cpu.run();

        assert!(cpu.reg_x == 0x15);
        assert!((cpu.status & 0b0000_0010) == 0);
        assert!((cpu.status & 0b1000_0000) == 0);
    }

    #[test]
    fn test_0xaa_tax_moves_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xAA, 0x00]);
        cpu.reset();

        cpu.acc_reg = 0x00;
        cpu.run();

        assert!((cpu.status & 0b0000_0010) != 0)
    }

    #[test]
    fn test_0xaa_tax_moves_sets_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xAA, 0x00]);
        cpu.reset();

        cpu.acc_reg = 0b1000_0000;
        cpu.run();

        assert!((cpu.status & 0b1000_0000) != 0)
    }

    #[test]
    fn text_0xe8_inc_reg_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xe8, 0x00]);
        cpu.reset();

        cpu.reg_x = 20;
        cpu.run();

        assert!(cpu.reg_x == 21);
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);
        cpu.reset();

        cpu.reg_x = 0xff; 
        cpu.run();

        assert_eq!(1, cpu.reg_x)
    }

    #[test] 
    fn test_update_negative_flag() {
        let mut cpu = CPU::new();
        let val = 0b1000_0000;
        cpu.update_negative_zero_flags(val);

        assert!((cpu.status & 0b1000_0000) != 0);
    }

    #[test] 
    fn test_update_zero_flag() {
        let mut cpu = CPU::new();
        let val = 0b0000_0000;
        cpu.update_negative_zero_flags(val);

        assert!((cpu.status & 0b0000_0010) != 0);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.reg_x, 0xc1)
    }

}
