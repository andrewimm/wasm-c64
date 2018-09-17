use vm::mem;

pub struct MemMap {
  pub mem: mem::Mem,
}

pub fn create_memmap() -> MemMap {
  let map = MemMap {
    mem: mem::create(),
  };
  // Init directional and port bits
  map.set_byte(0, 0xff);
  map.set_byte(1, 0x07);
}

impl MemMap {
  pub fn get_byte(&self, addr: u16) -> u8 {
    let port = self.mem.ram[1];
    if addr < 0xa000 { // 0x0000 - 0x9fff
      return self.mem.ram[addr as usize];
    }
    if addr < 0xc000 { // 0xa000 - 0xbfff
      // BASIC ROM or RAM
      if port & 3 == 3 {
        return self.mem.basic[(addr - 0xa000) as usize];
      }
      return self.mem.ram[addr as usize];
    }
    if addr < 0xd000 { // 0xc000 - 0xcfff
      return self.mem.ram[addr as usize];
    }
    if addr < 0xe000 { // 0xd000 - 0xdfff
      // RAM, CHARGEN ROM, I/O
      if port & 3 > 0 {
        if port & 4 == 0 {
          return self.mem.char_gen[(addr - 0xd000) as usize];
        }
        if port & 4 == 4 {
          // I/O
          if addr >= 0xd800 && addr < 0xdc00 {
            return self.mem.color_ram[(addr - 0xd800) as usize];
          }
          // Need to implement other I/O
          return 0;
        }
      }
      return self.mem.ram[addr as usize];
    }
    // 0xe000 - 0xffff
    // KERNAL ROM or RAM
    if port & 2 == 2 {
      return self.mem.kernal[(addr - 0xe000) as usize];
    }
    return self.mem.ram[addr as usize];
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    let port = self.mem.ram[1];
    if addr < 0xa000 { // 0x0000 - 0x9fff
      self.mem.ram[addr as usize] = value;
      return;
    }
    if addr < 0xc000 { // 0xa000 - 0xbfff
      // BASIC ROM or RAM
      if port & 3 == 3 {
        return;
      }
      self.mem.ram[addr as usize] = value;
      return;
    }
    if addr < 0xd000 { // 0xc000 - 0xcfff
      self.mem.ram[addr as usize] = value;
      return;
    }
    if addr < 0xe000 { // 0xd000 - 0xdfff
      // RAM, CHARGEN ROM, I/O
      if port & 3 > 0 {
        if port & 4 == 0 {
          return;
        }
        if port & 4 == 4 {
          // I/O
          if addr >= 0xd800 && addr < 0xdc00 {
            self.mem.color_ram[(addr - 0xd800) as usize] = value;
            return;
          }
          // Need to implement other I/O
          return;
        }
      }
      self.mem.ram[addr as usize] = value;
      return;
    }
    // 0xe000 - 0xffff
    // KERNAL ROM or RAM
    if port & 2 == 2 {
      return;
    }
    self.mem.ram[addr as usize] = value;
  }
}