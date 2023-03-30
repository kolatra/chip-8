#[allow(dead_code)]
mod cpu;
mod display;
mod instructions;

use std::error::Error;

const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;
const FPS: u32 = 60;
const ROMS: [&str; 9] = [
    "chip8-test-suite/bin/1-chip8-logo.ch8",
    "chip8-test-suite/bin/2-ibm-logo.ch8",
    "chip8-test-suite/bin/3-corax+.ch8",
    "chip8-test-suite/bin/4-flags.ch8",
    "chip8-test-suite/bin/5-quirks.ch8",
    "chip8-test-suite/bin/6-keypad.ch8",
    "./test_opcode.ch8",
    "./delay_timer_test.ch8",
    "./random_number_test.ch8",
];

fn main() -> Result<(), Box<dyn Error>> {
    let mut cpu = cpu::CPU::new();
    let path = ROMS[3];
    let rom = std::fs::read(path).unwrap();
    println!("ROM: {} size:{:?} bytes", path, rom.len());
    cpu.load_rom(rom);
    cpu.run()
}

#[cfg(test)]
mod tests {
    use crate::cpu;

    #[test]
    fn it_works() {
        let cpu = cpu::CPU::new();
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
        let mut cpu = cpu::CPU::new();
        let rom = std::fs::read("chip8-test-suite/bin/1-chip8-logo.ch8").unwrap();
        cpu.load_rom(rom);
        cpu.emulate_cycle();
        assert_eq!(cpu.pc, 0x200 + 2);
    }

    #[test]
    fn test_shift() {
        // 8xye 8xy6
        let mut cpu = cpu::CPU::new();
        cpu.registers[0] = 0b0000_0001;

        cpu.print_registers();
        cpu.parse_opcode(0x812e);
        cpu.print_registers();
    }
}
