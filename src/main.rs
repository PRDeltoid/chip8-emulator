/************
CHIP8 Memory Map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM

V Regs are 1 byte long (u8)
Opcodes are 2 bytes long (u16). This means we must combine 2 1-byte numbers in memory into a single 2-byte number
This is done by rotating the leading number (the big end) by 8 bits. This will create a 2-byte number with 1 byte of zeros at the little end
We then bitwise OR our 2 byte number and our 1-byte number that we want to combine. All of the 1s in the 1-byte number are kept in the final result

Rust does not allow Hex literals in code, so most hex will be converted to decimal before being entered.
Their decimal equivalence and purpose should be noted in the comments or via constants

To extract nibbles as individual numbers, we mask the nibble and then rotate that nibble to the right until it is in the "1"s place
************/
#![feature(duration_as_u128)]
extern crate piston_window;
extern crate rand;


use std::fs::File;
use std::ops::Range;
use std::io::{stdin, stdout, Read, Write};
use std::time::Duration;
use std::thread::sleep;

use piston_window::*;

const FIRST_NIBBLE_MASK: u16 = 0xF000;  //Grabs first nibble only
const SECOND_NIBBLE_MASK: u16 = 0x0F00; //Grabs second nibble only
const THIRD_NIBBLE_MASK: u16 = 0x00F0;
const FOURTH_NIBBLE_MASK: u16 = 0x000F;

const LAST_TWO_MASK: u16 = 0x00FF;      //Grabs the last two nibbles
const LAST_THREE_MASK: u16 = 0x0FFF;    //Grabs last three nibbles only

pub struct Chip8 {
    memory: [u8; 4096], //General purpose memory
    v: [u8; 16],        //General purpose registers. Register 16 is the "carry flag"

    i: u16,             //Index register
    pc: u16,            //Program counter (instruction pointer)

    screen: [u8; 64 * 32], //Array for storing screen pixels. Screen is 64 x 32 pixels
    draw_flag: bool,

    delay_timer: u8,    //Counts down at 60Hz speed to zero
    sound_timer: u8,    //Same as above, system buzzer sounds when it reaches zero

    stack: [u16; 16],   //Stack for program execution. Use to return to calling program after called program is finished
    sp: u16,            //Stack pointer, to keep track of what is currently the "top"

