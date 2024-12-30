#![allow(non_snake_case)]

use rand::Rng;

const memory_size: usize = 4096;
const num_reg: usize = 16;
const stack_size: usize = 16;
const screen_width: usize = 64;
const screen_height: usize = 64;
const num_keys: usize = 16;
const fontset_size: usize = 80;

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
        let mut new_chip = Self {
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
        };
        new_chip.memory[..fontset_size].copy_from_slice(&fontset);

        new_chip
    }
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[(self.sp + 1) as usize]
    }

    pub fn loadProgram(&mut self, data: &[u8]) {
        let start = 0x200 as usize;
        let end = (0x200 as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.graphics
    }
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    fn executeCycle(&mut self) {
        //fetch opcode
        let higher_byte = self.memory[self.pc as usize] as u16;
        let lower_byte = self.memory[(self.pc + 1) as usize] as u16;
        self.opcode = higher_byte << 8 | lower_byte;

        //decode and execute opcode
        let d1 = (self.opcode & 0xF000) >> 12;
        let d2 = (self.opcode & 0x0F00) >> 8;
        let d3 = (self.opcode & 0x00F0) >> 4;
        let d4 = self.opcode & 0x000F;
        match (d1, d2, d3, d4) {
            // NOP
            (0, 0, 0, 0) => {}
            // CLEAR
            (0, 0, 0xE, 0) => self.graphics = [false; screen_width * screen_height],
            // RET
            (0, 0, 0xE, 0xE) => {
                let return_addr = self.pop();
                self.pc = return_addr;
            }
            // JMP
            (1, _, _, _) => {
                let nnn = self.opcode & 0xFFF;
                self.pc = nnn;
            }
            // CALL
            (2, _, _, _) => {
                let nnn = self.opcode & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // SKIP if VX = NN
            (3, _, _, _) => {
                let nn = (self.opcode & 0xFF) as u8;
                let x = d2 as usize;
                if (self.v[x] == nn) {
                    self.pc += 2;
                }
            }
            // SKIP if VX =/= NN
            (4, _, _, _) => {
                let nn = (self.opcode & 0xFF) as u8;
                let x = d2 as usize;
                if (self.v[x] != nn) {
                    self.pc += 2;
                }
            }
            // SKIP if VX == VY
            (5, _, _, 0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                if (self.v[x] == self.v[y]) {
                    self.pc += 2;
                }
            }
            // SET VX = NN
            (6, _, _, _) => {
                let x = d2 as usize;
                let nn = (self.opcode & 0xFF) as u8;
                self.v[x] = nn;
            }
            // INC VX by NN
            (7, _, _, _) => {
                let x = d2 as usize;
                let nn = (self.opcode & 0xFF) as u8;
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            // SET VX = VY
            (8, _, _, 0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v[x] = self.v[y];
            }
            // SET VX = VX | VY
            (8, _, _, 1) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v[x] = self.v[x] | self.v[y];
            }
            // SET VX = VX & VY
            (8, _, _, 2) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v[x] = self.v[x] & self.v[y];
            }
            // SET VX = VX XOR VY
            (8, _, _, 3) => {
                let x = d2 as usize;
                let y = d3 as usize;
                self.v[x] = self.v[x] ^ self.v[y];
            }
            // SET VX = VX + VY
            (8, _, _, 4) => {
                let x = d2 as usize;
                let y = d3 as usize;
                let (vx, carry) = self.v[x].overflowing_add(self.v[y]);
                let vf = if carry { 1 } else { 0 };
                self.v[x] = vx;
                self.v[15] = vf;
            }
            // SET VX = VX - VY
            (8, _, _, 5) => {
                let x = d2 as usize;
                let y = d3 as usize;
                let (vx, borrow) = self.v[x].overflowing_sub(self.v[y]);
                let vf = if borrow { 0 } else { 1 };
                self.v[x] = vx;
                self.v[15] = vf;
            }
            // SET VX = VX >>=1
            (8, _, _, 6) => {
                let x = d2 as usize;
                let lsb = self.v[x] & 1;
                self.v[x] = self.v[x] >> 1;
                self.v[15] = lsb;
            }
            // SET VX = VY - VX
            (8, _, _, 7) => {
                let x = d2 as usize;
                let y = d3 as usize;
                let (vx, borrow) = self.v[y].overflowing_sub(self.v[x]);
                let vf = if borrow { 0 } else { 1 };
                self.v[x] = vx;
                self.v[15] = vf;
            }
            // SET VX = VX <<=1
            (8, _, _, 0xE) => {
                let x = d2 as usize;
                let lsb = (self.v[x] >> 7) & 1;
                self.v[x] = self.v[x] << 1;
                self.v[15] = lsb;
            }
            // SKIP if VX != VY
            (9, _, _, 0x0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                if (self.v[x] != self.v[y]) {
                    self.pc += 2;
                }
            }
            // SETS IR = NNN
            (0xA, _, _, _) => {
                let nnn = self.opcode & 0xFFF;
                self.ir = nnn;
            }
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = self.opcode & 0xFFF;
                self.pc = self.v[0] as u16 + nnn;
            }
            // SET VX = rand & nn
            (0xC, _, _, _) => {
                let x = d2 as usize;
                let nn = (self.opcode & 0xFF) as usize;
                let rand_num = rand::thread_rng().gen_range(0..=255);
                self.v[x] = (rand_num & nn) as u8;
            }
            // SKIP if key[VX] is pressed
            (0xE, _, 0x9, 0xE) => {
                let x = d2 as usize;
                if (self.keys[self.v[x] as usize] == true) {
                    self.pc += 2
                }
            }
            // SKIP if key[VX] isn't pressed
            (0xE, _, 0xA, 0x1) => {
                let x = d2 as usize;
                if (self.keys[self.v[x] as usize] == false) {
                    self.pc += 2
                }
            }

            _ => {}
        }

        //update timers
        if self.delay_timer > 1 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 1 {
            self.sound_timer -= 1;
        }
    }
}
