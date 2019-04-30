#[derive(Copy, Clone)]
pub struct Sprite {
  pub y_position: u8,
  pub x_position: u8,
  pub tile_index: u8,
  pub palette: u8,
  pub flip_horizontal: bool,
  pub flip_vertical: bool,
  pub has_bg_priority: bool,
}

impl Sprite {
  pub fn new() -> Sprite {
    Sprite {
      y_position: 0xff,
      x_position: 0,
      tile_index: 0,
      palette: 0,
      flip_horizontal: false,
      flip_vertical: false,
      has_bg_priority: true,
    }
  }

  pub fn set_oam_byte(&mut self, addr: u8, value: u8) {
    match addr {
      0 => self.y_position = value,
      1 => self.tile_index = value,
      2 => {
        self.palette = value & 3;
        self.has_bg_priority = value & 0x20 == 0;
        self.flip_horizontal = value & 0x40 == 0x40;
        self.flip_vertical = value & 0x80 == 0x80;
      },
      3 => self.x_position = value,
      _ => (),
    };
  }

  pub fn get_oam_byte(&mut self, addr: u8) -> u8 {
    match addr {
      0 => self.y_position,
      1 => self.tile_index,
      2 => {
        let priority_bit = if self.has_bg_priority { 0 } else { 0x20 };
        let flip_horiz_bit = if self.flip_horizontal { 0x40 } else { 0 };
        let flip_vert_bit = if self.flip_vertical { 0x80 } else { 0 };
        self.palette | priority_bit | flip_horiz_bit | flip_vert_bit
      }
      3 => self.x_position,
      _ => 0,
    }
  }
}