
pub struct CIA {
  // CIA 1
  keys: [u8;8], // 64 bits for key matrix, in 8 8-bit rows
  port_a_1: u8,
  mask_a_1: u8,

  timer_a_1_interrupt: bool,
  timer_a_1_interrupt_enabled: bool,
  timer_a_1_enabled: bool,
  timer_a_1_latch: u16,
  timer_a_1_value: u16,
  timer_a_1_restart: bool,
  timer_a_1_register: u8,

  // CIA 2
}

impl CIA {
  pub fn new() -> CIA {
    return CIA {
      keys: [0, 0, 0, 0, 0, 0, 0, 0],
      port_a_1: 0,
      mask_a_1: 0xff,
      timer_a_1_interrupt: false,
      timer_a_1_interrupt_enabled: false,
      timer_a_1_enabled: false,
      timer_a_1_latch: 0,
      timer_a_1_value: 0,
      timer_a_1_restart: false,
      timer_a_1_register: 0,
    };
  }

  pub fn get_byte(&mut self, addr: u16) -> u8 {
    match addr % 16 {
      0x00 => self.port_a_1,
      0x01 => {
        let port_inv = !self.port_a_1;
        let mut col = 0;
        let mut row = 0;
        while col < 8 {
          let shift = port_inv >> col;
          if shift & 1 == 1 {
            row = self.keys[col as usize];
            col = 7;
          }
          col += 1;
        }
        !row
      },
      0x02 => self.mask_a_1,

      0x04 => (self.timer_a_1_value & 0xff) as u8,
      0x05 => ((self.timer_a_1_value & 0xff00) >> 8) as u8,
      
      0x0d => {
        let mut status = 0;
        if self.timer_a_1_interrupt {
          status = status | 1 | 128;
          self.timer_a_1_interrupt = false;
        }
        status
      },
      0x0e => self.timer_a_1_register,

      _ => 0,
    }
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    match addr % 16 {
      0x00 => {
        let mut port_a = self.port_a_1;
        port_a = port_a | (value & self.mask_a_1);
        port_a = port_a & !(!value & self.mask_a_1);
        self.port_a_1 = port_a;
      },
      0x01 => (),
      0x02 => self.mask_a_1 = value,

      0x04 => {
        let high = self.timer_a_1_latch & 0xff00;
        self.timer_a_1_latch = high | (value as u16);
      },
      0x05 => {
        let low = self.timer_a_1_latch & 0xff;
        self.timer_a_1_latch = ((value as u16) << 8) | low;
      },

      0x0d => {
        let set = value & 0x80 != 0;
        if value & 1 == 1 {
          self.timer_a_1_interrupt_enabled = set;
        }
      },
      0x0e => {
        self.timer_a_1_enabled = value & 1 != 0;
        self.timer_a_1_restart = value & 8 == 0;
        if value & 0x10 != 0 {
          self.timer_a_1_value = self.timer_a_1_latch;
        }
        self.timer_a_1_register = value;
      },
      _ => (),
    };
  }

  pub fn keydown(&mut self, index: u8) {
    if index > 63 {
      return;
    }
    let row = (index / 8) as usize;
    let shift = index % 8;
    let orig = self.keys[row];
    self.keys[row] = orig | (1 << shift);
  }

  pub fn keyup(&mut self, index: u8) {
    if index > 63 {
      return;
    }
    let row = (index / 8) as usize;
    let shift = index % 8;
    let orig = self.keys[row];
    self.keys[row] = orig & !(1 << shift);
  }

  pub fn update_timers(&mut self, cycles: u8) -> bool {
    if !self.timer_a_1_enabled {
      return false;
    }
    // not accurate, really needs to be 1.023 cycles per update
    let (value, underflow) = self.timer_a_1_value.overflowing_sub(cycles as u16);
    if underflow {
      if self.timer_a_1_restart {
        self.timer_a_1_value = self.timer_a_1_latch;
      } else {
        self.timer_a_1_enabled = false;
      }
      return true;
    }
    self.timer_a_1_value = value;
    return false;
  }
}

#[cfg(test)]
mod tests {
  use cia::CIA;

  #[test]
  fn port_a_masking() {
    let mut cia = CIA::new();
    cia.set_byte(2, 0b11110000);
    cia.set_byte(0, 0b11011000);
    assert_eq!(cia.get_byte(0), 0b11010000);
    cia.set_byte(0, 0b01101101);
    assert_eq!(cia.get_byte(0), 0b01100000);
  }
}