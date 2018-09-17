pub struct Mem {
  pub ram: Box<[u8; 0x10000]>,
  pub color_ram: Box<[u8; 0x400]>,

  pub kernal: Box<[u8; 0x2000]>,
  pub char_gen: Box<[u8; 0x1000]>,
  pub basic: Box<[u8; 0x2000]>,
}

pub fn create() -> Mem {
  return Mem {
    ram: box [0; 0x10000],
    color_ram: box [0; 0x400],

    kernal: box [0; 0x2000],
    char_gen: box [0; 0x1000],
    basic: box [0; 0x2000],
  };
}