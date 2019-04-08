use super::mapper::{ChrMem, Config, Mapper};

pub struct MMC1 {
  shifter: u8,

  register_control: u8,
  register_chr0: u8,
  register_chr1: u8,
  register_prg: u8,

  prg_ram: Box<[u8; 0x2000]>,
  prg_rom: Box<[u8; 0x40000]>,
  chr_mem: ChrMem,

  config: Config,
}

impl MMC1 {
  pub fn new(config: Config) -> MMC1 {
    let chr = if config.chr_rom_size == 0 {
      ChrMem::Ram(box [0; 0x2000])
    } else {
      let size = (config.chr_rom_size as usize) * 16 * 1024;
      let mem = Vec::with_capacity(size);
      ChrMem::Rom(mem.into_boxed_slice())
    };
    MMC1 {
      shifter: 0x10,
      register_control: 0,
      register_chr0: 0,
      register_chr1: 0,
      register_prg: 0,

      prg_ram: box [0; 0x2000],
      prg_rom: box [0; 0x40000],
      chr_mem: chr,

      config: config,
    }
  }
}

impl Mapper for MMC1 {
  fn cpu_set_byte(&mut self, addr: u16, value: u8) {
    if addr < 0x6000 {
      // not supported
      return;
    }
    if addr < 0x8000 {
      // write to ram, if enabled
      let prg = self.register_prg;
      if prg & 0x10 != 0 {
        // PRG ram disabled
        return;
      }
      let bank = prg & 0xf;


      return;
    }
    // Addresses 0x8000-0xffff are connected to a shift register
    if value & 0x80 != 0 {
      // Reset shifter, write control
      self.shifter = 0x10;
      self.register_control = self.register_control | 0x0c;
      return;
    }
    let should_write = self.shifter & 1 != 0;
    self.shifter = self.shifter >> 1;
    let lsb = value & 1;
    if lsb != 0 {
      self.shifter = self.shifter | 0x10;
    }
    if should_write { // Shifter is full, write it and reset
      match addr {
        0x8000...0x9fff => {
          self.register_control = self.shifter & 0x1f;
        },
        0xa000...0xbfff => {
          self.register_chr0 = self.shifter & 0x1f;
        },
        0xc000...0xdfff => {
          self.register_chr1 = self.shifter & 0x1f;
        },
        0xe000...0xffff => {
          self.register_prg = self.shifter & 0x1f;
        },
        _ => (),
      }
      self.shifter = 0x10;
    }
  }

