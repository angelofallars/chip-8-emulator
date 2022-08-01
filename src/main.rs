use std::fs;
use std::io::{self, Read, Write};
use std::{fs::File, process::exit};
use std::{thread, time};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "chip-8-emulator")]
#[clap(author = "Angelo Fallaria <ba.fallaria@gmail.com")]
#[clap(version = "1.0")]
#[clap(about = "An emulator for chip-8 games written in Rust.")]
struct Args {
    #[clap(value_parser)]
    file_name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
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

        // println!("{:#04x}", x);
        // println!("{:#04x}", y);
        // println!("{:#04x}", n);
        // println!("{:#04x}", nn);
        // println!("{:#04x}", nnn);
        //
        // println!("{:#?}", register);
        // println!("{:#?}", index_register);

        program_counter += 2;

        // decode instruction
        match instruction & 0xF000 {
            0x0000 => {
                if instruction == 0x00E0 {
                    for i in 0..64 {
                        for j in 0..32 {
                            display[i][j] = false;
                        }
                    }
                }
            }
            0x1000 => {
                program_counter = nnn;
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
            0xA000 => {
                index_register = nnn;
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
            _ => {}
        }
        // execute instruction
        print_display(display);
        thread::sleep(millis);
    }
}

fn print_display(display: [[bool; 32]; 64]) {
    print!("\x1B[2J");

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    handle.write_all(b"\x1B[2J").unwrap();

    for i in 0..32 {
        for j in 0..64 {
            if display[j][i] == true {
                handle.write(b"\xE2\x96\x88").unwrap();
            } else {
                handle.write(b" ").unwrap();
            }
        }
        handle.write_all(b"\n").unwrap();
    }
    handle.flush().unwrap();
}
