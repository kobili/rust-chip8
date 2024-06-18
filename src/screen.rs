extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ptr::null;
use std::time::Duration;

struct Screen {
    sdl_context: sdl2::Sdl,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

impl Screen {
    pub fn new(name: &str) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem  = sdl_context.video().unwrap();

        let window = video_subsystem.window(name, 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas()
            .accelerated()
            .build()
            .unwrap();

        let texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext> = canvas.texture_creator();

        Self {
            sdl_context: sdl_context,
            canvas: canvas,
            texture_creator: texture_creator,
        }
    }

    pub fn create_texture(&self) -> sdl2::render::Texture {
        self.texture_creator.create_texture(
            sdl2::pixels::PixelFormatEnum::RGBA8888, sdl2::render::TextureAccess::Streaming, 64, 32
        ).unwrap()
    }

    pub fn update(&mut self, texture: &mut sdl2::render::Texture, display_memory: &[[u32; 64]; 32], pitch: u32) {
        let mut buffer : [u8; 64 * 32] = [0; 64 * 32];

        // flatten the display input
        for i in 0..display_memory.len() {
            let row = display_memory[i];
            let row_offset = i + row.len();
            for j in 0..row.len() {
                buffer[j + row_offset] = row[j] as u8;
            }
        }

        match texture.update(Option::None, &buffer, pitch.try_into().unwrap()) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };

        self.canvas.clear();

        match self.canvas.copy(texture, Option::None, Option::None) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }

        self.canvas.present();
    }
}

pub fn draw_screen() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
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

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
