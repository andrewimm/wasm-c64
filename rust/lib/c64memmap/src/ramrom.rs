pub struct RamRom {
  pub ram: Box<[u8; 0x10000]>,
  pub color_ram: Box<[u8; 0x400]>,

  pub kernal: Box<[u8; 0x2000]>,
  pub char_gen: Box<[u8; 0x1000]>,
  pub basic: Box<[u8; 0x2000]>,
}

impl RamRom {
  pub fn new() -> RamRom {
    return RamRom {
      ram: box [0; 0x10000],
      color_ram: box [0; 0x400],

      kernal: box [0; 0x2000],
      char_gen: box [0; 0x1000],
      basic: box [0; 0x2000],
    };
  }

  pub fn kernal_ptr(&mut self) -> *mut u8 {
    let ptr = &mut self.kernal[0] as *mut u8;
    return ptr;
  }

  pub fn char_ptr(&mut self) -> *mut u8 {
    let ptr = &mut self.char_gen[0] as *mut u8;
    return ptr;
  }

  pub fn basic_ptr(&mut self) -> *mut u8 {
    let ptr = &mut self.basic[0] as *mut u8;
    return ptr;
  }

  pub fn color_ptr(&mut self) -> *mut u8 {
    let ptr = &mut self.color_ram[0] as *mut u8;
    return ptr;
  }

  pub fn ram_ptr(&mut self) -> *mut u8 {
    let ptr = &mut self.ram[0] as *mut u8;
    return ptr;
  }

  pub fn screen_ptr(&mut self) -> *mut u8 {
    let ptr = &mut self.ram[0x400] as *mut u8;
    return ptr;
  }

  pub fn initialize_kernal_rom(&mut self, rom: &'static [u8;0x2000]) {
    for i in 0..0x2000 {
      self.kernal[i] = rom[i];
    }
  }

  pub fn initialize_basic_rom(&mut self, rom: &'static [u8;0x2000]) {
    for i in 0..0x2000 {
      self.basic[i] = rom[i];
    }
  }

  pub fn initialize_char_rom(&mut self, rom: &'static [u8;0x1000]) {
    for i in 0..0x1000 {
      self.char_gen[i] = rom[i];
    }
  }
}