use std::cmp;

pub struct Sprite {
  pub x: u16,
  pub y: u8,
  pub color: u8,
  pub enabled: bool,
  pub double_height: bool,
  pub double_width: bool,
}

impl Sprite {
  pub fn new() -> Sprite {
    return Sprite {
      x: 0,
      y: 0,
      color: 0,
      enabled: false,
      double_height: false,
      double_width: false,
    };
  }

  pub fn set_x_low(&mut self, low: u8) {
    let x = self.x & 0x100 | (low as u16);
    self.x = x;
  }

  pub fn set_x_high(&mut self, high: u8) {
    let x = self.x & 0xff | ((high as u16) << 8);
    self.x = x;
  }

  pub fn set_y(&mut self, y: u8) {
    self.y = y;
  }
}

#[derive(PartialEq)]
pub enum ScreenHeight {
  TwentyFour,
  TwentyFive,
}

#[derive(PartialEq)]
pub enum ScreenWidth {
  ThirtyEight,
  Forty,
}

#[derive(PartialEq)]
pub enum Mode {
  Text,
  Bitmap,
}

pub enum DerivedGraphicsMode {
  StandardCharMode = 0,
  MulticolorCharMode = 1,
  StandardBitmapMode = 2,
  MulticolorBitmapMode = 3,
  ExtendedBackgroundColorMode = 4,
  Invalid = 5,
}

pub struct VIC {
  pub sprites: [Sprite;8],
  pub vertical_scroll: u8,
  screen_height: ScreenHeight,
  mode: Mode,
  pub screen_on: bool,
  extended_bg: bool,
  raster_interrupt_line: u16,
  current_raster_line: u16,
  pub horizontal_scroll: u8,
  screen_width: ScreenWidth,
  multicolor: bool,

  pub border_color: u8,
  pub background_color: u8,
  pub background_color_e1: u8,
  pub background_color_e2: u8,
  pub background_color_e3: u8,
  pub sprite_color_e1: u8,
  pub sprite_color_e2: u8,
}

impl VIC {
  pub fn new() -> VIC {
    return VIC {
      sprites: [
        Sprite::new(),
        Sprite::new(),
        Sprite::new(),
        Sprite::new(),
        Sprite::new(),
        Sprite::new(),
        Sprite::new(),
        Sprite::new(),
      ],
      vertical_scroll: 0x3,
      screen_height: ScreenHeight::TwentyFour,
      mode: Mode::Text,
      screen_on: true,
      extended_bg: false,
      raster_interrupt_line: 0,
      current_raster_line: 0,
      horizontal_scroll: 0,
      screen_width: ScreenWidth::Forty,
      multicolor: false,

      border_color: 0,
      background_color: 0,
      background_color_e1: 0,
      background_color_e2: 0,
      background_color_e3: 0,
      sprite_color_e1: 0,
      sprite_color_e2: 0,
    };
  }

