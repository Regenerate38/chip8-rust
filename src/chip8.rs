#![allow(non_snake_case)]

const memory_size: usize = 4096;
const num_reg: usize = 16;
const stack_size: usize = 16;
const screen_width: usize = 64;
const screen_height: usize = 64;
const num_keys: usize = 16;

const fontset: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

pub struct Chip8 {
    memory: [u8; memory_size],

    stack: [u16; stack_size],
    sp: u16,
    // VF is carry flag in addition, no borrow flag in subtraction and sets in pixel collision in draw mode.
    v: [u8; num_reg], //Registers
    ir: u16,          //Instruction register
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    opcode: u16,
    graphics: [bool; screen_width * screen_height],
    keys: [bool; num_keys],
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            sp: 0,
            ir: 0,
            memory: [0; memory_size],
            stack: [0; stack_size],
            v: [0; num_reg],
            keys: [false; num_keys],
            graphics: [false; screen_width * screen_height],
            delay_timer: 0,
            sound_timer: 0,
            opcode: 0,
            pc: 0x200,
        }
    }

    fn loadProgram() {}

    fn executeCycle() {
        //fetch opcode

        //decode and execute opcode

        //update timers
    }
}

impl Chip8 {
    fn decodeOperand() {}
}
