use vm::memmap::MemMap;

pub struct CPU {
  acc: u8, // accumulator
  x: u8,
  y: u8,
  status: u8, // status register
  pc: u16, // program counter
  stack: u8, // stack pointer
}

// flag offsets
const FLAG_OFFSET_C: u8 = 0;
const FLAG_OFFSET_Z: u8 = 1;
const FLAG_OFFSET_I: u8 = 2;
const FLAG_OFFSET_D: u8 = 3;
const FLAG_OFFSET_B: u8 = 4;
const FLAG_OFFSET_V: u8 = 6;
const FLAG_OFFSET_N: u8 = 7;

const FLAG_C: u8 = 1 << FLAG_OFFSET_C; // Carry
const FLAG_Z: u8 = 1 << FLAG_OFFSET_Z; // Zero
const FLAG_I: u8 = 1 << FLAG_OFFSET_I; // Interrupt Disable
const FLAG_D: u8 = 1 << FLAG_OFFSET_D; // Decimal Mode
const FLAG_B: u8 = 1 << FLAG_OFFSET_B; // Break
const FLAG_V: u8 = 1 << FLAG_OFFSET_V; // Overflow
const FLAG_N: u8 = 1 << FLAG_OFFSET_N; // Negative

pub enum Register {
  Acc,
  X,
  Y,
  Status,
  Stack,
}

pub fn create_cpu() -> CPU {
  CPU {
    acc: 0,
    x: 0,
    y: 0,
    status: 0,
    pc: 0,
    stack: 0,
  }
}

impl CPU {
  pub fn reset(&mut self, mem: MemMap) {
    self.acc = 0;
    self.x = 0;
    self.y = 0;
    self.status = 0;
    self.stack = 0xfd;

    let pc_low = mem.get_byte(0xfffc) as u16;
    let pc_high = mem.get_byte(0xfffd) as u16;
    self.pc = (pc_high << 8) | pc_low;
  }

  pub fn set_pc(&mut self, pc: u16) {
    self.pc = pc;
  }

  pub fn get_register(&self, reg: Register) -> u8 {
    match reg {
      Register::Acc => self.acc,
      Register::X => self.x,
      Register::Y => self.y,
      Register::Status => self.status,
      Register::Stack => self.stack,
    }
  }

  pub fn test_flags_n_z(&mut self, value: u8) {
    let mut status = self.status;
    if value & (1 << 7) > 0 {
      status = status | FLAG_N;
    } else {
      status = status & !FLAG_N;
    }
    if value == 0 {
      status = status | FLAG_Z;
    } else {
      status = status & !FLAG_Z;
    }
    self.status = status;
  }

  #[inline]
  pub fn set_carry_flag(&mut self, set: bool) {
    let status = self.status;
    if set {
      self.status = status | FLAG_C;
    } else {
      self.status = status & !FLAG_C;
    }
  }

  #[inline]
  pub fn set_overflow_flag(&mut self, set: bool) {
    let status = self.status;
    if set {
      self.status = status | FLAG_V;
    } else {
      self.status = status & !FLAG_V;
    }
  }

  pub fn jump_pc(&mut self, offset: u8) {
    let start = self.pc;
    if offset & 0x80 == 0 {
      self.pc = start.wrapping_add(offset as u16);
    } else {
      let abs = !offset + 1;
      self.pc = start.wrapping_sub(abs as u16);
    }
  }

  pub fn push(&mut self, mem: &mut MemMap, value: u8) {
    let addr: u16 = 0x100 + (self.stack as u16);
    mem.set_byte(addr, value);
    self.stack = self.stack.wrapping_sub(1);
  }

  pub fn pop(&mut self, mem: &mut MemMap) -> u8 {
    self.stack = self.stack.wrapping_add(1);
    let addr: u16 = 0x100 + (self.stack as u16);
    return mem.get_byte(addr);
  }

  #[inline]
  pub fn get_addr_absolute(&self, mem: &MemMap) -> u16 {
    let low = mem.get_byte(self.pc + 1) as u16;
    let high = mem.get_byte(self.pc + 2) as u16;
    low | (high << 8)
  }

  #[inline]
  pub fn get_addr_absolute_x(&self, mem: &MemMap) -> u16 {
    let low = mem.get_byte(self.pc + 1) as u16;
    let high = mem.get_byte(self.pc + 2) as u16;
    (low | (high << 8)) + (self.x as u16)
  }

  #[inline]
  pub fn get_addr_absolute_y(&self, mem: &MemMap) -> u16 {
    let low = mem.get_byte(self.pc + 1) as u16;
    let high = mem.get_byte(self.pc + 2) as u16;
    (low | (high << 8)) + (self.y as u16)
  }

  #[inline]
  pub fn get_addr_zeropage(&self, mem: &MemMap) -> u16 {
    mem.get_byte(self.pc + 1) as u16
  }

  #[inline]
  pub fn get_addr_zeropage_y(&self, mem: &MemMap) -> u16 {
    (mem.get_byte(self.pc + 1) as u16).wrapping_add(self.y as u16)
  }

  #[inline]
  pub fn get_addr_zeropage_x(&self, mem: &MemMap) -> u16 {
    (mem.get_byte(self.pc + 1) as u16).wrapping_add(self.x as u16)
  }

  #[inline]
  pub fn get_addr_indexed_indirect(&self, mem: &MemMap) -> u16 {
    let src = (mem.get_byte(self.pc + 1) as u16).wrapping_add(self.x as u16);
    mem.get_byte(src) as u16
  }

