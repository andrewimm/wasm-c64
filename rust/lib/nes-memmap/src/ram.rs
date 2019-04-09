pub struct RAM {
  pub ram: Box<[u8; 0x800]>,
}

impl RAM {
  pub fn new() -> RAM {
    return RAM {
      ram: box [0; 0x800],
    };
  }

  pub fn get_byte(&mut self, addr: u16) -> u8 {
    return self.ram[addr as usize];
  }

  pub fn set_byte(&mut self, addr: u16, value: u8) {
    self.ram[addr as usize] = value;
  }
}