use crate::fonts;
use rand::prelude::*;
use std::fs;

const STARTING_PC_OFFSET: u16 = 0x200;
pub const GFX_WIDTH: usize = 64;
pub const GFX_HEIGHT: usize = 32;

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    stack: [u16; 16],
    pub input: [u8; 16],
    pub gfx: [u8; GFX_WIDTH * GFX_HEIGHT],

    i: u16,
    pc: u16,
    sp: u16,

    delay_timer: u8,
    sound_timer: u8,

    draw_flag: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c8 = Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            input: [0; 16],
            gfx: [0; GFX_WIDTH * GFX_HEIGHT],

            i: 0,
            pc: STARTING_PC_OFFSET,
            sp: 0,

            delay_timer: 0,
            sound_timer: 0,

            draw_flag: false,
        };

        for i in 0..fonts::FONTS.len() {
            c8.memory[i] = fonts::FONTS[i];
        }

        return c8;
    }

    pub fn is_draw_ready(&self) -> bool {
        self.draw_flag
    }

    pub fn tick(&mut self) {
        self.draw_flag = false;

        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }

        let op_code = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as u16);
        self.exec_op(op_code);
    }

    pub fn debug_gfx_to_stdout(&self) {
        for col in 0..GFX_HEIGHT {
            for row in 0..GFX_WIDTH {
                if self.gfx[col * GFX_WIDTH + row] == 0 {
                    print!("-")
                } else {
                    print!("X")
                }
            }

            print!("\n");
        }
    }

    pub fn load(&mut self, file_path: &str) {
        let data = fs::read(file_path).unwrap();
        for (i, it) in data.iter().enumerate() {
            self.memory[self.pc as usize + i] = *it;
        }
    }

    fn exec_op(&mut self, opcode: u16) {
        let codes = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;

        let x = codes.1 as usize;
        let y = codes.2 as usize;
        let n = codes.3 as usize;

        let mut pc_step: u16 = 2;

        match codes {
            (0x0, 0x0, 0xE, 0x0) => self.gfx.fill(0),
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp = self.sp.wrapping_sub(1);
                self.pc = self.stack[self.sp as usize];
                return;
            }
            (0x1, _, _, _) => {
                self.pc = nnn;
                return;
            }
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp = self.sp.wrapping_add(1);
                self.pc = nnn;
                return;
            }
            (0x3, _, _, _) => {
                if self.v[x] == nn {
                    pc_step = self.skip_next();
                }
            }
            (0x4, _, _, _) => {
                if self.v[x] != nn {
                    pc_step = self.skip_next();
                }
            }
            (0x5, _, _, 0x0) => {
                if self.v[x] == self.v[y] {
                    pc_step = self.skip_next();
                }
            }
            (0x6, _, _, _) => self.v[x] = nn,
            (0x7, _, _, _) => self.v[x] = self.v[x].wrapping_add(nn),
            (0x8, _, _, 0x0) => self.v[x] = self.v[y],
            (0x8, _, _, 0x1) => self.v[x] |= self.v[y],
            (0x8, _, _, 0x2) => self.v[x] &= self.v[y],
            (0x8, _, _, 0x3) => self.v[x] ^= self.v[y],
            (0x8, _, _, 0x4) => {
                if self.v[y] > (0xFF - self.v[x]) {
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.v[x] = self.v[x].wrapping_add(self.v[y]);
            }
            (0x8, _, _, 0x5) => {
                if self.v[y] > self.v[x] {
                    self.v[0xF] = 0;
                } else {
                    self.v[0xF] = 1;
                }
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            }
            (0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            (0x8, _, _, 0x7) => {
                if self.v[x] > self.v[y] {
                    self.v[0xF] = 0;
                } else {
                    self.v[0xF] = 1;
                }
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            }
            (0x8, _, _, 0xE) => {
                self.v[0xF] = (self.v[x] >> 7) & 1;
                self.v[x] <<= 1;
            }

            (0x9, _, _, 0x0) => {
                if self.v[x] != self.v[y] {
                    pc_step = self.skip_next();
                }
            }
            (0xA, _, _, _) => self.i = nnn,
            (0xB, _, _, _) => {
                self.pc = (self.v[0] as u16 + nnn) as u16;
                return; // Jump to address by not letting pc_step increment self.pc
            }
            (0xC, _, _, _) => self.v[x] = rand::random::<u8>() & nn,
            (0xD, _, _, _) => {
                self.draw_flag = true;
                self.v[0xF] = 0;

                for y_line in 0..n {
                    let px = self.memory[self.i as usize + y_line];

                    for x_line in 0u8..8 {
                        if (px & (0x80 >> x_line)) != 0 {
                            // if drawing causes any pixel to be erased set the
                            // collision flag to 1
                            if self.gfx[(self.v[x] as usize + x_line as usize + ((self.v[y] as usize + y_line) * 64))] == 1 {
                                self.v[0xF] = 1;
                            }

                            // set pixel value by using XOR
                            self.gfx[self.v[x] as usize + x_line as usize + ((self.v[y] as usize + y_line) * 64)] ^= 1;
                        }
                    }
                }
            }
            (0xE, _, 0x9, 0xE) => {
                if self.input[self.v[x] as usize] == 1 {
                    pc_step = self.skip_next();
                }
            }
            (0xE, _, 0xA, 0x1) => {
                if self.input[self.v[x] as usize] != 1 {
                    pc_step = self.skip_next();
                }
            }
            (0xF, _, 0x0, 0x7) => self.v[x] = self.delay_timer,
            (0xF, _, 0x0, 0xA) => {
                let mut input_pressed = false;
                for i in 0..self.input.len() {
                    if self.input[i] == 1 {
                        self.v[x] = i as u8;
                        input_pressed = true;
                        break;
                    }
                }
                if !input_pressed {
                    return;
                }
            }

            (0xF, _, 0x1, 0x5) => self.delay_timer = self.v[x],
            (0xF, _, 0x1, 0x8) => self.sound_timer = self.v[x],
            (0xF, _, 0x1, 0xE) => self.i += self.v[x] as u16,
            (0xF, _, 0x2, 0x9) => self.i = (self.v[x] * fonts::BYTES_PER_LINE) as u16,
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i as usize] = self.v[x] / 100;
                self.memory[self.i as usize + 1] = (self.v[x] % 100) / 10;
                self.memory[self.i as usize + 2] = self.v[x] % 10;
            }
            (0xF, _, 0x5, 0x5) => {
                for register_index in 0..x + 1 {
                    self.memory[self.i as usize + register_index] = self.v[register_index];
                }
            }
            (0xF, _, 0x6, 0x5) => {
                for register_index in 0..x + 1 {
                    self.v[register_index] = self.memory[self.i as usize + register_index];
                }
            }
            _ => println!(
                "UNREACHED CODE {:#02X} {} {} {} {} {}",
                opcode, nnn, nn, x, y, n
            ),
        }
        self.pc += pc_step;
    }

    fn skip_next(&mut self) -> u16 {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_new_all_variables_and_arrays_are_zeroed_out() {
        let c8 = Chip8::new();

        for m in c8.v {
            assert_eq!(m, 0);
        }

        for m in c8.stack {
            assert_eq!(m, 0);
        }

        for m in c8.input {
            assert_eq!(m, 0);
        }

        for bit in c8.gfx {
            assert_eq!(bit, 0);
        }

        assert_eq!(c8.i, 0);
        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        assert_eq!(c8.sp, 0);
        assert_eq!(c8.delay_timer, 0);
        assert_eq!(c8.sound_timer, 0);
    }

    #[test]
    fn on_new_should_load_fonts_into_memory() {
        let c8 = Chip8::new();
        for i in 0..80 {
            assert_eq!(c8.memory[i], fonts::FONTS[i]);
        }
    }

    #[test]
    fn op_00e0_should_clear_the_screen_and_inc_counter() {
        let mut c8 = Chip8::new();

        c8.gfx.fill(1);

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x00E0);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        for bit in c8.gfx {
            assert_eq!(bit, 0);
        }
    }

    #[test]
    fn op_00ee() {
        let mut c8 = Chip8::new();

        let pc_step = 2;
        c8.sp = 2;
        c8.stack[c8.sp as usize - 1] = 255;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x00EE);
        assert_eq!(c8.pc, 255);

        assert_eq!(c8.sp, 1);
    }

    #[test]
    // Skips the next instruction if VX equals NN
    fn op_3xnn_should_skip_next_instruction() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x3000);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if VX equals NN
    fn op_3xnn_should_not_skip_next() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x3001);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }

    #[test]
    // Skips the next instruction if VX does not equal NN
    fn op_4xnn_should_skip_next() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x4001);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if VX does not equal NN
    fn op_4xnn_should_not_skip_next() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x4000);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }

    #[test]
    // Skips the next instruction if VX equals VY
    fn op_5xnn_should_skip_next() {
        let mut c8 = Chip8::new();

        let (x_val, y_val) = (0x01, 0x01);

        c8.v[0] = x_val;
        c8.v[1] = y_val;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x5010);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if VX equals VY
    fn op_5xnn_should_not_skip_next() {
        let mut c8 = Chip8::new();

        let (x_val, y_val) = (0x01, 0x02);

        c8.v[0] = x_val;
        c8.v[2] = y_val;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x5020);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }

    #[test]
    // Sets VX to NN.
    fn op_6xnn() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.v[3], 0x00);

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x6312);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x0012);
    }

    #[test]
    // Adds NN to VX (carry flag is not changed).
    fn op_7xnn() {
        let mut c8 = Chip8::new();

        c8.v[3] = 0x02;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x7312);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x02 + 0x0012);
    }

    #[test]
    // Sets VX to the value of VY.
    fn op_8xy0() {
        let mut c8 = Chip8::new();

        c8.v[3] = 0x02;
        c8.v[4] = 0x03;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8340);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x03);
    }

    #[test]
    // Sets VX to VX or VY. (bitwise OR operation)
    fn op_8xy1() {
        let mut c8 = Chip8::new();

        c8.v[3] = 0b101;
        c8.v[4] = 0b110;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8341);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0b101 | 0b110);
    }

    #[test]
    // Sets VX to VX and VY. (bitwise AND operation)
    fn op_8xy2() {
        let mut c8 = Chip8::new();

        c8.v[3] = 0b101;
        c8.v[4] = 0b011;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8342);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0b101 & 0b011);
    }

    #[test]
    // Sets VX to VX xor VY.
    fn op_8xy3() {
        let mut c8 = Chip8::new();

        c8.v[3] = 0b101;
        c8.v[4] = 0b011;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8343);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0b101 ^ 0b011);
    }

    #[test]
    // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
    fn op_8xy4_should_carry() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0xFF;
        c8.v[4] = 0x01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8344);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0xFFu8.wrapping_add(0x01));
        assert_eq!(c8.v[0xF], 1);
    }

    #[test]
    // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there is not.
    fn op_8xy4_should_not_carry() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0xFE;
        c8.v[4] = 0x01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8344);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0xFE + 0x01);
        assert_eq!(c8.v[0xF], 0);
    }

    #[test]
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy5_should_borrow() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0x01;
        c8.v[4] = 0x02;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8345);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x01u8.wrapping_sub(0x02));
        assert_eq!(c8.v[0xF], 0);
    }

    #[test]
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy5_should_not_borrow() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0x02;
        c8.v[4] = 0x01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8345);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x02 - 0x01);
        assert_eq!(c8.v[0xF], 1);
    }

    #[test]
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy5_should_not_borrow_if_x_and_y_are_the_same() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0x01;
        c8.v[4] = 0x01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8345);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x01 - 0x01);
        assert_eq!(c8.v[0xF], 1);
    }

    #[test]
    // Stores the least significant bit of VX in VF and then shifts VX to the right by 1.[b]
    fn op_8xy6() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0b01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8306);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0b00);
        assert_eq!(c8.v[0xF], 1);
    }

    #[test]
    // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy7_should_not_borrow() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0b10;
        c8.v[4] = 0b11;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8347);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0b01);
        assert_eq!(c8.v[0xF], 1);
    }

    #[test]
    // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
    fn op_8xy7_should_borrow() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0x01;
        c8.v[4] = 0x00;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x8347);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0x00u8.wrapping_sub(0x01));
        assert_eq!(c8.v[0xF], 0);
    }

    #[test]
    // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    fn op_8xye() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 0b01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x830E);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[3], 0b10);
        assert_eq!(c8.v[0xF], 0);
    }

    #[test]
    // Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block);
    fn op_9xy40_should_skip() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 1;
        c8.v[4] = 2;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x9340);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if VX does not equal VY. (Usually the next instruction is a jump to skip a code block);
    fn op_9xy40_should_not_skip() {
        let mut c8 = Chip8::new();
        c8.v[0xF] = u8::MAX;

        c8.v[3] = 2;
        c8.v[4] = 2;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0x9340);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }

    #[test]
    // Sets I to the address NNN.
    fn op_annn() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xAEF1);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.i, 0x0EF1);
    }

    #[test]
    // Jumps to the address NNN plus V0.
    fn op_bnnn() {
        let mut c8 = Chip8::new();

        c8.v[0x0] = 0x01;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xB123);

        assert_eq!(c8.pc, 0x01 + 0x0123);
    }

    // #[test]
    // // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
    // fn op_cxnn() {
    //     let mut c8 = Chip8::new();
    //
    //     assert_eq!(c8.pc, STARTING_PC_OFFSET);
    //     c8.exec_op(0xC133);
    //
    //     let mut rng = rng::test::rng(538);
    //
    //     assert_eq!(c8.v[1], 123 & 0x0033);
    // }

    #[test]
    /*
       Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
       Each row of 8 pixels is read as bit-coded starting from memory location I; I value does not change after the execution of this instruction.
       As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that does not happen.
    */
    fn op_dxyn() {
        let mut c8 = Chip8::new();

        let x = 2;
        let y = 3;

        c8.v[x] = 0;
        c8.v[y] = 1;

        for i in 0..8usize {
            c8.memory[c8.i as usize + i] = 1;
        }

        assert_eq!(c8.is_draw_ready(), false);
        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xD233);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
        assert_eq!(c8.is_draw_ready(), true);

        c8.debug_gfx_to_stdout();
    }

    #[test]
    // Skips the next instruction if the key stored in VX is pressed (usually the next instruction is a jump to skip a code block).
    fn op_ex9e_should_skip() {
        let mut c8 = Chip8::new();

        c8.v[0] = 0x0;
        c8.v[1] = 0x01;
        c8.input[c8.v[0] as usize] = 1;
        c8.input[c8.v[1] as usize] = 0;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xE09E);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if the key stored in VX is pressed (usually the next instruction is a jump to skip a code block).
    fn op_ex9e_should_not_skip() {
        let mut c8 = Chip8::new();

        c8.v[0] = 0x0;
        c8.v[1] = 0x01;

        c8.input[c8.v[0] as usize] = 0;
        c8.input[c8.v[1] as usize] = 1;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xE09E);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }

    #[test]
    // Skips the next instruction if the key stored in VX is not pressed (usually the next instruction is a jump to skip a code block).
    fn op_exa1_should_skip() {
        let mut c8 = Chip8::new();

        c8.v[0] = 0x0;
        c8.v[1] = 0x01;
        c8.input[c8.v[0] as usize] = 0;
        c8.input[c8.v[1] as usize] = 1;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xE0A1);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 4);
    }

    #[test]
    // Skips the next instruction if the key stored in VX is not pressed (usually the next instruction is a jump to skip a code block).
    fn op_exa1_should_not_skip() {
        let mut c8 = Chip8::new();

        c8.v[0] = 0x0;
        c8.v[1] = 0x01;

        c8.input[c8.v[0] as usize] = 1;
        c8.input[c8.v[1] as usize] = 0;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xE0A1);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);
    }

    #[test]
    // Sets VX to the value of the delay timer.
    fn op_fx07() {
        let mut c8 = Chip8::new();

        c8.delay_timer = 5;
        c8.v[1] = 0;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF107);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[1], c8.delay_timer);
    }

    #[test]
    // A key press is awaited, and then stored in VX (blocking operation, all instruction halted until next key event).
    fn op_fx0a() {
        let mut c8 = Chip8::new();

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF10A);
        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        assert_eq!(c8.v[1], 0);

        c8.input[10] = 1;
        c8.exec_op(0xF10A);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.v[1], 10);
    }

    #[test]
    // Sets the delay timer to VX.
    fn op_fx15() {
        let mut c8 = Chip8::new();

        c8.v[2] = 4;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF215);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.delay_timer, c8.v[2]);
    }

    #[test]
    // Sets the sound timer to VX.
    fn op_fx18() {
        let mut c8 = Chip8::new();

        c8.v[2] = 4;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF218);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.sound_timer, c8.v[2]);
    }

    #[test]
    // Adds VX to I. VF is not affected.
    fn op_fx1e() {
        let mut c8 = Chip8::new();

        c8.v[2] = 4;
        c8.v[0xF] = u8::MAX;
        c8.i = 3;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF21E);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.i, 3 + 4);
        assert_eq!(c8.v[0xF], u8::MAX);
    }

    #[test]
    // Sets I to the location of the sprite for the character in VX.
    // Characters 0-F (in hexadecimal) are represented by a 4x5 font.
    fn op_fx29() {
        let mut c8 = Chip8::new();

        let font_to_find = 0x02;
        c8.v[1] = font_to_find;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF129);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.i, (font_to_find * fonts::BYTES_PER_LINE) as u16);
    }

    #[test]
    // Stores the binary-coded decimal representation of VX,
    // with the hundreds digit in memory at location in I,
    // the tens digit at location I+1, and the ones digit at location I+2.
    fn op_fx33() {
        let mut c8 = Chip8::new();

        c8.v[1] = 201;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF133);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        assert_eq!(c8.memory[c8.i as usize], 2);
        assert_eq!(c8.memory[c8.i as usize + 1], 0);
        assert_eq!(c8.memory[c8.i as usize + 2], 1);
    }

    #[test]
    // Stores from V0 to VX (including VX) in memory, starting at address I.
    // The offset from I is increased by 1 for each value written,
    // but I itself is left unmodified.
    fn op_fx55() {
        let mut c8 = Chip8::new();

        c8.v[0] = 2;
        c8.v[1] = 3;
        c8.v[2] = 5;
        c8.v[3] = 9;
        c8.v[4] = 11;

        c8.i = 50;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF455);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        for i in 0..5 {
            assert_eq!(c8.memory[c8.i as usize + i as usize], c8.v[i]);
        }
    }

    #[test]
    // Fills from V0 to VX (including VX) with values from memory,
    // starting at address I.
    // The offset from I is increased by 1 for each value read,
    // but I itself is left unmodified.
    fn op_fx65() {
        let mut c8 = Chip8::new();

        c8.i = 50;

        c8.memory[c8.i as usize + 0] = 2;
        c8.memory[c8.i as usize + 1] = 3;
        c8.memory[c8.i as usize + 2] = 5;
        c8.memory[c8.i as usize + 3] = 9;
        c8.memory[c8.i as usize + 4] = 11;

        assert_eq!(c8.pc, STARTING_PC_OFFSET);
        c8.exec_op(0xF465);
        assert_eq!(c8.pc, STARTING_PC_OFFSET + 2);

        for i in 0..4 + 1 {
            assert_eq!(c8.v[i], c8.memory[c8.i as usize + i as usize]);
        }
    }
}
