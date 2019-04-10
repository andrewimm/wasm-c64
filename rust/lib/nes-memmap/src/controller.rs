pub struct Controller {
  pub a: bool,
  pub b: bool,
  pub select: bool,
  pub start: bool,
  pub up: bool,
  pub down: bool,
  pub left: bool,
  pub right: bool,

  latching: bool,
  latch: u8,
}

impl Controller {
  pub fn new() -> Controller {
    Controller {
      a: false,
      b: false,
      select: false,
      start: false,
      up: false,
      down: false,
      left: false,
      right: false,

      latching: false,
      latch: 0,
    }
  }
  pub fn begin_latch(&mut self) {
    self.latching = true;
    self.latch = if self.a { 0x01 } else { 0 };
    if self.b {
      self.latch = self.latch | 0x02;
    }
    if self.select {
      self.latch = self.latch | 0x04;
    }
    if self.start {
      self.latch = self.latch | 0x08;
    }
    if self.up {
      self.latch = self.latch | 0x10;
    }
    if self.down {
      self.latch = self.latch | 0x20;
    }
    if self.left {
      self.latch = self.latch | 0x40;
    }
    if self.right {
      self.latch = self.latch | 0x80;
    }
  }

  pub fn end_latch(&mut self) {
    self.latching = false;
  }

  pub fn read_latch(&mut self) -> u8 {
    let value = self.latch & 1;
    if !self.latching {
      self.latch = self.latch >> 1;
    }
    value
  }
}

#[cfg(test)]
mod tests {
  use crate::controller::Controller;

  #[test]
  fn read_controller() {
    let mut c = Controller::new();
    assert_eq!(c.latch, 0);
    c.a = true;
    c.select = true;
    assert_eq!(c.latch, 0);
    c.begin_latch();
    assert_eq!(c.latch, 5);
    assert_eq!(c.read_latch(), 1);
    assert_eq!(c.read_latch(), 1);
    c.end_latch();
    assert_eq!(c.read_latch(), 1);
    assert_eq!(c.read_latch(), 0);
    assert_eq!(c.read_latch(), 1);
    assert_eq!(c.read_latch(), 0);
  }
}