extern crate sdl2;

use chip8;
use chip8::Chip8;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

/*
std::pair<SDL_Window *, SDL_Renderer *> initSDL() {
    if (SDL_Init(SDL_INIT_VIDEO) < 0) {
        std::cout << "Failed to initialize the SDL2 library\n";
        exit(-1);
    }

    SDL_Window *window = SDL_CreateWindow("The Game Of Life",
                                          SDL_WINDOWPOS_CENTERED,
                                          SDL_WINDOWPOS_CENTERED,
                                          640,
                                          480,
                                          SDL_WINDOW_SHOWN);

    if (!window) {
        std::cout << "Failed to create window\n";
        exit(-1);
    }

    auto *renderer = SDL_CreateRenderer(window, -1,
                                        SDL_RENDERER_ACCELERATED | SDL_RENDERER_TARGETTEXTURE);

    return std::make_pair(window, renderer);
}
 */

fn main() {
    let mut c8 = Chip8::new();
    c8.load("test_opcode.ch8");

    println!("Hello, world!");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Chip8 Rust", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
