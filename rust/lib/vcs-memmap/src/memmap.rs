use mos6510::memory::Memory;
use crate::tia::TIA;
use crate::riot::RIOT;

pub struct MemMap {
  pub ram: [u8;128],
  pub rom: [u8;0x2000],
  pub tia: TIA,
  pub riot: RIOT,
  enable_bankswitch: bool,
  bank_no: u16,
}

impl MemMap {
  pub fn new() -> MemMap {
    MemMap {
      ram: [0;128],
      rom: [0;0x2000],
      tia: TIA::new(),
      riot: RIOT::new(),
      enable_bankswitch: false,
      bank_no: 0,
    }
  }

  pub fn load_rom(&mut self, rom: Box<[u8]>) {
    for i in 0..rom.len() {
      self.rom[i] = rom[i];
    }
    if rom.len() > 0x1000 {
      self.enable_bankswitch = true;
    }
  }
}

impl Memory for MemMap {
  fn get_byte(&mut self, addr: u16) -> u8 {
    let dest = addr & 0x1fff;
    if dest < 0x80 {
      // TIA
      // For reading, the TIA only uses 4 lines
      let tia_addr = dest & 0xf;

      return 0;
    }
    if dest < 0x100 {
      return self.ram[(dest - 0x80) as usize];
    }
    if dest < 0x180 {
      return 0;
    }
    if dest < 0x200 {
      // RAM is mirrored at 0x180-0x1ff
      return self.ram[(dest - 0x180) as usize];
    }
    if dest < 0x280 {
      return 0;
    }
    if dest < 0x300 {
      // RIOT
      if dest == 0x280 {
        return self.riot.get_port_a_data();
      }
      if dest == 0x284 {
        return self.riot.timer_count_remaining();
      }
      return 0;
    }
    if dest < 0x1000 {
      return 0;
    }
    if dest < 0x2000 {
      // ROM
      // F8 Bankswitching
      if dest == 0x1ff8 && self.enable_bankswitch {
        self.bank_no = 0;
        return 0;
      }
      if dest == 0x1ff9 && self.enable_bankswitch {
        self.bank_no = 0;
        return 0;
      }
      return self.rom[(dest - 0x1000 + self.bank_no * 0x1000) as usize];
    }
    return 0;
  }

  fn set_byte(&mut self, addr: u16, value: u8) {
    let dest = addr & 0x1fff;
    if dest < 0x80 {
      // TIA
      self.tia.set_byte(addr, value);
      return;
    }
    if dest < 0x100 {
      self.ram[(dest - 0x80) as usize] = value;
      return;
    }
    if dest < 0x180 {
      return;
    }
    if dest < 0x200 {
      self.ram[(dest - 0x180) as usize] = value;
      return;
    }
    if dest < 0x280 {
      return;
    }
    if dest < 0x300 {
      // RIOT
      if dest == 0x294 {
        self.riot.set_timer_1(value);
        return;
      }
      if dest == 0x295 {
        self.riot.set_timer_8(value);
        return;
      }
      if dest == 0x296 {
        self.riot.set_timer_64(value);
        return;
      }
      if dest == 0x297 {
        self.riot.set_timer_1024(value);
        return;
      }
      return;
    }
    if dest < 0x1000 {
      return;
    }
    if dest < 0x2000 {
      // ROM
      return;
    }
  }
}