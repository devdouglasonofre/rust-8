use minifb::{Window, Key};

use crate::{HEIGHT, WIDTH};

#[derive(Debug)]
pub struct Chip8CPU {
    memory: Vec<u16>,
    vram: Vec<u16>,
    registers: Vec<u8>,
    curr_keys: Vec<u8>,
    old_keys: Vec<u8>,
    call_stack: Vec<u16>,
    i: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8CPU {
    pub fn initialize() -> Chip8CPU {
        Chip8CPU {
            memory: vec![0; 4096],
            vram: vec![0; 2048],
            registers: vec![0; 16],
            curr_keys: vec![0; 16],
            old_keys: vec![0; 16],
            call_stack: vec![],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        for (index, value) in rom_data.iter().enumerate() {
            self.memory[index + 0x200] = *value as u16;
        }
        println!("{:?}", self.memory);
    }

    pub fn decrease_timers_value(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn get_sound_timer_value(&mut self) -> u8 {
        return self.sound_timer;
    }

    pub fn register_current_pressed_keys(&mut self, window: &Window) {
        for (index, key) in self.curr_keys.iter_mut().enumerate() {
            let current_key = Chip8CPU::map_hex_value_to_key(index as u8);
            if window.is_key_down(current_key) {
                *key = 1.into();
            }
            if window.is_key_released(current_key) {
                *key = 0.into();
            }
        }
    }

    pub fn clone_current_to_old_keys(&mut self, window: &Window) {
       self.old_keys = self.curr_keys.clone();
    }


    pub fn run(&mut self) {
        let first_byte = self.memory[self.pc as usize];
        let second_byte = self.memory[(self.pc + 1) as usize];

        let current_instruction_binary = ((first_byte as u16) << 8) | second_byte as u16;

        let nnn = current_instruction_binary & 0x0FFF;
        let nn = second_byte;
        let n = current_instruction_binary & 0xF;

        let leading_nibble = (current_instruction_binary >> 12) & 0xF;
        let register_x = (current_instruction_binary >> 8) & 0xF;
        let register_y = (current_instruction_binary >> 4) & 0xF;

        println!("{}", format!("{:04X}", current_instruction_binary)); 

        if current_instruction_binary == 0x0 {
            return;
        }

        self.pc += 0x2;

        let mut should_halt = false;

        match leading_nibble {
            0x0 => match nnn {
                0x0E0 => self.clear_screen(),
                0x0EE => self.remove_from_call_stack_and_get_address(),
                _ => self.set_pointer(nnn),
            },
            0x1 => self.set_pointer(nnn),
            0x2 => self.add_address_to_call_stack(nnn),
            0x3 => self.skip_if_equal_to_register(register_x, nn),
            0x4 => self.skip_if_different_from_register(register_x, nn),
            0x5 => self.skip_if_both_registers_are_equal(register_x, register_y),
            0x6 => self.set_value_to_register(register_x, nn),
            0x7 => self.add_with_value_then_save(register_x, nn),
            0x8 => match n {
                0x0 => self.replace_x_with_y(register_x, register_y),
                0x1 => self.set_register_x_to_binary_OR_from_both_registers(register_x, register_y),
                0x2 => self.set_register_x_to_binary_AND_from_both_registers(register_x, register_y),
                0x3 => self.set_register_x_to_binary_XOR_from_both_registers(register_x, register_y),
                0x4 => self.add_registers_with_overflow_carry(register_x, register_y),
                0x5 => self.subtract_registers_with_overflow_borrow(register_x, register_y),
                0x6 => self.binary_shift_register_value_and_save_least_significant(register_x, register_y),
                0x7 => self.minus_register_and_check_if_has_borrow(register_y, register_x),
                0xe => self.binary_shift_register_value_and_save_most_significant(register_x, register_y),
                _ => {}
            },
            0x9 => self.skip_if_registers_values_are_different(register_x, register_y),
            0xA => self.set_i_to_address(nnn),
            0xB => self.set_pc_to_register_value_plus_address(nnn),
            0xC => self.generate_and_register_random_number(register_x, nn), // RNG
            0xD => self.register_to_vram(register_x, register_y, n),
            0xE => match nn {
                0x9E => self.skip_if_key_vx_is_pressed(register_x),
                0xA1 => self.skip_if_key_vx_is_released(register_x),
                _ => {},
            } // Key Registers
            0xF => match nn {
                0x07 => self.store_current_delay_value_to_x(register_x),
                0x0A => self.wait_until_key_press(register_x, &mut should_halt), // Key Press
                0x15 => self.set_delay_timer_value(register_x), // Delay
                0x18 => self.set_sound_timer_value(register_x), // Sound
                0x1E => self.set_i_to_sum_of_itself_with_register_value(register_x),
                0x29 => {} // Character Data
                0x33 => self.binary_coded_decimal(register_x),
                0x55 => self.copy_register_data_to_i_place(register_x),
                0x65 => self.copy_register_data_from_i_place(register_x),
                _ => {}
            },
            _ => {}
        }

        if should_halt {
            self.pc -= 0x2;
        }
    }

    fn wait_until_key_press(&mut self, register_x: u16, should_halt: &mut bool) {
        let mut key_is_pressed = false;
        let mut pressed_key = 0;
        for (index, value) in self.curr_keys.iter().enumerate() {
            if *value == 1 {
                key_is_pressed = true;
                pressed_key = index;
            }
        }
        if key_is_pressed {
            self.registers[register_x as usize] = pressed_key as u8;
            self.curr_keys[pressed_key] = 0;
        } else {
            *should_halt = true;
        }
    }

    fn skip_if_key_vx_is_released(&mut self, register_x: u16) {
        let current_key_state = self.curr_keys[self.registers[register_x as usize] as usize];
        if  current_key_state == 0 {
            self.pc += 0x2;
        }
    }

    fn skip_if_key_vx_is_pressed(&mut self, register_x: u16) {
        if self.curr_keys[self.registers[register_x as usize] as usize] == 0x1 {
            self.pc += 0x2;
        }
    }

    fn map_hex_value_to_key(key: u8) -> Key {
        match key {
            0x1 => Key::Key1,
            0x2 => Key::Key2,
            0x3 => Key::Key3,
            0xC => Key::Key4,
            0x4 => Key::Q,
            0x5 => Key::W,
            0x6 => Key::E,
            0xD => Key::R,
            0x7 => Key::A,
            0x8 => Key::S,
            0x9 => Key::D,
            0xE => Key::F,
            0xA => Key::Z,
            0x0 => Key::X,
            0xB => Key::C,
            0xF => Key::V, 
            _ => Key::V 
        }
    }

    fn set_sound_timer_value(&mut self, register_x: u16) {
        self.sound_timer = self.registers[register_x as usize];
    }

    fn set_delay_timer_value(&mut self, register_x: u16) {
        self.delay_timer = self.registers[register_x as usize];
    }

    fn store_current_delay_value_to_x(&mut self, register_x: u16) {
        self.registers[register_x as usize] = self.delay_timer;
    }

    fn generate_and_register_random_number(&mut self, register_x: u16, nn: u16) {
        let random_number = rand::random::<u8>();
        self.registers[register_x as usize] = random_number & (nn as u8);
    }

    fn minus_register_and_check_if_has_borrow(&mut self, register_y: u16, register_x: u16) {
        let (result, had_carry) = self.registers[register_y as usize]
            .overflowing_sub(self.registers[register_x as usize]);
        let mut had_borrow_integer = 0;
        if had_carry {
            had_borrow_integer = 1;
        };
        self.registers[register_x as usize] = result;
        self.registers[0xF] = had_borrow_integer;
    }

    fn binary_shift_register_value_and_save_most_significant(&mut self, register_x: u16, register_y: u16) {
        let most_significant = self.registers[register_x as usize] >> 7;
        self.registers[register_x as usize] = self.registers[register_y as usize] << 1;
        self.registers[0xF] = most_significant;
    }

    fn binary_shift_register_value_and_save_least_significant(&mut self, register_x: u16, register_y: u16) {
        let least_significant = self.registers[register_x as usize] & 0x1;
        self.registers[register_x as usize] = self.registers[register_y as usize] >> 1;
        self.registers[0xF] = least_significant;
    }

    fn subtract_registers_with_overflow_borrow(&mut self, register_x: u16, register_y: u16) {
        let (result, had_carry) = self.registers[register_x as usize]
            .overflowing_sub(self.registers[register_y as usize]);
        let mut had_borrow_integer = 0;
        if had_carry {
            had_borrow_integer = 1;
        };
        self.registers[register_x as usize] = result;
        self.registers[0xF] = had_borrow_integer;
    }

    fn add_registers_with_overflow_carry(&mut self, register_x: u16, register_y: u16) {
        let (result, had_carry) = self.registers[register_x as usize]
            .overflowing_add(self.registers[register_y as usize]);
        let mut had_carry_integer = 0;
        if had_carry {
            had_carry_integer = 1;
        };
        self.registers[register_x as usize] = result;
        self.registers[0xF] = had_carry_integer;
    }

    fn set_register_x_to_binary_XOR_from_both_registers(&mut self, register_x: u16, register_y: u16) {
        self.registers[register_x as usize] =
            self.registers[register_x as usize] ^ self.registers[register_y as usize]
    }

    fn set_register_x_to_binary_AND_from_both_registers(&mut self, register_x: u16, register_y: u16) {
        self.registers[register_x as usize] =
            self.registers[register_x as usize] & self.registers[register_y as usize]
    }

    fn set_register_x_to_binary_OR_from_both_registers(&mut self, register_x: u16, register_y: u16) {
        self.registers[register_x as usize] =
            self.registers[register_x as usize] | self.registers[register_y as usize]
    }

    fn skip_if_registers_values_are_different(&mut self, register_x: u16, register_y: u16) {
        if self.registers[register_x as usize] != self.registers[register_y as usize] {
            self.pc = self.pc + 0x2;
        }
    }

    fn set_i_to_address(&mut self, nnn: u16) {
        self.i = nnn
    }

    fn set_pc_to_register_value_plus_address(&mut self, nnn: u16) {
        self.pc = nnn + self.registers[0] as u16
    }

    fn register_to_vram(&mut self, register_x: u16, register_y: u16, n: u16) {
        let x = self.registers[register_x as usize] as u32 & WIDTH as u32 - 1;
        let y = self.registers[register_y as usize] as u32 & HEIGHT as u32 - 1;
        let number_of_bytes = n;
        self.registers[0xF] = 0;
        let mut px_pos;
        for row in 0..number_of_bytes {
            if (row as u32 + y) >= WIDTH as u32 {
                break;
            };

            let data = self.memory[(self.i as u32 + row as u32) as usize];

            for col in 0..8 {
                if (col + x) >= WIDTH as u32 {
                    break;
                }; // column is past screen limit

                if (data >> (7 - col) & 0x1) == 0 {
                    continue;
                }; // data bit is 0 so no draw needed, skip

                px_pos = (x + col) + (y + row as u32) * WIDTH as u32; // fetch location of screen pixel

                if px_pos > 2048 {
                    return;
                }

                if self.vram[px_pos as usize] == 1 {
                    self.registers[0xF] = 1;
                } // if screen pixel is also 1, then collide

                self.vram[px_pos as usize] ^= 1 // don't forget to flip screen bit
            }
        }
    }

    fn set_i_to_sum_of_itself_with_register_value(&mut self, register_x: u16) {
        self.i = self.i + self.registers[register_x as usize] as u16
    }

    fn binary_coded_decimal(&mut self, register_x: u16) {
        let mut decimal_number = self.registers[register_x as usize] as u16;
        for index in (0..(2 + 1)).rev() {
            self.memory[(self.i + index) as usize] = decimal_number % 10;
            decimal_number /= 10;
        }
    }

    fn copy_register_data_to_i_place(&mut self, register_x: u16) {
        for index in 0..(register_x + 1) {
            self.memory[(self.i + index) as usize] =
                self.registers[index as usize] as u16;
        }
    }

    fn copy_register_data_from_i_place(&mut self, register_x: u16) {
        for index in 0..(register_x + 1) {
            self.registers[index as usize] =
                self.memory[(self.i + index) as usize] as u8;
        }
    }

    fn replace_x_with_y(&mut self, register_x: u16, register_y: u16) {
        self.registers[register_x as usize] = self.registers[register_y as usize]
    }

    fn add_with_value_then_save(&mut self, register_x: u16, nn: u16) {
        self.registers[register_x as usize] = self.registers[register_x as usize].overflowing_add(nn as u8).0;
    }

    fn set_value_to_register(&mut self, register_x: u16, nn: u16) {
        self.registers[register_x as usize] = nn as u8
    }

    fn skip_if_both_registers_are_equal(&mut self, register_x: u16, register_y: u16) {
        if self.registers[register_x as usize] == self.registers[register_y as usize] {
            self.pc = self.pc + 0x2;
        }
    }

    fn skip_if_different_from_register(&mut self, register_x: u16, nn: u16) {
        if self.registers[register_x as usize] != nn as u8 {
            self.pc = self.pc + 0x2;
        }
    }

    fn skip_if_equal_to_register(&mut self, register_x: u16, nn: u16) {
        if self.registers[register_x as usize] == nn as u8 {
            self.pc = self.pc + 0x2;
        }
    }

    fn add_address_to_call_stack(&mut self, nnn: u16) {
        self.call_stack.push(self.pc);
        self.pc = nnn;
    }

    fn set_pointer(&mut self, nnn: u16) {
        if nnn > 0x200 {
            self.pc = nnn;
        }
    }

    fn remove_from_call_stack_and_get_address(&mut self) {
        self.pc = self.call_stack.pop().unwrap()
    }

    pub fn get_display_data(&self) -> &[u16] {
        return &self.vram;
    }

    fn clear_screen(&mut self) {
        self.vram = vec![0; 2048];
    }
}