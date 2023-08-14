#[allow(dead_code)]
mod c8;
mod display;

const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;
const FPS: u32 = 144;

const FONTSET: [u8; 80] = [
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

const ROMS: [&str; 10] = [
    "pong.rom",
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

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut cpu = c8::Cpu::new();
    cpu.load_rom(ROMS[1]).await;
    cpu.single_step = false;
    cpu.boot().await
}

#[cfg(test)]
mod tests {
    use crate::c8;

    #[tokio::test]
    async fn test_init() {
        let cpu = c8::Cpu::new();
        assert_eq!(cpu.vx[0], 0);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.i, 0);
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.stack[0], 0);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.sound_timer, 0);
        assert_eq!(cpu.display[0], [0; 64]);
        
        for k in cpu.keys {
            assert!(!k);
        }
    }

    #[tokio::test]
    async fn test_cycle() {
        let mut cpu = c8::Cpu::new();
        cpu.emulate_cycle().await;
        assert_eq!(cpu.pc, 0x200 + 2);
    }
}
