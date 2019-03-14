use cpu::CPU;
use flags;
use memory::Memory;

impl CPU {
  pub fn interrupt_request(&mut self, mem: &mut Memory) {
    if (self.status & flags::FLAG_INTERRUPT_DISABLE) != 0 {
      return
    }
    self.interrupt(mem, 0xfffe);
  }

  pub fn nonmaskable_interrupt(&mut self, mem: &mut Memory) {
    self.interrupt(mem, 0xfffa);
  }

  fn interrupt(&mut self, mem: &mut Memory, vector: u16) {
    let pc = self.pc;
    let status = self.status;
    self.push(mem, (pc >> 8) as u8);
    self.push(mem, (pc & 0xff) as u8);
    self.push(mem, status);
    self.status = status | flags::FLAG_INTERRUPT_DISABLE;
    let dest_low = mem.get_byte(vector) as u16;
    let dest_high = mem.get_byte(vector + 1) as u16;
    self.pc = (dest_high << 8) | dest_low;
  }

  pub fn brk(&mut self, mem: &mut Memory) {
    // Byte after BRK instruction is padding
    // After an interrupt resumes execution, it returns to the address two bytes
    // after the original BRK opcode
    let pc = self.pc.wrapping_add(2);
    let status = self.status | (1 << 5) | flags::FLAG_BRK;
    let vector = 0xfffe;
    self.push(mem, (pc >> 8) as u8);
    self.push(mem, (pc & 0xff) as u8);
    self.push(mem, status);
    let dest_low = mem.get_byte(vector) as u16;
    let dest_high = mem.get_byte(vector) as u16;
    self.pc = (dest_high << 8) | dest_low;
  }
}

#[cfg(test)]
mod tests {
  use cpu::CPU;
  use flags;
  use memory::Memory;
  use memory::mock::MockBigMem;

  #[test]
  fn irq() {
    let mut cpu = CPU::new();
    let mut mem = MockBigMem::new();
    cpu.stack = 0xfc;
    cpu.pc = 0x1234;
    mem.set_byte(0xfffe, 0x20);
    mem.set_byte(0xffff, 0x40);
    cpu.status = flags::FLAG_CARRY | flags::FLAG_INTERRUPT_DISABLE;
    cpu.interrupt_request(&mut mem);
    assert_eq!(cpu.pc, 0x1234);

    cpu.status = flags::FLAG_CARRY;
    cpu.interrupt_request(&mut mem);
    assert_eq!(cpu.pc, 0x4020);
    assert_eq!(mem.get_byte(0x1fc), 0x12);
    assert_eq!(mem.get_byte(0x1fb), 0x34);
    assert_eq!(mem.get_byte(0x1fa), flags::FLAG_CARRY);
  }
}