#[allow(dead_code)]
mod arch;
mod display;
mod instructions;

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

pub type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let rom = ROMS[3];
    let mut cpu = arch::CPU::new(rom.to_string());
    cpu.single_step = true;

    if let Err(e) = cpu.boot().await {
        println!("Error: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::arch;

    #[tokio::test]
    async fn it_works() {
        let cpu = arch::CPU::new("".to_string());
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

    #[tokio::test]
    async fn emulate_cycle() {
        let mut cpu = arch::CPU::new("chip8-test-suite/bin/1-chip8-logo.ch8".to_string());
        cpu.emulate_cycle().await;
        assert_eq!(cpu.pc, 0x200 + 2);
    }
}
