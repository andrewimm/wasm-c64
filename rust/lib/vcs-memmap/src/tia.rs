pub enum ScanlineState {
  VSync,
  VBlank,
  HBlank,
  Pixel(u8, u8, u8), // x, y, color
  Overscan,
}

pub enum ExecState {
  Block,
  Run,
}

pub struct TIA {
  scanline: u16,
  horiz_clock: u8,

  vsync_enabled: bool,
  vblank_enabled: bool,
  block_until_hsync: bool,

  bg_color: u8,
  playfield_color: u8,
  player_0_color: u8,
  player_1_color: u8,

  playfield: [bool;20],
  playfield_reflect: bool,
  playfield_use_player_color: bool,
  playfield_has_priority: bool,

  player_0_graphics: u8,
  player_0_mirror: bool,
  player_0_position: u8,
  player_0_offset: u8,
  player_1_graphics: u8,
  player_1_mirror: bool,
  player_1_position: u8,
  player_1_offset: u8,
  missile_0_length: u8,
  missile_0_position: u8,
  missile_0_offset: u8,
  missile_0_enabled: bool,
  missile_1_length: u8,
  missile_1_position: u8,
  missile_1_offset: u8,
  missile_1_enabled: bool,
  ball_length: u8,
  ball_position: u8,
  ball_offset: u8,
  ball_enabled: bool,
}

