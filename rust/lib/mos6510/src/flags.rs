use cpu::CPU;

pub const FLAG_OFFSET_C: u8 = 0;
pub const FLAG_OFFSET_Z: u8 = 1;
pub const FLAG_OFFSET_I: u8 = 2;
pub const FLAG_OFFSET_D: u8 = 3;
pub const FLAG_OFFSET_B: u8 = 4;
// 5th bit is only set when pushing status from BRK / PHP
pub const FLAG_OFFSET_V: u8 = 6;
pub const FLAG_OFFSET_N: u8 = 7;

pub const FLAG_CARRY: u8 = 1 << FLAG_OFFSET_C; // Carry
pub const FLAG_ZERO: u8 = 1 << FLAG_OFFSET_Z; // Zero
pub const FLAG_INTERRUPT_DISABLE: u8 = 1 << FLAG_OFFSET_I; // Interrupt Disable
pub const FLAG_DECIMAL: u8 = 1 << FLAG_OFFSET_D; // Decimal Mode
pub const FLAG_BRK: u8 = 1 << FLAG_OFFSET_B; // Break
pub const FLAG_OVERFLOW: u8 = 1 << FLAG_OFFSET_V; // Overflow
pub const FLAG_NEGATIVE: u8 = 1 << FLAG_OFFSET_N; // Negative

impl CPU {
  pub fn test_flag_zero(&mut self, value: u8) {
    let mut status = self.status;
    if value == 0 {
      status = status | FLAG_ZERO;
    } else {
      status = status & !FLAG_ZERO;
    }
    self.status = status;
  }

  pub fn test_flag_negative(&mut self, value: u8) {
    let status = self.status;
    if value & (1 << 7) > 0 {
      self.status = status | FLAG_NEGATIVE;
    } else {
      self.status = status & !FLAG_NEGATIVE;
    }
  }

  pub fn set_flag_carry(&mut self, carry: bool) {
    let status = self.status;
    if carry {
      self.status = status | FLAG_CARRY;
    } else {
      self.status = status & !FLAG_CARRY;
    }
  }

  pub fn set_flag_overflow(&mut self, overflow: bool) {
    let status = self.status;
    if overflow {
      self.status = status | FLAG_OVERFLOW;
    } else {
      self.status = status & !FLAG_OVERFLOW;
    }
  }
}