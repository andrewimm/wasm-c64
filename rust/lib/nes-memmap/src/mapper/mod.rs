mod mapper;
mod mmc1;

use self::mapper::Mapper;

// Create a Mapper instance from an iNes ROM
pub fn create_mapper(rom: &Vec<u8>) -> Result<Box<impl mapper::Mapper>, &'static str> {
  let header = &rom[0..16];
  if header[0] != 0x4e || header[1] != 0x45 || header[2] != 0x53 || header[3] != 0x1a {
    return Err("Invalid ROM header")
  }

  let config = mapper::Config {
    prg_rom_size: header[4],
    chr_rom_size: header[5],
    mirroring: if header[6] & 1 == 1 { mapper::Mirroring::Horizontal } else { mapper::Mirroring::Vertical },
    contains_ram: header[6] & 2 == 2,
  };
  let mapper_low = (header[6] & 0xf0) >> 4;
  let mapper_high = header[7] & 0xf0;
  let mapper = mapper_low | mapper_high;

  let m = match mapper {
    0x01 => Ok(Box::new(mmc1::MMC1::new(config))),
    _ => Err("Unsupported Mapper ID")
  };

  match m {
    Ok(mut mmc) => {
      let prg_start = 16;
      let prg_end = 0x4000 * header[4] as usize;
      let chr_end = prg_end + 0x2000 * header[5] as usize;
      mmc.set_prg_rom(&rom[prg_start..prg_end]);
      mmc.set_chr_rom(&rom[prg_end..chr_end]);
      Ok(mmc)
    },
    Err(msg) => Err(msg),
  }
}