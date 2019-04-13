enum Timer {
  Cycle1,
  Cycle8,
  Cycle64,
  Cycle1024,
}

pub struct RIOT {
  timer_cycles: u16,
  timer_type: Timer,
  timer_count: u8,

  pub joystick_0_left: bool,
  pub joystick_0_right: bool,
  pub joystick_0_up: bool,
  pub joystick_0_down: bool,
}

impl RIOT {
  pub fn new() -> RIOT {
    RIOT {
      timer_cycles: 0,
      timer_type: Timer::Cycle1,
      timer_count: 0,

      joystick_0_left: false,
      joystick_0_right: false,
      joystick_0_up: false,
      joystick_0_down: false,
    }
  }

  pub fn set_timer_1(&mut self, count: u8) {
    self.timer_type = Timer::Cycle1;
    self.timer_cycles = 1 * 3;
    self.timer_count = count;
  }

  pub fn set_timer_8(&mut self, count: u8) {
    self.timer_type = Timer::Cycle8;
    self.timer_cycles = 8 * 3;
    self.timer_count = count;
  }

  pub fn set_timer_64(&mut self, count: u8) {
    self.timer_type = Timer::Cycle64;
    self.timer_cycles = 64 * 3;
    self.timer_count = count;
  }

  pub fn set_timer_1024(&mut self, count: u8) {
    self.timer_type = Timer::Cycle1024;
    self.timer_cycles = 1024 * 3;
    self.timer_count = count;
  }

  pub fn increment_clock(&mut self) {
    if self.timer_cycles > 0 {
      self.timer_cycles -= 1;
    }
    if self.timer_cycles == 0 {
      self.timer_cycles = match self.timer_type {
        Timer::Cycle1 => 1 * 3,
        Timer::Cycle8 => 8 * 3,
        Timer::Cycle64 => 64 * 3,
        Timer::Cycle1024 => 1024 * 3,
      };
      if self.timer_count > 0 {
        self.timer_count -= 1;
      }
    }
  }

  pub fn timer_count_remaining(&self) -> u8 {
    self.timer_count
  }

  pub fn get_port_a_data(&self) -> u8 {
    (if self.joystick_0_right { 0 } else { 0x80 }) |
    (if self.joystick_0_left { 0 } else { 0x40 }) |
    (if self.joystick_0_down { 0 } else { 0x20 }) |
    (if self.joystick_0_up { 0 } else { 0x10 }) | 0xf
  }
}