use crate::mapper::mapper::{ChrMem, Config, Mapper, Mirroring};

pub struct NROM {
  prg_ram: Box<[u8; 0x2000]>,
  prg_rom: Box<[u8; 0x4000]>,
  chr_mem: ChrMem,

  config: Config,
}

impl NROM {
  pub fn new(config: Config) -> NROM {
    let chr = if config.chr_rom_size == 0 {
      ChrMem::Ram(box [0; 0x2000])
    } else {
      ChrMem::Rom(box [0; 0x2000])
    };
    NROM {
      prg_ram: box [0; 0x2000],
      prg_rom: box [0; 0x4000],
      chr_mem: chr,

      config: config,
    }
  }
}

impl Mapper for NROM {
  fn cpu_set_byte(&mut self, addr: u16, value: u8) {
    if addr < 0x6000 {
      // not supported
      return;
    }
    if addr < 0x8000 {
      self.prg_ram[(addr - 0x6000) as usize] = value;
      return;
    }
  }

  fn cpu_get_byte(&mut self, addr: u16) -> u8 {
    if addr < 0x6000 {
      // not supported
      return 0;
    }
    if addr < 0x8000 {
      // 8KB PRG RAM
      return self.prg_ram[(addr - 0x6000) as usize];
    }
    if addr < 0xc000 {
      // First PRG ROM bank
      return self.prg_rom[(addr - 0x8000) as usize];
    }
    // Second PRG ROM bank, mirror of first
    return self.prg_rom[(addr - 0xc000) as usize];
  }

  fn ppu_get_byte(&self, addr: u16) -> u8 {
    if addr >= 0x2000 {
      return 0;
    }
    match &self.chr_mem {
      ChrMem::Ram(mem) => {
        mem[addr as usize]
      },
      ChrMem::Rom(mem) => {
        mem[addr as usize]
      },
    }
  }

  fn ppu_set_byte(&mut self, addr: u16, value: u8) {
    if addr >= 0x2000 {
      return;
    }
    if let ChrMem::Ram(mem) = &mut self.chr_mem {
      mem[addr as usize] = value;
    }
  }

  fn ppu_get_mirrored_address(&self, addr: u16) -> u16 {
    let mode = &self.config.mirroring;
    match mode {
      Mirroring::Vertical => {
        let offset = addr - 0x2000;
        0x2000 + (offset & 0x7ff)
      },
      Mirroring::Horizontal => {
        let offset = addr - 0x2000;
        let page = (addr & 0x800) >> 1;
        0x2000 + (page | (offset & 0x3ff))
      }
    }
  }

  fn set_prg_rom(&mut self, rom: &[u8]) {
    for i in 0..rom.len() {
      self.prg_rom[i] = rom[i];
    }
  }

  fn set_chr_rom(&mut self, rom: &[u8]) {
    if let ChrMem::Rom(mem) = &mut self.chr_mem {
      for i in 0..rom.len() {
        mem[i] = rom[i];
      }
    }
  }

  fn get_pattern_0_ptr(&self) -> *const u8 {
    match &self.chr_mem {
      ChrMem::Ram(mem) => &mem[0] as *const u8,
      ChrMem::Rom(mem) => &mem[0] as *const u8,
    }
  }

  fn get_pattern_1_ptr(&self) -> *const u8 {
    match &self.chr_mem {
      ChrMem::Ram(mem) => &mem[0x1000] as *const u8,
      ChrMem::Rom(mem) => &mem[0x1000] as *const u8,
    }
  }

  fn get_nametable_offsets(&self) -> (usize, usize, usize, usize) {
    (
      self.ppu_get_mirrored_address(0x2000) as usize - 0x2000,
      self.ppu_get_mirrored_address(0x2400) as usize - 0x2000,
      self.ppu_get_mirrored_address(0x2800) as usize - 0x2000,
      self.ppu_get_mirrored_address(0x2c00) as usize - 0x2000,
    )
  }
}
