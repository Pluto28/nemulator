struct CPU {
    acc_reg: u8,
    pc: u16,
    status: u8,
    pub reg_x: u8
}


impl CPU {
    pub fn new() -> Self {
        Self {
            acc_reg: 0,
            pc: 0,
            status: 0,
            reg_x: 0
        }
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

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.pc = 0;

        loop {
            let opcode = program.get(self.pc as usize).unwrap();
            self.pc += 1; 

            match *opcode {
                0xA9 => {
                    let param = program.get(self.pc as usize).unwrap();
                    self.pc += 1;

                    self.lda(*param);
                },
                0xE8 => {
                    self.inx()
                },
                0xAA => {
                    self.tax()
                },
                0x00 => {
                    return;
                },
                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod test{ 
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0x05, 0x00]);
        assert!(cpu.acc_reg == 0x05);
        
        // Check if negative flag is set, which it shouldn't
        assert!(cpu.status & 0b1000_0000 == 0);
        
        // Check if result zero flag is set, which it shouldn't
        assert!(cpu.status & 0b0000_0010 == 0)
    }

    #[test]
    fn test_0xa_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0x00, 0x00]);
        // Check if result zero flag is set, which it should
        assert!(cpu.status & 0b0000_0010 != 0)
    }

    #[test]
    fn test_0xa_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xA9, 0xFF, 0x00]);
        // Check if result zero flag is set, which it should
        assert!(cpu.status & 0b1000_0000 != 0)
    }

    #[test]
    fn test_0xaa_tax_moves_a_to_x() {
        let mut cpu = CPU::new();
        cpu.acc_reg = 0x15;
        cpu.interpret(vec![0xAA, 0x00]);

        assert!(cpu.reg_x == 0x15);
        assert!((cpu.status & 0b0000_0010) == 0);
        assert!((cpu.status & 0b1000_0000) == 0);
    }

    #[test]
    fn test_0xaa_tax_moves_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.acc_reg = 0x0;
        cpu.interpret(vec![0xAA, 0x00]);

        assert!((cpu.status & 0b0000_0010) != 0)
    }

    #[test]
    fn test_0xaa_tax_moves_sets_negative_flag() {
        let mut cpu = CPU::new();
        cpu.acc_reg = 0b1000_0000;
        cpu.interpret(vec![0xAA, 0x00]);

        assert!((cpu.status & 0b1000_0000) != 0)
    }

    #[test]
    fn text_0xe8_inc_reg_x() {
        let mut cpu = CPU::new();
        cpu.reg_x = 20;
        cpu.interpret(vec![0xe8, 0x00]);
        assert!(cpu.reg_x == 21);
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.reg_x = 0xff; 
        cpu.interpret(vec![0xe8, 0xe8, 0x00]);

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
        cpu.interpret(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.reg_x, 0xc1)
    }

}
