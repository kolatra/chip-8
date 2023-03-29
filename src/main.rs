use std::collections::HashMap;
#[allow(dead_code)]
use std::time::Duration;

use sdl2::{event::Event, keyboard::Keycode};

mod arch;
mod display;

const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;
const FPS: u32 = 144;
const ROMS: [&str; 6] = [
    "chip8-test-suite/bin/1-chip8-logo.ch8",
    "chip8-test-suite/bin/2-ibm-logo.ch8",
    "chip8-test-suite/bin/3-corax+.ch8",
    "chip8-test-suite/bin/4-flags.ch8",
    "chip8-test-suite/bin/5-quirks.ch8",
    "chip8-test-suite/bin/6-keypad.ch8",
];

fn main() -> Result<(), String> {
    let mut cpu = arch::CPU::new();
    let rom = std::fs::read(ROMS[4]).unwrap();
    cpu.load_rom(rom);

    let sdl_context = sdl2::init()?;
    let mut display_driver = display::DisplayDriver::new(&sdl_context, "chip-8 emulator");
    let mut event_pump = sdl_context.event_pump()?;

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

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        let new_keys = event_pump.keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect::<Vec<_>>();
        for key in keymap.keys() {
            if new_keys.contains(key) {
                cpu.keys[keymap[key]] = true;
            } else {
                cpu.keys[keymap[key]] = false;
            }
        }

        cpu.emulate_cycle();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
        if cpu.draw_flag {
            display_driver.draw(&cpu.display);
            cpu.draw_flag = false;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::arch;

    #[test]
    fn it_works() {
        let cpu = arch::CPU::new();
        assert_eq!(cpu.registers[0], 0);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.index, 0);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.stack[0], 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);
        assert_eq!(cpu.keys[0], false);
        assert_eq!(cpu.display[0], [0; 64]);
    }

    #[test]
    fn emulate_cycle() {
        let mut cpu = arch::CPU::new();
        let rom = std::fs::read("chip8-test-suite/bin/1-chip8-logo.ch8").unwrap();
        cpu.load_rom(rom);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }
}
