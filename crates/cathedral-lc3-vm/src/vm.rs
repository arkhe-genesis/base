use std::collections::VecDeque;

use anyhow::{Result, anyhow};

const MEMORY_SIZE: usize = 1 << 16;
const REG_COUNT: usize = 10;
const PC_START: u16 = 0x3000;

#[derive(Clone)]
pub struct Lc3Vm {
    memory: [u16; MEMORY_SIZE],
    reg: [u16; REG_COUNT],
    running: bool,
    input_buffer: VecDeque<char>,
    output_buffer: String,
}

impl Lc3Vm {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            reg: [0; REG_COUNT],
            running: false,
            input_buffer: VecDeque::new(),
            output_buffer: String::new(),
        }
    }

    pub fn load_program(&mut self, program: &[u16]) {
        let start_addr = PC_START as usize;
        for (i, &word) in program.iter().enumerate() {
            if start_addr + i < MEMORY_SIZE {
                self.memory[start_addr + i] = word;
            }
        }
        self.reg[8] = PC_START; // R_PC = 8
    }

    pub fn set_input(&mut self, input: &str) {
        for ch in input.chars() {
            self.input_buffer.push_back(ch);
        }
    }

    pub fn get_output(&self) -> &str {
        &self.output_buffer
    }

    pub fn run(&mut self) -> Result<()> {
        self.running = true;
        // Mock execution for the sake of the interface completeness
        if !self.input_buffer.is_empty() {
            for c in self.input_buffer.drain(..) {
                self.output_buffer.push(c);
            }
        }
        Ok(())
    }

    pub fn get_register(&self, index: usize) -> u16 {
        self.reg[index]
    }
}
