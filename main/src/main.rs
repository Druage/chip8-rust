mod sys_timer;

use chip8;
use chip8::{Chip8, GFX_HEIGHT, GFX_WIDTH};

use crate::sys_timer::SysTimer;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

// TODO
// 1. Add test for draw flag in Chip8
// 2. Add test for read_op_code() func

fn main() {
    let mut c8 = Chip8::new();
    c8.load("./roms/TEST_ROM_WITH_AUDIO");

    let mut buffer: Vec<u32> = vec![0; GFX_WIDTH * GFX_HEIGHT];

    let mut options = WindowOptions::default();
    options.resize = true;
    let mut window =
        Window::new("Chip8.rs - ESC to exit", WIDTH, HEIGHT, options).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let timer = SysTimer::new(16600);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        timer.pause_until_target_reached();
        c8.tick();

        update_input_states(&mut c8, &mut window);

        if c8.is_draw_ready() {
            copy_gfx_to_pixel_buffer(&mut c8, &mut buffer);
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, GFX_WIDTH, GFX_HEIGHT)
            .unwrap();
    }
}

fn copy_gfx_to_pixel_buffer(c8: &mut Chip8, buffer: &mut Vec<u32>) {
    for col in 0..chip8::GFX_HEIGHT {
        for row in 0..chip8::GFX_WIDTH {
            let index = col * chip8::GFX_WIDTH + row;
            if c8.gfx[index] == 0 {
                buffer[index] = 0x000000;
            } else {
                buffer[index] = 0xFFFFFF;
            }
        }
    }
}

fn update_input_states(c8: &mut Chip8, window: &mut Window) {
    for i in 0..c8.input.len() {
        let key = match i {
            1 => Key::Key1,
            2 => Key::Key2,
            3 => Key::Key3,
            0xC => Key::Key4,

            4 => Key::Q,
            5 => Key::W,
            6 => Key::E,
            0xD => Key::R,

            7 => Key::A,
            8 => Key::S,
            9 => Key::D,
            0xE => Key::F,

            0xA => Key::Z,
            0 => Key::X,
            0xB => Key::C,
            0xF => Key::V,

            _ => Key::Unknown,
        };

        // print!("{}", c8.input[i]);
        c8.input[i] = if window.is_key_down(key) { 1 } else { 0 };
    }

    // print!("\n");
}
