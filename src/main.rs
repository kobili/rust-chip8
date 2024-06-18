extern crate sdl2;
mod emulator;
mod screen;

use emulator::chip8::Chip8;

use sdl2::keyboard::Keycode;
use sdl2::render::{
    TextureCreator, 
    TextureAccess,
};
use sdl2::video::WindowContext;
use sdl2::pixels::PixelFormatEnum;
use std::collections::HashMap;


fn main() {
    // setup sdl2 resources
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem  = sdl_context.video().unwrap();

    let screen_width = 64;
    let screen_height = 32;
    let video_scale = 12_u32;

    let window = video_subsystem.window("Chip8", screen_width * video_scale, screen_height * video_scale)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        .accelerated()
        .build()
        .unwrap();

    let texture_creator: TextureCreator<WindowContext> = canvas.texture_creator();

    let mut texture = texture_creator.create_texture(
        PixelFormatEnum::RGBA8888, TextureAccess::Streaming, screen_width, screen_height
    ).unwrap();

    let mut event_pump = match sdl_context.event_pump() {
        Ok(value) => value,
        Err(e) => panic!("{}", e),
    };

    let key_mapping = HashMap::from([
        (Keycode::NUM_1, 1),
        (Keycode::NUM_2, 2),
        (Keycode::NUM_3, 3),
        (Keycode::NUM_4, 0xC),
    
        (Keycode::Q, 4),
        (Keycode::W, 5),
        (Keycode::E, 6),
        (Keycode::R, 0xD),
    
        (Keycode::A, 7),
        (Keycode::S, 8),
        (Keycode::D, 9),
        (Keycode::F, 0xE),
    
        (Keycode::Z, 0xA),
        (Keycode::X, 0),
        (Keycode::C, 0xB),
        (Keycode::V, 0xF),
    ]);

    let mut c8 = Chip8::new("./examples/test_opcode.ch8");

    let cycle_delay = 4;
    let video_pitch = (std::mem::size_of_val(&c8.get_display_memory()[0][0]) * 64) as u32;
    
    let mut last_cycle_time = std::time::SystemTime::now();

    let mut quit = false;

    while !quit {
        quit = screen::process_input(&mut event_pump, c8.get_keypad(), &key_mapping);

        let current_time = std::time::SystemTime::now();
        let dt = match current_time.duration_since(last_cycle_time) {
            Ok(v) => v.as_millis(),
            Err(e) => panic!("{}", e),
        };

        if dt > cycle_delay {
            last_cycle_time = current_time;

            c8.cycle();

            screen::update(&mut canvas, &mut texture, &c8.get_display_memory(), video_pitch as u32);
        }
    }
}
