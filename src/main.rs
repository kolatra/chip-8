#[allow(dead_code)]
use std::time::Duration;

use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

mod arch;
mod display;

const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;
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
    let rom = std::fs::read(ROMS[2]).unwrap();
    cpu.load_rom(rom);

    let sdl_context = sdl2::init()?;
    let mut display_driver = display::DisplayDriver::new(&sdl_context, "chip-8 emulator");
    let mut event_pump = sdl_context.event_pump()?;

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

        cpu.emulate_cycle();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
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
        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.stack[0], 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);
        assert_eq!(cpu.keys[0], 0);
        assert_eq!(cpu.display[0], 0);
    }

    #[test]
    fn emulate_cycle() {
        let mut cpu = arch::CPU::new();
        let rom = std::fs::read("chip8-test-suite/bin/1-chip8-logo.ch8").unwrap();
        cpu.load_rom(rom);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 2);
    }
}
