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

        match opcode & 0xf000 {
            0x0000 => match opcode {
                0x00E0 => self.clear_screen(),
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            0x1000 => self.jump(opcode),
            0x6000 => self.set_reg(opcode),
            0x7000 => self.add(opcode),
            0xa000 => self.set_index(opcode),
            0xd000 => self.draw(opcode),
            _ => panic!("Unknown opcode: {:x}", opcode),
        }
    }

    pub fn clear_screen(&mut self) {
        self.display = [[0; 64]; 32];
    }

    pub fn jump(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.pc = nnn;
    }

    pub fn set_reg(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] = nn;
    }

    pub fn add(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let nn = (opcode & 0x00FF) as u8;
        self.registers[x] += nn;
    }

    pub fn set_index(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.index = nnn;
    }

    pub fn draw(&mut self, opcode: u16) {
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
}