    key: [u8; 16],     //Hex based keypad
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096], //Initialize our memory
            v: [0; 16],        //Zero out our registers
            i: 0,
            pc: 512,           //program counter starts at 0x200 (system data comes before)
            screen: [0; 64 * 32],
            draw_flag: false,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
        }
    }

    pub fn initialize(&mut self) {
    }

    //Increments the program counter to pull the next opcode
    fn next_instruction(&mut self) {
        self.pc += 2;
    }

    //Loads font sprites into memory starting at location 0x0000 to 0x01FF
    pub fn load_font(&mut self, font_path: &str) {
        let font = File::open(font_path).unwrap();
        let mut i = 0;

        for byte in font.bytes() {
            self.memory[i] = byte.unwrap();
            i += 1;
            //Prevent malformed font file from overwriting program data
            if i >= 512 { break; }
        }
    }

    pub fn set_key(&mut self, key: u8, value: u8) {
        self.key[key as usize] = value;
        println!("key {} set to {}", key, value);
    }

    //Loads a ROM into memory starting at location 0x0200
    pub fn load_rom(&mut self, rom_path: &str) {
        let rom = File::open(rom_path).unwrap();
        let mut i = 512;

        for byte in rom.bytes() {
            self.memory[i] = byte.unwrap();
            i += 1;
        }

        /*Print a small memory map for debugging purposes
        for i in 512..550 {
            println!("{}: {:#04X}", i, self.memory[i])
        }*/
    }

    //Reads two bytes from memory and combines them into a single opcode number
    fn read_opcode(&mut self) -> u16 {
        //Grab the first half of the opcode as 2-byte, shifted 8 bits left
        let opcode1: u16 = (self.memory[self.pc as usize] as u16) << 8;
        //Grab second half of opcode as 2-byte
        let opcode2: u16 = self.memory[(self.pc + 1) as usize] as u16;
        //OR the two two-byte numbers (one "big end" and one "small end") to combine them
        let opcode = opcode1 | opcode2;

        opcode
    }

    fn draw(&mut self, window: &mut PistonWindow, event: &Event) {
        let pixel_size = 8.0; // self.pixel_size as f64;
        let x_size = 64; //self.x_size as usize;
        let y_size = 32; //self.y_size as usize;
        window.draw_2d(event, |c, g| {

            //Step over each x "pixel"
            for x in 0..x_size as usize {
                //Step over each y "pixel" for each x above
                for y in 0..y_size as usize {
                    //If the screen contains a 1 at the current pixel...
                    if self.screen[x + (y * x_size as usize)] == 1 {
                        let x_pos = x as f64 * pixel_size;
                        let y_pos = y as f64 * pixel_size;
                        //println!("Drawing rect at x:{}, y:{}", x_pos, y_pos);
                        Rectangle::new([1.0, 1.0, 1.0, 1.0])
                            .draw([x_pos, y_pos, pixel_size, pixel_size], &c.draw_state, c.transform, g)
                    }
                }
            }
        });
    }

    fn clear(&mut self, window: &mut PistonWindow, event: &Event) {
        window.draw_2d(event, |_context, graphics| {
            clear(color::BLACK, graphics);
        });
    }

    //Pulls the current opcode in memory (at program counter) and performs it's required operations
    pub fn emulate_cycle(&mut self, window: &mut PistonWindow, event: &Event) {
        //Fetch opcode
        let opcode = self.read_opcode();

        //Print opcode as a 6-digit hex number, including leading zeros and "0x" notation.
        print!("Opcode: {:#06X} - ", opcode); //ie 0x0012

        //Decode and execute opcode
        //Check our first hex digit (nibble)
        match opcode & FIRST_NIBBLE_MASK {
            //0x0NNN opcodes
            0x0000 => {
                match opcode & FOURTH_NIBBLE_MASK {
                    //0x0000 opcode (clear screen)
                    0x0000 => {
                        println!("Clear Screen");
                        self.clear(window, event);
                        self.next_instruction();
                    },
                    //0x00EE opcode (return from sub-process)
                    0x000E => {
                        println!("Returning to {:#06X}", self.stack[self.sp as usize]);
                        //Set program counter to the address at the top of the stack
                        self.pc = self.stack[self.sp as usize];
                        //Move the stack pointer down one to "pop" the previous stack information
                        self.sp -= 1;
                        self.next_instruction()
                    },
                    _ => { println!("Unknown 0x000N opcode")}
                }
            },
            //0x1NNN opcode (jmp nnn)
            0x1000 => {
                self.pc = opcode & LAST_THREE_MASK;
                println!("Jumping to {:#06X}", self.pc);
            },
            //0x2NNN opcode (call subroutine: push pc to stack, jmp nnn)
            0x2000 => {
                //Move stack pointer up one because we are "pushing" data in
                self.sp += 1;
                //Push the current program counter into the stack at the "top"
                self.stack[self.sp as usize] = self.pc;
                //Jump to address NNN
                self.pc = opcode & LAST_THREE_MASK;
                println!("Call routine at {:#06X}", self.pc-512);
            },
            //0x3XKK opcode (Skp next instruction if Vx == kk)
            0x3000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                println!("SE V[{}] ({}), {}", x, self.v[x], kk);
                if self.v[x] == kk {
                    //Skip next instruction by adding 2 to the program counter (skipping 2 bytes or 1 opcode)
                    self.next_instruction();
                }
                self.next_instruction();
            },
            //0x4XKK opcode (Skp next instruction if Vx != kk)
            0x4000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                println!("SNE V[{}] ({}), {}", x, self.v[x], kk);
                if self.v[x] != kk {
                    //Skip next instruction by adding 2 to the program counter (skipping 2 bytes or 1 opcode)
                    self.next_instruction();
                }
                self.next_instruction();
            },
            //0x5XY0 (Skp next instruction if Vx == Vy)
            0x5000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let y = ((opcode & THIRD_NIBBLE_MASK) >> 4) as usize;
                println!("SE V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                if self.v[x] == self.v[y] {
                    self.next_instruction();
                }
                self.next_instruction();
            },
            //0x6XKK (Load Vx with kk)
            0x6000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                println!("Load V[{}] ({}) with {}", x, self.v[x], kk);
                self.v[x] = kk;
                self.next_instruction();
            },
            //0x7XKK (Add Vx, kk)
            0x7000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u16;
                println!("Add V[{}] ({}) with {}", x, self.v[x], kk);
                //Add and keep only the last byte by masking.
                self.v[x] = (((self.v[x] as u16) + kk) & 0x00FF) as u8;
                self.next_instruction();
            },
            //0x8XYN (Vx/Vy operations)
            0x8000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let y = ((opcode & THIRD_NIBBLE_MASK) >> 4) as usize;
                //println!("X: {}, Y: {}", x, y );
                match opcode & FOURTH_NIBBLE_MASK  {
                    //0x8XY0 (MOV v[x], v[y])
                    0x0000 => {
                        println!("Mov V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        self.v[x] = self.v[y];
                    },
                    //0x8XY1 (OR v[x], v[y])
                    0x0001 => {
                        println!("Or V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        self.v[x] = self.v[x] | self.v[y];
                    },
                    //0x8XY2 (AND v[x], v[y])
                    0x0002 => {
                        println!("And V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        self.v[x] = self.v[x] & self.v[y];
                    },
                    //0x8XY3 (XOR v[x], v[y])
                    0x0003 => {
                        println!("Xor V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        self.v[x] = self.v[x] ^ self.v[y];
                    },
                    //0x8XY4 (ADD v[x], v[y])
                    0x0004 => {
                        println!("Add V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        //Set carry if addition goes over 8 bits
                        let (_, overflow) = self.v[x].overflowing_add(self.v[y]);
                        if overflow {
                            self.v[0x0f] = 1;
                        } else {
                            self.v[0x0f] = 0;
                        }
                    },
                    //0x8XY5 (SUB v[x], v[y])
                    0x0005 => {
                        println!("Sub V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        if self.v[x] > self.v[y] {
                            self.v[0x0f] = 1;
                        } else {
                            self.v[0x0f] = 0;
                        }
                        self.v[x] = self.v[x] - self.v[y];
                    },
                    //0x8XY6 (SHR v[x], 1)
                    0x0006 => {
                        println!("Shift Right V[{}] ({}), 1", x, self.v[x]);
                        //If Most Significant Bit is 1, set VF to 1
                        if(opcode & 0b1000_0000) == 0b1000_0000 {
                            self.v[0x0f] = 1;
                        }
                        self.v[x] = self.v[x] >> 1;
                    },
                    //0x8XY7 (SUBN v[x], v[y])
                    0x0007 => {
                        println!("Subn V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                        if self.v[y] > self.v[x] {
                            self.v[0x0f] = 1;
                        } else {
                            self.v[0x0f] = 0;
                        }
                        self.v[x] = self.v[x] - self.v[y];
                    },
                    //0x8XY6 (SHL v[x], 1)
                    0x000E => {
                        println!("Shift Left V[{}] ({}), 1", x, self.v[x]);
                        //If Least Significant Bit is 1, set VF to 1
                        if (opcode & 0b0000_0001) == 0b0000_0001 {
                            self.v[0x0f] = 1;
                        }
                        self.v[x] = self.v[x] << 1;
                    },
                    _ => { println!("Unknown 0x800N opcode")}
                }
                //None of the 8NNN opcodes affect the PC, so we can increment it at the end no matter what
                self.next_instruction();
            },
            //0x9XY0 (Skip next instruction if Vx != Vy
            0x9000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                let y = ((opcode & THIRD_NIBBLE_MASK) >> 4) as usize;

                println!("SNE V[{}] ({}), V[{}] ({})", x, self.v[x], y, self.v[y]);
                if self.v[x] != self.v[y] {
                    self.next_instruction();
                }
                self.next_instruction();
            },
            //0xANNN opcode (mv i, NNN)
            0xA000 => {
                self.i = opcode & LAST_THREE_MASK;
                println!("Changing index to {:}d", self.i);
                self.next_instruction();
            },
            //0xBNNN opcode (jmp NNN + V0)
            0xB000 => {
                println!("Jmp NNN + V[0]");
                self.pc = (opcode & LAST_THREE_MASK) + self.v[0] as u16;
            },
            //0xCXNN opcode (rnd Vx, byte AND NN)
            0xC000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) >> 8;
                let n = opcode & LAST_TWO_MASK;
                let rand = rand::random::<u16>();

                println!("V[{}] ({}) = n: {} & {}", x, self.v[x as usize], n, rand);
                self.v[x as usize] = (rand & n) as u8;
                self.next_instruction();

            }
            //0xDxyn opcode
            0xD000 => {
                //X Coord to draw at
                let x = self.v[((opcode & SECOND_NIBBLE_MASK) >> 8) as usize] as usize;
                //Y Coord to draw at
                let y = self.v[((opcode & THIRD_NIBBLE_MASK) >> 4) as usize] as usize;
                //line height of the sprite (width is ALWAYS 8)
                let height = (opcode & FOURTH_NIBBLE_MASK) as usize;

                //Unset our collision flag
                self.v[0x0F] = 0;

                println!("Draw Sprite starting at mem[{}] at loc x:{}, y:{} with height:{}", self.i, x, y, height);

                //Holds the current pixel data
                let mut pixel_line: u8;

                //For each line in the sprite from 0 to the sprite's height
                for yline in 0..height {
                    //Grab our sprite's 8-bit pixel line at this spot
                    pixel_line = self.memory[self.i as usize + yline];
                    //For each pixel (bit) in the line... (always width of 8, remember!)
                    for xline in 0..8 {
                        //If the current bit is set...
                        if (pixel_line >> xline) & 0b00000001 != 0 { //this hack separates each bit in the pixel line by masking it and then rotating the bits to the right until they are in the 1s place
                            //Check for pixel collision
                            let index: usize = x + xline + ((y + yline) * 64);

                            if index >= 2048 {
                                //break;
                                continue;
                            }

                            if self.screen[x + xline + ((y + yline) * 64)] == 1 {
                                //If there is a collision, set the collision register VF to 1
                                self.v[0xF] = 1;
                            }
                            //Set the value of the line by XORing our sprite's current line onto it
                            self.screen[x + xline + ((y + yline) * 64)] ^= 1;
                        }
                    }
                }
                //Tell the screen that it has to refresh after this operation
                self.draw_flag = true;
                self.next_instruction();
            },
            //0xE0NN opcodes
            0xE000 => {
                match opcode & LAST_TWO_MASK {
                    //0xEx9E Skip next instruct if key with value of Vx is pressed
                    0x009E => {
                        let x = ((opcode & THIRD_NIBBLE_MASK) >> 8) as usize;
                        println!("SN if Key[{}] is pressed", x);
                        if self.key[x] == 1 {
                            self.next_instruction();
                        }
                        self.next_instruction();
                    },
                    //0xEx9E Skip next instruct if key with value of Vx is not pressed
                    0x00A1 => {
                        let x = ((opcode & THIRD_NIBBLE_MASK) >> 8) as usize;
                        println!("SN if Key[{}] is not pressed", x);
                        if self.key[x] == 0 {
                            self.next_instruction();
                        }
                        self.next_instruction();
                    },
                    _ => {
                        println!("Unknown 0xE000 opcode");
                    }
                }
            },
            //0xFXNN opcodes
            0xF000 => {
                let x = ((opcode & SECOND_NIBBLE_MASK) >> 8) as usize;
                match opcode & LAST_TWO_MASK  {
                    //0xFX07 (mv v[x], delay_timer)
                    0x0007 => {
                        println!("Mv V[{}] ({}), delay_timer", x, self.v[x]);
                        self.v[x] = self.delay_timer;
                        self.next_instruction();
                    },
                    //Wait for key press, store value of key in Vx
                    //All execution stops until a key is pressed
                    0x000A => {
                        let x = (opcode & THIRD_NIBBLE_MASK) >> 8;
                        println!("Wait for key press {}", self.v[x as usize]);
                    },
                    //0xFX15 (mov delay_timer, v[x])
                    0x0015 => {
                        println!("Mov delay_timer, V[{}] ({})", x, self.v[x]);
                        self.delay_timer = self.v[x];
                        self.next_instruction();
                    },
                    //0xFX18 (mov sound_timer, v[x])
                    0x0018 => {
                        println!("Mov sound_timer, V[{}] ({})", x, self.v[x]);
                        self.sound_timer = self.v[x];
                        self.next_instruction();
                    },
                    //0xFX1E (add i, v[x])
                    0x001E => {
                        println!("Add V[{}] ({}) to index", x, self.v[x]);
                        self.i += self.v[x] as u16;
                        self.next_instruction();
                    },
                    0x0029 => {
                        println!("Set I = location of sprite for digit Vx");
                        self.next_instruction();
                    },
                    0x0033 => {
                        println!("Store BCD of Vx in memory at location i, i+1, i+2");
                        //Take each numbers place in V[x] and separate them to store in separate memory locations
                        let bcd = self.v[x];
                        self.memory[self.i as usize] = bcd / 100;
                        self.memory[self.i as usize + 1] = (bcd / 10) % 10;
                        self.memory[self.i as usize + 2] = (bcd % 100) % 10;

                        self.next_instruction();
                    },
                    0x0055 => {
                        println!("Stores registers V0 through V{} in memory starting at location {:#06X}", x, self.i);
                        for n in 0..x {
                           self.memory[self.i as usize + n] = self.v[n];
                        }
                        self.next_instruction();
                    },
                    0x0065 => {
                        println!("Read registers V0 through Vx from memory starting at location I");
                        for n in 0..x {
                            self.v[n] = self.memory[self.i as usize + n];
                        }
                        self.next_instruction();
                    },
                    _ => { println!("Unknown 0xF0NN opcode")},
                }
            }
            _ => {
                println!("Unknown opcode {:}", opcode);
            },
        }

        //Update timer(s)
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                //Make a beep noise
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }

        if self.draw_flag == true {
            //Draw the screen
            self.draw(window, event);
            //println!("Draw Screen");

            //Unset our draw flag for the next op
            self.draw_flag = false;
        }

    }

    //Print the bytes in memory between the given range (for debugging purposes)
    pub fn print_memory(&self, range: Range<usize>) {
        for i in range {
            println!("{:#04X}", self.memory[i]);
        }
    }
}

