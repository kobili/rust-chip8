extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{
    Canvas,
    Texture,
};
use sdl2::EventPump;
use sdl2::video::Window;
use std::collections::HashMap;


pub fn update(canvas: &mut Canvas<Window>, texture: &mut Texture, display_memory: &[[u32; 64]; 32], pitch: u32) {
    let mut buffer : [u8; 64 * 32 * 4] = [0; 64 * 32 * 4];

    // flatten the display input
    // TODO: Might cause a slowdown if we do this with every cpu cycle; try to flatten the display memory at the emulator level
    for i in 0..display_memory.len() {
        let row = display_memory[i];
        let row_offset = i * row.len();
        for j in 0..row.len() {
            let start_index = j + row_offset;
            buffer[4*start_index..][..4].copy_from_slice(&row[j].to_le_bytes());
        }
    }

    match texture.update(Option::None, &buffer, pitch.try_into().unwrap()) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    };

    canvas.clear();

    match canvas.copy(texture, Option::None, Option::None) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }

    canvas.present();
}


pub fn process_input(event_pump: &mut EventPump, keys: &mut [u8; 16], key_mapping: &HashMap<Keycode, u8>) -> bool {
    let mut quit = false;

    'event_poll: for event in event_pump.poll_iter() {
        match event {
            Event::Quit{..} => {
                quit = true;
                break 'event_poll;
            },
            Event::KeyDown { keycode, ..} => {
                match keycode {
                    Some(Keycode::ESCAPE) => {
                        quit = true;
                        break 'event_poll;
                    },
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
                        if key_mapping.contains_key(&keycode) {
                            let key_value = *key_mapping.get(&keycode).unwrap();
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
