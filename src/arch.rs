pub struct CPU {
    pub registers: [u8; 16],
    pub memory: [u8; 4096],
    pub index: u16,
    pub pc: u16,
    pub stack: [u16; 16],
    pub sp: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    // 0-f
    pub keys: [u8; 16],
    pub display: [[u8; 64]; 32],

    pub draw_flag: bool,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keys: [0; 16],
            display: [[0; 64]; 32],
            draw_flag: false,
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }

    pub fn emulate_cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        if opcode == 0 {
            return;
        }
        println!("{:#04x}", opcode);

        match opcode & 0xf000 {
            0x0000 => match opcode {
                0x00E0 => self.clear_screen(),
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            0x1000 => self.jump(opcode),
            0x2000 => self.call(opcode),
            0x3000 => self.skip_if_equal(opcode),
            0x4000 => self.skip_if_not_equal(opcode),
            0x5000 => self.skip_if_equal_2(opcode),
            0x6000 => self.set_reg(opcode),
            0x7000 => self.add(opcode),
            0x8000 => match opcode & 0x000f {
                0x0000 => self.set_reg_2(opcode),
                0x0001 => self.or(opcode),
                0x0002 => self.and(opcode),
                0x0003 => self.xor(opcode),
                0x0004 => self.add_2(opcode),
                0x0005 => self.sub(opcode),
                0x0006 => self.shr(opcode),
                0x0007 => self.sub_2(opcode),
                0x000E => self.shl(opcode),
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            0x9000 => self.skip_if_not_equal_2(opcode),
            0xa000 => self.set_index(opcode),
            0xd000 => self.draw(opcode),
            0xf000 => match opcode & 0x00ff {
                0x0033 => self.store_bcd(opcode),
                0x0055 => self.store_registers(opcode),
                0x0065 => self.load_registers(opcode),
                0x001e => self.add_index(opcode),
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            _ => panic!("Unknown opcode: {:x}", opcode),
        }
    }

    fn jump(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.pc = nnn;
    }

    fn call(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    fn clear_screen(&mut self) {
        self.display = [[0; 64]; 32];
    }

    fn skip_if_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        if self.registers[x] == nn {
            self.pc += 2;
        }
    }

    fn skip_if_not_equal(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        if self.registers[x] != nn {
            self.pc += 2;
        }
    }

    fn skip_if_equal_2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] == self.registers[y] {
            self.pc += 2;
        }
    }

    fn set_reg(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        println!("x:{} nn:{}", x, nn);
        self.registers[x] = nn;
    }

    fn add(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        println!("{} + {}", self.registers[x], nn);
        let _ = self.registers[x].wrapping_add(nn);
    }

    fn set_reg_2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] = self.registers[y];
    }

    fn or(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] |= self.registers[y];
    }

    fn and(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] &= self.registers[y];
    }

    fn xor(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[x] ^= self.registers[y];
    }

    fn add_2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 1 } else { 0 };
    }

    fn sub(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 0 } else { 1 };
    }

    fn shr(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[0xF] = self.registers[x] & 0x1;
        self.registers[x] >>= 1;
    }

    fn sub_2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 0 } else { 1 };
    }

    fn shl(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.registers[0xF] = self.registers[x] >> 7;
        self.registers[x] <<= 1;
    }

    fn skip_if_not_equal_2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }

    fn set_index(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.index = nnn;
    }

    fn draw(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as usize;

        let x = self.registers[x] as usize % 64;
        let y = self.registers[y] as usize % 32;

        let mut pixel: u8;
        self.registers[0xF] = 0;
        for yline in 0..n {
            pixel = self.memory[(self.index + yline as u16) as usize];
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    if self.display[y + yline][x + xline] == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.display[y + yline][x + xline] ^= 1;
                }
            }
        }
        self.draw_flag = true;
    }

    fn store_bcd(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let value = self.registers[x];
        self.memory[self.index as usize] = value / 100;
        self.memory[self.index as usize + 1] = (value / 10) % 10;
        self.memory[self.index as usize + 2] = (value % 100) % 10;
    }

    fn store_registers(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.memory[self.index as usize + i] = self.registers[i];
        }
    }

    fn load_registers(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        for i in 0..=x {
            self.registers[i] = self.memory[self.index as usize + i];
        }
    }

    fn add_index(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.index += self.registers[x] as u16;
    }
}
