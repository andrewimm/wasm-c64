use cpu::CPU;

pub trait Memory {
  fn get_byte(&mut self, addr: u16) -> u8;
  fn set_byte(&mut self, addr: u16, value: u8);
}

#[inline]
pub fn memory_get_short(mem: &mut Memory, addr: u16) -> u16 {
  let low = mem.get_byte(addr) as u16;
  let high = mem.get_byte(addr + 1) as u16;
  (high << 8) | low
}

impl CPU {
  pub fn reset(&mut self, mem: &mut Memory) {
    self.acc = 0;
    self.x = 0;
    self.y = 0;
    self.status = 0;
    self.stack = 0xfd;

    self.pc = memory_get_short(mem, 0xfffc);
  }

  pub fn push(&mut self, mem: &mut Memory, value: u8) {
    let addr: u16 = 0x100 + (self.stack as u16);
    mem.set_byte(addr, value);
    self.stack = self.stack.wrapping_sub(1);
  }

  pub fn pop(&mut self, mem: &mut Memory) -> u8 {
    self.stack = self.stack.wrapping_add(1);
    let addr: u16 = 0x100 + (self.stack as u16);
    mem.get_byte(addr)
  }

  /**
   * Addressing modes for operations
   * Most instructions operate on non-register values, because the 6510 has few
   * registers (and rapid access to zero-page addresses)
   * Depending on the instruction, external values may come directly from
   * program memory, or from another location.
   * These methods do the fetching and math to compute the memory address needed
   * to execute the instruction.
   */
  #[inline]
  pub fn get_address_immediate(&self) -> u16 {
    self.pc + 1
  }

  #[inline]
  pub fn get_address_relative(&self) -> u16 {
    self.pc + 1
  }

  #[inline]
  pub fn get_address_zeropage(&self, mem: &mut Memory) -> u16 {
    mem.get_byte(self.pc + 1) as u16
  }

  #[inline]
  pub fn get_address_zeropage_x(&self, mem: &mut Memory) -> u16 {
    (mem.get_byte(self.pc + 1) as u16).wrapping_add(self.x as u16)
  }

  #[inline]
  pub fn get_address_zeropage_y(&self, mem: &mut Memory) -> u16 {
    (mem.get_byte(self.pc + 1) as u16).wrapping_add(self.y as u16)
  }

  #[inline]
  pub fn get_address_absolute(&self, mem: &mut Memory) -> u16 {
    let low = mem.get_byte(self.pc + 1) as u16;
    let high = mem.get_byte(self.pc + 2) as u16;
    low | (high << 8)
  }

  #[inline]
  pub fn get_address_absolute_x(&self, mem: &mut Memory) -> u16 {
    let low = mem.get_byte(self.pc + 1) as u16;
    let high = mem.get_byte(self.pc + 2) as u16;
    (low | (high << 8)) + (self.x as u16)
  }

  #[inline]
  pub fn get_address_absolute_y(&self, mem: &mut Memory) -> u16 {
    let low = mem.get_byte(self.pc + 1) as u16;
    let high = mem.get_byte(self.pc + 2) as u16;
    (low | (high << 8)) + (self.y as u16)
  }

  #[inline]
  pub fn get_address_indirect(&self, mem: &mut Memory) -> u16 {
    let src_low = mem.get_byte(self.pc + 1) as u16;
    let src_high = mem.get_byte(self.pc + 2) as u16;
    let src = src_low | (src_high << 8);
    let low = mem.get_byte(src) as u16;
    let high = mem.get_byte(src + 1) as u16;
    (low | (high << 8))
  }

  #[inline]
  pub fn get_address_indexed_indirect(&self, mem: &mut Memory) -> u16 {
    let src = (mem.get_byte(self.pc + 1) as u16).wrapping_add(self.x as u16);
    let low = mem.get_byte(src) as u16;
    let high = mem.get_byte(src + 1) as u16;
    (low | (high << 8))
  }

