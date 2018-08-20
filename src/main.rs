/************
CHIP8 Memory Map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM

V Regs are 1 byte long (u8)
Opcodes are 2 bytes long (u16). This means we must combine 2 1-byte numbers in memory into a single 2-byte number
This is done by rotating the leading number (the big end) by 8 bits. This will create a 2-byte number with 1 byte of zeros at the little end
We then bitwise OR our 2 byte number and our 1-byte number that we want to combine. All of the 1s in the 1-byte number are kept in the final result
************/

pub struct Chip8 {
    opcode: u16,        //Opcode
    memory: [u8; 4096], //General purpose memory
    v: [u8; 16],        //General purpose registers. Register 16 is the "carry flag"

    i: u16,             //Index register
    pc: usize,            //Program counter (instruction pointer)

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
            opcode: 0,
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0,
            screen: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
        }
    }
}

fn main() {
    let mut chip8= Chip8::new();

    chip8.memory[0] = 161; //A1 in Hex
    chip8.memory[1] = 35;  //23 in Hex

    emulate_cycle(&mut chip8);

}


fn _initialize(_chip8: &mut Chip8) {
    //Initialize the registers and counters on the chip
}

fn emulate_cycle(chip8: &mut Chip8) {
    //Fetch opcode

    //Grab the first half of the opcode as byte
    let mut opcode8: u8 = chip8.memory[chip8.pc];
    //Cast byte as 2 bytes
    let mut opcode = opcode8 as u16;
    //Shift the first half by 8 bits, so first byte is now is the "big end" of a 2 byte number
    opcode = opcode << 8;
    println!("Shifted opcode is {:}", opcode);
    //Grab next byte
    opcode8 = chip8.memory[chip8.pc+1];
    //Convert to 2-byte number (again)
    let mut opcode2: u16 = opcode8 as u16;

    //OR the two two-byte (one "big end" and one "small end") to combine them
    opcode = opcode | opcode2;
    println!("Final opcode is {:}", opcode);

    //Increase program counter by 2 (since we just consumed two bytes)
    chip8.pc = chip8.pc + 2

    //Decode it
    //Execute it

    //Update timer(s)
}