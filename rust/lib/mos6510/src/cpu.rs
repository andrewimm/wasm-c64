pub struct CPU {
  pub acc: u8, // accumulator
  pub x: u8,
  pub y: u8,
  pub status: u8, // status register
  pub pc: u16, // program counter
  pub stack: u8, // stack pointer
}

pub enum Register {
  Acc,
  X,
  Y,
  Status,
  Stack,
}

impl CPU {
  pub fn new() -> CPU {
    CPU {
      acc: 0,
      x: 0,
      y: 0,
      status: 0,
      pc: 0,
      stack: 0,
    }
  }
}