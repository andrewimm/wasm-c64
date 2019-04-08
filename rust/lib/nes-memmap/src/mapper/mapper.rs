pub trait Mapper {
  fn cpu_get_byte(&mut self, addr: u16) -> u8;
  fn cpu_set_byte(&mut self, addr: u16, value: u8);
  fn ppu_get_byte(&self, addr: u16) -> u8;
  fn ppu_get_mirrored_address(&self, addr: u16) -> u16;

  fn set_prg_rom(&mut self, rom: &[u8]);
  fn set_chr_rom(&mut self, rom: &[u8]);
}

pub enum Mirroring {
  Horizontal,
  Vertical,
}

pub enum ChrMem {
  Rom(Box<[u8]>),
  Ram(Box<[u8]>),
}

pub struct Config {
  pub prg_rom_size: u8,
  pub chr_rom_size: u8,
  pub mirroring: Mirroring,
  pub contains_ram: bool,
}