use crate::mapper::Mapper;
use crate::sprite::Sprite;

// VRAM increment after each access of PPUDATA
#[derive(PartialEq)]
enum VRAMIncrement {
  Across, // add 1
  Down, // add 32
}

#[derive(PartialEq)]
pub enum PatternTable {
  Base, // 0x0000
  Offset, // 0x1000
}

type Palette = (u8, u8, u8);

struct Colors {
  background: u8,
  bg_0: Palette,
  bg_1: Palette,
  bg_2: Palette,
  bg_3: Palette,
  sprite_0: Palette,
  sprite_1: Palette,
  sprite_2: Palette,
  sprite_3: Palette,
}

impl Colors {
  pub fn new() -> Colors {
    Colors {
      background: 0,
      bg_0: (0, 0, 0),
      bg_1: (0, 0, 0),
      bg_2: (0, 0, 0),
      bg_3: (0, 0, 0),
      sprite_0: (0, 0, 0),
      sprite_1: (0, 0, 0),
      sprite_2: (0, 0, 0),
      sprite_3: (0, 0, 0),
    }
  }
}

#[derive(Copy, Clone)]
struct SpritePixel {
  index: u8, // sprite index
  pixel: u8, // palette and color data
}

struct SpriteScanline {
  pub pixels: [SpritePixel;256],
}

impl SpriteScanline {
  pub fn new() -> SpriteScanline {
    SpriteScanline {
      pixels: [SpritePixel{index: 0xff, pixel: 0xff};256],
    }
  }

  pub fn get_pixel(&self, x: u8) -> &SpritePixel {
    &self.pixels[x as usize]
  }
}

pub struct PPU2 {
  // internal registers
  pub v: u16, // current vram address
  pub t: u16, // temporary vram address
  pub x: u8, // fine x-scroll (masked with 0x3)
  pub w: u8, // write toggle (0 or 1)
  status: u8,

  // masks
  greyscale: bool,
  show_left_bg: bool,
  show_left_sprites: bool,
  show_bg: bool,
  show_sprites: bool,
  emphasize_red: bool,
  emphasize_green: bool,
  emphasize_blue: bool,

  vram_increment: VRAMIncrement,
  sprite_pattern: PatternTable,
  background_pattern: PatternTable,
  double_height_sprites: bool,
  nmi_enabled: bool,
  oam_addr: u8,

  // drawing registers
  scanline: u16,
  cycle: u16,
  read_nametable: u8,
  read_attribute: u8,
  read_bitmap_low: u8,
  read_bitmap_high: u8,
  needs_interrupt: bool,

  colors: Colors,

  // memory
  ciram: [u8;0x800],
  sprites: Vec<Sprite>,
  secondary_oam: [Sprite;8],
  sprite_line: SpriteScanline,
  buffer: Box<[u8; 256 * 240]>,
}

