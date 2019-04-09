//use apu::APU;
use crate::mapper::Mapper;
use crate::ppu::PPU;
use crate::ram::RAM;
use mos6510::memory::Memory;

pub struct MemMap {
  //pub apu: APU,
  pub ppu: PPU,
  pub ram: RAM,
  pub mapper: Box<Mapper>,
}

impl Memory for MemMap {
  fn get_byte(&mut self, addr: u16) -> u8 {
    if addr < 0x2000 { // RAM
      let dest = addr % 0x800;
      return self.ram.get_byte(dest);
    }
    if addr < 0x4000 { // PPU
      let dest = (addr - 0x2000) % 8;
      return self.ppu.get_byte(addr, &self.mapper);
    }
    if addr < 0x4018 { // APU + I/O
      let dest = addr - 0x4000;
      //return self.apu.get_byte(dest);
      return 0;
    }
    if addr < 0x4020 { // disabled
      return 0;
    }
    // maps to cartridge ROM
    return self.mapper.cpu_get_byte(addr);
  }

  fn set_byte(&mut self, addr: u16, value: u8) {
    if addr < 0x2000 { // RAM
      let dest = addr & 0x7ff;
      self.ram.set_byte(dest, value);
      return;
    }
    if addr < 0x4000 { // PPU
      let dest = (addr - 0x2000) % 8;
      self.ppu.set_byte(dest, value, &mut self.mapper);
      return;
    }
    if addr < 0x4018 { // APU + I/O
      let dest = addr - 0x4000;
      //return self.apu.get_byte(dest);
      return;
    }
    if addr < 0x4020 { // disabled
      return;
    }
    // maps to cartridge ROM
    self.mapper.cpu_set_byte(addr, value);
    return;
  }
}

impl MemMap {
  pub fn new(mapper: Box<Mapper>) -> MemMap {
    let mut map = MemMap {
      //apu: APU::new(),
      ppu: PPU::new(),
      ram: RAM::new(),
      mapper: mapper,
    };

    return map;
  }

  pub fn get_pattern_0_ptr(&self) -> *const u8 {
    self.mapper.get_pattern_0_ptr()
  }

  pub fn get_pattern_1_ptr(&self) -> *const u8 {
    self.mapper.get_pattern_1_ptr()
  }
}
