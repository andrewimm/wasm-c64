use crate::apu::APU;
use crate::controller::Controller;
use crate::mapper::Mapper;
use crate::ppu::PPU;
use crate::ppu2::PPU2;
use crate::ram::RAM;
use mos6510::memory::Memory;

pub struct MemMap {
  pub apu: Box<APU>,
  pub ppu: PPU,
  pub ppu2: PPU2,
  pub ram: RAM,
  pub mapper: Box<Mapper>,
  pub controller_0: Controller,

  needs_dma: bool,
  pub dma_source: u16,
}

impl Memory for MemMap {
  fn get_byte(&mut self, addr: u16) -> u8 {
    if addr < 0x2000 { // RAM
      let dest = addr % 0x800;
      return self.ram.get_byte(dest);
    }
    if addr < 0x4000 { // PPU
      let dest = (addr - 0x2000) % 8;
      //self.ppu.get_byte(addr, &self.mapper);
      return self.ppu2.get_byte(addr, &self.mapper);
    }
    if addr < 0x4018 { // APU + I/O
      if addr == 0x4016 {
        return self.controller_0.read_latch();
      }
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
      //self.ppu.set_byte(dest, value, &mut self.mapper);
      self.ppu2.set_byte(dest, value, &mut self.mapper);
      return;
    }
    if addr < 0x4018 { // APU + I/O
      if addr == 0x4000 {
        self.apu.set_square_0_properties(value >> 6, value & 0x20 == 0x20, value & 0x10 == 0x10, value & 0xf);
        return;
      }
      if addr == 0x4001 {
        return;
      }
      if addr == 0x4002 {
        self.apu.set_square_0_timer_low(value);
        return;
      }
      if addr == 0x4003 {
        self.apu.set_square_0_timer_high(value & 7);
        self.apu.set_square_0_length(value >> 3);
        return;
      }
      if addr == 0x4004 {
        self.apu.set_square_1_properties(value >> 6, value & 0x20 == 0x20, value & 0x10 == 0x10, value & 0xf);
        return;
      }
      if addr == 0x4005 {
        return;
      }
      if addr == 0x4006 {
        self.apu.set_square_1_timer_low(value);
        return;
      }
      if addr == 0x4007 {
        self.apu.set_square_1_timer_high(value & 7);
        self.apu.set_square_1_length(value >> 3);
        return;
      }
      if addr == 0x4008 {
        self.apu.set_triangle_properties(value & 0x80 == 0x80, value & 0x7f);
        return;
      }
      if addr == 0x4009 {
        return;
      }
      if addr == 0x400a {
        self.apu.set_triangle_timer_low(value);
        return;
      }
      if addr == 0x400b {
        self.apu.set_triangle_timer_high(value & 7);
        self.apu.set_triangle_length(value >> 3);
        return;
      }
      if addr == 0x4014 { // DMA
        self.dma_source = (value as u16) << 8;
        self.needs_dma = true;
        return;
      }
      if addr == 0x4015 {
        self.apu.toggle_square_0(value & 1 == 1);
        self.apu.toggle_square_1(value & 2 == 2);
        self.apu.toggle_triangle(value & 4 == 4);
        return;
      }
      if addr == 0x4016 {
        if value & 1 == 1 {
          self.controller_0.begin_latch();
        } else {
          self.controller_0.end_latch();
        }
        return;
      }
      if addr == 0x4017 {
        //println!("0x4017 {:x}", value);
      }
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
  pub fn new(mapper: Box<Mapper>, apu: Box<APU>) -> MemMap {
    let mut map = MemMap {
      apu: apu,
      controller_0: Controller::new(),
      ppu: PPU::new(),
      ppu2: PPU2::new(),
      ram: RAM::new(),
      mapper: mapper,

      needs_dma: false,
      dma_source: 0,
    };

    return map;
  }

  pub fn dma_requested(&mut self) -> bool {
    if self.needs_dma {
      self.needs_dma = false;
      true
    } else {
      false
    }
  }

  pub fn dma_copy(&mut self) {
    for i in 0..256 {
      let byte = self.get_byte(self.dma_source + i);
      //self.ppu.write_oam(byte);
      self.ppu2.write_oam(byte);
    }
  }

  pub fn get_pattern_0_ptr(&self) -> *const u8 {
    self.mapper.get_pattern_0_ptr()
  }

  pub fn get_pattern_1_ptr(&self) -> *const u8 {
    self.mapper.get_pattern_1_ptr()
  }

  pub fn increment_clock(&mut self) {
    self.ppu2.increment_clock(&self.mapper);
  }
}
