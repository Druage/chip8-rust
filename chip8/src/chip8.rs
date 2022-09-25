use crate::fonts;

const STARTING_PC_OFFSET: u16 = 0x200;

pub struct Chip8 {
    memory: [i32; 4096],
    registers: [u8; 16],
    stack: [u16; 16],
    input: [u8; 16],
    gfx: [u8; 64 * 32],

    opcode: u16,
    i: u16,
    pc: u16,
    sp: u16,

    delay_timer: u8,
    sound_timer: u8,
}

pub fn new() -> Chip8 {
    let mut c8 = Chip8 {
        memory: [0; 4096],
        registers: [0; 16],
        stack: [0; 16],
        input: [0; 16],
        gfx: [0; 64 * 32],

        opcode: 0,
        i: 0,
        pc: STARTING_PC_OFFSET,
        sp: 0,

        delay_timer: 0,
        sound_timer: 0,
    };

    for i in 0..fonts::FONTS.len() {
        c8.memory[i] = fonts::FONTS[i] as i32;
    }

    return c8;
}

impl Chip8 {
    pub fn print(&self) {
        println!("Hello World");
    }

    pub fn exec_op(&mut self, code: u16) {
        let codes = (
            (code & 0xF000) >> 12 as u8,
            (code & 0x0F00) >> 8 as u8,
            (code & 0x00F0) >> 4 as u8,
            (code & 0x000F) as u8
        );

        let nnn = code & 0x0FFF;
        let nn = (code & 0x00FF) as u8;

        let x = codes.1 as usize;
        let y = codes.2 as usize;
        let n = codes.3 as usize;

        let mut pc_step: u16 = 2;

        match codes {
            (0x0, 0x0, 0xE, 0x0) => {
                self.gfx.fill(0);
            }
            (0x0, 0x0, 0xE, 0xE) => println!("Return from subrouting"),

            (0x3, _, _, _) => {
                if self.registers[x] == nn {
                    pc_step = self.skip_next();
                }
            }

            _ => println!("UNREACHED CODE {} {} {} {} {}", nnn, nn, x, y, n)
        }

        self.pc += pc_step;
    }

    fn skip_next(&mut self) -> u16 {
        return 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_new_all_variables_and_arrays_are_zeroed_out() {
        let c8 = new();
        for m in c8.registers {
            assert_eq!(m, 0);
        }

        for m in c8.stack {
            assert_eq!(m, 0);
        }

        for m in c8.input {
            assert_eq!(m, 0);
        }

        for m in c8.gfx {
            assert_eq!(m, 0);
        }
        assert_eq!(c8.opcode, 0);
        assert_eq!(c8.i, 0);
        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        assert_eq!(c8.sp, 0);
        assert_eq!(c8.delay_timer, 0);
        assert_eq!(c8.sound_timer, 0);
    }

    #[test]
    fn on_new_should_load_fonts_into_memory() {
        let c8 = new();
        for i in 0..80 {
            assert_eq!(c8.memory[i], fonts::FONTS[i] as i32);
        }
    }

    #[test]
    fn should_clear_the_screen_and_inc_counter() {
        let mut c8 = new();

        for i in 0..c8.gfx.len() {
            c8.gfx[i] = 1;
        }

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x00E0);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        for x in c8.gfx {
            assert_eq!(x, 0);
        }
    }

    #[test]
    // Skips the next instruction if VX equals NN
    fn op_3xnn_should_skip_next_instruction() {
        let mut c8 = new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x3000);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if VX equals NN
    fn op_3xnn_should_not_skip_next() {
        let mut c8 = new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x3001);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }
}
