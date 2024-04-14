struct CPU {
    acc_reg: u8,
    pc: u16,
    status: u8,
}


impl CPU {
    pub fn new() -> Self {
        Self {
            acc_reg: 0,
            pc: 0,
            status: 0
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.pc = 0;

        loop {
            let opcode = program.get(self.pc as usize).unwrap();
            self.pc += 1; 

            match *opcode {
                0xA9 => {
                    let param = program.get(self.pc as usize).unwrap();
                    self.acc_reg = *param;
                    self.pc += 1;

                    if self.acc_reg == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    if (self.acc_reg & 0b1000_0000) != 0 {
                       self.status = self.status | 0b1000_0000;
                    } else  {
                        self.status = self.status & 0b0111_1111;
                    }
                },
                0x00 => {
                    return;
                }
                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod test{ 
    use super::*;

    #[test]
    fn test_0xA9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0x05, 0x00]);
        assert!(cpu.acc_reg == 0x05);
        
        // Check if negative flag is set, which it shouldn't
        assert!(cpu.status & 0b1000_0000 == 0);
        
        // Check if result zero flag is set, which it shouldn't
        assert!(cpu.status & 0b0000_0010 == 0)
    }

    #[test]
    fn test_0xA_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0x00, 0x00]);
        // Check if result zero flag is set, which it should
        assert!(cpu.status & 0b0000_0010 != 0)
    }

    #[test]
    fn test_0xA_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0xFF, 0x00]);
        // Check if result zero flag is set, which it should
        assert!(cpu.status & 0b1000_0000 != 0)
    }
}