impl TIA {
  pub fn new() -> TIA {
    TIA {
      scanline: 0,
      horiz_clock: 0,

      vsync_enabled: false,
      vblank_enabled: false,
      block_until_hsync: false,

      bg_color: 0,
      playfield_color: 0,
      player_0_color: 0,
      player_1_color: 0,

      playfield: [false;20],
      playfield_reflect: false,
      playfield_use_player_color: false,
      playfield_has_priority: false,

      player_0_graphics: 0,
      player_0_mirror: false,
      player_0_position: 240,
      player_0_offset: 0,
      player_1_graphics: 0,
      player_1_mirror: false,
      player_1_position: 240,
      player_1_offset: 0,
      missile_0_length: 1,
      missile_0_position: 240,
      missile_0_offset: 0,
      missile_0_enabled: false,
      missile_1_length: 1,
      missile_1_position: 240,
      missile_1_offset: 0,
      missile_1_enabled: false,
      ball_length: 1,
      ball_position: 240,
      ball_offset: 0,
      ball_enabled: false,
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr {
      0x00 => { // VSYNC
        if value & 0x02 != 0 {
          self.vsync_enabled = true;
          self.scanline = 0;
          self.horiz_clock = 0;
        } else {
          self.vsync_enabled = false;
        }
      },
      0x01 => { // VBLANK
        self.vblank_enabled = (value & 0xc2 != 0);
      },
      0x02 => { // WSYNC
        self.block_until_hsync = true;
      },
      0x03 => { // RSYNC

      },
      0x04 => { // NUSIZ 0
        let size = (value >> 4) & 3;
        self.missile_0_length = 1 << size;
      },
      0x05 => { // NUSIZ 1
        let size = (value >> 4) & 3;
        self.missile_1_length = 1 << size;
      },

      0x06 => { // Player 0 Color
        self.player_0_color = value;
      },
      0x07 => { // Player 1 Color
        self.player_1_color = value;
      },
      0x08 => { // Playfield Color
        self.playfield_color = value;
      },
      0x09 => { // BG Color
        //println!("SET BG {:x} {}", value, self.horiz_clock);
        self.bg_color = value;
      },
      0x0a => { // CTRLPF
        let ball_size = (value >> 4) & 3;
        self.ball_length = 1 << ball_size;
        self.playfield_reflect = value & 1 == 1;
        self.playfield_use_player_color = value & 2 == 2;
        self.playfield_has_priority = value & 4 == 4;
      },

      0x0d => { // Playfield Reg 0
        self.playfield[0] = value & 0x10 == 0x10;
        self.playfield[1] = value & 0x20 == 0x20;
        self.playfield[2] = value & 0x40 == 0x40;
        self.playfield[3] = value & 0x80 == 0x80;
      },
      0x0e => { // Playfield Reg 1
        self.playfield[4] = value & 0x80 == 0x80;
        self.playfield[5] = value & 0x40 == 0x40;
        self.playfield[6] = value & 0x20 == 0x20;
        self.playfield[7] = value & 0x10 == 0x10;
        self.playfield[8] = value & 0x08 == 0x08;
        self.playfield[9] = value & 0x04 == 0x04;
        self.playfield[10] = value & 0x02 == 0x02;
        self.playfield[11] = value & 0x01 == 0x01;
      },
      0x0f => { // Playfield Reg 2
        self.playfield[12] = value & 0x01 == 0x01;
        self.playfield[13] = value & 0x02 == 0x02;
        self.playfield[14] = value & 0x04 == 0x04;
        self.playfield[15] = value & 0x08 == 0x08;
        self.playfield[16] = value & 0x10 == 0x10;
        self.playfield[17] = value & 0x20 == 0x20;
        self.playfield[18] = value & 0x40 == 0x40;
        self.playfield[19] = value & 0x80 == 0x80;
      },
      0x10 => { // Reset Player 0
        self.player_0_position = self.horiz_clock + 9;
      },
      0x11 => { // Reset Player 1
        self.player_1_position = self.horiz_clock + 9;
      },
      0x12 => { // Reset Missile 0
        self.missile_0_position = self.horiz_clock + 9;
      },
      0x13 => { // Reset Missile 1
        self.missile_1_position = self.horiz_clock + 9;
      },
      0x14 => { // Reset Ball
        self.ball_position = self.horiz_clock + 9;
      },

      0x1b => { // Player 0 Graphics
        self.player_0_graphics = value;
      },
      0x1c => { // Player 1 Graphics
        self.player_1_graphics = value;
      },
      0x1d => { // Missile 0 Enable
        self.missile_0_enabled = value & 2 == 2;
      },
      0x1e => { // Missile 1 Enable
        self.missile_1_enabled = value & 2 == 2;
      }
      0x1f => {
        self.ball_enabled = value & 2 == 2;
      },
      0x20 => { // Player 0 Horiz Offset
        self.player_0_offset = value >> 4;
      },
      0x21 => { // Player 1 Horiz Offset
        self.player_1_offset = value >> 4;
      },
      0x22 => { // Missile 0 Horiz Offset
        self.missile_0_offset = value >> 4;
      },
      0x23 => { // Missile 1 Horiz Offset
        self.missile_1_offset = value >> 4;
      },
      0x24 => { // Ball Horiz Offset
        self.ball_offset = value >> 4;
      },

      0x2a => { // Apply Horiz Motion Offset
        if self.player_0_offset & 0x08 == 0x08 {
          self.player_0_position = self.player_0_position.wrapping_add(((!self.player_0_offset) & 0x7) + 1);
          if self.player_0_position > 228 {
            self.player_0_position -= 160;
          }
        } else {
          self.player_0_position = self.player_0_position.wrapping_sub(self.player_0_offset);
          if self.player_0_position < 68 {
            self.player_0_position += 160;
          }
        }
        if self.player_1_offset & 0x08 == 0x08 {
          self.player_1_position = self.player_1_position.wrapping_add(((!self.player_1_offset) & 0x7) + 1);
          if self.player_1_position > 228 {
            self.player_1_position -= 160;
          }
        } else {
          self.player_1_position = self.player_1_position.wrapping_sub(self.player_1_offset);
          if self.player_1_position < 68 {
            self.player_1_position += 160;
          }
        }
        if self.missile_0_offset & 0x08 == 0x08 {
          self.missile_0_position = self.missile_0_position.wrapping_add(((!self.missile_0_offset) & 0x7) + 1);
          if self.missile_0_position > 228 {
            self.missile_0_position -= 160;
          }
        } else {
          self.missile_0_position = self.missile_0_position.wrapping_sub(self.missile_0_offset);
          if self.missile_0_position < 68 {
            self.missile_0_position += 160;
          }
        }
        if self.missile_1_offset & 0x08 == 0x08 {
          self.missile_1_position = self.missile_1_position.wrapping_add(((!self.missile_1_offset) & 0x7) + 1);
          if self.missile_1_position > 228 {
            self.missile_1_position -= 160;
          }
        } else {
          self.missile_1_position = self.missile_1_position.wrapping_sub(self.missile_1_offset);
          if self.missile_1_position < 68 {
            self.missile_1_position += 160;
          }
        }
        if self.ball_offset & 0x08 == 0x08 {
          self.ball_position = self.ball_position.wrapping_add(((!self.ball_offset) & 0x7) + 1);
          if self.ball_position > 228 {
            self.ball_position -= 160;
          }
        } else {
          self.ball_position = self.ball_position.wrapping_sub(self.ball_offset);
          if self.ball_position < 68 {
            self.ball_position += 160;
          }
        }
      },
      0x2b => { // Horiz Motion Clear
        self.player_0_offset = 0;
        self.player_1_offset = 0;
        self.missile_0_offset = 0;
        self.missile_1_offset = 0;
        self.ball_offset = 0;
      },
      _ => (),
    }
  }

  pub fn increment_scanline(&mut self) {
    self.scanline += 1;
    if self.scanline >= 262 {
      self.scanline -= 262;
    }
  }

  pub fn increment_clock(&mut self, cycles: u8) {
    self.horiz_clock += cycles;
    if self.horiz_clock >= 228 {
      self.block_until_hsync = false;
      self.horiz_clock -= 228;
      self.increment_scanline();
    }
  }

  pub fn get_scanline_state(&self) -> ScanlineState {
    if self.scanline < 4 {
      return ScanlineState::VSync;
    }
    if self.scanline < 40 {
      return ScanlineState::VBlank;
    }
    if self.scanline < 232 {
      if self.horiz_clock < 68 {
        return ScanlineState::HBlank;
      }
      let x = self.horiz_clock - 68;
      let pixel_color = self.get_pixel_color(x);
      return ScanlineState::Pixel(x, self.scanline as u8 - 40, pixel_color);
    }
    return ScanlineState::Overscan;
  }

  pub fn get_exec_state(&self) -> ExecState {
    if self.block_until_hsync {
      return ExecState::Block;
    }
    return ExecState::Run;
  }

  fn get_pixel_color(&self, x_position: u8) -> u8 {
    if self.vblank_enabled {
      return 0;
    }
    let abs_pos = 68 + x_position;
    let position_0 = self.player_0_position;
    if position_0 < 228 {
      if abs_pos >= position_0 {
        let offset = abs_pos - position_0;
        if offset < 8 {
          if self.player_0_graphics & (1 << (7 - offset)) != 0 {
            return self.player_0_color;
          }
        }
      }
    }
    let position_1 = self.player_1_position;
    if position_1 < 228 {
      if abs_pos >= position_1 {
        let offset = abs_pos - position_1;
        if offset < 8 {
          if self.player_1_graphics & (1 << (7 - offset)) != 0 {
            return self.player_1_color;
          }
        }
      }
    }
    if self.missile_0_enabled {
      let m0 = self.missile_0_position;
      if m0 < 228 {
        if abs_pos >= m0 {
          let offset = abs_pos - m0;
          if offset <= self.missile_0_length {
            return self.player_0_color;
          }
        }
      }
    }
    if self.missile_1_enabled {
      let m1 = self.missile_1_position;
      if m1 < 228 {
        if abs_pos >= m1 {
          let offset = abs_pos - m1;
          if offset <= self.missile_1_length {
            return self.player_1_color;
          }
        }
      }
    }
    if self.ball_enabled {
      let b = self.ball_position;
      if b < 228 {
        if abs_pos >= b {
          let offset = abs_pos - b;
          if offset <= self.ball_length {
            return self.playfield_color;
          }
        }
      }
    }
    let mut playfield_index = x_position / 4;
    let mut left_half = true;
    if playfield_index >= 20 {
      if self.playfield_reflect {
        playfield_index = 19 - (playfield_index - 20);
      } else {
        playfield_index -= 20;
      }
      left_half = false;
    }
    if self.playfield[playfield_index as usize] {
      if self.playfield_use_player_color {
        return if left_half { self.player_0_color } else { self.player_1_color };
      }
      return self.playfield_color;
    }
    return self.bg_color;
  }
}