mod mapper;
mod mmc1;
mod nrom;

pub use self::mapper::Mapper;

// Create a Mapper instance from an iNes ROM
pub fn create_mapper(rom: &Vec<u8>) -> Box<Mapper> {
  let header = &rom[0..16];
  if header[0] != 0x4e || header[1] != 0x45 || header[2] != 0x53 || header[3] != 0x1a {
    panic!("Invalid ROM header");
  }

  let config = mapper::Config {
    prg_rom_size: header[4],
    chr_rom_size: header[5],
    mirroring: if header[6] & 1 == 1 { mapper::Mirroring::Horizontal } else { mapper::Mirroring::Vertical },
    contains_ram: header[6] & 2 == 2,
  };
  let mapper_low = (header[6] & 0xf0) >> 4;
  let mapper_high = header[7] & 0xf0;
  let mapper_id = mapper_low | mapper_high;

  let mut mapper: Box<Mapper> = match mapper_id {
    0x00 => Box::new(nrom::NROM::new(config)),
    0x01 => Box::new(mmc1::MMC1::new(config)),
    _ => panic!("Unsupported Mapper ID"),
  };

  let prg_start = 16;
  let prg_end = 16 + 0x4000 * header[4] as usize;
  let chr_end = prg_end + 0x2000 * header[5] as usize;
  mapper.set_prg_rom(&rom[prg_start..prg_end]);
  mapper.set_chr_rom(&rom[prg_end..chr_end]);
  mapper
}