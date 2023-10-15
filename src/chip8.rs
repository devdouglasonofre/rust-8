use crate::{DISPLAY_DATA_ADDRESS, WIDTH};

#[derive(Debug)]
pub struct Chip8CPU {
    memory: Vec<u16>,
    registers: Vec<u8>,
    i: u16,
    pc: u16,
}
impl Chip8CPU {
    pub fn initialize() -> Chip8CPU {
        Chip8CPU {
            memory: vec![0; 4096],
            registers: vec![0; 16],
            i: 0,
            pc: 0x200,
        }
    }

    pub fn load_rom(&mut self, rom_data: Vec<u8>) {
        for (index, value) in rom_data.iter().enumerate() {
            self.memory[index + 0x200] = *value as u16;
        }
        println!("{:?}", self.memory);
    }

    pub fn run(&mut self) {
        let first_byte = self.memory[self.pc as usize];
        let second_byte = self.memory[(self.pc + 1) as usize];

        let current_instruction_binary = ((first_byte as u16) << 8) | second_byte as u16;

        let memory_address = current_instruction_binary & 0x0FFF;
        let hexadecimal_byte = (current_instruction_binary >> 8) & 0xFF;
        let hexadecimal_nibble = current_instruction_binary & 0xF;

        let leading_nimble = (current_instruction_binary >> 12) & 0xF;
        let register_x = (current_instruction_binary >> 8) & 0xF;
        let register_y = (current_instruction_binary >> 4) & 0xF;

        println!("{}", format!("{:04X}", current_instruction_binary));

        self.pc += 0x2;

        match leading_nimble {
            0x0 => match hexadecimal_nibble {
                0x0 => Self::clear_screen(self),
                0xE => Self::remove_from_routine_stack(self),
                _ => {
                    if memory_address > 0x200 {
                        self.pc = memory_address
                    }
                }
            },
            0x1 => self.pc = memory_address,
            0x2 => Self::add_to_routine_stack(self, memory_address),
            0x3 => {
                if self.registers[register_x as usize] == hexadecimal_byte as u8 {
                    self.pc = self.pc + 0x2;
                }
            }
            0x4 => {
                if self.registers[register_x as usize] != hexadecimal_byte as u8 {
                    self.pc = self.pc + 0x2;
                }
            }
            0x5 => {
                if self.registers[register_x as usize] == self.registers[register_y as usize] {
                    self.pc = self.pc + 0x2;
                }
            }
            0x6 => self.registers[register_x as usize] = hexadecimal_byte as u8,
            0x7 => {
                let mut value_to_set: u16 =
                    (self.registers[register_x as usize] as u16) + hexadecimal_byte;
                if value_to_set > 255 {
                    value_to_set = value_to_set - 255;
                }
                println!("{} {}", self.registers[register_x as usize], value_to_set);
                self.registers[register_x as usize] = value_to_set as u8
            }
            0x8 => match hexadecimal_nibble {
                0x0 => self.registers[register_x as usize] = self.registers[register_y as usize],
                0x1 => {
                    self.registers[register_x as usize] =
                        self.registers[register_x as usize] | self.registers[register_y as usize]
                }
                0x2 => {
                    self.registers[register_x as usize] =
                        self.registers[register_x as usize] & self.registers[register_y as usize]
                }
                0x3 => {
                    self.registers[register_x as usize] =
                        self.registers[register_x as usize] ^ self.registers[register_y as usize]
                }
                0x4 => {
                    let mut had_carry = 0;
                    if (self.registers[register_x as usize] as u32)
                        + (self.registers[register_y as usize] as u32)
                        > 255
                    {
                        had_carry = 1;
                    };
                    self.registers[register_x as usize] =
                        self.registers[register_x as usize] + self.registers[register_y as usize];
                    self.registers[0xF] = had_carry;
                }
                0x5 => {
                    let mut had_borrow = 0;
                    if self.registers[register_x as usize] > self.registers[register_y as usize] {
                        had_borrow = 1;
                    };
                    self.registers[register_x as usize] =
                        self.registers[register_x as usize] - self.registers[register_y as usize];

                    self.registers[0xF] = had_borrow;
                }
                0x6 => {
                    let lease_significant = self.registers[register_x as usize] & 0x1;
                    self.registers[register_x as usize] = self.registers[register_y as usize] >> 1;
                    self.registers[0xF] = lease_significant;
                }
                0x7 => {
                    let mut had_borrow = 0;
                    if self.registers[register_y as usize] > self.registers[register_x as usize] {
                        had_borrow = 1;
                    };
                    self.registers[register_x as usize] =
                        self.registers[register_y as usize] - self.registers[register_x as usize];

                    self.registers[0xF] = had_borrow;
                }
                0xe => {
                    let most_significant = self.registers[register_x as usize] >> 7;
                    self.registers[register_x as usize] = self.registers[register_y as usize] << 1;
                    self.registers[0xF] = most_significant;
                }
                _ => {}
            },
            0x9 => {
                if self.registers[register_x as usize] != self.registers[register_y as usize] {
                    self.pc = self.pc + 0x2;
                }
            }
            0xA => self.i = memory_address,
            0xB => self.pc = memory_address + self.registers[0] as u16,
            0xC => {} // RNG
            0xD => {
                let mut x = self.registers[register_x as usize];
                if (x > 0x3F) {
                    x = x % 0x40;
                }
                let mut y = self.registers[register_y as usize];
                if (y > 0x1F) {
                    y = y % 0x20;
                }

                let number_of_bytes = hexadecimal_nibble;

                let mut pixel_has_changed_to_unset = false;

                let total_bytes =
                    &self.memory[self.i as usize..(self.i + number_of_bytes) as usize].to_vec();

                for (i, &value) in total_bytes.iter().enumerate() {
                    let mut pixel_pointer =
                        DISPLAY_DATA_ADDRESS as u8 + x + y * WIDTH as u8 + WIDTH as u8 * i as u8;
                    let binary_data = Self::convert_decimal_byte_to_binary(value);

                    for element in binary_data {
                        let previous_pixel_value = self.memory[pixel_pointer as usize];
                        if element as u16 != previous_pixel_value {
                            self.memory[pixel_pointer as usize] = 1;
                        } else {
                            self.memory[pixel_pointer as usize] = 0;
                        }
                        if previous_pixel_value != self.memory[pixel_pointer as usize]
                            && self.memory[pixel_pointer as usize] == 0
                        {
                            pixel_has_changed_to_unset = true;
                        }
                        pixel_pointer += 1;
                    }
                    pixel_pointer += WIDTH as u8;
                }

                if pixel_has_changed_to_unset {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            0xE => {} // Key Registers
            0xF => match hexadecimal_byte {
                0x07 => {} // Delay
                0x0A => {} // Key Press
                0x15 => {} // Delay
                0x18 => {} // Sound
                0x1E => self.i = self.i + self.registers[register_x as usize] as u16,
                0x29 => {} // Character Data
                0x33 => {} // Binary Coded Decimal
                0x55 => {
                    for index in 0..register_x {
                        self.memory[(self.i + index) as usize] =
                            self.registers[index as usize] as u16;
                    }
                    self.i = self.i + register_x + 1;
                }
                0x65 => {
                    for index in 0..register_x {
                        self.registers[index as usize] =
                            self.memory[(self.i + index) as usize] as u8;
                    }
                    self.i = self.i + register_x + 1;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn get_display_data(&self) -> &[u16] {
        return &self.memory[DISPLAY_DATA_ADDRESS as usize..4096];
    }

    fn add_to_routine_stack(&mut self, memory_address: u16) {
        for index in 0..11 {
            if self.memory[index] as u16 == 0x0 {
                self.memory[index] = memory_address as u16;
                self.pc = memory_address;
                break;
            }
        }
    }

    fn remove_from_routine_stack(&mut self) {
        for index in 0..11 {
            if self.memory[index] as u16 == 0x0 {
                self.memory[index - 1] = 0x0;
                self.pc = self.memory[index - 2];
                break;
            }
        }
    }

    fn clear_screen(&mut self) {
        for index in 1..352 {
            self.memory[index + DISPLAY_DATA_ADDRESS] = 0x0;
        }
    }

    fn convert_decimal_byte_to_binary(value: u16) -> Vec<u8> {
        let mut binary_vec = Vec::new();
        let mut quotient = value;
        while quotient > 0 {
            binary_vec.insert(0, (quotient % 2) as u8);
            quotient /= 2;
        }
        while binary_vec.len() < 8 {
            binary_vec.insert(0, 0);
        }
        binary_vec
    }
}

//? Subroutines goes on emulator memory

//? Delay timer stop running for wait

//? Sound multipler by beep
