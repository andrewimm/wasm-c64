use crate::mapper::Mapper;

// Address of the base nametable
enum NametableAddress {
  Table0, // 0x2000
  Table1, // 0x2400
  Table2, // 0x2800
  Table3, // 0x2c00
}

// VRAM increment after each access of PPUDATA
#[derive(PartialEq)]
enum VRAMIncrement {
  Across, // add 1
  Down, // add 32
}

// Address of patterns for square 8x8 sprites
#[derive(PartialEq)]
enum SpriteTableAddress {
  Base, // 0x0000
  Offset, // 0x1000
}

// Address of pattersn for background
#[derive(PartialEq)]
enum BackgroundTableAddress {
  Base, // 0x0000
  Offset, // 0x1000
}

pub struct PPU {
  nametable_address: NametableAddress,
  vram_increment: VRAMIncrement,
  square_sprite_address: SpriteTableAddress,
  background_address: BackgroundTableAddress,
  double_width_sprites: bool,
  nmi_enabled: bool,

  greyscale: bool,
  show_left_sprites: bool,
  show_left_bg: bool,
  show_bg: bool,
  show_sprites: bool,
  emphasize_red: bool,
  emphasize_green: bool,
  emphasize_blue: bool,

  pub oamaddr: u8,
  oamdata: u8,

  ppuaddr: u8,
  ppudata: u8,

  latch: u8,
  scroll_x: u8,
  scroll_y: u8,
  write_scroll_y: bool,

  vblank: bool,
  scanline: u16,
  cpu_cycles: u8,
  needs_interrupt: bool,

  ppu_address: u16,
  write_ppu_address_high: bool,
  
  oam: [u8;0x100],
  ciram: [u8;0x800],
}

impl PPU {
  pub fn new() -> PPU {
    PPU {
      nametable_address: NametableAddress::Table0,
      vram_increment: VRAMIncrement::Across,
      square_sprite_address: SpriteTableAddress::Base,
      background_address: BackgroundTableAddress::Base,
      double_width_sprites: false,
      nmi_enabled: false,

      greyscale: false,
      show_left_sprites: false,
      show_left_bg: false,
      show_bg: false,
      show_sprites: false,
      emphasize_red: false,
      emphasize_green: false,
      emphasize_blue: false,

      oamaddr: 0,
      oamdata: 0,

      ppuaddr: 0,
      ppudata: 0,

      latch: 0,
      scroll_x: 0,
      scroll_y: 0,
      write_scroll_y: false,

      vblank: false,
      scanline: 0,
      cpu_cycles: 0,
      needs_interrupt: false,

      ppu_address: 0,
      write_ppu_address_high: true,
      
      oam: [0;0x100],
      ciram: [0;0x800],
    }
  }

  pub fn add_cpu_cycles(&mut self, cycles: u8) {
    self.cpu_cycles += cycles;
    if self.cpu_cycles > 113 {
      self.cpu_cycles = 0;
      self.increment_scanline();
    }
  }

  pub fn set_scanline(&mut self, line: u16) {
    self.scanline = line;
    self.vblank = line > 240 && line < 261;
  }

  pub fn increment_scanline(&mut self) {
    self.scanline = self.scanline + 1;
    if self.scanline > 262 {
      self.scanline = 0;
    }
    if self.scanline == 240 {
      self.vblank = true;
      if self.nmi_enabled {
        self.needs_interrupt = true;
      }
    }
    if self.scanline == 261 {
      self.vblank = false;
    }
  }

