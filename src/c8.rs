use crate::display::DisplayDriver;
use sdl2::{event::Event, keyboard::Keycode};
use std::{collections::HashMap, time::Duration};

pub struct Cpu {
    pub memory: [u8; 4096],
    pub stack: [u16; 16],

    pub pc: u16,
    pub sp: u8,
    pub vx: [u8; 16],
    pub i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,

    pub keys: [bool; 16],
    pub display: [[u8; 64]; 32],

    pub single_step: bool,
    pub draw_flag: bool,
    pub rom_loaded: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            memory: [0; 4096],
            stack: [0; 16],
            pc: 0x200,
            sp: 0,
            vx: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; 16],
            display: [[0; 64]; 32],
            single_step: false,
            draw_flag: false,
            rom_loaded: false,
        }
    }

    pub async fn load_rom(&mut self, path: &str) {
        if self.rom_loaded {
            println!("[!] ROM already loaded");
            return;
        }

        let rom = std::fs::read(path).unwrap();
        println!("[*] ROM: {} size: {:?} bytes", path, rom.len());

        for (i, byte) in rom.into_iter().enumerate() {
            self.memory[0x200 + i] = byte;
        }

        self.rom_loaded = true;
    }

    pub async fn boot(&mut self) -> Result<(), crate::Error> {
        let title = if self.single_step {
            "CHIP-8 Emulator (Single Step)"
        } else {
            "CHIP-8 Emulator"
        };
        let sdl_context = sdl2::init()?;
        let keymap = self.load_keys();
        let mut display_driver = DisplayDriver::new(&sdl_context, title);
        let mut event_pump = sdl_context.event_pump()?;
        let mut old_keys = vec![];
        let mut step = false;

        loop {
            for event in event_pump.poll_iter() {
                match self.check_keys(&mut step, event) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => continue,
                }
            }

            for _ in 0..8 {
                if !self.single_step || step {
                    self.emulate_cycle().await;
                    step = false;
                }
            }

            let new_keys = event_pump
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect::<Vec<_>>();

            for key in keymap.keys() {
                self.keys[keymap[key]] = old_keys.contains(key);
            }

            old_keys.clear();
            old_keys.extend(new_keys);

            if self.draw_flag {
                display_driver.draw(&self.display);
                self.draw_flag = false;
            }

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / crate::FPS));
        }
    }

    fn load_keys(&self) -> HashMap<Keycode, usize> {
        let mut keymap = HashMap::new();
        keymap.insert(Keycode::Num1, 0x1);
        keymap.insert(Keycode::Num2, 0x2);
        keymap.insert(Keycode::Num3, 0x3);
        keymap.insert(Keycode::Num4, 0xC);
        keymap.insert(Keycode::Q, 0x4);
        keymap.insert(Keycode::W, 0x5);
        keymap.insert(Keycode::E, 0x6);
        keymap.insert(Keycode::R, 0xD);
        keymap.insert(Keycode::A, 0x7);
        keymap.insert(Keycode::S, 0x8);
        keymap.insert(Keycode::D, 0x9);
        keymap.insert(Keycode::F, 0xE);
        keymap.insert(Keycode::Z, 0xA);
        keymap.insert(Keycode::X, 0x0);
        keymap.insert(Keycode::C, 0xB);
        keymap.insert(Keycode::V, 0xF);

        keymap
    }

    fn check_keys(&mut self, step: &mut bool, event: Event) -> Result<bool, crate::Error> {
        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => self.print_registers(),

            Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } => self.single_step = !self.single_step,

            Event::KeyDown {
                keycode: Some(Keycode::J),
                ..
            } => {
                if !self.single_step {
                    return Ok(false);
                } else {
                    *step = true;
                }
            }

            Event::Quit { .. } => return Err("Shutting down".into()),

            _ => return Ok(false),
        }

        Ok(true)
    }

    fn opcode(&self) -> u16 {
        (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16
    }

    pub async fn emulate_cycle(&mut self) {
        let opcode = self.opcode();

        self.pc += 2;
        if opcode == 0 {
            return;
        }

        if self.single_step {
            println!("{:#04x}", opcode);
        }

        self.execute(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            println!("BEEP!");
            self.sound_timer -= 1;
        }
    }

    fn print_registers(&self) {
        println!("[*] Registers:");
        for (i, reg) in self.vx.iter().enumerate() {
            println!("[*] V{:X}: {:#04x}", i, reg);
        }
        println!("[*] Index: {:#04x}", self.i);
        println!("[*] PC: {:#04x}", self.pc);
        println!("[*] SP: {:#04x}", self.sp);
        println!("[*] Delay timer: {:#04x}", self.delay_timer);
        println!("[*] Sound timer: {:#04x}", self.sound_timer);
    }

    fn draw(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as usize;

        let x = self.vx[x] as usize % 64;
        let y = self.vx[y] as usize % 32;

        let mut pixel: u8;
        self.vx[0xF] = 0;
        for yline in 0..n {
            pixel = self.memory[(self.i + yline as u16) as usize];
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    let ia = y + yline;
                    let ib = x + xline;

                    if ia >= 32 || ib >= 64 {
                        continue;
                    }

                    if self.display[ia][ib] == 1 {
                        self.vx[0xF] = 1;
                    }
                    self.display[y + yline][x + xline] ^= 1;
                }
            }
        }
        self.draw_flag = true;
    }

    fn execute(&mut self, opcode: u16) {
        let nibbles = (
            ((opcode & 0xf000) >> 12) as u8,
            ((opcode & 0x0f00) >> 8) as u8,
            ((opcode & 0x00f0) >> 4) as u8,
            (opcode & 0x000f) as u8,
        );

        match nibbles {
            // CLS
            (0x0, 0x0, 0xE, 0x0) => self.display = [[0; 64]; 32],
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }

            // JP addr
            (0x1, _, _, _) => self.pc = opcode & 0x0fff,

            // CALL addr
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0fff;
            }

            // SE Vx, byte
            (0x3, _, _, _) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                if self.vx[x] == nn {
                    self.pc += 2;
                }
            }

            // SNE Vx, byte
            (0x4, _, _, _) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                if self.vx[x] != nn {
                    self.pc += 2;
                }
            }

            // SE Vx, Vy
            (0x5, _, _, 0x0) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let y = ((opcode & 0x00f0) >> 4) as usize;
                if self.vx[x] == self.vx[y] {
                    self.pc += 2;
                }
            }

            // LD Vx, byte
            (0x6, _, _, _) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                self.vx[x] = nn;
            }

            // ADD Vx, byte
            (0x7, _, _, _) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                self.vx[x] = self.vx[x].wrapping_add(nn);
            }

            // LD Vx, Vy
            (0x8, _, _, _) => match nibbles.3 {
                // LD Vx, Vy
                0x0 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.vx[x] = self.vx[y];
                }

                // OR Vx, Vy
                0x1 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.vx[x] |= self.vx[y];
                }

                // AND Vx, Vy
                0x2 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.vx[x] &= self.vx[y];
                }

                // XOR Vx, Vy
                0x3 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    self.vx[x] ^= self.vx[y];
                }

                // ADD Vx, Vy
                0x4 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;

                    let (res, overflow) = self.vx[x].overflowing_add(self.vx[y]);
                    self.vx[x] = res;
                    self.vx[0xF] = overflow as u8;
                }

                // SUB Vx, Vy
                0x5 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;

                    let (res, overflow) = self.vx[x].overflowing_sub(self.vx[y]);
                    self.vx[x] = res;
                    self.vx[0xF] = overflow as u8;
                }

                // SHR Vx, Vy
                0x6 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let vf = self.vx[x] & 1;
                    self.vx[x] >>= 1;
                    self.vx[0xf] = vf;
                }

                // SUBN Vx, Vy
                0x7 => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let y = ((opcode & 0x00f0) >> 4) as usize;
                    let (result, overflow) = self.vx[y].overflowing_sub(self.vx[x]);
                    self.vx[x] = result;
                    self.vx[0xf] = overflow as u8;
                }

                // SHL Vx, Vy
                0xe => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let vf = self.vx[x] >> 7;
                    self.vx[x] <<= 1;
                    self.vx[0xf] = vf;
                }

                _ => panic!("Unknown opcode: {:#x}", opcode),
            }

            // SNE Vx, Vy
            (0x9, _, _, 0x0) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let y = ((opcode & 0x00f0) >> 4) as usize;
                if self.vx[x] != self.vx[y] {
                    self.pc += 2;
                }
            }

            // LD I, addr
            (0xa, _, _, _) => {
                let nnn = opcode & 0x0fff;
                self.i = nnn;
            }

            // JP V0, addr
            (0xb, _, _, _) => {
                let nnn = opcode & 0x0fff;
                self.pc = self.vx[0] as u16 + nnn;
            }

            // RND Vx, byte
            (0xc, _, _, _) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                let nn = (opcode & 0x00ff) as u8;
                self.vx[x] = rand::random::<u8>() & nn;
            }

            // DRW Vx, Vy, nibble
            (0xd, _, _, _) => self.draw(opcode),

            (0xe, _, _, _) => {
                let x = ((opcode & 0x0f00) >> 8) as usize;
                match opcode & 0x00ff {
                    // SKP Vx
                    0x009e => {
                        if self.keys[self.vx[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    // SKNP Vx
                    0x00a1 => {
                        if !self.keys[self.vx[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => panic!("Unknown opcode: {:x}", opcode),
                }
            }

            (0xf, _, _, _) => match (nibbles.2, nibbles.3) {
                // LD Vx, DT
                (0x0, 0x7) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    self.vx[x] = self.delay_timer;
                }

                // LD Vx, K
                (0x0, 0xa) => {
                    let _x = ((opcode & 0x0f00) >> 8) as usize;
                }

                // LD DT, Vx
                (0x1, 0x5) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    self.delay_timer = self.vx[x];
                }

                // Sets I to the location of the sprite for the character in VX.
                (0x1, 0x8) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let vx = self.vx[x] as usize;
                    self.i = crate::FONTSET[vx] as u16;
                }

                // LD B, Vx
                (0x3, 0x3) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    let value = self.vx[x];
                    self.memory[self.i as usize] = value / 100;
                    self.memory[self.i as usize + 1] = (value / 10) % 10;
                    self.memory[self.i as usize + 2] = (value % 100) % 10;
                }

                // LD [I], Vx
                (0x5, 0x5) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    for i in 0..=x {
                        self.memory[self.i as usize + i] = self.vx[i];
                    }
                }

                // LD Vx, [I]
                (0x6, 0x5) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    for i in 0..=x {
                        self.vx[i] = self.memory[self.i as usize + i];
                    }
                }

                // ADD I, Vx
                (0x1, 0xe) => {
                    let x = ((opcode & 0x0f00) >> 8) as usize;
                    self.i += self.vx[x] as u16;
                }

                _ => panic!("Unknown opcode: {:x}", opcode),
            },
            _ => panic!("Unknown opcode: {:x}", opcode),
        }
    }
}
