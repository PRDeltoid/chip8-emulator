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
************/
use std::io;
use std::io::prelude::*;
use std::fs::File;

const FIRST_NIBBLE_MASK: u16 = 0xF000;  //Grabs first nibble only
const SECOND_NIBBLE_MASK: u16 = 0x0F00; //Grabs second nibble only
const THIRD_NIBBLE_MASK: u16 = 0x00F0;
const FOURTH_NIBBLE_MASK: u16 = 0x000F;

const LAST_THREE_MASK: u16 = 0x0FFF;    //Grabs last three nibbles only

pub struct Chip8 {
    opcode: u16,        //Opcode
    memory: [u8; 4096], //General purpose memory
    v: [u8; 16],        //General purpose registers. Register 16 is the "carry flag"

    i: u16,             //Index register
    pc: u16,            //Program counter (instruction pointer)

    screen: [u8; 64 * 32], //Array for storing screen pixels. Screen is 64 x 32 pixels

    delay_timer: u8,    //Counts down at 60Hz speed to zero
    sound_timer: u8,    //Same as above, system buzzer sounds when it reaches zero

    stack: [u16; 16],   //Stack for program execution. Use to return to calling program after called program is finished
    sp: u16,            //Stack pointer, to keep track of what is currently the "top"

    key: [u8; 16],     //Hex based keypad
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            opcode: 0,         //Blank opcode
            memory: [0; 4096], //Initialize our memory
            v: [0; 16],        //Zero out our registers
            i: 0,
            pc: 512,           //program counter starts at 0x200 (system data comes before)
            screen: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
        }
    }

    pub fn initialize(&mut self) {
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let mut rom = File::open(rom_path).unwrap();
        let mut i = 512;

        for byte in rom.bytes() {
            self.memory[i] = byte.unwrap();
            i += 1;
        }

        //Print a small memory map for debugging purposes
        for i in 512..550 {
            println!("{}: {:#04X}", i, self.memory[i])
        }
    }

    fn read_opcode(&mut self) -> u16 {
        //Grab the first half of the opcode as 2-byte, shifted 8 bits left
        let opcode1: u16 = (self.memory[self.pc as usize] as u16) << 8;
        //Grab second half of opcode as 2-byte
        let opcode2: u16 = self.memory[(self.pc + 1) as usize] as u16;
        //OR the two two-byte numbers (one "big end" and one "small end") to combine them
        let opcode = opcode1 | opcode2;

        opcode
    }

    pub fn emulate_cycle(&mut self) {
        //Fetch opcode
        let opcode = self.read_opcode();

        //Print opcode as a 6-digit hex number, including leading zeros and "0x" notation.
        println!("Opcode is {:#06X}", opcode); //ie 0x0012

        //Decode and execute opcode
        //Check our first hex digit (nibble)
        match opcode & FIRST_NIBBLE_MASK {
            //0x0NNN opcodes
            0x0000 => {
                match opcode & FOURTH_NIBBLE_MASK {
                    //0x0000 opcode (clear screen)
                    0x0000 => { println!("Clear Screen") },
                    //0x00EE opcode (return from sub-process)
                    0x000E => { println!("Return") },
                    _ => { println!("Unknown 0x000N opcode")}
                }
            },
            //0xANNN opcode (mv i, NNN)
            0xA000 => {
                self.i = opcode & LAST_THREE_MASK;  //4095 is 0x0fff in hex (our mask to grab xxx from above)
                self.pc += 2;
                println!("Changing index to {:}d", self.i)
            },
            //0xBNNN opcode (jmp NNN + V0)
            0xB000 => {
                self.pc = (opcode & LAST_THREE_MASK) + self.v[0] as u16;

            }
            default => {
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
            }
            self.sound_timer -= 1;
        }
    }
}

fn main() {
    //Create and initialize our Chip8 object
    let mut chip8 = Chip8::new();

    chip8.initialize();
    chip8.load_rom("foo.rom");

    //Manually load in some opcodes for testing
    //Program memory starts at address 512 (0x200)
    /*chip8.memory[512] = 161; //A1 in Hex
    chip8.memory[513] = 35;  //23 in Hex*/

    //Our memory now contains [A1, 23]. This is the opcode A123.
    //This is the opcode for "load index with address 0x123" (0xANNN, where NNN is the address)

    //Emulate a CPU cycle
    chip8.emulate_cycle();
}
