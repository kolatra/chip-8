use crate::display::DisplayDriver;
use sdl2::{event::Event, keyboard::Keycode};
use std::{collections::HashMap, time::Duration};

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
    pub keys: [bool; 16],
    pub display: [[u8; 64]; 32],
    pub fontset: Vec<u8>,

    pub single_step: bool,
    pub draw_flag: bool,
    pub rom_loaded: bool,
    pub rom_path: String
}

impl CPU {
    pub fn new(rom_path: String) -> CPU {
        let fontset = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        CPU {
            registers: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keys: [false; 16],
            display: [[0; 64]; 32],
            fontset,
            single_step: false,
            draw_flag: false,
            rom_loaded: false,
            rom_path
        }
    }

    pub async fn boot(&mut self) -> Result<(), crate::Error> {
        if !self.rom_loaded {
            self.load_rom().await;
        }

        let sdl_context = sdl2::init()?;
        let title = if self.single_step {
            "CHIP-8 Emulator (Single Step)"
        } else {
            "CHIP-8 Emulator"
        };
        let mut display_driver = DisplayDriver::new(&sdl_context, &title);
        let mut event_pump = sdl_context.event_pump()?;
        let keymap = self.load_keys().await;
        let mut old_keys = vec![];
        let mut step = false;

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => self.print_registers().await,

                    Event::KeyDown {
                        keycode: Some(Keycode::H),
                        ..
                    } => self.single_step = !self.single_step,

                    Event::KeyDown {
                        keycode: Some(Keycode::J),
                        ..
                    } => {
                        if !self.single_step {
                            continue;
                        }

                        step = true;
                    },

                    Event::Quit { .. } => break 'running,

                    _ => continue,
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
                if old_keys.contains(key) {
                    self.keys[keymap[key]] = true;
                } else {
                    self.keys[keymap[key]] = false;
                }
            }

            old_keys.clear();
            old_keys.extend(new_keys);

            if self.draw_flag {
                display_driver.draw(&self.display);
                self.draw_flag = false;
            }

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / crate::FPS));
        }

        Ok(())
    }

    pub async fn load_rom(&mut self) {
        let path = self.rom_path.clone();
        let rom = std::fs::read(&path).unwrap();
        println!("ROM: {} size:{:?} bytes", path, rom.len());

        for (i, byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
        self.rom_loaded = true;
    }

    pub async fn load_keys(&self) -> HashMap<Keycode, usize> {
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

    pub async fn emulate_cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        if opcode == 0 {
            return;
        }

        if self.single_step {
            println!("{:#04x}", opcode);
        }

        self.parse_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            println!("BEEP!");
            self.sound_timer -= 1;
        }
    }

    pub async fn print_registers(&self) {
        println!("Registers:");
        for (i, reg) in self.registers.iter().enumerate() {
            println!("V{:X}: {:#04x}", i, reg);
        }
        println!("Index: {:#04x}", self.index);
        println!("PC: {:#04x}", self.pc);
        println!("SP: {:#04x}", self.sp);
        println!("Delay timer: {:#04x}", self.delay_timer);
        println!("Sound timer: {:#04x}", self.sound_timer);
    }
}
