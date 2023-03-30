use crate::cpu::CPU;

impl CPU {
    pub fn parse_opcode(&mut self, opcode: u16) {
        match opcode & 0xf000 {
            0x0000 => match opcode {
                // CLS
                0x00E0 => self.display = [[0; 64]; 32],
                // RET
                0x00EE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            // JP addr
            0x1000 => self.pc = opcode & 0x0fff,
            // CALL addr
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0fff;
            }
            // SE Vx, byte
            0x3000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                if self.registers[x] == nn {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte
            0x4000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                if self.registers[x] != nn {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy
            0x5000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let y = ((opcode & 0x00f0) >> 4) as usize;
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte
            0x6000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                self.registers[x] = nn;
            }
            // ADD Vx, byte
            0x7000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                self.registers[x] = self.registers[x].wrapping_add(nn);
            }
            0x8000 => match opcode & 0x000f {
                // LD Vx, Vy
                0x0000 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.registers[x] = self.registers[y];
                }
                // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
                // OR Vx, Vy
                0x0001 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.registers[x] |= self.registers[y];
                }
                // AND Vx, Vy
                0x0002 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.registers[x] &= self.registers[y];
                }
                // XOR Vx, Vy
                0x0003 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.registers[x] ^= self.registers[y];
                }
                // ADD Vx, Vy
                0x0004 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
                    self.registers[x] = result;
                    self.registers[0xf] = if overflow { 1 } else { 0 };
                }
                // SUB Vx, Vy
                0x0005 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
                    self.registers[x] = result;
                    self.registers[0xf] = if overflow { 0 } else { 1 };
                }
                // SHR Vx, Vy
                0x0006 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let vf = self.registers[x] & 1;
                    self.registers[x] >>= 1;
                    self.registers[0xf] = vf;
                }
                // SUBN Vx, Vy
                0x0007 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
                    self.registers[x] = result;
                    self.registers[0xf] = if overflow { 0 } else { 1 };
                }
                // SHL Vx, Vy
                0x000e => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let vf = self.registers[x] >> 7;
                    self.registers[x] <<= 1;
                    self.registers[0xf] = vf;
                }
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            // SNE Vx, Vy
            0x9000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let y = ((opcode & 0x00f0) >> 4) as usize;
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }
            // LD I, addr
            0xa000 => {
                let nnn = opcode & 0x0fff;
                self.index = nnn;
            }
            // JP V0, addr
            0xb000 => {
                let nnn = opcode & 0x0fff;
                self.pc = self.registers[0] as u16 + nnn;
            }
            // DRW Vx, Vy, nibble
            0xd000 => self.draw(opcode),
            0xe000 => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                match opcode & 0x00ff {
                    // SKP Vx
                    0x009e => {
                        if self.keys[self.registers[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    // SKNP Vx
                    0x00a1 => {
                        if !self.keys[self.registers[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => panic!("Unknown opcode: {:x}", opcode),
                }
            }
            0xf000 => match opcode & 0x00ff {
                // LD Vx, K
                0x000a => {
                    let _x = ((opcode & 0x0f00) >> 8) as usize;
                }
                // LD Vx, DT
                0x0007 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    self.registers[x] = self.delay_timer;
                }
                // LD DT, Vx
                0x0015 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    self.delay_timer = self.registers[x];
                }
                // Sets I to the location of the sprite for the character in VX.
                0x0029 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let vx = self.registers[x] as usize;
                    self.index = self.fontset[vx] as u16;
                }
                // LD B, Vx
                0x0033 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let value = self.registers[x];
                    self.memory[self.index as usize] = value / 100;
                    self.memory[self.index as usize + 1] = (value / 10) % 10;
                    self.memory[self.index as usize + 2] = (value % 100) % 10;
                }
                // LD [I], Vx
                0x0055 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    for i in 0..=x {
                        self.memory[self.index as usize + i] = self.registers[i];
                    }
                }
                // LD Vx, [I]
                0x0065 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    for i in 0..=x {
                        self.registers[i] = self.memory[self.index as usize + i];
                    }
                }
                // ADD I, Vx
                0x001e => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    self.index += self.registers[x] as u16;
                }
                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            _ => panic!("Unknown opcode: {:x}", opcode),
        }
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
                    let ia = y + yline;
                    let ib = x + xline;

                    if ia >= 32 || ib >= 64 {
                        continue;
                    }

                    if self.display[ia][ib] == 1 {
                        self.registers[0xF] = 1;
                    }
                    self.display[y + yline][x + xline] ^= 1;
                }
            }
        }
        self.draw_flag = true;
    }
}
