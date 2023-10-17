use minifb::{Key, Window, WindowOptions, Scale};
use std::fs;

mod chip8;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    let mut chip8 = chip8::Chip8CPU::initialize();

    let current_rom: Vec<u8> = fs::read("./rom/4-flags.ch8").unwrap();

    chip8.load_rom(current_rom);

    chip8.run();

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window =
        Window::new("Rust-8", WIDTH, HEIGHT, {
            WindowOptions { 
                resize: true, scale: Scale::X4, scale_mode: minifb::ScaleMode::Stretch,
                ..WindowOptions::default()
            }
        }).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600  / 4)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.run();

        let display_data = chip8.get_display_data();

        for (index, &value) in display_data.iter().enumerate() {
            buffer[index] = get_pixel_state(value);
        }
        


        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn get_pixel_state(pixel: u16) -> u32 {
    if pixel == 0x0 {
       return from_u8_rgb(0, 0, 0);
    } else {
        return from_u8_rgb(255, 255, 255);
    }
}

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}
