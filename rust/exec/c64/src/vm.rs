use mos6510::cpu::CPU;
use c64memmap::memmap::MemMap;

pub struct VM {
  pub cpu: CPU,
  pub mem: MemMap,
}

const CYCLES_PER_MS: u32 = 1023;

const CHAR_ROM: &[u8;0x1000] = include_bytes!("rom/char.bin");
const KERNAL_ROM: &[u8;0x2000] = include_bytes!("rom/kernal.bin");
const BASIC_ROM: &[u8;0x2000] = include_bytes!("rom/basic.bin");

impl VM {
  pub fn new() -> VM {
    let mut vm = VM {
      cpu: CPU::new(),
      mem: MemMap::new(),
    };
    vm.mem.ram_rom.initialize_char_rom(CHAR_ROM);
    vm.mem.ram_rom.initialize_kernal_rom(KERNAL_ROM);
    vm.mem.ram_rom.initialize_basic_rom(BASIC_ROM);

    vm.reset();
    return vm;
  }

  pub fn step(&mut self) -> u8 {
    return self.cpu.step(&mut self.mem);
  }

  pub fn run_for_ms(&mut self, ms: u32) {
    let cycles = CYCLES_PER_MS * ms;
    let mut ran = 0;
    while ran < cycles {
      let step_time = self.step();
      if self.mem.cia.update_timers(step_time) {
        // cia interrupt fired
        self.cpu.interrupt_request(&mut self.mem);
      }
      ran += step_time as u32;
    }
  }

  pub fn reset(&mut self) {
    self.cpu.reset(&mut self.mem);
  }
}