  pub fn should_interrupt(&mut self) -> bool {
    if self.needs_interrupt {
      self.needs_interrupt = false;
      true
    } else {
      false
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8, mapper: &mut Box<Mapper>) {
    match addr & 7 {
      0 => { // controller
        self.nametable_address = match value & 3 {
          1 => NametableAddress::Table1,
          2 => NametableAddress::Table2,
          3 => NametableAddress::Table3,
          _ => NametableAddress::Table0,
        };
        self.vram_increment = if value & 4 == 4 { VRAMIncrement::Down } else { VRAMIncrement::Across };
        self.square_sprite_address = if value & 8 == 8 { SpriteTableAddress::Offset } else { SpriteTableAddress::Base };
        self.background_address = if value & 0x10 == 0x10 { BackgroundTableAddress::Offset } else { BackgroundTableAddress::Base };
        self.double_width_sprites = value & 0x20 == 0x20;
        // ignoring EXT pins for now, they're unused in stock NES
        self.nmi_enabled = value & 0x80 == 0x80;
        self.latch = value;
      },
      1 => { // rendering mask
        self.greyscale = value & 1 == 1;
        self.show_left_bg = value & 2 == 2;
        self.show_left_sprites = value & 4 == 4;
        self.show_bg = value & 8 == 8;
        self.show_sprites = value & 0x10 == 0x10;
        self.emphasize_red = value & 0x20 == 0x20;
        self.emphasize_green = value & 0x40 == 0x40;
        self.emphasize_blue = value & 0x80 == 0x80;
        self.latch = value;
      },
      2 => { // status, read-only
        self.latch = value;
      },
      3 => { // set OAM Addr
        self.oamaddr = value;
        self.latch = value;
      },
      4 => { // set OAM Data
        self.oam[self.oamaddr as usize] = value;
        self.oamaddr.wrapping_add(1);

        self.latch = value;
      },
      5 => { // PPU Scroll position
        if self.write_scroll_y {
          self.scroll_y = value;
        } else {
          self.scroll_x = value;
        }
        self.write_scroll_y = !self.write_scroll_y;
        self.latch = value;
      },
      6 => { // PPU Address
        if self.write_ppu_address_high {
          let low = self.ppu_address & 0xff;
          self.ppu_address = (low | ((value as u16) << 8)) & 0x3fff;
        } else {
          let high = self.ppu_address & 0xff00;
          self.ppu_address = (high | (value as u16)) & 0x3fff;
        }
        self.write_ppu_address_high = !self.write_ppu_address_high;
        self.latch = value;
      },
      7 => { // PPU Data
        if self.ppu_address < 0x2000 {
          // pattern table
          mapper.ppu_set_byte(self.ppu_address, value);
        } else if self.ppu_address < 0x3000 {
          // nametables
          let dest = mapper.ppu_get_mirrored_address(self.ppu_address);
          self.ciram[(dest - 0x2000) as usize] = value;
        }
        let increment = if self.vram_increment == VRAMIncrement::Across { 1 } else { 32 };
        self.ppu_address = self.ppu_address.wrapping_add(increment);
      },
      _ => (),
    };
  }

  pub fn get_byte(&mut self, addr: u16, mapper: &Box<Mapper>) -> u8 {
    match addr % 8 {
      0 => { // controller
        self.latch
      },
      1 => { // rendering mask
        self.latch
      },
      2 => { // status, read-only
        let low = self.latch & 0x1f;
        let vblank_bit = if self.vblank { 0x80 } else { 0 };
        self.vblank = false;
        self.write_scroll_y = false;
        self.write_ppu_address_high = true;
        low | vblank_bit
      },
      3 => { // OAM Addr
        self.latch
      },
      4 => { // OAM Data
        self.oam[self.oamaddr as usize]
      },
      5 => { // PPU Scroll position
        self.latch
      },
      6 => { // PPU Address
        self.latch
      },
      7 => { // PPU Data
        if self.ppu_address < 0x2000 {
          // pattern table
          mapper.ppu_get_byte(self.ppu_address)
        } else if self.ppu_address < 0x3000 {
          // nametables
          let dest = mapper.ppu_get_mirrored_address(self.ppu_address);
          self.ciram[(dest - 0x2000) as usize]
        } else {
          0
        }
      },
      _ => 0,
    }
  }
}
