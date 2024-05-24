use std::{collections::HashMap, usize, task::Wake};

struct OpsInfo {
    info: HashMap<u8, OpCode>,
}

struct CPU {
    pub acc_reg: u8,
    pub pc: u16,
    pub status: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    memory: [u8; 0xffff],
}

struct OpCode {
    opcode: u8,
    instruction: String,
    addressing_mode: AddressingMode,
    cycle_count: u8,
    size: u8,
}

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
    IndirectX,
    IndirectY,
    IndexedDirect,
    IndirectedIndex,
    Noneaddressing,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            acc_reg: 0,
            pc: 0,
            status: 0,
            reg_x: 0,
            reg_y: 0,
            memory: [0; 0xffff],
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match *mode {
            AddressingMode::Immediate => self.pc,
            AddressingMode::ZeroPage => self.mem_read(self.pc) as u16,
            AddressingMode::ZeroPageX => {
                let page_addr = self.mem_read(self.pc);
                let addr = page_addr.wrapping_add(self.reg_x) as u16;
                addr
            }
            AddressingMode::ZeroPageY => {
                let page_addr = self.mem_read(self.pc);
                let addr = page_addr.wrapping_add(self.reg_y) as u16;
                addr
            }
            AddressingMode::Absolute => self.mem_read_u16(self.pc),
            AddressingMode::AbsoluteY => {
                let page_addr = self.mem_read_u16(self.pc);
                let addr = page_addr.wrapping_add(self.reg_y as u16);
                addr
            }
            AddressingMode::AbsoluteX => {
                let page_addr = self.mem_read_u16(self.pc);
                let addr = page_addr.wrapping_add(self.reg_y as u16);
                addr
            }
            AddressingMode::Indirect => {
                let base = self.mem_read(self.pc);

                let lb = self.mem_read(base as u16);
                let hb = self.mem_read(base.wrapping_add(1) as u16);

                ((hb as u16) << 8) | (lb as u16)
            }
            AddressingMode::IndirectX => {
                let base: u8 = self.mem_read(self.pc) + self.reg_x;

                let lb = self.mem_read(base as u16);
                let hb = self.mem_read(base.wrapping_add(1) as u16);

                ((hb as u16) << 8) | (lb as u16)
            }
            AddressingMode::IndirectY => {
                let base: u8 = self.mem_read(self.pc) + self.reg_x;

                let lb = self.mem_read(base as u16);
                let hb = self.mem_read(base.wrapping_add(1) as u16);

                let deref_base = ((hb as u16) << 8) | (lb as u16);
                let deref = deref_base.wrapping_add(self.reg_y as u16);

                deref
            }

            _ => todo!(),
        }
    }

    fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
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
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn inx(&mut self) {
        self.reg_x = self.reg_x.wrapping_add(1);
        self.update_negative_zero_flags(self.reg_x);
    }

    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);

        let mem_val: u8 = self.mem_read(addr);
        let carry_flag = self.status & 0b0000_0001;
        let mut value: u16 = 0;

        value = value
            .wrapping_add(mem_val.into())
            .wrapping_add(self.acc_reg.into())
            .wrapping_add(carry_flag.into());

        // set the carry flag
        if value > 255 {
            self.status = (self.status & 0b1111_1110) | 0b0000_0001;
        }

        // Check if overflow
        let overflow = (mem_val ^ value as u8) & (self.acc_reg ^ value as u8) & 0x80;
        self.status = (self.status & 0b1011_1111) | (overflow >> 1);
        // println!("{} {} {} {} {:#b}", mem_val, self.acc_reg, carry_flag, value, self.status);

        // Update the accumulator with the result of the operation
        self.acc_reg = value as u8;

        self.update_negative_zero_flags(value as u8);
    }

    pub fn run(&mut self) {
        let ops_info = create_ops_info();

        loop {
            let opcode = self.mem_read(self.pc);
            self.pc += 1;

            match opcode {
                0x00 => {
                    self.reset();
                }
                0x69 => {
                    self.adc(&AddressingMode::Immediate);
                    self.pc += ops_info.get(&0x69).unwrap().size as u16 - 1;
                }

                _ => break,
            }
        }
    }
}

