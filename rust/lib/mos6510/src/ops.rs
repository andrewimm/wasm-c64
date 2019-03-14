use cpu::CPU;
use flags;
use memory::Memory;

impl CPU {
  pub fn ora(&mut self, mem: &Memory, addr: u16) {
    let orig = self.acc;
    let value = mem.get_byte(addr);
    let result = orig | value;
    self.acc = result;
    self.test_flag_zero(result);
    self.test_flag_negative(result);
  }

  pub fn asl(&mut self, value: u8) -> u8 {
    let carry = value & 0x80 == 0x80;
    let result = value << 1;
    self.test_flag_zero(result);
    self.test_flag_negative(result);
    self.set_flag_carry(carry);
    result
  }

  pub fn php(&mut self, mem: &mut Memory) {
    let value = self.status;
    self.push(mem, value);
  }

  pub fn clc(&mut self) {
    let status = self.status & !flags::FLAG_CARRY;
    self.status = status;
  }

  pub fn and(&mut self, mem: &mut Memory, addr: u16) {
    let orig = self.acc;
    let value = mem.get_byte(addr);
    self.acc = orig & value;
  }

  pub fn bit(&mut self, mem: &Memory, addr: u16) {
    let orig = self.acc;
    let value = mem.get_byte(addr);
    let mut status = self.status;
    if value & (1 << 7) > 0 {
      status = status | flags::FLAG_NEGATIVE;
    } else {
      status = status & !flags::FLAG_NEGATIVE;
    }
    if value & (1 << 6) > 0 {
      status = status | flags::FLAG_OVERFLOW;
    } else {
      status = status & !flags::FLAG_OVERFLOW;
    }
    if orig & value == 0 {
      status = status | flags::FLAG_ZERO;
    } else {
      status = status & !flags::FLAG_ZERO;
    }
    self.status = status;
  }

  pub fn rol(&mut self, value: u8) -> u8 {
    let had_carry = self.status & 1 == 1;
    let carry = value & 0x80 == 0x80;
    let mut result = value << 1;
    if had_carry {
      result = result | 1;
    }
    self.test_flag_zero(result);
    self.test_flag_negative(result);
    self.set_flag_carry(carry);
    result
  }

  pub fn plp(&mut self, mem: &Memory) {
    let status = self.pop(mem);
    self.status = status & 0b11001111;
  }

  pub fn sec(&mut self) {
    let status = self.status;
    self.status = status | flags::FLAG_CARRY;
  }

  pub fn rti(&mut self, mem: &Memory) {
    let status = self.pop(mem);
    let pc_low = self.pop(mem) as u16;
    let pc_high = self.pop(mem) as u16;
    self.status = status & 0b11001111;
    self.pc = (pc_high << 8) | pc_low;
  }

  pub fn eor(&mut self, mem: &Memory, addr: u16) {
    let orig = self.acc;
    let value = mem.get_byte(addr);
    let result = orig ^ value;
    self.acc = result;
    self.test_flag_zero(result);
    self.test_flag_negative(result);
  }

  pub fn lsr(&mut self, value: u8) -> u8 {
    let carry = value & 1 == 1;
    let result = value >> 1;
    self.test_flag_zero(result);
    self.test_flag_negative(result);
    self.set_flag_carry(carry);
    result
  }

  pub fn pha(&mut self, mem: &mut Memory) {
    let value = self.acc;
    self.push(mem, value);
  }

  pub fn cli(&mut self) {
    let status = self.status & !flags::FLAG_INTERRUPT_DISABLE;
    self.status = status;
  }

  pub fn rts(&mut self, mem: &Memory) {
    let low = self.pop(mem) as u16;
    let high = self.pop(mem) as u16;
    let ret = (high << 8) | low;
    self.pc = ret;
  }

  pub fn adc(&mut self, value: u8) {
    let orig = self.acc;
    let (mut total, mut overflow) = orig.overflowing_add(value);
    let mut v =
      (orig & 0b10000000 == value & 0b10000000) &&
      (orig & 0b10000000 != total & 0b10000000);
    if self.status & flags::FLAG_CARRY > 0 {
      let (carry_total, carry_overflow) = total.overflowing_add(1);
      v = v || (total & 0b10000000 == 0 && carry_total & 0b10000000 > 0);
      total = carry_total;
      overflow = overflow || carry_overflow;
    }
    self.test_flag_negative(total);
    self.test_flag_zero(total);
    self.set_flag_carry(overflow);
    self.set_flag_overflow(v);
    self.acc = total;
  }
  
  pub fn sbc(&mut self, mem: &Memory, addr: u16) {
    let value = !mem.get_byte(addr);
    self.adc(value);
  }

  pub fn ror(&mut self, value: u8) -> u8 {
    let had_carry = self.status & 1 == 1;
    let carry = value & 1 == 1;
    let mut result = value >> 1;
    if had_carry {
      result = result | 0x80;
    }
    self.test_flag_negative(result);
    self.test_flag_zero(result);
    self.set_flag_carry(carry);
    result
  }

