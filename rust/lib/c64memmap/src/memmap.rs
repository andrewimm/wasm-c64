extern crate mos6510;

use cia::CIA;
use ramrom::RamRom;
use sid::SID;
use vic::VIC;
use self::mos6510::memory::Memory;

pub struct MemMap {
  pub ram_rom: RamRom,
  pub cia: CIA,
  pub sid: SID,
  pub vic: VIC,
}

impl Memory for MemMap {
  fn get_byte(&mut self, addr: u16) -> u8 {
    let port = self.ram_rom.ram[1];
    if addr < 0xa000 { // 0x0000 - 0x9fff
      return self.ram_rom.ram[addr as usize];
    }
    if addr < 0xc000 { // 0xa000 - 0xbfff
      // BASIC ROM or RAM
      if port & 3 == 3 {
        return self.ram_rom.basic[(addr - 0xa000) as usize];
      }
      return self.ram_rom.ram[addr as usize];
    }
    if addr < 0xd000 { // 0xc000 - 0xcfff
      return self.ram_rom.ram[addr as usize];
    }
    if addr < 0xe000 { // 0xd000 - 0xdfff
      // RAM, CHARGEN ROM, I/O
      if port & 3 > 0 {
        if port & 4 == 0 {
          return self.ram_rom.char_gen[(addr - 0xd000) as usize];
        }
        if port & 4 == 4 {
          // I/O
          if addr < 0xd400 {
            // VIC II
            return self.vic.get_byte(addr - 0xd000);
          }
          if addr < 0xd800 {
            // SID
            return self.sid.get_byte(addr - 0xd400);
          }
          if addr < 0xdc00 {
            // Color RAM
            return self.ram_rom.color_ram[(addr - 0xd800) as usize];
          }
          if addr < 0xdd00 {
            // CIA 1
            return self.cia.get_byte(addr - 0xdc00);
          }
          if addr < 0xde00 {
            // CIA 2
            return self.cia.get_byte(addr - 0xdd00);
          }
          if addr < 0xdf00 {
            // I/O 1
            return 0;
          }
          // I/O 2
          return 0;
        }
      }
      return self.ram_rom.ram[addr as usize];
    }
    // 0xe000 - 0xffff
    // KERNAL ROM or RAM
    if port & 2 == 2 {
      return self.ram_rom.kernal[(addr - 0xe000) as usize];
    }
    return self.ram_rom.ram[addr as usize];
  }

  fn set_byte(&mut self, addr: u16, value: u8) {
    let port = self.ram_rom.ram[1];
    if addr < 0xa000 { // 0x0000 - 0x9fff
      self.ram_rom.ram[addr as usize] = value;
      return;
    }
    if addr < 0xc000 { // 0xa000 - 0xbfff
      // BASIC ROM or RAM
      if port & 3 == 3 {
        return;
      }
      self.ram_rom.ram[addr as usize] = value;
      return;
    }
    if addr < 0xd000 { // 0xc000 - 0xcfff
      self.ram_rom.ram[addr as usize] = value;
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
          if addr < 0xd400 {
            // VIC II
            self.vic.set_byte(addr - 0xd000, value);
            return;
          }
          if addr < 0xd800 {
            // SID
            self.sid.set_byte(addr - 0xd400, value);
            return;
          }
          if addr < 0xdc00 {
            // Color RAM
            self.ram_rom.color_ram[(addr - 0xd800) as usize] = value;
            return;
          }
          if addr < 0xdd00 {
            // CIA 1
            self.cia.set_byte(addr - 0xdc00, value);
            return;
          }
          if addr < 0xde00 {
            // CIA 2
            self.cia.set_byte(addr - 0xdd00, value);
            return;
          }
          if addr < 0xdf00 {
            // I/O 1
            return;
          }
          // I/O 2
          return;
        }
      }
      self.ram_rom.ram[addr as usize] = value;
      return;
    }
    // 0xe000 - 0xffff
    // KERNAL ROM or RAM
    if port & 2 == 2 {
      return;
    }
    self.ram_rom.ram[addr as usize] = value;
  }
}

impl MemMap {
  pub fn new() -> MemMap {
    let mut map = MemMap {
      ram_rom: RamRom::new(),
      cia: CIA::new(),
      sid: SID::new(),
      vic: VIC::new(),
    };
    // Init directional and port bits
    map.set_byte(0, 0x2f);
    map.set_byte(1, 0x37);
    return map;
  }

  pub fn set_basic_rom(&mut self, bytes: Vec<u8>, offset: usize) {
    for i in 0..bytes.len() {
      self.ram_rom.basic[i + offset] = bytes[i];
    }
  }
}