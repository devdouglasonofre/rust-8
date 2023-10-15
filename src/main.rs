use minifb::{Key, Window, WindowOptions};
use std::fs;

const WIDTH: usize = 64 * 4;
const HEIGHT: usize = 32 * 4;

fn main() {
    let current_rom = fs::read("./rom/2-ibm-logo.ch8").unwrap();
    print!("{:#?}", current_rom);
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Rust-8",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}