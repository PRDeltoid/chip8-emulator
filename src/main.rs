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
use std::io::prelude::*;
use std::fs::File;

const FIRST_NIBBLE_MASK: u16 = 0xF000;  //Grabs first nibble only
const SECOND_NIBBLE_MASK: u16 = 0x0F00; //Grabs second nibble only
const THIRD_NIBBLE_MASK: u16 = 0x00F0;
const FOURTH_NIBBLE_MASK: u16 = 0x000F;

const LAST_TWO_MASK: u16 = 0x00FF;      //Grabs the last two nibbles
const LAST_THREE_MASK: u16 = 0x0FFF;    //Grabs last three nibbles only

pub struct Chip8 {
    _opcode: u16,        //Opcode
    memory: [u8; 4096], //General purpose memory
    v: [u8; 16],        //General purpose registers. Register 16 is the "carry flag"
    vf: u8,             //Carry flag

    i: u16,             //Index register
    pc: u16,            //Program counter (instruction pointer)

    _screen: [u8; 64 * 32], //Array for storing screen pixels. Screen is 64 x 32 pixels

    delay_timer: u8,    //Counts down at 60Hz speed to zero
    sound_timer: u8,    //Same as above, system buzzer sounds when it reaches zero

    stack: [u16; 16],   //Stack for program execution. Use to return to calling program after called program is finished
    sp: u16,            //Stack pointer, to keep track of what is currently the "top"

    _key: [u8; 16],     //Hex based keypad
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            _opcode: 0,         //Blank opcode
            memory: [0; 4096], //Initialize our memory
            v: [0; 16],        //Zero out our registers
            vf: 0,
            i: 0,
            pc: 512,           //program counter starts at 0x200 (system data comes before)
            _screen: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            _key: [0; 16],
        }
    }

    pub fn initialize(&mut self) {
    }

    pub fn load_rom(&mut self, rom_path: &str) {
        let rom = File::open(rom_path).unwrap();
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
            //0x1NNN opcode (jmp nnn)
            0x1000 => {
                self.pc = opcode & LAST_THREE_MASK;
                println!("Jumping to {:}d", self.pc);
            },
            //0x2NNN opcode (call subroutine: push pc to stack, jmp nnn)
            0x2000 => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = opcode & LAST_THREE_MASK;
                println!("Jumping to {:}d", self.pc);
            },
            //0x3XKK opcode (Skp next instruction if Vx == kk)
            0x3000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                if self.v[x] == kk {
                    //Skip next instruction by adding 2 to the program counter (skipping 2 bytes or 1 opcode)
                    self.pc += 2;
                }
            },
            //0x4XKK opcode (Skp next instruction if Vx != kk)
            0x4000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                if self.v[x] != kk {
                    //Skip next instruction by adding 2 to the program counter (skipping 2 bytes or 1 opcode)
                    self.pc += 2;
                }
            },
            //0x5XY0 (Skp next instruction if Vx == Vy)
            0x5000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let y = (opcode & THIRD_NIBBLE_MASK) as usize;
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            },
            //0x6XKK (Load Vx with kk)
            0x6000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                self.v[x] = kk;
            },
            //0x7XKK (Add Vx, kk)
            0x7000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let kk = (opcode & LAST_TWO_MASK) as u8;
                self.v[x] += kk;
            },
            //0x8XYN (Vx/Vy operations)
            0x8000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let y = (opcode & THIRD_NIBBLE_MASK) as usize;
                match opcode & FOURTH_NIBBLE_MASK  {
                    //0x8XY0 (MOV v[x], v[y])
                    0x0000 => {
                        self.v[x] = self.v[y];
                    },
                    //0x8XY1 (OR v[x], v[y])
                    0x0001 => {
                        self.v[x] = self.v[x] | self.v[y];
                    },
                    //0x8XY2 (AND v[x], v[y])
                    0x0002 => {
                        self.v[x] = self.v[x] & self.v[y];
                    },
                    //0x8XY3 (XOR v[x], v[y])
                    0x0003 => {
                        self.v[x] = self.v[x] ^ self.v[y];
                    },
                    //0x8XY4 (ADD v[x], v[y])
                    0x0004 => {
                        //Set carry if addition goes over 8 bits
                        if (self.v[x] + self.v[y]) >= 255  {
                            self.vf = 1;
                        } else {
                            self.vf = 0;
                        }
                        self.v[x] = ((self.v[x] + self.v[y]) & LAST_TWO_MASK as u8) as u8; //only store lowest 8 bits, no matter what
                    },
                    //0x8XY5 (SUB v[x], v[y])
                    0x0005 => {
                        if self.v[x] > self.v[y] {
                            self.vf = 1;
                        } else {
                            self.vf = 0;
                        }
                        self.v[x] = self.v[x] - self.v[y];
                    },
                    //0x8XY6 (SHR v[x], 1)
                    0x0006 => {
                        self.v[x] = self.v[x] >> 1;
                        //Need to set VF if least-significant bit is 1
                    },
                    //0x8XY7 (SUBN v[x], v[y])
                    0x0007 => {
                        if self.v[y] > self.v[x] {
                            self.vf = 1;
                        } else {
                            self.vf = 0;
                        }
                        self.v[x] = self.v[x] - self.v[y];
                    },
                    //0x8XY6 (SHL v[x], 1)
                    0x000E => {
                        self.v[x] = self.v[x] << 1;
                        //Need to set VF if most-significant bit is 1.
                    },
                    _ => {},
                }
            },
            //0x9XY0 (Skip next instruction if Vx != Vy
            0x9000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                let y = (opcode & THIRD_NIBBLE_MASK) as usize;
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            },
            //0xANNN opcode (mv i, NNN)
            0xA000 => {
                self.i = opcode & LAST_THREE_MASK;
                self.pc += 2;
                println!("Changing index to {:}d", self.i)
            },
            //0xBNNN opcode (jmp NNN + V0)
            0xB000 => {
                self.pc = (opcode & LAST_THREE_MASK) + self.v[0] as u16;

            }
            //0xFXNN opcodes
            0xF000 => {
                let x = (opcode & SECOND_NIBBLE_MASK) as usize;
                match opcode & LAST_TWO_MASK  {
                    //0xFX07 (mv v[x], delay_timer)
                    0x0007 => {
                        self.v[x] = self.delay_timer;
                    },
                    //0xFX15 (mov delay_timer, v[x])
                    0x0015 => {
                        self.delay_timer  = self.v[x];
                    },
                    //0xFX18 (mov sound_timer, v[x])
                    0x0018 => {
                        self.sound_timer = self.v[x];
                    },
                    //0xFX1E (add i, v[x])
                    0x001E => {
                        self.i += self.v[x] as u16;
                    },
                    _ => {},
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