  #[inline]
  pub fn get_address_indirect_indexed(&self, mem: &mut Memory) -> u16 {
    let src = mem.get_byte(self.pc + 1) as u16;
    let low = mem.get_byte(src) as u16;
    let high = mem.get_byte(src + 1) as u16;
    let pointer = low | (high << 8);
    pointer.wrapping_add(self.y as u16)
  }
}

#[cfg(test)]
pub mod mock {
  use memory::Memory;

  pub struct MockMem {
    pub ram: Box<[u8; 0x2000]>,
  }

  impl MockMem {
    pub fn new() -> MockMem {
      return MockMem {
        ram: Box::new([0; 0x2000]),
      };
    }
  }

  impl Memory for MockMem {
    fn get_byte(&mut self, addr: u16) -> u8 {
      return self.ram[(addr % 0x2000) as usize];
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
      self.ram[(addr % 0x2000) as usize] = value;
    }
  }

  pub struct MockBigMem {
    pub ram: Box<[u8; 0x10000]>,
  }

  impl MockBigMem {
    pub fn new() -> MockBigMem {
      return MockBigMem {
        ram: Box::new([0; 0x10000]),
      };
    }
  }

  impl Memory for MockBigMem {
    fn get_byte(&mut self, addr: u16) -> u8 {
      return self.ram[addr as usize];
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
      self.ram[addr as usize] = value;
    }
  }
}

#[cfg(test)]
mod tests {
  use cpu::CPU;
  use memory::mock::MockMem;

  #[test]
  fn pushpop() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.stack = 0xfd;
    cpu.push(&mut mem, 0xa1);
    cpu.push(&mut mem, 0xb2);
    cpu.push(&mut mem, 0xc3);
    assert!(cpu.pop(&mut mem) == 0xc3);
    assert!(cpu.pop(&mut mem) == 0xb2);
    assert!(cpu.pop(&mut mem) == 0xa1);
  }

  #[test]
  fn addr_absolute() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    mem.ram[100] = 0x41;
    mem.ram[101] = 0xc0;
    cpu.pc = 99;
    assert!(cpu.get_address_absolute(&mut mem) == 0xc041);
  }

  #[test]
  fn addr_absolute_x() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    mem.ram[100] = 0xb0;
    mem.ram[101] = 0x08;
    cpu.pc = 99;
    cpu.x = 5;
    assert!(cpu.get_address_absolute_x(&mut mem) == 0x8b5);
  }

  #[test]
  fn addr_absolute_y() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    mem.ram[100] = 0xbb;
    mem.ram[101] = 0xaa;
    cpu.pc = 99;
    cpu.y = 2;
    assert!(cpu.get_address_absolute_y(&mut mem) == 0xaabd);
  }

  #[test]
  fn addr_indirect() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    mem.ram[0x20] = 0x40;
    mem.ram[0x21] = 0x00;
    mem.ram[0x40] = 0xca;
    mem.ram[0x41] = 0xb0;
    cpu.pc = 0x1f;
    assert!(cpu.get_address_indirect(&mut mem) == 0xb0ca);
  }

  #[test]
  fn addr_indirect_x() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    mem.ram[0x20] = 0x01;
    mem.ram[0x21] = 0x00;
    mem.ram[0x05] = 0x11;
    mem.ram[0x06] = 0x5c;
    cpu.pc = 0x1f;
    cpu.x = 4;
    assert!(cpu.get_address_indexed_indirect(&mut mem) == 0x5c11);
  }

  #[test]
  fn addr_indirect_y() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    mem.ram[0x20] = 0x33;
    mem.ram[0x21] = 0x00;
    mem.ram[0x33] = 0xc0;
    mem.ram[0x34] = 0xab;
    cpu.pc = 0x1f;
    cpu.y = 0xd;
    assert!(cpu.get_address_indirect_indexed(&mut mem) == 0xabcd);
  }
}