impl OpCode {
    fn new(
        opcode: u8,
        instruction: String,
        cycle_count: u8,
        size: u8,
        addressing_mode: AddressingMode,
    ) -> Self {
        Self {
            opcode,
            instruction,
            cycle_count,
            size,
            addressing_mode,
        }
    }
}

pub fn create_ops_info() -> HashMap<u8, OpCode> {
    let mut hash: HashMap<u8, OpCode> = HashMap::new();

    hash.insert(
        0x00,
        OpCode::new(0x00, "BRK".to_string(), 7, 1, AddressingMode::Implicit),
    );

    // ADC
    hash.insert(
        0x69,
        OpCode::new(0x69, "ADC".to_string(), 2, 2, AddressingMode::Immediate),
    );
    hash.insert(
        0x65,
        OpCode::new(0x65, "ADC".to_string(), 3, 2, AddressingMode::ZeroPage),
    );
    hash.insert(
        0x75,
        OpCode::new(0x75, "ADC".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );
    hash.insert(
        0x6D,
        OpCode::new(0x6D, "ADC".to_string(), 4, 3, AddressingMode::Absolute),
    );
    hash.insert(
        0x7D,
        OpCode::new(
            0x7D,
            "ADC".to_string(),
            4, /* +1 if page crossed */
            3,
            AddressingMode::AbsoluteX,
        ),
    );
    hash.insert(
        0x79,
        OpCode::new(
            0x79,
            "ADC".to_string(),
            4, /* +1 is page crossed */
            3,
            AddressingMode::AbsoluteY,
        ),
    );
    hash.insert(
        0x61,
        OpCode::new(0x61, "ADC".to_string(), 6, 2, AddressingMode::IndirectX),
    );
    hash.insert(
        0x71,
        OpCode::new(
            0x71,
            "ADC".to_string(),
            5, /* +1 if page crossed */
            2,
            AddressingMode::IndirectY,
        ),
    );

    // AND
    hash.insert(
        0x29,
        OpCode::new(0x29, "AND".to_string(), 2, 2, AddressingMode::Immediate),
    );
    hash.insert(
        0x25,
        OpCode::new(0x25, "AND".to_string(), 3, 2, AddressingMode::ZeroPage),
    );
    hash.insert(
        0x35,
        OpCode::new(0x35, "AND".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );
    hash.insert(
        0x2D,
        OpCode::new(0x2D, "AND".to_string(), 4, 3, AddressingMode::Absolute),
    );
    hash.insert(
        0x3D,
        OpCode::new(
            0x3D,
            "AND".to_string(),
            4, /* +1 if page crossed */
            3,
            AddressingMode::AbsoluteX,
        ),
    );
    hash.insert(
        0x39,
        OpCode::new(
            0x39,
            "AND".to_string(),
            4, /* +1 if page crossed */
            3,
            AddressingMode::AbsoluteY,
        ),
    );
    hash.insert(
        0x21,
        OpCode::new(0x21, "AND".to_string(), 6, 2, AddressingMode::IndirectX),
    );
    hash.insert(
        0x31,
        OpCode::new(
            0x31,
            "AND".to_string(),
            5, /* +1 if page crossed */
            2,
            AddressingMode::IndirectY,
        ),
    );

    // ASL
    hash.insert(
        0x0A,
        OpCode::new(0x0A, "ASL".to_string(), 2, 1, AddressingMode::Accumulator),
    );
    hash.insert(
        0x06,
        OpCode::new(0x06, "ASL".to_string(), 5, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0x16,
        OpCode::new(0x16, "ASL".to_string(), 6, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0x0E,
        OpCode::new(0x0E, "ASL".to_string(), 6, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0x1E,
        OpCode::new(0x1E, "ASL".to_string(), 7, 3, AddressingMode::AbsoluteX),
    );

    // BCC
    hash.insert(
        0x90,
        OpCode::new(
            0x90,
            "BCC".to_string(),
            2, /*+1 if branche succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BCS
    hash.insert(
        0xB0,
        OpCode::new(
            0xB0,
            "BCS".to_string(),
            2, /*+1 if branche succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BEQ
    hash.insert(
        0x90,
        OpCode::new(
            0x90,
            "BEQ".to_string(),
            2, /*+1 if branche succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BIT
    hash.insert(
        0x24,
        OpCode::new(0x24, "BIT".to_string(), 2, 3, AddressingMode::ZeroPage),
    );

    hash.insert(
        0x2C,
        OpCode::new(0x2C, "BIT".to_string(), 3, 4, AddressingMode::Absolute),
    );

    // BMI
    hash.insert(
        0x30,
        OpCode::new(
            0x30,
            "BMI".to_string(),
            2, /*+1 if branche succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BNE
    hash.insert(
        0xD0,
        OpCode::new(
            0xD0,
            "BNE".to_string(),
            2, /*+1 if branch succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BPL
    hash.insert(
        0x10,
        OpCode::new(
            0x10,
            "BPL".to_string(),
            2, /*+1 if branch succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BRK
    hash.insert(
        0x00,
        OpCode::new(
            0x00,
            "BRK".to_string(),
            7, /*+1 if branch succeeds +2 if a new page*/
            1,
            AddressingMode::Implicit,
        ),
    );

    // BVC
    hash.insert(
        0x50,
        OpCode::new(
            0x50,
            "BVC".to_string(),
            2, /*+1 if branch succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // BVS
    hash.insert(
        0x70,
        OpCode::new(
            0x70,
            "BVS".to_string(),
            2, /*+1 if branch succeeds +2 if a new page*/
            2,
            AddressingMode::Relative,
        ),
    );

    // CLC
    hash.insert(
        0x18,
        OpCode::new(0x18, "CLC".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // CLD
    hash.insert(
        0xD8,
        OpCode::new(0xD8, "CLD".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // CLI
    hash.insert(
        0x58,
        OpCode::new(0x58, "CLI".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // CLV
    hash.insert(
        0xB8,
        OpCode::new(0xB8, "CLV".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // CMP
    hash.insert(
        0xC9,
        OpCode::new(0xC9, "CMP".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0xC5,
        OpCode::new(0xC5, "CMP".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xD5,
        OpCode::new(0xD5, "CMP".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0xCD,
        OpCode::new(0xCD, "CMP".to_string(), 4, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0xDD,
        OpCode::new(
            0xDD,
            "CMP".to_string(),
            4, /*+1 if page crossed*/
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    hash.insert(
        0xD9,
        OpCode::new(
            0xD9,
            "CMP".to_string(),
            4, /*+1 if page crossed*/
            3,
            AddressingMode::AbsoluteY,
        ),
    );

    hash.insert(
        0xC1,
        OpCode::new(0xC1, "CMP".to_string(), 6, 2, AddressingMode::IndirectX),
    );

    hash.insert(
        0xD1,
        OpCode::new(
            0xD1,
            "CMP".to_string(),
            5, /*+1 if page crossed*/
            2,
            AddressingMode::IndirectY,
        ),
    );

    // CPX
    hash.insert(
        0xE0,
        OpCode::new(0xE0, "CPX".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0xE4,
        OpCode::new(0xE4, "CPX".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xEC,
        OpCode::new(0xEC, "CPX".to_string(), 4, 2, AddressingMode::Absolute),
    );

    // CPY
    hash.insert(
        0xC0,
        OpCode::new(0xC0, "CPY".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0xC4,
        OpCode::new(0xC4, "CPY".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xCC,
        OpCode::new(0xCC, "CPY".to_string(), 4, 3, AddressingMode::Immediate),
    );

    // DEC
    hash.insert(
        0xC6,
        OpCode::new(0xC6, "DEC".to_string(), 5, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xD6,
        OpCode::new(0xD6, "DEC".to_string(), 6, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0xCE,
        OpCode::new(0xCE, "DEC".to_string(), 6, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0xDE,
        OpCode::new(0xDE, "DEC".to_string(), 7, 3, AddressingMode::AbsoluteX),
    );

    // DEX
    hash.insert(
        0xCA,
        OpCode::new(0xCA, "DEX".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // DEY
    hash.insert(
        0x88,
        OpCode::new(0x88, "DEY".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // EOR
    hash.insert(
        0x49,
        OpCode::new(0x49, "EOR".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0x45,
        OpCode::new(0x45, "EOR".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0x55,
        OpCode::new(0x55, "EOR".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0x4D,
        OpCode::new(0x4D, "EOR".to_string(), 4, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0x5D,
        OpCode::new(
            0x5D,
            "EOR".to_string(),
            4, /* +1 if page crossed */
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    hash.insert(
        0x59,
        OpCode::new(
            0x59,
            "EOR".to_string(),
            4, /* +1 if page crossed */
            3,
            AddressingMode::AbsoluteY,
        ),
    );

    hash.insert(
        0x41,
        OpCode::new(0x41, "EOR".to_string(), 6, 2, AddressingMode::IndirectX),
    );

    hash.insert(
        0x51,
        OpCode::new(
            0x51,
            "EOR".to_string(),
            5, /* +1 if page crossed */
            2,
            AddressingMode::IndirectY,
        ),
    );

    // INC
    hash.insert(
        0xE6,
        OpCode::new(0xE6, "INC".to_string(), 5, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xF6,
        OpCode::new(0xF6, "INC".to_string(), 6, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0xEE,
        OpCode::new(0xEE, "INC".to_string(), 6, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0xFE,
        OpCode::new(0xFE, "INC".to_string(), 7, 3, AddressingMode::AbsoluteX),
    );

    // INX
    hash.insert(
        0xE8,
        OpCode::new(0xE8, "INX".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // INY
    hash.insert(
        0xC8,
        OpCode::new(0xC8, "INX".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // JMP
    hash.insert(
        0x4C,
        OpCode::new(0x4C, "JMP".to_string(), 3, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0x4C,
        OpCode::new(0x4C, "JMP".to_string(), 3, 3, AddressingMode::Indirect),
    );

    // JSR
    hash.insert(
        0x20,
        OpCode::new(0x20, "JMP".to_string(), 6, 3, AddressingMode::Absolute),
    );

    // LDA
    hash.insert(
        0xA9,
        OpCode::new(0xA9, "LDA".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0xA5,
        OpCode::new(0xA5, "LDA".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xB5,
        OpCode::new(0xB5, "LDA".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0xAD,
        OpCode::new(0xAD, "LDA".to_string(), 4, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0xBD,
        OpCode::new(
            0xBD,
            "LDA".to_string(),
            4, /* (+1 if page is crossed) */
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    hash.insert(
        0xB9,
        OpCode::new(
            0xB9,
            "LDA".to_string(),
            4, /* (+1 if page is crossed) */
            3,
            AddressingMode::AbsoluteY,
        ),
    );

    hash.insert(
        0xA1,
        OpCode::new(0xA1, "LDA".to_string(), 6, 2, AddressingMode::IndirectX),
    );

    hash.insert(
        0xB1,
        OpCode::new(
            0xB1,
            "LDA".to_string(),
            5, /* (+1 if page is crossed) */
            2,
            AddressingMode::IndirectY,
        ),
    );

    // LDX
    hash.insert(
        0xA2,
        OpCode::new(0xA2, "LDX".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0xA6,
        OpCode::new(0xA6, "LDX".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xB6,
        OpCode::new(0xB6, "LDX".to_string(), 4, 2, AddressingMode::ZeroPageY),
    );

    hash.insert(
        0xAE,
        OpCode::new(0xAE, "LDX".to_string(), 4, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0xBE,
        OpCode::new(
            0xBE,
            "LDX".to_string(),
            4, /*+1 if page crossed*/
            3,
            AddressingMode::AbsoluteY,
        ),
    );

    // LDY
    hash.insert(
        0xA0,
        OpCode::new(0xA0, "LDY".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0xA4,
        OpCode::new(0xA4, "LDY".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0xB4,
        OpCode::new(0xB4, "LDY".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0xAC,
        OpCode::new(0xAC, "LDY".to_string(), 4, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0xAC,
        OpCode::new(
            0xAC,
            "LDY".to_string(),
            4, /* +1 if page is crossed */
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    // LSR
    hash.insert(
        0x4A,
        OpCode::new(0x4A, "LSR".to_string(), 2, 1, AddressingMode::Accumulator),
    );

    hash.insert(
        0x46,
        OpCode::new(0x46, "LSR".to_string(), 5, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0x56,
        OpCode::new(0x56, "LSR".to_string(), 6, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0x4E,
        OpCode::new(0x4E, "LSR".to_string(), 6, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0x5E,
        OpCode::new(0x5E, "LSR".to_string(), 7, 3, AddressingMode::AbsoluteX),
    );

    // NOP
    hash.insert(
        0xEA,
        OpCode::new(0xEA, "NOP".to_string(), 2, 1, AddressingMode::Implicit),
    );

    // ORA
    hash.insert(
        0x09,
        OpCode::new(0x09, "ORA".to_string(), 2, 2, AddressingMode::Immediate),
    );

    hash.insert(
        0x05,
        OpCode::new(0x05, "ORA".to_string(), 3, 2, AddressingMode::ZeroPage),
    );

    hash.insert(
        0x15,
        OpCode::new(0x15, "ORA".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0x15,
        OpCode::new(0x15, "ORA".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0x15,
        OpCode::new(0x15, "ORA".to_string(), 4, 2, AddressingMode::ZeroPageX),
    );

    hash.insert(
        0x0D,
        OpCode::new(0x0D, "ORA".to_string(), 4, 3, AddressingMode::Absolute),
    );

    hash.insert(
        0x1D,
        OpCode::new(
            0x1D,
            "ORA".to_string(),
            4, /*+1 if page crossed*/
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    hash.insert(
        0x19,
        OpCode::new(
            0x19,
            "ORA".to_string(),
            4 /*+1 if page crossed*/, 
            3,
            AddressingMode::AbsoluteY,
        ),
    );

    hash.insert(
        0x01,
        OpCode::new(
            0x01,
            "ORA".to_string(),
            6 /*+1 if page crossed*/, 
            2,
            AddressingMode::IndirectX,
        ),
    );

    hash.insert(
        0x11,
        OpCode::new(
            0x11,
            "ORA".to_string(),
            5 /*+1 if page crossed*/, 
            2,
            AddressingMode::IndirectY,
        ),
    );

    // PHA
    hash.insert(
        0x48,
        OpCode::new(
            0x48,
            "PHA".to_string(),
            3 /*+1 if page crossed*/, 
            1,
            AddressingMode::Implicit,
        ),
    );

    // PHP
    hash.insert(
        0x08,
        OpCode::new(
            0x08,
            "PHP".to_string(),
            3 /*+1 if page crossed*/, 
            1,
            AddressingMode::Implicit,
        ),
    );

    // PLA
    hash.insert(
        0x68,
        OpCode::new(
            0x68,
            "PLA".to_string(),
            4 /*+1 if page crossed*/, 
            1,
            AddressingMode::Implicit,
        ),
    );

    // PLP
    hash.insert(
        0x28,
        OpCode::new(
            0x28,
            "PLP".to_string(),
            4, 
            1,
            AddressingMode::Implicit,
        ),
    );

    // ROL
    hash.insert(
        0x2A,
        OpCode::new(
            0x2A,
            "ROL".to_string(),
            2,
            1,
            AddressingMode::Accumulator,
        ),
    );

    hash.insert(
        0x26,
        OpCode::new(
            0x26,
            "ROL".to_string(),
            5,
            2,
            AddressingMode::ZeroPage,
        ),
    );

    hash.insert(
        0x36,
        OpCode::new(
            0x36,
            "ROL".to_string(),
            6,
            2,
            AddressingMode::ZeroPageX,
        ),
    );

    hash.insert(
        0x2E,
        OpCode::new(
            0x2E,
            "ROL".to_string(),
            6,
            3,
            AddressingMode::Absolute,
        ),
    );

    hash.insert(
        0x3E,
        OpCode::new(
            0x3E,
            "ROL".to_string(),
            7,
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    // ROR
    hash.insert(
        0x6A,
        OpCode::new(
            0x6A,
            "ROR".to_string(),
            2,
            1,
            AddressingMode::Accumulator,
        ),
    );

    hash.insert(
        0x66,
        OpCode::new(
            0x66,
            "ROR".to_string(),
            5,
            2,
            AddressingMode::ZeroPage,
        ),
    );

    hash.insert(
        0x76,
        OpCode::new(
            0x76,
            "ROR".to_string(),
            6,
            2,
            AddressingMode::ZeroPageX,
        ),
    );

    hash.insert(
        0x6E,
        OpCode::new(
            0x6E,
            "ROR".to_string(),
            6,
            3,
            AddressingMode::Absolute,
        ),
    );

    hash.insert(
        0x7E,
        OpCode::new(
            0x7E,
            "ROR".to_string(),
            7,
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    // RTI
    hash.insert(
        0x40,
        OpCode::new(
            0x40,
            "RTI".to_string(),
            6,
            1,
            AddressingMode::Implicit,
        ),
    );
     
    // RTS
    hash.insert(
        0x60,
        OpCode::new(
            0x60,
            "RTS".to_string(),
            6,
            1,
            AddressingMode::Implicit,
        ),
    );

    // SBC
    hash.insert(
        0xE9,
        OpCode::new(
            0xE9,
            "SBC".to_string(),
            2,
            2,
            AddressingMode::Immediate,
        ),
    );

    hash.insert(
        0xE5,
        OpCode::new(
            0xE5,
            "SBC".to_string(),
            3,
            2,
            AddressingMode::ZeroPage,
        ),
    );

    hash.insert(
        0xF5,
        OpCode::new(
            0xF5,
            "SBC".to_string(),
            4,
            2,
            AddressingMode::ZeroPageX,
        ),
    );

    hash.insert(
        0xED,
        OpCode::new(
            0xED,
            "SBC".to_string(),
            4,
            3,
            AddressingMode::Absolute,
        ),
    );

    hash.insert(
        0xFD,
        OpCode::new(
            0xFD,
            "SBC".to_string(),
            4 /* +1 if page crossed */,
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    hash.insert(
        0xF9,
        OpCode::new(
            0xF9,
            "SBC".to_string(),
            4 /* +1 if page crossed */,
            3,
            AddressingMode::AbsoluteY,
        ),
    );
    
    hash.insert(
        0xE1,
        OpCode::new(
            0xE1,
            "SBC".to_string(),
            6 /* +1 if page crossed */,
            2,
            AddressingMode::IndirectX,
        ),
    );

    hash.insert(
        0xF1,
        OpCode::new(
            0xF1,
            "SBC".to_string(),
            5 /* +1 if page crossed */,
            2,
            AddressingMode::IndirectY,
        ),
    );

    // SEC
    hash.insert(
        0x38,
        OpCode::new(
            0x38,
            "SEC".to_string(),
            2 /* +1 if page crossed */,
            1,
            AddressingMode::Implicit,
        ),
    );

    // SED
    hash.insert(
        0xF8,
        OpCode::new(
            0xF8,
            "SED".to_string(),
            2 /* +1 if page crossed */,
            1,
            AddressingMode::Implicit,
        ),
    );


    // SEI
    hash.insert(
        0x78,
        OpCode::new(
            0x78,
            "STI".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    // STA
    hash.insert(
        0x85,
        OpCode::new(
            0x85,
            "STA".to_string(),
            3,
            2,
            AddressingMode::ZeroPage,
        ),
    );
   
    hash.insert(
        0x95,
        OpCode::new(
            0x95,
            "STA".to_string(),
            4,
            2,
            AddressingMode::Absolute,
        ),
    );

    hash.insert(
        0x8D,
        OpCode::new(
            0x8D,
            "STA".to_string(),
            4,
            3,
            AddressingMode::AbsoluteX,
        ),
    );

    hash.insert(
        0x99,
        OpCode::new(
            0x99,
            "STA".to_string(),
            5,
            3,
            AddressingMode::AbsoluteY,
        ),
    );

    hash.insert(
        0x81,
        OpCode::new(
            0x81,
            "STA".to_string(),
            6,
            2,
            AddressingMode::IndirectX,
        ),
    );

    hash.insert(
        0x91,
        OpCode::new(
            0x91,
            "STA".to_string(),
            6,
            2,
            AddressingMode::IndirectY,
        ),
    );

    // STX 
    hash.insert(
        0x86,
        OpCode::new(
            0x86,
            "STX".to_string(),
            3,
            2,
            AddressingMode::ZeroPage,
        ),
    );

    hash.insert(
        0x96,
        OpCode::new(
            0x96,
            "STX".to_string(),
            4,
            2,
            AddressingMode::ZeroPageY,
        ),
    );

    hash.insert(
        0x8E,
        OpCode::new(
            0x8E,
            "STX".to_string(),
            4,
            3,
            AddressingMode::Absolute,
        ),
    );

    // STY
    hash.insert(
        0x84,
        OpCode::new(
            0x84,
            "STY".to_string(),
            3,
            2,
            AddressingMode::ZeroPage,
        ),
    );

    hash.insert(
        0x94,
        OpCode::new(
            0x94,
            "STY".to_string(),
            4,
            2,
            AddressingMode::ZeroPageX,
        ),
    );

    hash.insert(
        0x8C,
        OpCode::new(
            0x8C,
            "STY".to_string(),
            4,
            3,
            AddressingMode::Absolute,
        ),
    );

    // TAX
    hash.insert(
        0xAA,
        OpCode::new(
            0xAA,
            "TAX".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    // TAY
    hash.insert(
        0xA8,
        OpCode::new(
            0xA8,
            "TAY".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    // TSX
    hash.insert(
        0xBA,
        OpCode::new(
            0xBA,
            "TSX".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    // TXA
    hash.insert(
        0x8A,
        OpCode::new(
            0x8A,
            "TXA".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    // TXS
    hash.insert(
        0x9A,
        OpCode::new(
            0x9A,
            "TXS".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    // TYA
    hash.insert(
        0x98,
        OpCode::new(
            0x98,
            "TYA".to_string(),
            2,
            1,
            AddressingMode::Implicit,
        ),
    );

    return hash;
}

#[cfg(test)]
mod test {
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

    #[test]
    fn test_adc_overflow() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 80, 0x33]);

        cpu.acc_reg = 80;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0100_0000 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_not_overflow() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 10, 0x33]);

        cpu.acc_reg = 80;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0100_0000 & cpu.status) == 0)
    }

    #[test]
    fn test_adc_not_underflow() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 0xd0 /* - 10*/, 0x33]);

        cpu.acc_reg = 0xd0; // -48
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0100_0000 & cpu.status) == 0)
    }

    #[test]
    fn test_adc_underflow() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 0x90 /*-112*/, 0x33]);

        cpu.acc_reg = 0xd0; // -48
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0100_0000 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_overflow_flag_negative_positive_numbers() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 0xa0 /*10*/, 0x33]);

        cpu.acc_reg = 0xd0; // -48
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0100_0000 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_carry_set_80_208_acc() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 80, 0x33]);

        cpu.acc_reg = 208;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0000_0001 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_carry_set_208_80_acc() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 208, 0x33]);

        cpu.acc_reg = 80;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0000_0001 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_carry_set_208_144_acc() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 208, 0x33]);

        cpu.acc_reg = 144;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0000_0001 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_carry_set_208_208() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 208, 0x33]);

        cpu.acc_reg = 208;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0000_0001 & cpu.status) != 0)
    }

    #[test]
    fn test_adc_carry_doesnt_set_numbers() {
        let mut cpu = CPU::new();

        cpu.load(vec![0x69, 40, 0x33]);

        cpu.acc_reg = 208;
        cpu.pc = cpu.mem_read_u16(0xfffc);
        cpu.run();

        assert!((0b0000_0001 & cpu.status) == 0)
    }
}