  pub fn pla(&mut self, mem: &Memory) {
    let acc = self.pop(mem);
    self.acc = acc;
    self.test_flag_negative(acc);
    self.test_flag_zero(acc);
  }

  pub fn sei(&mut self) {
    let status = self.status;
    self.status = status | flags::FLAG_INTERRUPT_DISABLE;
  }

  pub fn sta(&mut self, mem: &mut Memory, addr: u16) {
    let value = self.acc;
    mem.set_byte(addr, value);
  }

  pub fn stx(&mut self, mem: &mut Memory, addr: u16) {
    let value = self.x;
    mem.set_byte(addr, value);
  }

  pub fn sty(&mut self, mem: &mut Memory, addr: u16) {
    let value = self.y;
    mem.set_byte(addr, value);
  }

  pub fn dey(&mut self) {
    let result = self.y.wrapping_sub(1);
    self.y = result;
    self.test_flag_negative(result);
    self.test_flag_zero(result);
  }

  pub fn txa(&mut self) {
    let value = self.x;
    self.acc = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn tya(&mut self) {
    let value = self.y;
    self.acc = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn txs(&mut self) {
    let value = self.x;
    self.stack = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn ldy(&mut self, mem: &Memory, addr: u16) {
    let value = mem.get_byte(addr);
    self.y = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn lda(&mut self, mem: &Memory, addr: u16) {
    let value = mem.get_byte(addr);
    self.acc = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn ldx(&mut self, mem: &Memory, addr: u16) {
    let value = mem.get_byte(addr);
    self.x = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn tay(&mut self) {
    let value = self.acc;
    self.y = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn tax(&mut self) {
    let value = self.acc;
    self.x = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn clv(&mut self) {
    let status = self.status & !flags::FLAG_OVERFLOW;
    self.status = status;
  }

  pub fn tsx(&mut self) {
    let value = self.stack;
    self.x = value;
    self.test_flag_negative(value);
    self.test_flag_zero(value);
  }

  pub fn compare(&mut self, a: u8, b: u8) {
    let value = a.wrapping_sub(b);
    self.test_flag_negative(value);
    self.test_flag_zero(value);
    let status = self.status;
    if b > a {
      self.status = status | flags::FLAG_CARRY;
    } else {
      self.status = status & !flags::FLAG_CARRY;
    }
  }

  pub fn cpy(&mut self, mem: &Memory, addr: u16) {
    let value = mem.get_byte(addr);
    let y = self.y;
    self.compare(y, value);
  }

  pub fn cpx(&mut self, mem: &Memory, addr: u16) {
    let value = mem.get_byte(addr);
    let x = self.x;
    self.compare(x, value);
  }

  pub fn cmp(&mut self, mem: &Memory, addr: u16) {
    let value = mem.get_byte(addr);
    let acc = self.acc;
    self.compare(acc, value);
  }

  pub fn iny(&mut self) {
    let result = self.y.wrapping_add(1);
    self.y = result;
    self.test_flag_negative(result);
    self.test_flag_zero(result);
  }

  pub fn inx(&mut self) {
    let result = self.x.wrapping_add(1);
    self.x = result;
    self.test_flag_negative(result);
    self.test_flag_zero(result);
  }

  pub fn inc(&mut self, mem: &mut Memory, addr: u16) {
    let result = mem.get_byte(addr).wrapping_add(1);
    mem.set_byte(addr, result);
    self.test_flag_negative(result);
    self.test_flag_zero(result);
  }

  pub fn dex(&mut self) {
    let result = self.x.wrapping_sub(1);
    self.x = result;
    self.test_flag_negative(result);
    self.test_flag_zero(result);
  }

  pub fn dec(&mut self, mem: &mut Memory, addr: u16) {
    let result = mem.get_byte(addr).wrapping_sub(1);
    mem.set_byte(addr, result);
    self.test_flag_negative(result);
    self.test_flag_zero(result);
  }

  pub fn cld(&mut self) {
    let status = self.status & !flags::FLAG_DECIMAL;
    self.status = status;
  }

  pub fn sed(&mut self) {
    let status = self.status | flags::FLAG_DECIMAL;
    self.status = status;
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
  
  pub fn kil(&mut self) {

  }
}

#[cfg(test)]
mod tests {
  use cpu::CPU;

  #[test]
  fn compare() {
    let mut cpu = CPU::new();
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
  fn jump() {
    let mut cpu = CPU::new();
    cpu.pc = 0x1000;
    cpu.jump_pc(0x70);
    assert_eq!(cpu.pc, 0x1070);
    cpu.jump_pc(0xf);
    assert_eq!(cpu.pc, 0x107f);
    cpu.jump_pc(0x80);
    assert_eq!(cpu.pc, 0xfff);
  }
}