extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{
    Canvas,
    TextureCreator, 
    Texture,
    TextureAccess,
};
use sdl2::{
    Sdl,
    EventPump,
};
use sdl2::video::{
    Window,
    WindowContext,
};
use sdl2::pixels::PixelFormatEnum;

pub struct Screen {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    event_pump: EventPump,

    key_mapping: std::collections::HashMap<Keycode, u8>,
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

        let texture_creator: TextureCreator<WindowContext> = canvas.texture_creator();

        let event_pump = match sdl_context.event_pump() {
            Ok(value) => value,
            Err(e) => panic!("{}", e),
        };

        let keymapping = std::collections::HashMap::from([
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

        Self {
            sdl_context: sdl_context,
            canvas: canvas,
            texture_creator: texture_creator,
            event_pump: event_pump,
            key_mapping: keymapping,
        }
    }

    pub fn create_texture(&self) -> Texture {
        self.texture_creator.create_texture(
            PixelFormatEnum::RGBA8888, TextureAccess::Streaming, 64, 32
        ).unwrap()
    }

    pub fn update(&mut self, texture: &mut Texture, display_memory: &[[u32; 64]; 32], pitch: u32) {
        let mut buffer : [u8; 64 * 32] = [0; 64 * 32];

        // flatten the display input
        // TODO: Might cause a slowdown; try to flatten the display memory at the emulator level
        for i in 0..display_memory.len() {
            let row = display_memory[i];
            let row_offset = i * row.len();
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

    /// Polls an SDL EventPump for key events. Updates the input keymap accordingly.
    /// 
    /// If a quit event occurs, returns true. Otherwise false.
    pub fn process_input(&mut self, keys: &mut [u8; 16]) -> bool {
        let mut quit = false;

        'event_poll: for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit{..} => {
                    quit = true;
                    break 'event_poll;
                },
                Event::KeyDown { keycode, ..} => {
                    match keycode {
                        Some(Keycode::NUM_1) => keys[1] = 1,
                        Some(Keycode::NUM_2) => keys[2] = 1,
                        Some(Keycode::NUM_3) => keys[3] = 1,
                        Some(Keycode::NUM_4) => keys[0xC] = 1,
                        Some(Keycode::Q) => keys[4] = 1,
                        Some(Keycode::W) => keys[5] = 1,
                        Some(Keycode::E) => keys[6] = 1,
                        Some(Keycode::R) => keys[0xD] = 1,

                        Some(Keycode::A) => keys[7] = 1,
                        Some(Keycode::S) => keys[8] = 1,
                        Some(Keycode::D) => keys[9] = 1,
                        Some(Keycode::F) => keys[0xE] = 1,

                        Some(Keycode::Z) => keys[0xA] = 1,
                        Some(Keycode::X) => keys[0] = 1,
                        Some(Keycode::C) => keys[0xB] = 1,
                        Some(Keycode::V) => keys[0xF] = 1,
                        Some(_) | None => {},
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    match keycode {
                        Some(keycode) => {
                            if self.key_mapping.contains_key(&keycode) {
                                let key_value = *self.key_mapping.get(&keycode).unwrap();
                                keys[usize::from(key_value)] = 0;
                            }
                        },
                        None => {},
                    }
                }
                _ => {},
            }
        }
        quit
    }
}
