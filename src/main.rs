use std::fs;
use std::io::{self, Read, Write};
use std::{fs::File, process::exit};
use std::{thread, time};

use ::rand::random;
use clap::Parser;
use macroquad::prelude::*;

#[derive(Parser, Debug)]
#[clap(name = "chip-8-emulator")]
#[clap(author = "Angelo Fallaria <ba.fallaria@gmail.com")]
#[clap(version = "1.0")]
#[clap(about = "An emulator for chip-8 games written in Rust.")]
struct Args {
    #[clap(value_parser)]
    file_name: String,
}

#[macroquad::main("CHIP-8-Emulator")]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args = Args::parse();

    let file = fs::read(args.file_name)?;

    let mut memory: [u8; 4096] = [0; 4096];
    let mut display: [[bool; 32]; 64] = [[false; 32]; 64];
    let mut program_counter: u16 = 512;
    let mut index_register: u16 = 0;
    let mut stack: Vec<u16> = Vec::new();
    let mut delay_timer: u8 = 0;
    let mut sound_timer: u8 = 0;
    let mut register: [u8; 16] = [0; 16];
    let mut keypad: [bool; 16] = [false; 16];

    let millis = time::Duration::from_millis(5);

    // Load the program in memory
    for i in 0..file.len() {
        memory[512 + i] = file[i];
    }

    loop {
        // fetch instruction
        let instruction: u16;
        instruction = ((memory[program_counter as usize] as u16) << 8)
            | (memory[program_counter as usize + 1] as u16);

        let x: usize = ((instruction & 0x0F00) >> 8).try_into().unwrap();
        let y: usize = ((instruction & 0x00F0) >> 4).try_into().unwrap();
        let n: u8 = ((instruction & 0x000F) >> 0).try_into().unwrap();
        let nn: u8 = ((instruction & 0x00FF) >> 0).try_into().unwrap();
        let nnn = (instruction & 0x0FFF) >> 0;

        program_counter += 2;

        // decode instruction
        match instruction & 0xF000 {
            0x0000 => match instruction & 0x0FFF {
                0x00E0 => {
                    for i in 0..64 {
                        for j in 0..32 {
                            display[i][j] = false;
                        }
                    }
                }
                0x00EE => {
                    let address = stack.pop().unwrap();
                    program_counter = address;
                }
                _ => {}
            },
            0x1000 => {
                program_counter = nnn;
            }
            0x2000 => {
                stack.push(program_counter);
                program_counter = nnn;
            }
            0x3000 => {
                if register[x] == nn {
                    program_counter += 2;
                }
            }
            0x4000 => {
                if register[x] != nn {
                    program_counter += 2;
                }
            }
            0x5000 => {
                if register[x] == register[y] {
                    program_counter += 2;
                }
            }
            0x9000 => {
                if register[x] != register[y] {
                    program_counter += 2;
                }
            }
            0x6000 => {
                register[x] = nn;
            }
            0x7000 => {
                if register[x] as usize + nn as usize > 0xFF {
                    register[x] = 0xFF;
                } else {
                    register[x] += nn;
                }
            }
            0x8000 => match instruction & 0x000F {
                0x0000 => {
                    register[x] = register[y];
                }
                0x0001 => {
                    register[x] = register[x] | register[y];
                }
                0x0002 => {
                    register[x] = register[x] & register[y];
                }
                0x0003 => {
                    register[x] = register[x] ^ register[y];
                }
                0x0004 => {
                    if register[x] as usize + register[y] as usize >= 0xFF {
                        register[x] = 0xFF;
                        register[0xF] = 1;
                    } else {
                        register[x] += register[y];
                        register[0xF] = 0;
                    }
                }
                0x0005 => {
                    if register[x] >= register[y] {
                        register[x] = register[x] - register[y];
                        register[0xF] = 1;
                    } else {
                        register[x] = 0;
                        register[0xF] = 0;
                    }
                }
                0x0007 => {
                    if register[y] >= register[x] {
                        register[x] = register[y] - register[x];
                        register[0xF] = 1;
                    } else {
                        register[x] = 0;
                        register[0xF] = 0;
                    }
                }
                0x0006 => {
                    let shifted_out_bit = register[x] & 0x01;
                    register[x] = register[x] >> 1;
                    register[0xF] = shifted_out_bit;
                }
                0x000E => {
                    let shifted_out_bit = register[x] & 0x80 >> 7;
                    register[x] = register[x] << 1;
                    register[0xF] = shifted_out_bit;
                }
                _ => {}
            },
            0xA000 => {
                index_register = nnn;
            }
            0xB000 => {
                program_counter = nnn + register[0] as u16;
            }
            0xC000 => {
                register[x] = random::<u8>() & nn;
            }
            0xD000 => {
                // draw
                let mut x_coord = (register[x] % 64) as usize;
                let mut y_coord = (register[y] % 32) as usize;
                register[0xF] = 0;

                for i in 0..n {
                    let byte = memory[index_register as usize + i as usize];

                    // Reset the x coordinates for every row
                    let mut x_coord = x_coord;

                    for j in 0..8 {
                        let current_bit = (byte >> (7 - j)) & 0b1;

                        if current_bit == 1 && display[x_coord][y_coord] == true {
                            display[x_coord][y_coord] = false;
                            register[0xF] = 1;
                        }
                        if current_bit == 1 && display[x_coord][y_coord] == false {
                            display[x_coord][y_coord] = true;
                        }

                        if x_coord >= 63 {
                            break;
                        }

                        x_coord += 1;
                    }

                    y_coord += 1;
                    if y_coord >= 31 {
                        break;
                    }
                }
            }
            0xE000 => match instruction & 0x00FF {
                0x009E => {
                    if keypad[register[x] as usize] == true {
                        program_counter += 2;
                    }
                }
                0x00A1 => {
                    if keypad[register[x] as usize] == false {
                        program_counter += 2;
                    }
                }
                _ => {}
            },
            0xF000 => match instruction & 0x00FF {
                0x0007 => {
                    register[x] = delay_timer;
                }
                0x0015 => {
                    delay_timer = register[x];
                }
                0x0018 => {
                    sound_timer = register[x];
                }
                0x001E => {
                    if index_register + register[x] as u16 > 1000 {
                        index_register = 1000;
                        register[0xF] = 1;
                    } else {
                        index_register += register[x] as u16;
                    }
                }
                0x000A => {
                    let mut some_key_pressed = false;

                    for i in 0..16 {
                        if keypad[i] == true {
                            some_key_pressed = true;
                            register[x] = i.try_into().unwrap();
                            break;
                        }
                    }

                    if !some_key_pressed {
                        program_counter -= 2;
                    }
                }
                0x0029 => {}
                0x0033 => {
                    let number = register[x];
                    let digit_one = (number / 100) % 100;
                    let digit_two = (number / 10) % 10;
                    let digit_three = number % 10;

                    memory[index_register as usize] = digit_one;
                    memory[index_register as usize + 1] = digit_two;
                    memory[index_register as usize + 2] = digit_three;
                }
                0x0055 => {
                    let num = register[x];
                    for i in 0..num {
                        memory[index_register as usize + i as usize] = register[i as usize];
                    }
                }
                0x0065 => {
                    let num = register[x];
                    for i in 0..num {
                        register[i as usize] = memory[index_register as usize + i as usize];
                    }
                }
                _ => {}
            },
            _ => {}
        }
        // execute instruction
        update_keypad(keypad);
        print_display(display);

        thread::sleep(millis);
        next_frame().await
    }
}

const PIXEL_SIZE: usize = 16;

fn print_display(display: [[bool; 32]; 64]) {
    clear_background(BLACK);
    for i in 0..32 {
        for j in 0..64 {
            if display[j][i] == true {
                draw_rectangle(
                    (j * PIXEL_SIZE) as f32,
                    (i * PIXEL_SIZE) as f32,
                    PIXEL_SIZE as f32,
                    PIXEL_SIZE as f32,
                    WHITE,
                );
            }
        }
    }
}

fn update_keypad(mut keypad: [bool; 16]) {
    let keycodes = [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Q,
        KeyCode::W,
        KeyCode::E,
        KeyCode::R,
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
        KeyCode::Z,
        KeyCode::X,
        KeyCode::C,
        KeyCode::V,
    ];

    for i in 0..16 {
        if is_key_down(keycodes[i]) {
            keypad[i] = true;
        } else {
            keypad[i] = false;
        }
    }
}