  pub fn get_byte(&self, addr: u16) -> u8 {
    match addr {
      0x00 => (self.sprites[0].x & 0xff) as u8,
      0x01 => self.sprites[0].y,
      0x02 => (self.sprites[1].x & 0xff) as u8,
      0x03 => self.sprites[1].y,
      0x04 => (self.sprites[2].x & 0xff) as u8,
      0x05 => self.sprites[2].y,
      0x06 => (self.sprites[3].x & 0xff) as u8,
      0x07 => self.sprites[3].y,
      0x08 => (self.sprites[4].x & 0xff) as u8,
      0x09 => self.sprites[4].y,
      0x0a => (self.sprites[5].x & 0xff) as u8,
      0x0b => self.sprites[5].y,
      0x0c => (self.sprites[6].x & 0xff) as u8,
      0x0d => self.sprites[6].y,
      0x0e => (self.sprites[7].x & 0xff) as u8,
      0x0f => self.sprites[7].y,
      0x10 =>
        (((self.sprites[0].x & 0x100) >> 8) |
        ((self.sprites[1].x & 0x100) >> 7) |
        ((self.sprites[2].x & 0x100) >> 6) |
        ((self.sprites[3].x & 0x100) >> 5) |
        ((self.sprites[4].x & 0x100) >> 4) |
        ((self.sprites[5].x & 0x100) >> 3) |
        ((self.sprites[6].x & 0x100) >> 2) |
        ((self.sprites[7].x & 0x100) >> 1)) as u8,
      0x11 => {
        let mut register = self.vertical_scroll;
        if self.screen_height == ScreenHeight::TwentyFive {
          register = register | 0x8;
        }
        if self.screen_on {
          register = register | 0x10;
        }
        if self.mode == Mode::Bitmap {
          register = register | 0x20;
        }
        if self.extended_bg {
          register = register | 0x40;
        }
        let raster_high = ((self.current_raster_line & 0x100) >> 1) as u8;
        register | raster_high
      },
      0x12 => (self.current_raster_line & 0xff) as u8,
      0x13 => 0, // light pen low
      0x14 => 0, // light pen high
      0x15 => {
        let mut enabled: u8 = 0;
        if self.sprites[0].enabled {
          enabled = enabled | 1;
        }
        if self.sprites[1].enabled {
          enabled = enabled | 2;
        }
        if self.sprites[2].enabled {
          enabled = enabled | 4;
        }
        if self.sprites[3].enabled {
          enabled = enabled | 8;
        }
        if self.sprites[4].enabled {
          enabled = enabled | 16;
        }
        if self.sprites[5].enabled {
          enabled = enabled | 32;
        }
        if self.sprites[6].enabled {
          enabled = enabled | 64;
        }
        if self.sprites[7].enabled {
          enabled = enabled | 128;
        }
        enabled
      },
      0x16 => {
        let mut register = self.horizontal_scroll | 0xc0;
        if self.screen_width == ScreenWidth::Forty {
          register = register | 0x8;
        }
        if self.multicolor {
          register = register | 0x10;
        }
        register
      },
      0x17 => {
        let mut double_height: u8 = 0;
        if self.sprites[0].double_height {
          double_height = double_height | 1;
        }
        if self.sprites[1].double_height {
          double_height = double_height | 2;
        }
        if self.sprites[2].double_height {
          double_height = double_height | 4;
        }
        if self.sprites[3].double_height {
          double_height = double_height | 8;
        }
        if self.sprites[4].double_height {
          double_height = double_height | 16;
        }
        if self.sprites[5].double_height {
          double_height = double_height | 32;
        }
        if self.sprites[6].double_height {
          double_height = double_height | 64;
        }
        if self.sprites[7].double_height {
          double_height = double_height | 128;
        }
        double_height
      },
      0x18 => 0,
      0x19 => {
        let mut status = 0;
        if self.current_raster_line == self.raster_interrupt_line {
          status = status | 1;
        }
        // todo: handle other interrupts
        status
      },
      0x1a => {
        // enabled interrupts
        0
      },
      0x1b => {
        // sprite priority
        0
      },
      0x1c => {
        // sprite multicolor
        0
      },
      0x1d => {
        let mut double_width: u8 = 0;
        if self.sprites[0].double_width {
          double_width = double_width | 1;
        }
        if self.sprites[1].double_width {
          double_width = double_width | 2;
        }
        if self.sprites[2].double_width {
          double_width = double_width | 4;
        }
        if self.sprites[3].double_width {
          double_width = double_width | 8;
        }
        if self.sprites[4].double_width {
          double_width = double_width | 16;
        }
        if self.sprites[5].double_width {
          double_width = double_width | 32;
        }
        if self.sprites[6].double_width {
          double_width = double_width | 64;
        }
        if self.sprites[7].double_width {
          double_width = double_width | 128;
        }
        double_width
      },
      0x1e => {
        // current sprite-sprite collisions
        0
      },
      0x1f => {
        // current sprite-background collisions
        0
      },
      0x20 => self.border_color & 0xf,
      0x21 => self.background_color & 0xf,
      0x22 => self.background_color_e1 & 0xf,
      0x23 => self.background_color_e2 & 0xf,
      0x24 => self.background_color_e3 & 0xf,
      0x25 => self.sprite_color_e1 & 0xf,
      0x26 => self.sprite_color_e2 & 0xf,
      0x27 => self.sprites[0].color & 0xf,
      0x28 => self.sprites[1].color & 0xf,
      0x29 => self.sprites[2].color & 0xf,
      0x2a => self.sprites[3].color & 0xf,
      0x2b => self.sprites[4].color & 0xf,
      0x2c => self.sprites[5].color & 0xf,
      0x2d => self.sprites[6].color & 0xf,
      0x2e => self.sprites[7].color & 0xf,
      _ => 0,
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr {
      0x00 => self.sprites[0].set_x_low(value),
      0x01 => self.sprites[0].set_y(value),
      0x02 => self.sprites[1].set_x_low(value),
      0x03 => self.sprites[1].set_y(value),
      0x04 => self.sprites[2].set_x_low(value),
      0x05 => self.sprites[2].set_y(value),
      0x06 => self.sprites[3].set_x_low(value),
      0x07 => self.sprites[3].set_y(value),
      0x08 => self.sprites[4].set_x_low(value),
      0x09 => self.sprites[4].set_y(value),
      0x0a => self.sprites[5].set_x_low(value),
      0x0b => self.sprites[5].set_y(value),
      0x0c => self.sprites[6].set_x_low(value),
      0x0d => self.sprites[6].set_y(value),
      0x0e => self.sprites[7].set_x_low(value),
      0x0f => self.sprites[7].set_y(value),
      0x10 => {
        self.sprites[0].set_x_high(value & 1);
        self.sprites[1].set_x_high(value & 2);
        self.sprites[2].set_x_high(value & 4);
        self.sprites[3].set_x_high(value & 8);
        self.sprites[4].set_x_high(value & 16);
        self.sprites[5].set_x_high(value & 32);
        self.sprites[6].set_x_high(value & 64);
        self.sprites[7].set_x_high(value & 128);
      },
      0x11 => {
        self.vertical_scroll = value & 0x7;
        self.screen_height = if value & 0x8 == 0 { ScreenHeight::TwentyFour } else { ScreenHeight::TwentyFive };
        self.screen_on = value & 0x10 == 0x10;
        self.mode = if value & 0x20 == 0 { Mode::Text } else { Mode::Bitmap };
        self.extended_bg = value & 0x40 == 0x40;
        let raster_high = ((value as u16) & 0x80) << 1;
        let line = self.raster_interrupt_line & 0xf | raster_high;
        self.raster_interrupt_line = line;
      },
      0x12 => {
        let line = self.raster_interrupt_line & 0xf0 | (value as u16);
        self.raster_interrupt_line = line;
      },
      0x13 => (),
      0x14 => (),
      0x15 => {
        self.sprites[0].enabled = (value & 1) != 0;
        self.sprites[1].enabled = (value & 2) != 0;
        self.sprites[2].enabled = (value & 4) != 0;
        self.sprites[3].enabled = (value & 8) != 0;
        self.sprites[4].enabled = (value & 16) != 0;
        self.sprites[5].enabled = (value & 32) != 0;
        self.sprites[6].enabled = (value & 64) != 0;
        self.sprites[7].enabled = (value & 128) != 0;
      },
      0x16 => {
        self.horizontal_scroll = value & 0x7;
        self.screen_width = if value & 0x8 == 0 { ScreenWidth::ThirtyEight } else { ScreenWidth::Forty };
        self.multicolor = value & 0x10 == 0x10;
      },
      0x17 => {
        self.sprites[0].double_height = (value & 1) != 0;
        self.sprites[1].double_height = (value & 2) != 0;
        self.sprites[2].double_height = (value & 4) != 0;
        self.sprites[3].double_height = (value & 8) != 0;
        self.sprites[4].double_height = (value & 16) != 0;
        self.sprites[5].double_height = (value & 32) != 0;
        self.sprites[6].double_height = (value & 64) != 0;
        self.sprites[7].double_height = (value & 128) != 0;
      },
      0x18 => {
        // memory setup, to be implemented
      },
      0x19 => {
        // acknowledge interrupts
      },
      0x1a => {
        // enable interrupts
      },
      0x1b => {
        // sprite priority
      },
      0x1c => {
        // sprite multicolor
      },
      0x1d => {
        self.sprites[0].double_width = (value & 1) != 0;
        self.sprites[1].double_width = (value & 2) != 0;
        self.sprites[2].double_width = (value & 4) != 0;
        self.sprites[3].double_width = (value & 8) != 0;
        self.sprites[4].double_width = (value & 16) != 0;
        self.sprites[5].double_width = (value & 32) != 0;
        self.sprites[6].double_width = (value & 64) != 0;
        self.sprites[7].double_width = (value & 128) != 0;
      },
      0x1e => {
        // enable sprite-sprite collision
      },
      0x1f => {
        // enable sprite-background collision
      },
      0x20 => self.border_color = value & 0xf,
      0x21 => self.background_color = value & 0xf,
      0x22 => self.background_color_e1 = value & 0xf,
      0x23 => self.background_color_e2 = value & 0xf,
      0x24 => self.background_color_e3 = value & 0xf,
      0x25 => self.sprite_color_e1 = value & 0xf,
      0x26 => self.sprite_color_e2 = value & 0xf,
      0x27 => self.sprites[0].color = value & 0xf,
      0x28 => self.sprites[1].color = value & 0xf,
      0x29 => self.sprites[2].color = value & 0xf,
      0x2a => self.sprites[3].color = value & 0xf,
      0x2b => self.sprites[4].color = value & 0xf,
      0x2c => self.sprites[5].color = value & 0xf,
      0x2d => self.sprites[6].color = value & 0xf,
      0x2e => self.sprites[7].color = value & 0xf,
      _ => (),
    };
  }

  pub fn get_graphics_mode_bits(&self) -> u8 {
    let mcm = if self.multicolor { 1 } else { 0 };
    let bmm = if self.mode == Mode::Bitmap { 2 } else { 0 };
    let ecm = if self.extended_bg { 4 } else { 0 };
    mcm | bmm | ecm
  }

  pub fn get_graphics_mode(&self) -> DerivedGraphicsMode {
    match self.get_graphics_mode_bits() {
      0 => DerivedGraphicsMode::StandardCharMode,
      1 => DerivedGraphicsMode::MulticolorCharMode,
      2 => DerivedGraphicsMode::StandardBitmapMode,
      3 => DerivedGraphicsMode::MulticolorBitmapMode,
      4 => DerivedGraphicsMode::ExtendedBackgroundColorMode,
      _ => DerivedGraphicsMode::Invalid,
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn sprite_position() {

  }
}