//Simple system("pause") equivalent in Rust.
fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

fn key_translator(button: Button) -> u8 {
    match button {
        Button::Keyboard(Key::D1) => 0,
        Button::Keyboard(Key::D2) => 1,
        Button::Keyboard(Key::D3) => 2,
        Button::Keyboard(Key::D4) => 3,
        Button::Keyboard(Key::Q) => 4,
        Button::Keyboard(Key::W) => 5,
        Button::Keyboard(Key::E) => 7,
        Button::Keyboard(Key::R) => 8,
        Button::Keyboard(Key::A) => 9,
        Button::Keyboard(Key::S) => 0x0A,
        Button::Keyboard(Key::D) => 0x0B,
        Button::Keyboard(Key::F) => 0x0C,
        Button::Keyboard(Key::Z) => 0x0D,
        Button::Keyboard(Key::X) => 0x0E,
        Button::Keyboard(Key::C) => 0x0F,
        Button::Keyboard(Key::V) => 0x10,
        _ => 255,
    }

}

fn main() {
    let height: u32 = 64 * 8; //x_size as u32 * pixel_size as u32;
    let width: u32 = 32 * 8; //y_size as u32 * pixel_size as u32;

    let mut window: PistonWindow = WindowSettings::new(
        "Chip8",
        [height, width]
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    //Update screen, even when no input is given
    //This makes sure our emulation cycle (which is tied to game loop) keeps running
    window.set_lazy(false);

    //Create and initialize our Chip8 object
    let mut chip8 = Chip8::new();
    chip8.initialize();

    //Load up our font into reserved system memory
    chip8.load_font("font.c8");

    //Load up our ROM into program memory
    chip8.load_rom("Particle.ch8");

    while let Some(e) = window.next() {

        //Set/unset keys
        if let Some(button) = e.button_args() {
            //Key translation (1234, qwer, asdf, zxcv hex keyboard)
            let key = key_translator(button.button);

            let state = match button.state {
                ButtonState::Press => 1,
                ButtonState::Release => 0,
            };

            //If a key was pressed, set its state
            if key != 255 {
                chip8.set_key(key, state);
            } else {
                println!("Unknown key");
            }
        };

        //While the program counter is within an acceptable range...
        if chip8.pc > 4096 {
            println!("Accessing invalid memory, aborting");
            return;
        }
        //Emulate a CPU cycle
        chip8.emulate_cycle(&mut window, &e);

        //Since we emulate WAYYY fast, sleep for 16ms to make it about 60Hz emulation speed
        sleep(Duration::from_millis(16));
    }
    //Pause after execution to observe the state of the screen
    pause();

}
