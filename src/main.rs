/************
CHIP8 Memory Map:
0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
0x200-0xFFF - Program ROM and work RAM

V Regs are 1 byte long (u8)
Opcodes are 2 bytes long (u16)
************/


fn main() {
    let opcode: u16;        //Opcode
    let memory: [u8; 4096]; //General purpose memory
    let V: [u8; 16];        //General purpose registers. Register 16 is the "carry flag"

    let I: u16;              //Index register
    let pc: u16;            //Program counter (instruction pointer)

    let screen: [u8; 64 * 32]; //Array for storing screen pixels. Screen is 64 x 32 pixels

    let delay_timer: u8;    //Counts down at 60Hz speed to zero
    let sound_timer: u8;    //Same as above, system buzzer sounds when it reaches zero

    let stack: [u16; 16];   //Stack for program execution. Use to return to calling program after called program is finished
    let sp: u16;            //Stack pointer, to keep track of what is currently the "top"

    let key: [u8; 16];      //Hex based keypad

}


fn initialize() {
    //Initialize registers and memory
}

fn emulate_cycle() {
    //Fetch opcode
    //Decode it
    //Execute it

    //Update timer(s)
}