  fn cpu_get_byte(&mut self, addr: u16) -> u8 {
    if addr < 0x6000 {
      // not supported
      return 0;
    }
    if addr < 0x8000 {
      // 8KB PRG RAM, if available
      if self.register_prg & 0x10 != 0 {
        // PRG ram disabled
        return 0;
      }
      return self.prg_ram[(addr - 0x6000) as usize];
    }
    let mode = self.register_control & 0xc;
    if addr < 0xc000 {
      // 16KB PRG ROM
      let offset = addr as u32 - 0x8000;
      if mode & 0x8 == 0 {
        // 32KB bank mode
        let bank = self.register_prg as u32 & 0xe;
        return self.prg_rom[(offset + bank * 0x4000) as usize];
      }
      if mode == 0x8 {
        // first bank fixed at 0x0000
        return self.prg_rom[offset as usize];
      }
      if mode == 0x9 {
        // switch mode
        let bank = self.register_prg as u32 & 0xf;
        return self.prg_rom[(offset + bank * 0x4000) as usize];
      }
      return 0;
    }
    // 16KB PRG ROM
    let offset = addr as u32 - 0xc000;
    if mode & 0x8 == 0 {
      // 32KB bank mode
      let bank = (self.register_prg as u32 & 0xe) | 1 ;
      return self.prg_rom[(offset + bank * 0x4000) as usize];
    }
    if mode == 0x8 {
      // switch mode
      let bank = self.register_prg as u32 & 0xf;
      return self.prg_rom[(offset + bank * 0x4000) as usize];
    }
    if mode == 0x9 {
      // last bank fixed at 0xc000
      let bank = self.config.prg_rom_size as u32 - 1;
      return self.prg_rom[(offset + bank * 0x4000) as usize];
    }

    return 0;
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
        let separate_bank_mode = self.register_control & 0x10 != 0;
        if addr < 0x1000 {
          // CHR 0
          let mut bank = self.register_chr0 as u32;
          if !separate_bank_mode {
            bank = bank & 0x1e;
          }
          return mem[(addr as u32 + bank * 0x2000) as usize];
        }
        if addr < 0x2000 {
          // CHR 1
          let mut bank = self.register_chr1 as u32;
          if !separate_bank_mode {
            bank = (self.register_chr0 as u32 & 0x1e) | 1;
          }
          return mem[((addr as u32 & 0xfff) + bank * 0x2000) as usize];
        }
        // invalid
        return 0;
      },
    }
  }

  fn ppu_get_mirrored_address(&self, addr: u16) -> u16 {
    let mode = self.register_control & 3;
    match mode {
      0 => {
        // one screen, lower bank
        let offset = addr - 0x2000;
        0x2000 + (offset & 0x3ff)
      },
      1 => {
        // one screen, upper bank
        let offset = addr - 0x2000;
        0x2400 + (offset & 0x3ff)
      },
      2 => {
        // vertical
        let offset = addr - 0x2000;
        0x2000 + (offset & 0x7ff)
      },
      3 => {
        // horizontal
        let offset = addr - 0x2000;
        0x2000 + (offset & 0xbff)
      }
      _ => addr,
    }
  }

  fn set_prg_rom(&mut self, rom: &[u8]) {
    for i in 0..rom.len() {
      self.prg_rom[i] = rom[i];
    }
  }

  fn set_chr_rom(&mut self, rom: &[u8]) {
    if let ChrMem::Ram(mem) = &mut self.chr_mem {
      for i in 0..rom.len() {
        mem[i] = rom[i];
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use mapper::mapper::{Config, Mirroring, Mapper};
  use mapper::mmc1::MMC1;

  fn create_mmc() -> MMC1 {
    MMC1::new(Config{
      prg_rom_size: 0,
      chr_rom_size: 0,
      mirroring: Mirroring::Horizontal,
      contains_ram: true,
    })
  }

  #[test]
  fn test_write_register_control() {
    let mut mmc = create_mmc();
    mmc.register_control = 0;
    let mut data = 0xe;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xe);
    data = 0x1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xe);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xe);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xe);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xe);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0x1);
  }

  #[test]
  fn test_reset_shifter() {
    let mut mmc = create_mmc();
    mmc.register_control = 0;
    let mut data = 0xf;
    mmc.cpu_set_byte(0x8000, data);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0);
    // now, reset
    mmc.cpu_set_byte(0x8000, data | 0x80);
    assert_eq!(mmc.register_control, 0xc);
    data = 0x8;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xc);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xc);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xc);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0xc);
    data = data >> 1;
    mmc.cpu_set_byte(0x8000, data);
    assert_eq!(mmc.register_control, 0x8);
  }

  #[test]
  fn test_vertical_mirroring() {
    let mut mmc = create_mmc();
    mmc.register_control = 0xe;
    assert_eq!(mmc.ppu_get_mirrored_address(0x2000), 0x2000);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2040), 0x2040);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2111), 0x2111);
    assert_eq!(mmc.ppu_get_mirrored_address(0x23ff), 0x23ff);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2400), 0x2400);
    assert_eq!(mmc.ppu_get_mirrored_address(0x24ff), 0x24ff);
    assert_eq!(mmc.ppu_get_mirrored_address(0x27ff), 0x27ff);

    assert_eq!(mmc.ppu_get_mirrored_address(0x2800), 0x2000);
    assert_eq!(mmc.ppu_get_mirrored_address(0x28fc), 0x20fc);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2aaa), 0x22aa);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2c00), 0x2400);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2e20), 0x2620);
  }

  #[test]
  fn test_horizontal_mirroring() {
    let mut mmc = create_mmc();
    mmc.register_control = 0xf;
    assert_eq!(mmc.ppu_get_mirrored_address(0x2000), 0x2000);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2040), 0x2040);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2111), 0x2111);
    assert_eq!(mmc.ppu_get_mirrored_address(0x23ff), 0x23ff);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2400), 0x2000);
    assert_eq!(mmc.ppu_get_mirrored_address(0x24ff), 0x20ff);
    assert_eq!(mmc.ppu_get_mirrored_address(0x27ff), 0x23ff);

    assert_eq!(mmc.ppu_get_mirrored_address(0x2800), 0x2800);
    assert_eq!(mmc.ppu_get_mirrored_address(0x28fc), 0x28fc);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2aaa), 0x2aaa);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2c00), 0x2800);
    assert_eq!(mmc.ppu_get_mirrored_address(0x2e20), 0x2a20);
  }
}