impl PPU2 {
  pub fn new() -> PPU2 {
    let mut sprites = Vec::with_capacity(64);
    for _ in 0..64 {
      sprites.push(Sprite::new());
    }
    let secondary: [Sprite; 8] = [
      Sprite::new(),
      Sprite::new(),
      Sprite::new(),
      Sprite::new(),
      Sprite::new(),
      Sprite::new(),
      Sprite::new(),
      Sprite::new(),
    ];
    PPU2 {
      v: 0,
      t: 0,
      x: 0,
      w: 0,
      status: 0,

      greyscale: false,
      show_left_bg: false,
      show_left_sprites: false,
      show_bg: false,
      show_sprites: false,
      emphasize_red: false,
      emphasize_green: false,
      emphasize_blue: false,

      vram_increment: VRAMIncrement::Across,
      sprite_pattern: PatternTable::Base,
      background_pattern: PatternTable::Base,
      double_height_sprites: false,
      nmi_enabled: false,
      oam_addr: 0,

      scanline: 0,
      cycle: 0,
      read_nametable: 0,
      read_attribute: 0,
      read_bitmap_low: 0,
      read_bitmap_high: 0,
      needs_interrupt: false,

      colors: Colors::new(),

      ciram: [0;0x800],
      sprites: sprites,
      secondary_oam: secondary,
      sprite_line: SpriteScanline::new(),
      buffer: box [0; 256 * 240],
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8, mapper: &mut Box<Mapper>) {
    match addr & 7 {
      0 => { // controller
        let nametable = (value & 3) as u16;
        self.t = (self.t & 0xf3ff) | (nametable << 10);
        self.vram_increment = if value & 4 == 4 { VRAMIncrement::Down } else { VRAMIncrement::Across };
        self.sprite_pattern = if value & 8 == 8 { PatternTable::Offset } else { PatternTable::Base };
        self.background_pattern = if value & 0x10 == 0x10 { PatternTable::Offset } else { PatternTable::Base };
        self.double_height_sprites = value & 0x20 == 0x20;
        self.nmi_enabled = value & 0x80 == 0x80;
      },
      1 => { // masks
        self.greyscale = value & 1 == 1;
        self.show_left_bg = value & 2 == 2;
        self.show_left_sprites = value & 4 == 4;
        self.show_bg = value & 8 == 8;
        self.show_sprites = value & 0x10 == 0x10;
        self.emphasize_red = value & 0x20 == 0x20;
        self.emphasize_green = value & 0x40 == 0x40;
        self.emphasize_blue = value & 0x80 == 0x80;
      },
      2 => {

      },
      3 => {
        self.oam_addr = value;
      },
      4 => {
        self.write_oam(value);
      },
      5 => { // scroll positions
        if self.w == 0 {
          let fine = value & 7;
          let coarse = ((value & 0xf8) >> 3) as u16;
          self.t = (self.t & 0xffe0) | coarse;
          self.x = fine;
          self.w = 1;
        } else {
          let fine = (value & 7) as u16;
          let coarse = ((value & 0xf8) >> 3) as u16;
          self.t = (self.t & 0x0c1f) | (fine << 12) | (coarse << 5);
          self.w = 0;
        }
      },
      6 => { // PPU Address
        if self.w == 0 {
          let high = (value & 0x3f) as u16;
          self.t = (self.t & 0xff) | (high << 8);
          self.w = 1;
        } else {
          self.t = (self.t & 0xff00) | (value as u16);
          self.v = self.t;
          self.w = 0;
        }
      },
      7 => {
        if self.scanline < 240 {
          if self.show_bg || self.show_sprites {
            println!("CIRAM WRITE DURING DISPLAY");
          }
        }
        if self.v < 0x2000 {
          // pattern table
          mapper.ppu_set_byte(self.v, value);
        } else if self.v < 0x3eff {
          // nametables
          let mut orig = self.v;
          if orig >= 0x3000 {
            orig -= 0x1000;
          }
          let dest = mapper.ppu_get_mirrored_address(orig);
          self.ciram[(dest - 0x2000) as usize] = value;
        } else if self.v < 0x3fff {
          let dest = self.v & 0x1f;
          match dest {
            0x0 => self.colors.background = value,
            0x1 => self.colors.bg_0.0 = value,
            0x2 => self.colors.bg_0.1 = value,
            0x3 => self.colors.bg_0.2 = value,
            0x4 => (),
            0x5 => self.colors.bg_1.0 = value,
            0x6 => self.colors.bg_1.1 = value,
            0x7 => self.colors.bg_1.2 = value,
            0x8 => (),
            0x9 => self.colors.bg_2.0 = value,
            0xa => self.colors.bg_2.1 = value,
            0xb => self.colors.bg_2.2 = value,
            0xc => (),
            0xd => self.colors.bg_3.0 = value,
            0xe => self.colors.bg_3.1 = value,
            0xf => self.colors.bg_3.2 = value,
            0x10 => self.colors.background = value,
            0x11 => self.colors.sprite_0.0 = value,
            0x12 => self.colors.sprite_0.1 = value,
            0x13 => self.colors.sprite_0.2 = value,
            0x14 => (),
            0x15 => self.colors.sprite_1.0 = value,
            0x16 => self.colors.sprite_1.1 = value,
            0x17 => self.colors.sprite_1.2 = value,
            0x18 => (),
            0x19 => self.colors.sprite_2.0 = value,
            0x1a => self.colors.sprite_2.1 = value,
            0x1b => self.colors.sprite_2.2 = value,
            0x1c => (),
            0x1d => self.colors.sprite_3.0 = value,
            0x1e => self.colors.sprite_3.1 = value,
            0x1f => self.colors.sprite_3.2 = value,
            _ => (),
          };
        }
        let increment = if self.vram_increment == VRAMIncrement::Across { 1 } else { 32 };
        self.v = self.v.wrapping_add(increment);
      },
      _ => (),
    }
  }

  pub fn get_byte(&mut self, addr: u16, mapper: &Box<Mapper>) -> u8 {
    match addr & 7 {
      0 => {
        0
      },
      1 => {
        0
      },
      2 => {
        let status = self.status;
        self.status = status & 0x60;
        self.w = 0;
        status
      },
      3 => {
        0
      },
      4 => { // OAM Data
        let index = (self.oam_addr >> 2) as usize;
        self.sprites[index].get_oam_byte(self.oam_addr & 3)
      },
      5 => {
        0
      },
      6 => {
        0
      },
      7 => {
        if self.v < 0x2000 {
          // pattern table
          mapper.ppu_get_byte(self.v)
        } else if self.v < 0x3000 {
          // nametables
          let dest = mapper.ppu_get_mirrored_address(self.v);
          self.ciram[(dest - 0x2000) as usize]
        } else {
          0
        }
      },
      _ => 0,
    }
  }

  pub fn increment_clock(&mut self, mapper: &Box<Mapper>) {
    if self.scanline < 240 {
      // Visible scanline
      if self.cycle == 0 {
        // idle on cycle 0
      } else if self.cycle <= 256 {
        if self.show_bg || self.show_sprites {
          let draw_cycle = self.cycle - 1;
          let step = draw_cycle & 7;
          self.load_tile_data((step >> 1) as u8, mapper);

          if step == 7 {
            if self.show_bg || self.show_sprites {
              // draw tile pixels
              let mut attr = self.read_attribute;
              if self.v & 2 == 2 {
                attr = attr >> 2;
              }
              if self.v & 0x40 == 0x40 {
                attr = attr >> 4;
              }
              let palette = match attr & 3 {
                1 => self.colors.bg_1,
                2 => self.colors.bg_2,
                3 => self.colors.bg_3,
                _ => self.colors.bg_0,
              };
              // need to account for x scroll
              for i in 0..8 {
                let buffer_addr = self.scanline * 256 + draw_cycle - 7 + i as u16;

                let mut sprite_px = 0;
                let mut sprite_has_priority = false;
                let mut sprite_index = 0xff;
                let mut bg_px = 0;
                if self.show_sprites {
                  let px = self.sprite_line.get_pixel(draw_cycle as u8 - 7 + i);
                  sprite_index = px.index;
                  sprite_has_priority = px.pixel & 0x10 == 0x10;
                  if px.index != 0xff && px.pixel & 3 != 0 {
                    // a sprite is on this pixel
                    sprite_px = px.pixel & 3;
                    let sprite_pal = match (px.pixel >> 2) & 3 {
                      1 => self.colors.sprite_1,
                      2 => self.colors.sprite_2,
                      3 => self.colors.sprite_3,
                      _ => self.colors.sprite_0,
                    };
                    self.buffer[buffer_addr as usize] = match sprite_px {
                      1 => sprite_pal.0,
                      2 => sprite_pal.1,
                      3 => sprite_pal.2,
                      _ => self.colors.background,
                    };
                  }
                }
                if self.show_bg {
                  // load bg
                  let high = (self.read_bitmap_high >> (7 - i)) & 1;
                  let low = (self.read_bitmap_low >> (7 - i)) & 1;
                  bg_px = (high << 1) | low;
                  if sprite_index == 0 && sprite_px != 0 && bg_px != 0 {
                    self.status = self.status | 0x40;
                  }
                }
                if sprite_px == 0 || (!sprite_has_priority && bg_px > 0) {
                  self.buffer[buffer_addr as usize] = match bg_px {
                    1 => palette.0,
                    2 => palette.1,
                    3 => palette.2,
                    _ => self.colors.background,
                  };
                }
              }
            }
            // increment scroll
            
            if (self.v & 0x1f) == 0x1f {
              self.v = self.v & !0x1f;
              self.v = self.v ^ 0x400;
            } else {
              self.v += 1;
            }
            if self.cycle == 256 {
              if (self.v & 0x7000) != 0x7000 {
                // if fine y < 7, increment fine y
                self.v += 0x1000;
              } else {
                // reset fine to 0
                self.v = self.v & !0x7000;
                let mut y = (self.v & 0x3e0) >> 5;
                if y == 29 {
                  y = 0;
                  self.v = self.v ^ 0x800;
                } else if y == 31 {
                  y = 0;
                } else {
                  y += 1;
                }
                self.v = (self.v & !0x3e0) | (y << 5);
              }
            }
          }
        }
      } else if self.cycle <= 320 {
        if self.cycle == 257 {
          if self.show_bg || self.show_sprites {
            let mask = 0x41f;
            self.v = (self.v & !mask) | (self.t & mask);
          }
        }
        if self.cycle == 320 {
          // load sprite data for next line
          if self.show_sprites {
            self.copy_to_secondary_oam(mapper);
          }
        }
      } else if self.cycle <= 336 {
        // load initial tile data for next line
      }
    } else if self.scanline == 240 {
      // do nothing
    } else {
      // VBlank
      if self.scanline == 241 && self.cycle == 1 {
        // mark the vblank
        self.status = self.status | 0x80;
        if self.nmi_enabled {
          self.needs_interrupt = true;
        }
      }
      if self.scanline == 261 {
        if self.cycle == 1 {
          self.status = 0;
          self.needs_interrupt = false;
          self.clear_secondary_oam();
        }
        if self.cycle >= 280 && self.cycle <= 304 {
          if self.show_bg || self.show_sprites {
            let mask = 0x7be0;
            self.v = (self.v & !mask) | (self.t & mask);
          }
        }
      }
    }

    self.cycle += 1;
    if self.cycle >= 341 {
      self.cycle = 0;
      self.scanline += 1;
    }
    if self.scanline >= 262 {
      self.scanline = 0;
    }
  }

  fn load_tile_data(&mut self, step: u8, mapper: &Box<Mapper>) {
    match step {
      0 => {
        // fetch nametable byte
        let nametable_addr = mapper.ppu_get_mirrored_address(0x2000 | (self.v & 0x0fff));
        self.read_nametable = self.ciram[nametable_addr as usize - 0x2000];
      },
      1 => {
        // fetch attribute byte
        let attr_addr = mapper.ppu_get_mirrored_address(0x23c0 | (self.v & 0x0c00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07));
        self.read_attribute = self.ciram[attr_addr as usize - 0x2000];
      },
      2 => {
        // fetch low tile byte
        let y_offset = (self.v >> 12) & 0xf;
        let base: u16 = if self.background_pattern == PatternTable::Base { 0 } else { 0x1000 };
        let tile = (self.read_nametable as u16) << 4;
        let addr = base | tile | y_offset;
        self.read_bitmap_low = mapper.ppu_get_byte(addr);
      },
      3 => {
        // fetch high tile byte
        let y_offset = (self.v >> 12) & 0xf;
        let base: u16 = if self.background_pattern == PatternTable::Base { 0 } else { 0x1000 };
        let tile = (self.read_nametable as u16) << 4;
        let addr = base | tile | 8 | y_offset;
        self.read_bitmap_high = mapper.ppu_get_byte(addr);
      },
      _ => (),
    }
  }

  pub fn write_oam(&mut self, value: u8) {
    let index = (self.oam_addr >> 2) as usize;
    self.sprites[index].set_oam_byte(self.oam_addr & 3, value);
    self.oam_addr = self.oam_addr.wrapping_add(1);
  }

  fn clear_secondary_oam(&mut self) {
    for i in 0..256 {
      self.sprite_line.pixels[i].index = 0xff;
    }
  }

  fn copy_to_secondary_oam(&mut self, mapper: &Box<Mapper>) {
    let mut s = 0;
    let mut index = 0;
    for i in 0..256 {
      self.sprite_line.pixels[i].index = 0xff;
    }
    let height = if self.double_height_sprites { 16 } else { 8 };
    for sprite in self.sprites.iter() {
      let ypos = sprite.y_position as u16;
      if ypos <= self.scanline {
        let mut offset = self.scanline - ypos;
        if offset < height {
          if sprite.flip_vertical {
            offset = height - offset - 1;
          }
          if s < 8 {
            let bank = ((sprite.tile_index & 1) as u16) << 12;
            let tile_index = ((sprite.tile_index as u16 & 0xfffe) * 16 + (offset & 0x7));
            let mut addr = bank | tile_index;
            if offset >= 8 {
              addr += 16;
            }
            let palette = sprite.palette << 2;
            let tile_low = mapper.ppu_get_byte(addr);
            let tile_high = mapper.ppu_get_byte(addr + 8);
            for i in 0..8 {
              let px = i + sprite.x_position as usize;
              if px < 256 {
                if self.sprite_line.pixels[px].index == 0xff {
                  let shift = if sprite.flip_horizontal { i } else { 7 - i };
                  self.sprite_line.pixels[px].index = index;
                  let low = (tile_low >> shift) & 1;
                  let high = (tile_high >> shift) & 1;
                  let priority = if sprite.has_bg_priority { 0x10 } else { 0 };
                  self.sprite_line.pixels[px].pixel = priority | palette | (high << 1) | low;
                }
              }
            }
            s += 1;
          } else {
            // Set sprite overflow bit
            self.status = self.status | 0x20;
          }
        }
      }
      index += 1;
    }
  }

  pub fn buffer_ptr(&self) -> *const u8 {
    &self.buffer[0] as *const u8
  }

  pub fn in_vblank(&self) -> bool {
    self.scanline >= 241
  }

  pub fn should_interrupt(&mut self) -> bool {
    if self.needs_interrupt {
      self.needs_interrupt = false;
      true
    } else {
      false
    }
  }

  pub fn dump(&self) {
    println!("DUMP CIRAM");
    for j in 0..30 {
      for i in 0..32 {
        print!("{:x} ", self.ciram[j * 32 + i]);
      }
      print!("\n");
    }
    println!("ATTR TABLE:");
    for j in 960..1024 {
      print!("{:x} ", self.ciram[j]);
    }
    print!("\n");
  }
}