  #[inline]
  pub fn get_addr_indirect_indexed(&self, mem: &MemMap) -> u16 {
    let src = mem.get_byte(self.pc + 1) as u16;
    (mem.get_byte(src) as u16).wrapping_add(self.y as u16)
  }

  #[inline]
  pub fn compare(&mut self, a: u8, b: u8) {
    let value = a.wrapping_sub(b);
    self.test_flags_n_z(value);
    let status = self.status;
    if b > a {
      self.status = status | FLAG_C;
    } else {
      self.status = status & !FLAG_C;
    }
  }

  pub fn adc(&mut self, orig: u8, value: u8) {
    let (mut total, mut overflow) = orig.overflowing_add(value);
    let mut v =
      (orig & 0b10000000 == value & 0b10000000) &&
      (orig & 0b10000000 != total & 0b10000000);
    if self.status & FLAG_C > 0 {
      let (carry_total, carry_overflow) = total.overflowing_add(1);
      v = v || (total & 0b10000000 == 0 && carry_total & 0b10000000 > 0);
      total = carry_total;
      overflow = overflow || carry_overflow;
    }
    self.test_flags_n_z(total);
    self.set_carry_flag(overflow);
    self.set_overflow_flag(v);
    self.acc = total;
  }

  pub fn step(&mut self, mem: &mut MemMap) -> u8 {
    let index = self.pc;
    let (byte_len, cycles) = match mem.get_byte(index) {
      0x00 => { // BRK
        let next_pc = index.wrapping_add(2);
        let cur_status = self.status;
        self.pc = next_pc;
        self.push(mem, (next_pc >> 8) as u8);
        self.push(mem, (next_pc & 0xff) as u8);
        self.push(mem, cur_status);
        self.status = cur_status | FLAG_B | FLAG_I;
        (0, 7)
      },

      0x01 => { // ORA (nn,X)
        let orig = self.acc;
        let addr = self.get_addr_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (2, 6)
      },

      0x02 => {
        (1, 1)
      },

      0x03 => {
        (1, 1)
      },

      0x04 => {
        (1, 1)
      },

      0x05 => { // ORA nn
        let orig = self.acc;
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (2, 3)
      },

      0x06 => { // ASL nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let carry = value & 0x80 == 0x80;
        let result = value << 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 5)
      },

      0x07 => {
        (1, 1)
      },

      0x08 => { // PHP
        let value = self.status;
        self.push(mem, value);
        (1, 3)
      },

      0x09 => { // ORA #nn
        let orig = self.acc;
        let value = mem.get_byte(self.pc + 1);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (2, 2)
      },

      0x0a => { // ASL A
        let value = self.acc;
        let carry = value & 0x80 == 0x80;
        let result = value << 1;
        self.acc = result;
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (1, 2)
      },

      0x0b => {
        (1, 1)
      },

      0x0c => {
        (1, 1)
      },

      0x0d => { // ORA nnnn
        let orig = self.acc;
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (3, 4)
      },

      0x0e => { // ASL nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let carry = value & 0x80 == 0x80;
        let result = value << 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 6)
      },

      0x0f => {
        (1, 1)
      },

      0x10 => { // BPL
        if self.status & FLAG_N == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0x11 => { // ORA (nn),Y
        let orig = self.acc;
        let addr = self.get_addr_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (2, 5)
      },

      0x12 => {
        (1, 1)
      },

      0x13 => {
        (1, 1)
      },

      0x14 => {
        (1, 1)
      },

      0x15 => { // ORA nn,X
        let orig = self.acc;
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (2, 4)
      },

      0x16 => { // ASL nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let carry = value & 0x80 == 0x80;
        let result = value << 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 6)
      },

      0x17 => {
        (1, 1)
      },

      0x18 => { // CLC
        let status = self.status & !FLAG_C;
        self.status = status;
        (1, 2)
      },

      0x19 => { // ORA nnnn,Y
        let orig = self.acc;
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (3, 4)
      },

      0x1a => {
        (1, 1)
      },

      0x1b => {
        (1, 1)
      },

      0x1c => {
        (1, 1)
      },

      0x1d => { // ORA nnnn,X
        let orig = self.acc;
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let result = orig | value;
        self.test_flags_n_z(result);
        self.acc = result;
        (3, 4)
      },

