use minifb::{Key, Window, WindowOptions, Scale};
use std::fs;
use rodio::{OutputStream, Sink};
use rodio::source::{SineWave, Source};

mod chip8;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
// Instructions per frame.
const DEFAULT_CLOCK_SPEED: u8 = 11;

fn main() {
    let mut chip8 = chip8::Chip8CPU::initialize();

    let current_rom: Vec<u8> = fs::read("./rom/6-keypad.ch8").unwrap();

    chip8.load_rom(current_rom);

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

        
    let refresh_rate = std::time::Duration::from_secs_f32(1.0/60.0);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(refresh_rate));
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        
        chip8.store_current_pressed_keys(&window);
        let mut bus_counter = DEFAULT_CLOCK_SPEED;
        while bus_counter > 0 {
            chip8.run();
            bus_counter -= 1;
        }
        
        chip8.store_current_released_keys(&window);

        chip8.decrease_timers_value();
        if chip8.get_sound_timer_value() > 0 {
            play_beep(&sink);
        } else {
            sink.stop()
        }
        
        let display_data = chip8.get_display_data();

        for (index, &value) in display_data.iter().enumerate() {
            buffer[index] = get_pixel_state(value);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

fn play_beep(sink: &Sink) {
    let source = SineWave::new(400.0).amplify(0.20);
    sink.append(source);
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
