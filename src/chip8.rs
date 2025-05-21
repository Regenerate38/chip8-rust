#![allow(non_snake_case)]

use rand::Rng;

const MEMORY_SIZE: usize = 4096;
const INSTRUCTION_LENGTH: u16 = 2;
const NUM_REG: usize = 16;
const STACK_SIZE: usize = 128;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 64;
const NUM_KEYS: usize = 16;
const PROGRAM_LOAD_ADDRESS: usize = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; 80] = [
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
    memory: [u8; MEMORY_SIZE],

    stack: [u16; STACK_SIZE],
    sp: u16,
    // VF is carry flag in addition, no borrow flag in subtraction and sets in pixel collision in draw mode.
    v: [u8; NUM_REG], //Registers
    ir: u16,          //Instruction register
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    opcode: u16,
    graphics: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keys: [bool; NUM_KEYS],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut new_chip = Self {
            sp: 0,
            ir: 0,
            memory: [0; MEMORY_SIZE],
            stack: [0; STACK_SIZE],
            v: [0; NUM_REG],
            keys: [false; NUM_KEYS],
            graphics: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            delay_timer: 0,
            sound_timer: 0,
            opcode: 0,
            pc: PROGRAM_LOAD_ADDRESS as u16,
        };
        new_chip.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_chip
    }
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn loadProgram(&mut self, data: &[u8]) -> Result<usize, String> {
        if data.len() > MEMORY_SIZE - PROGRAM_LOAD_ADDRESS {
            return Err("ROM too big, aborting".to_string());
        }
        let start = 0x200 as usize;
        let end = (0x200 as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
        Ok(data.len())
    }

    pub fn get_display(&self) -> &[bool] {
        &self.graphics
    }
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
        println!(
            "Key at index {} is {}",
            idx,
            if pressed { "pressed" } else { "released" }
        );
    }

    pub fn is_pixel_set(&self, x: usize, y: usize) -> bool {
        self.graphics[x + SCREEN_HEIGHT * y]
    }
    pub fn executeCycle(&mut self) {
        //fetch opcode
        let higher_byte = self.memory[self.pc as usize] as u16;
        let lower_byte = self.memory[(self.pc + 1) as usize] as u16;
        self.opcode = higher_byte << 8 | lower_byte;
        self.pc += INSTRUCTION_LENGTH;

        //decode and execute opcode
        let d1 = (self.opcode & 0xF000) >> 12;
        let d2 = (self.opcode & 0x0F00) >> 8;
        let d3 = (self.opcode & 0x00F0) >> 4;
        let d4 = self.opcode & 0x000F;
        match (d1, d2, d3, d4) {
            // NOP
            (0, 0, 0, 0) => {}
            // CLEAR
            (0, 0, 0xE, 0) => self.graphics = [false; SCREEN_WIDTH * SCREEN_HEIGHT],
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
                    self.pc += INSTRUCTION_LENGTH;
                }
            }
            // SKIP if VX =/= NN
            (4, _, _, _) => {
                let nn = (self.opcode & 0xFF) as u8;
                let x = d2 as usize;
                if (self.v[x] != nn) {
                    self.pc += INSTRUCTION_LENGTH;
                }
            }
            // SKIP if VX == VY
            (5, _, _, 0) => {
                let x = d2 as usize;
                let y = d3 as usize;
                if (self.v[x] == self.v[y]) {
                    self.pc += INSTRUCTION_LENGTH;
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
                    self.pc += INSTRUCTION_LENGTH;
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
                    self.pc += INSTRUCTION_LENGTH
                }
            }
            // SKIP if key[VX] isn't pressed
            (0xE, _, 0xA, 0x1) => {
                let x = d2 as usize;
                if (self.keys[self.v[x] as usize] == false) {
                    self.pc += INSTRUCTION_LENGTH
                }
            }
            // DRAW. this was hard
            (0xD, _, _, _) => {
                let x = self.v[d2 as usize] as u16;
                let y = self.v[d3 as usize] as u16;
                let n = d4 as usize;

                let mut flipped = false;

                for line_no in 0..n {
                    let address = self.ir + line_no as u16;
                    let pixels = self.memory[address as usize];

                    for cell in 0..8 {
                        if (pixels & (0b1000_0000 >> cell)) != 0 {
                            let x_draw = (x + cell) as usize % SCREEN_WIDTH;
                            let y_draw = (y + line_no as u16) as usize % SCREEN_HEIGHT;
                            let index = x_draw + SCREEN_HEIGHT * y_draw;
                            flipped = flipped | self.get_display()[index];
                            self.graphics[index] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }
            }
            // SET VX = DT
            (0xF, _, 0x0, 0x7) => {
                let x = d2 as usize;
                self.v[x] = self.delay_timer;
            }
            // WAIT KEY
            (0xF, _, 0x0, 0xA) => {
                let x = d2 as usize;
                let mut key_pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v[x] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }
                if !key_pressed {
                    self.pc -= INSTRUCTION_LENGTH;
                    println!("Waiting for key input...");
                }
            }
            // SET DT = VX
            (0xf, _, 0x1, 0x5) => {
                let x = d2 as usize;
                self.delay_timer = self.v[x];
            }
            // SET ST = VX
            (0xf, _, 0x1, 0x8) => {
                let x = d2 as usize;
                self.sound_timer = self.v[x];
            }
            // ADD IR, VX
            (0xf, _, 0x1, 0xe) => {
                let x = d2 as usize;
                self.ir = self.ir.wrapping_add(self.v[x] as u16);
            }
            // MOV IR, SPRITE[VX]. USING FONT SPRITES HERE
            (0xf, _, 0x2, 0x9) => {
                let x = d2 as usize;
                self.ir = self.v[x] as u16 * 5;
            }
            // MOV RAM, VX
            (0xf, _, 0x5, 0x5) => {
                let x = d2 as usize;
                // for i in (self.ir as usize)..(self.ir as usize + x) {
                //     self.memory[i] = self.v[i]
                // }
                for i in 0..x {
                    self.memory[(self.ir as usize) + i] = self.v[i];
                }
            }
            // MOV VX, RAM
            (0xf, _, 0x6, 0x5) => {
                let x = d2 as usize;
                // for i in (self.ir as usize)..(self.ir as usize + x) {
                //     self.v[i] = self.memory[i]
                // }
                for i in 0..x {
                    self.v[i] = self.memory[(self.ir as usize) + i];
                }
            }
            // BCD STUFF
            (0xF, _, 0x3, 0x3) => {
                let x = d2 as usize;
                let vx = self.v[x] as f32;

                let ones = (vx % 10.0) as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let hundreds = ((vx / 100.0) % 10.0).floor() as u8;

                self.memory[self.ir as usize] = hundreds;
                self.memory[(self.ir + 1) as usize] = tens;
                self.memory[(self.ir + 2) as usize] = ones;
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