      0x1e => { // ASL nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let carry = value & 0x80 == 0x80;
        let result = value << 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 7)
      },

      0x1f => {
        (1, 1)
      },

      0x20 => { // JSR
        let ret = self.pc + 2;
        self.push(mem, (ret >> 8) as u8);
        self.push(mem, (ret & 0xff) as u8);
        let low = mem.get_byte(self.pc + 1) as u16;
        let high = mem.get_byte(self.pc + 2) as u16;
        self.pc = (high << 8) | low;
        (0, 6)
      },

      0x21 => { // AND (nn,X)
        let orig = self.acc;
        let addr = self.get_addr_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (2, 6)
      },

      0x22 => {
        (1, 1)
      },

      0x23 => {
        (1, 1)
      },

      0x24 => { // BIT nn
        let orig = self.acc;
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let mut status = self.status;
        if value & (1 << 7) > 0 {
          status = status | FLAG_N;
        } else {
          status = status & !FLAG_N;
        }
        if value & (1 << 6) > 0 {
          status = status | FLAG_V;
        } else {
          status = status & !FLAG_V;
        }
        if orig & value == 0 {
          status = status | FLAG_Z;
        } else {
          status = status & !FLAG_Z;
        }
        self.status = status;
        (2, 3)
      }

      0x25 => { // AND nn
        let orig = self.acc;
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (2, 3)
      },

      0x26 => { // ROL nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 0x80 == 0x80;
        let mut result = value << 1;
        if had_carry {
          result = result | 1;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 5)
      },

      0x27 => {
        (1, 1)
      },

      0x28 => { // PLP
        let status = self.pop(mem);
        self.status = status;
        (1, 4)
      },

      0x29 => { // AND #nn
        let orig = self.acc;
        let value = mem.get_byte(self.pc + 1);
        self.acc = orig & value;
        (2, 2)
      },

      0x2a => { // ROL A
        let value = self.acc;
        let had_carry = self.status & 1 == 1;
        let carry = value & 0x80 == 0x80;
        let mut result = value << 1;
        if had_carry {
          result = result | 1;
        }
        self.acc = result;
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (1, 2)
      },

      0x2b => {
        (1, 1)
      },

      0x2c => { // BIT nnnn
        let orig = self.acc;
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let mut status = self.status;
        if value & (1 << 7) > 0 {
          status = status | FLAG_N;
        } else {
          status = status & !FLAG_N;
        }
        if value & (1 << 6) > 0 {
          status = status | FLAG_V;
        } else {
          status = status & !FLAG_V;
        }
        if orig & value == 0 {
          status = status | FLAG_Z;
        } else {
          status = status & !FLAG_Z;
        }
        self.status = status;
        (3, 4)
      },

      0x2d => { // AND nnnn
        let orig = self.acc;
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (3, 4)
      },

      0x2e => { // ROL nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 0x80 == 0x80;
        let mut result = value << 1;
        if had_carry {
          result = result | 1;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 6)
      },

      0x2f => {
        (1, 1)
      },

      0x30 => { // BMI
        if self.status & FLAG_N > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0x31 => { // AND (nn),Y
        let orig = self.acc;
        let addr = self.get_addr_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (2, 5)
      },

      0x32 => {
        (1, 1)
      },

      0x33 => {
        (1, 1)
      },

      0x34 => {
        (1, 1)
      },

      0x35 => { // AND nn,X
        let orig = self.acc;
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (2, 4)
      },

      0x36 => { // ROL nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 0x80 == 0x80;
        let mut result = value << 1;
        if had_carry {
          result = result | 1;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 6)
      },

      0x37 => {
        (1, 1)
      },

      0x38 => { // SEC
        let mut status = self.status;
        status = status | FLAG_C;
        self.status = status;
        (1, 2)
      },

      0x39 => { // AND nnnn,Y
        let orig = self.acc;
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (3, 4)
      },

      0x3a => {
        (1, 1)
      },

      0x3b => {
        (1, 1)
      },

      0x3c => {
        (1, 1)
      },

      0x3d => { // AND nnnn,X
        let orig = self.acc;
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        self.acc = orig & value;
        (3, 4)
      },

      0x3e => { // ROL nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 0x80 == 0x80;
        let mut result = value << 1;
        if had_carry {
          result = result | 1;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 7)
      },

      0x3f => {
        (1, 1)
      },

      0x40 => { // RTI
        let status = self.pop(mem);
        let pc_low = self.pop(mem) as u16;
        let pc_high = self.pop(mem) as u16;
        self.status = status;
        self.pc = (pc_high << 8) | pc_low;
        (1, 6)
      },

      0x41 => { // EOR (nn,X)
        let orig = self.acc;
        let addr = self.get_addr_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (2, 6)
      },

      0x42 => {
        (1, 1)
      },

      0x43 => {
        (1, 1)
      },

      0x44 => {
        (1, 1)
      },

      0x45 => { // EOR nn
        let orig = self.acc;
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (2, 3)
      },

      0x46 => { // LSR nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let carry = value & 1 == 1;
        let result = value >> 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 5)
      },

      0x47 => {
        (1, 1)
      },

      0x48 => { // PHA
        let value = self.acc;
        self.push(mem, value);
        (1, 3)
      },

      0x49 => { // EOR #nn
        let orig = self.acc;
        let value = mem.get_byte(self.pc + 1);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (2, 2)
      },

      0x4a => { // LSR A
        let value = self.acc;
        let carry = value & 1 == 1;
        let result = value >> 1;
        self.acc = result;
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (1, 2)
      },

      0x4b => {
        (1, 1)
      },

      0x4c => { // JMP nnnn
        let dest_low = mem.get_byte(self.pc + 1) as u16;
        let dest_high = mem.get_byte(self.pc + 2) as u16;
        let dest = (dest_high << 8) | dest_low;
        self.pc = dest;
        (0, 3)
      },

      0x4d => { // EOR nnnn
        let orig = self.acc;
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (2, 4)
      },

      0x4e => { // LSR nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let carry = value & 1 == 1;
        let result = value >> 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 6)
      },

      0x4f => {
        (1, 1)
      },

      0x50 => { // BVC
        if self.status & FLAG_V == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0x51 => { // EOR (nn),Y
        let orig = self.acc;
        let addr = self.get_addr_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (2, 5)
      },

      0x52 => {
        (1, 1)
      },

      0x53 => {
        (1, 1)
      },

      0x54 => {
        (1, 1)
      },

      0x55 => { // EOR nn,X
        let orig = self.acc;
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (2, 4)
      },

      0x56 => { // LSR nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let carry = value & 1 == 1;
        let result = value >> 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 6)
      },

      0x57 => {
        (1, 1)
      },

      0x58 => { // CLI
        let status = self.status & !FLAG_I;
        self.status = status;
        (1, 2)
      },

      0x59 => { // EOR nnnn,Y
        let orig = self.acc;
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (3, 4)
      },

      0x5a => {
        (1, 1)
      },

      0x5b => {
        (1, 1)
      },

      0x5c => {
        (1, 1)
      },

      0x5d => { // EOR nnnn,X
        let orig = self.acc;
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let result = orig ^ value;
        self.acc = result;
        self.test_flags_n_z(result);
        (3, 4)
      },

      0x5e => { // LSR nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let carry = value & 1 == 1;
        let result = value >> 1;
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 7)
      },

      0x5f => {
        (1, 1)
      },

      0x60 => { // RTS
        let low = self.pop(mem) as u16;
        let high = self.pop(mem) as u16;
        let ret = (high << 8) | low;
        self.pc = ret;
        (1, 6)
      },

      0x61 => { // ADC (nn,X)
        let orig = self.acc;
        let addr = self.get_addr_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (2, 6)
      },

      0x62 => {
        (1, 1)
      },

      0x63 => {
        (1, 1)
      },

      0x64 => {
        (1, 1)
      },

      0x65 => { // ADC nn
        let orig = self.acc;
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (2, 3)
      },

      0x66 => { // ROR nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 1 == 1;
        let mut result = value >> 1;
        if had_carry {
          result = result | 0x80;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 5)
      },

      0x67 => {
        (1, 1)
      },

      0x68 => { // PLA
        let acc = self.pop(mem);
        self.acc = acc;
        self.test_flags_n_z(acc);
        (1, 4)
      },

      0x69 => { // ADC #nn
        let orig = self.acc;
        let value = mem.get_byte(self.pc + 1);
        self.adc(orig, value);
        (2, 2)
      },

      0x6a => { // ROR
        let value = self.acc;
        let had_carry = self.status & 1 == 1;
        let carry = value & 1 == 1;
        let mut result = value >> 1;
        if had_carry {
          result = result | 0x80;
        }
        self.acc = result;
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (1, 2)
      },

      0x6b => {
        (1, 1)
      },

      0x6c => { // JMP (nnnn)
        let source_low = mem.get_byte(self.pc + 1) as u16;
        let source_high = mem.get_byte(self.pc + 2) as u16;
        let source = (source_high << 8) | source_low;
        let dest_low = mem.get_byte(source) as u16;
        let dest_high = mem.get_byte(source + 1) as u16;
        let dest = (dest_high << 8) | dest_low;
        self.pc = dest;
        (0, 3)
      },

      0x6d => { // ADC nnnn
        let orig = self.acc;
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (3, 4)
      },

      0x6e => { // ROR nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 1 == 1;
        let mut result = value >> 1;
        if had_carry {
          result = result | 0x80;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 6)
      },

      0x6f => {
        (1, 1)
      },

      0x70 => { // BVS
        if self.status & FLAG_V > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0x71 => { // ADC (nn),Y
        let orig = self.acc;
        let addr = self.get_addr_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (2, 6)
      },

      0x72 => {
        (1, 1)
      },

      0x73 => {
        (1, 1)
      },

      0x74 => {
        (1, 1)
      },

      0x75 => { // ADC nn,X
        let orig = self.acc;
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (2, 3)
      },

      0x76 => { // ROR nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 1 == 1;
        let mut result = value >> 1;
        if had_carry {
          result = result | 0x80;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (2, 6)
      },

      0x77 => {
        (1, 1)
      },

      0x78 => { // SEI
        let mut status = self.status;
        status = status | FLAG_I;
        self.status = status;
        (1, 2)
      },

      0x79 => { // ADC nnnn,Y
        let orig = self.acc;
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (3, 4)
      },

      0x7a => {
        (1, 1)
      },

      0x7b => {
        (1, 1)
      },

      0x7c => {
        (1, 1)
      },

      0x7d => { // ADC nnnn,X
        let orig = self.acc;
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        self.adc(orig, value);
        (3, 4)
      },

      0x7e => { // ROR nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let had_carry = self.status & 1 == 1;
        let carry = value & 1 == 1;
        let mut result = value >> 1;
        if had_carry {
          result = result | 0x80;
        }
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        self.set_carry_flag(carry);
        (3, 7)
      },

      0x7f => {
        (1, 1)
      },

      0x80 => {
        (1, 1)
      },

      0x81 => { // STA (nn,X)
        let value = self.acc;
        let addr = self.get_addr_indexed_indirect(mem);
        mem.set_byte(addr, value);
        (2, 6)
      },

      0x82 => {
        (1, 1)
      },

      0x83 => {
        (1, 1)
      },

      0x84 => { // STY nn
        let value = self.y;
        let addr = self.get_addr_zeropage(mem);
        mem.set_byte(addr, value);
        (2, 3)
      },

      0x85 => { // STA nn
        let value = self.acc;
        let addr = self.get_addr_zeropage(mem);
        mem.set_byte(addr, value);
        (2, 3)
      },

      0x86 => { // STX nn
        let value = self.x;
        let addr = self.get_addr_zeropage(mem);
        mem.set_byte(addr, value);
        (2, 3)
      },

      0x87 => {
        (1, 1)
      },

      0x88 => { // DEY
        let result = self.y.wrapping_sub(1);
        self.y = result;
        self.test_flags_n_z(result);
        (1, 2)
      },

      0x89 => {
        (1, 1)
      },

      0x8a => { // TXA
        let value = self.x;
        self.acc = value;
        self.test_flags_n_z(value);
        (1, 2)
      },

      0x8b => {
        (1, 1)
      },

      0x8c => { // STY nnnn
        let value = self.y;
        let addr = self.get_addr_absolute(mem);
        mem.set_byte(addr, value);
        (3, 4)
      },

      0x8d => { // STA nnnn
        let value = self.acc;
        let addr = self.get_addr_absolute(mem);
        mem.set_byte(addr, value);
        (3, 4)
      },

      0x8e => { // STX nnnn
        let value = self.x;
        let addr = self.get_addr_absolute(mem);
        mem.set_byte(addr, value);
        (3, 4)
      },

      0x8f => {
        (1, 1)
      },

      0x90 => { // BCC
        if self.status & FLAG_C == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0x91 => { // STA (nn),Y
        let value = self.acc;
        let addr = self.get_addr_indirect_indexed(mem);
        mem.set_byte(addr, value);
        (2, 6)
      },

      0x92 => {
        (1, 1)
      },

      0x93 => {
        (1, 1)
      },

      0x94 => { // STY nn,X
        let value = self.y;
        let addr = self.get_addr_zeropage_x(mem);
        mem.set_byte(addr, value);
        (2, 4)
      },

      0x95 => { // STA nn,X
        let value = self.acc;
        let addr = self.get_addr_zeropage_x(mem);
        mem.set_byte(addr, value);
        (2, 4)
      },

      0x96 => { // STX nn,Y
        let value = self.x;
        let addr = self.get_addr_absolute_y(mem);
        mem.set_byte(addr, value);
        (2, 4)
      },

      0x97 => {
        (1, 1)
      },

      0x98 => { // TYA
        let value = self.y;
        self.acc = value;
        self.test_flags_n_z(value);
        (1, 2)
      },

      0x99 => { // STA nnnn,Y
        let value = self.acc;
        let addr = self.get_addr_absolute_y(mem);
        mem.set_byte(addr, value);
        (3, 5)
      },

      0x9a => { // TXS
        let value = self.x;
        self.stack = value;
        self.test_flags_n_z(value);
        (1, 2)
      },

      0x9b => {
        (1, 1)
      },

      0x9c => {
        (1, 1)
      },

      0x9d => { // STA nnnn,X
        let value = self.acc;
        let addr = self.get_addr_absolute_x(mem);
        mem.set_byte(addr, value);
        (3, 5)
      },

      0x9e => {
        (1, 1)
      },

      0x9f => {
        (1, 1)
      },

      0xa0 => { // LDY #nn
        let value = mem.get_byte(self.pc + 1);
        self.y = value;
        self.test_flags_n_z(value);
        (2, 2)
      },

      0xa1 => { // LDA (nn,X)
        let addr = self.get_addr_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (2, 6)
      },

      0xa2 => { // LDX #nn
        let value = mem.get_byte(self.pc + 1);
        self.x = value;
        self.test_flags_n_z(value);
        (2, 2)
      },

      0xa3 => {
        (1, 1)
      },

      0xa4 => { // LDY nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        self.y = value;
        self.test_flags_n_z(value);
        (2, 3)
      },

      0xa5 => { // LDA nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (2, 3)
      },

      0xa6 => { // LDX nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        self.x = value;
        self.test_flags_n_z(value);
        (2, 3)
      },

      0xa7 => {
        (1, 1)
      },

      0xa8 => { // TAY
        let acc = self.acc;
        self.y = self.acc;
        self.test_flags_n_z(acc);
        (1, 2)
      },

      0xa9 => { // LDA #nn
        let value = mem.get_byte(self.pc + 1);
        self.acc = value;
        self.test_flags_n_z(value);
        (2, 2)
      },

      0xaa => { // TAX
        let acc = self.acc;
        self.x = self.acc;
        self.test_flags_n_z(acc);
        (1, 2)
      },

      0xab => {
        (1, 1)
      },

      0xac => { // LDY nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        self.y = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xad => { // LDA nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xae => { // LDX nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        self.x = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xaf => {
        (1, 1)
      },

      0xb0 => { // BCS
        if self.status & FLAG_C > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0xb1 => { // LDA (nn),Y
        let addr = self.get_addr_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (2, 5)
      },

      0xb2 => {
        (1, 1)
      },

      0xb3 => {
        (1, 1)
      },

      0xb4 => { // LDY nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        self.y = value;
        self.test_flags_n_z(value);
        (2, 4)
      },

      0xb5 => { // LDA nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (2, 4)
      },

      0xb6 => { // LDX nn,Y
        let addr = self.get_addr_zeropage_y(mem);
        let value = mem.get_byte(addr);
        self.x = value;
        self.test_flags_n_z(value);
        (2, 4)
      },

      0xb7 => {
        (1, 1)
      },

      0xb8 => { // CLV
        let status = self.status & !FLAG_V;
        self.status = status;
        (1, 2)
      },

      0xb9 => { // LDA nnnn,Y
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xba => { // TSX
        let stack = self.stack;
        self.x = stack;
        self.test_flags_n_z(stack);
        (1, 2)
      },

      0xbb => {
        (1, 1)
      },

      0xbc => { // LDY nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        self.y = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xbd => { // LDA nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        self.acc = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xbe => { // LDX nnnn,Y
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        self.x = value;
        self.test_flags_n_z(value);
        (3, 4)
      },

      0xbf => {
        (1, 1)
      },

      0xc0 => { // CPY #nn
        let value = mem.get_byte(self.pc + 1);
        let y = self.y;
        self.compare(y, value);
        (2, 2)
      },

      0xc1 => { // CMP (nn,X)
        let addr = self.get_addr_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (2, 6)
      },

      0xc2 => {
        (1, 1)
      },

      0xc3 => {
        (1, 1)
      },

      0xc4 => { // CPY nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let y = self.y;
        self.compare(y, value);
        (2, 3)
      },

      0xc5 => { // CMP nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (2, 3)
      },

      0xc6 => { // DEC nn
        let addr = self.get_addr_zeropage(mem);
        let result = mem.get_byte(addr).wrapping_sub(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (2, 5)
      },

      0xc7 => {
        (1, 1)
      },

      0xc8 => { // INY
        let result = self.y.wrapping_add(1);
        self.y = result;
        self.test_flags_n_z(result);
        (1, 2)
      },

      0xc9 => { // CMP #nn
        let value = mem.get_byte(self.pc + 1);
        let acc = self.acc;
        self.compare(acc, value);
        (2, 2)
      },

      0xca => { // DEX
        let result = self.x.wrapping_sub(1);
        self.y = result;
        self.test_flags_n_z(result);
        (1, 2)
      },

      0xcb => {
        (1, 1)
      },

      0xcc => { // CPY nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let y = self.y;
        self.compare(y, value);
        (3, 4)
      },

      0xcd => { // CMP nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (3, 4)
      },

      0xce => { // DEC nnnn
        let addr = self.get_addr_absolute(mem);
        let result = mem.get_byte(addr).wrapping_sub(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (3, 6)
      },

      0xcf => {
        (1, 1)
      },

      0xd0 => { // BNE
        if self.status & FLAG_Z == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0xd1 => { // CMP (nn),Y
        let addr = self.get_addr_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (2, 5)
      },

      0xd2 => {
        (1, 1)
      },

      0xd3 => {
        (1, 1)
      },

      0xd4 => {
        (1, 1)
      },

      0xd5 => { // CMP nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (2, 4)
      },

      0xd6 => { // DEC nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let result = mem.get_byte(addr).wrapping_sub(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (2, 6)
      },

      0xd7 => {
        (1, 1)
      },

      0xd8 => { // CLD
        let status = self.status & !FLAG_D;
        self.status = status;
        (1, 2)
      },

      0xd9 => { // CMP nnnn,Y
        let addr = self.get_addr_absolute_y(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (3, 4)
      },

      0xda => {
        (1, 1)
      },

      0xdb => {
        (1, 1)
      },

      0xdc => {
        (1, 1)
      },

      0xdd => { // CMP nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let value = mem.get_byte(addr);
        let acc = self.acc;
        self.compare(acc, value);
        (3, 4)
      },

      0xde => { // DEC nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let result = mem.get_byte(addr).wrapping_sub(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (3, 7)
      },

      0xdf => {
        (1, 1)
      },

      0xe0 => { // CPX #nn
        let value = mem.get_byte(self.pc + 1);
        let x = self.x;
        self.compare(x, value);
        (2, 2)
      },

      0xe1 => { // SBC (nn,X)
        let orig = self.acc;
        let addr = self.get_addr_indexed_indirect(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (2, 6)
      },

      0xe2 => {
        (1, 1)
      },

      0xe3 => {
        (1, 1)
      },

      0xe4 => { // CPX nn
        let addr = self.get_addr_zeropage(mem);
        let value = mem.get_byte(addr);
        let x = self.x;
        self.compare(x, value);
        (2, 3)
      },

      0xe5 => { // SBC nn
        let orig = self.acc;
        let addr = self.get_addr_zeropage(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (2, 3)
      },

      0xe6 => { // INC nn
        let addr = self.get_addr_zeropage(mem);
        let result = mem.get_byte(addr).wrapping_add(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (2, 5)
      },

      0xe7 => {
        (1, 1)
      },

      0xe8 => { // INX
        let result = self.x.wrapping_add(1);
        self.x = result;
        self.test_flags_n_z(result);
        (1, 2)
      },

      0xe9 => { // SBC #nn
        let orig = self.acc;
        let value = !mem.get_byte(self.pc + 1);
        self.adc(orig, value);
        (2, 2)
      },

      0xea => { // NOP
        (1, 2)
      },

      0xeb => {
        (1, 1)
      },

      0xec => { // CPX nnnn
        let addr = self.get_addr_absolute(mem);
        let value = mem.get_byte(addr);
        let x = self.x;
        self.compare(x, value);
        (3, 4)
      },

      0xed => { // SBC nnnn
        let orig = self.acc;
        let addr = self.get_addr_absolute(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (3, 4)
      },

      0xee => { // INC nnnn
        let addr = self.get_addr_absolute(mem);
        let result = mem.get_byte(addr).wrapping_add(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (3, 6)
      },

      0xef => {
        (1, 1)
      },

      0xf0 => { // BEQ
        if self.status & FLAG_Z > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (0, 3)
        } else {
          (2, 2)
        }
      },

      0xf1 => { // SBC (nn),Y
        let orig = self.acc;
        let addr = self.get_addr_indirect_indexed(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (2, 5)
      },

      0xf2 => {
        (1, 1)
      },

      0xf3 => {
        (1, 1)
      },

      0xf4 => {
        (1, 1)
      },

      0xf5 => { // SBC nn,X
        let orig = self.acc;
        let addr = self.get_addr_zeropage_x(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (2, 4)
      },

      0xf6 => { // INC nn,X
        let addr = self.get_addr_zeropage_x(mem);
        let result = mem.get_byte(addr).wrapping_add(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (2, 6)
      },

      0xf7 => {
        (1, 1)
      },

      0xf8 => { // SED
        let mut status = self.status;
        status = status | FLAG_D;
        self.status = status;
        (1, 2)
      },

      0xf9 => { // SBC nnnn,Y
        let orig = self.acc;
        let addr = self.get_addr_absolute_y(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (3, 4)
      },

      0xfa => {
        (1, 1)
      },

      0xfb => {
        (1, 1)
      },

      0xfc => {
        (1, 1)
      },

      0xfd => { // SBC nnnn,X
        let orig = self.acc;
        let addr = self.get_addr_absolute_x(mem);
        let value = !mem.get_byte(addr);
        self.adc(orig, value);
        (3, 4)
      },

      0xfe => { // INC nnnn,X
        let addr = self.get_addr_absolute_x(mem);
        let result = mem.get_byte(addr).wrapping_add(1);
        mem.set_byte(addr, result);
        self.test_flags_n_z(result);
        (3, 7)
      },

      0xff => {
        (1, 1)
      },

      _ => {
        (1, 0)
      },
    };
    self.pc += byte_len;
    return cycles;
  }
}

#[cfg(test)]
mod tests {
  use vm::cpu::create_cpu;
  use vm::memmap::create_memmap;

  #[test]
  fn adc() {
    let mut cpu = create_cpu();
    cpu.adc(0, 0);
    assert!(cpu.acc == 0);
    assert!(cpu.status & (1 << 7) == 0); // negative
    assert!(cpu.status & (1 << 1) == 1 << 1); // zero
    assert!(cpu.status & 1 == 0); // carry
  }

  #[test]
  fn compare() {
    let mut cpu = create_cpu();
    cpu.compare(0xf, 0x4);
    assert!(cpu.status & (1 << 7) == 0); // negative
    assert!(cpu.status & (1 << 1) == 0); // zero
    assert!(cpu.status & 1 == 0); // carry
    cpu.status = 0;
    cpu.compare(0xf0, 0x2);
    assert!(cpu.status & (1 << 7) > 0); // negative
    assert!(cpu.status & (1 << 1) == 0); // zero
    assert!(cpu.status & 1 == 0); // carry
    cpu.status = 0;
    cpu.compare(0xf, 0x12);
    assert!(cpu.status & (1 << 7) > 0); // negative
    assert!(cpu.status & (1 << 1) == 0); // zero
    assert!(cpu.status & 1 > 0); // carry
    cpu.status = 0;
    cpu.compare(0x20, 0x20);
    assert!(cpu.status & (1 << 7) == 0); // negative
    assert!(cpu.status & (1 << 1) > 0); // zero
    assert!(cpu.status & 1 == 0); // carry
  }

  #[test]
  fn push_pop() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.stack = 0xff;
    cpu.acc = 0;
    cpu.push(&mut mem, 0x20);
    cpu.push(&mut mem, 0x45);
    cpu.push(&mut mem, 0xab);
    assert_eq!(cpu.stack, 0xfc);
    assert_eq!(cpu.pop(&mut mem), 0xab);
    assert_eq!(cpu.pop(&mut mem), 0x45);
    assert_eq!(cpu.pop(&mut mem), 0x20);
    assert_eq!(cpu.stack, 0xff);
  }

  #[test]
  fn jump() {
    let mut cpu = create_cpu();
    cpu.pc = 0x1000;
    cpu.jump_pc(0x70);
    assert_eq!(cpu.pc, 0x1070);
    cpu.jump_pc(0xf);
    assert_eq!(cpu.pc, 0x107f);
    cpu.jump_pc(0x80);
    assert_eq!(cpu.pc, 0xfff);
  }

  #[test]
  fn subroutine_and_return() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x1000, 0x20); // JSR
    mem.set_byte(0x1001, 0x50);
    mem.set_byte(0x1002, 0x12);
    mem.set_byte(0x1250, 0xea); // NOP
    mem.set_byte(0x1251, 0x60); // RTS
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1250);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1251);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1003);
  }

  #[test]
  fn instruction_0x00() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x1000, 0x00);
    mem.set_byte(0x1001, 0xea);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1002);
  }

  #[test]
  fn instruction_0x06() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x0008, 0b01000101);
    mem.set_byte(0x1000, 0x06);
    mem.set_byte(0x1001, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 7);
    assert_eq!(mem.get_byte(0x0008), 0b10001010);
    mem.set_byte(0x1002, 0x06);
    mem.set_byte(0x1003, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1);
    assert_eq!(mem.get_byte(0x0008), 0b00010100);
    mem.set_byte(0x1004, 0x06);
    mem.set_byte(0x1005, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 0);
    assert_eq!(mem.get_byte(0x0008), 0b00101000);
    mem.set_byte(0x1006, 0x06);
    mem.set_byte(0x1007, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 0);
    assert_eq!(mem.get_byte(0x0008), 0b01010000);
    mem.set_byte(0x1008, 0x06);
    mem.set_byte(0x1009, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 7);
    assert_eq!(mem.get_byte(0x0008), 0b10100000);
    mem.set_byte(0x100a, 0x06);
    mem.set_byte(0x100b, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1);
    assert_eq!(mem.get_byte(0x0008), 0b01000000);
    mem.set_byte(0x100c, 0x06);
    mem.set_byte(0x100d, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 7);
    assert_eq!(mem.get_byte(0x0008), 0b10000000);
    mem.set_byte(0x100e, 0x06);
    mem.set_byte(0x100f, 0x08);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 + 2);
    assert_eq!(mem.get_byte(0x0008), 0);
  }

  #[test]
  fn instruction_0x10() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.status = 0;
    mem.set_byte(0x1000, 0x10);
    mem.set_byte(0x1001, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1005);
    cpu.status = 1 << 7;
    mem.set_byte(0x1005, 0x10);
    mem.set_byte(0x1006, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1007);
  }

  #[test]
  fn instruction_0x24() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x0005, 0x10);
    mem.set_byte(0x1000, 0x24);
    mem.set_byte(0x1001, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 2);
    cpu.acc = 0x30;
    mem.set_byte(0x1002, 0x24);
    mem.set_byte(0x1003, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 0);
    mem.set_byte(0x0005, 0x70);
    mem.set_byte(0x1004, 0x24);
    mem.set_byte(0x1005, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 6);
    mem.set_byte(0x0005, 0xb0);
    mem.set_byte(0x1006, 0x24);
    mem.set_byte(0x1007, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 7);
    mem.set_byte(0x0005, 0xd0);
    mem.set_byte(0x1008, 0x24);
    mem.set_byte(0x1009, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, (1 << 7) + (1 << 6));
  }

  #[test]
  fn instruction_0x2a() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.status = 1;
    cpu.acc = 0b01101100;
    mem.set_byte(0x1000, 0x2a);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0b11011001);
    assert_eq!(cpu.status & 1, 0);
    mem.set_byte(0x1001, 0x2a);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0b10110010);
    assert_eq!(cpu.status & 1, 1);
  }

  #[test]
  fn instruction_0x65() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.acc = 0x22;
    mem.set_byte(0x40, 0x33);
    mem.set_byte(0x1000, 0x65);
    mem.set_byte(0x1001, 0x40);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x55);
  }

  #[test]
  fn instruction_0x66() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x40, 0b10110011);
    mem.set_byte(0x1000, 0x66);
    mem.set_byte(0x1001, 0x40);
    cpu.step(&mut mem);
    assert_eq!(mem.get_byte(0x40), 0b01011001);
    assert_eq!(cpu.status, 1);
    mem.set_byte(0x1002, 0x66);
    mem.set_byte(0x1003, 0x40);
    cpu.step(&mut mem);
    assert_eq!(mem.get_byte(0x40), 0b10101100);
    assert_eq!(cpu.status, 1 | (1 << 7));
  }

  #[test]
  fn instruction_0x69() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x1000, 0x69);
    mem.set_byte(0x1001, 0x40);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x40);
    assert_eq!(cpu.status, 0);
    mem.set_byte(0x1002, 0x69);
    mem.set_byte(0x1003, 0x80);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0xc0);
    assert_eq!(cpu.status, 1 << 7);
    mem.set_byte(0x1004, 0x69);
    mem.set_byte(0x1005, 0x80);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x40);
    assert_eq!(cpu.status, (1 << 6) + 1);
    mem.set_byte(0x1006, 0x69);
    mem.set_byte(0x1007, 0x70);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0xb1);
    assert_eq!(cpu.status, (1 << 7) + (1 << 6));
  }

  #[test]
  fn instruction_0x6d() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    mem.set_byte(0x3124, 0x44);
    mem.set_byte(0x1000, 0x6d);
    mem.set_byte(0x1001, 0x24);
    mem.set_byte(0x1002, 0x31);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x44);
  }

  #[test]
  fn instruction_0x75() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.x = 0x2b;
    mem.set_byte(0x6b, 0x33);
    mem.set_byte(0x1000, 0x75);
    mem.set_byte(0x1001, 0x40);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x33);
  }

  #[test]
  fn instruction_0x79() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.y = 0x14;
    mem.set_byte(0x3138, 0x44);
    mem.set_byte(0x1000, 0x79);
    mem.set_byte(0x1001, 0x24);
    mem.set_byte(0x1002, 0x31);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x44);
  }

  #[test]
  fn instruction_0x7d() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.x = 0x23;
    mem.set_byte(0x3147, 0x67);
    mem.set_byte(0x1000, 0x7d);
    mem.set_byte(0x1001, 0x24);
    mem.set_byte(0x1002, 0x31);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x67);
  }

  #[test]
  fn instruction_0x86() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.x = 0x23;
    mem.set_byte(0x1000, 0x86);
    mem.set_byte(0x1001, 0x44);
    cpu.step(&mut mem);
    assert_eq!(mem.get_byte(0x44), 0x23);
  }

  #[test]
  fn instruction_0xe9() {
    let mut cpu = create_cpu();
    let mut mem = create_memmap();
    cpu.pc = 0x1000;
    cpu.acc = 0x43;
    cpu.status = 1;
    mem.set_byte(0x1000, 0xe9);
    mem.set_byte(0x1001, 0x12);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x31);
    cpu.status = 0;
    mem.set_byte(0x1002, 0xe9);
    mem.set_byte(0x1003, 0x4);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x2c);
    cpu.acc = 0x50;
    cpu.status = 1;
    mem.set_byte(0x1004, 0xe9);
    mem.set_byte(0x1005, 0xb0);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0xa0);
    assert!(cpu.status & (1 << 6) > 0);
  }
}